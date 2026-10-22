package research

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type Hit struct {
	URL   string `json:"url"`
	Title string `json:"title"`
}
type Evidence struct {
	ID          string `json:"id"`
	URL         string `json:"url"`
	Title       string `json:"title"`
	SourceID    string `json:"source_id"`
	Edition     string `json:"edition"`
	Kind        string `json:"kind"`
	FetchedAt   string `json:"fetched_at"`
	ContentHash string `json:"content_hash"`
	Content     string `json:"content"`
	Locator     string `json:"locator"`
	Term        string `json:"term"`
	Status      string `json:"status"`
}
type Client struct {
	HTTP     *http.Client
	Store    state.Store
	Sources  []workspace.ResearchSource
	Interval time.Duration
	mu       sync.Mutex
	last     map[string]time.Time
}

func New(c workspace.Config) *Client {
	return &Client{HTTP: &http.Client{Timeout: 30 * time.Second}, Store: state.Store{Root: c.Root}, Sources: c.ResearchSources, Interval: time.Second, last: map[string]time.Time{}}
}
func sameOrigin(a, b string) bool {
	x, e := url.Parse(a)
	if e != nil {
		return false
	}
	y, e := url.Parse(b)
	return e == nil && (x.Scheme == "https" || x.Scheme == "http") && x.Scheme == y.Scheme && x.Host == y.Host && y.User == nil
}
func (c *Client) get(ctx context.Context, base, target string) ([]byte, error) {
	if !sameOrigin(base, target) {
		return nil, errors.New("research URL ngoài origin được cấu hình")
	}
	parsed, _ := url.Parse(target)
	c.mu.Lock()
	if c.last == nil {
		c.last = map[string]time.Time{}
	}
	delay := time.Until(c.last[parsed.Host].Add(c.Interval))
	c.last[parsed.Host] = time.Now().Add(max(delay, 0))
	c.mu.Unlock()
	if delay > 0 {
		timer := time.NewTimer(delay)
		defer timer.Stop()
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-timer.C:
		}
	}
	client := *c.HTTP
	client.CheckRedirect = func(req *http.Request, via []*http.Request) error {
		if len(via) >= 5 || !sameOrigin(base, req.URL.String()) {
			return errors.New("research redirect ngoài origin/loop")
		}
		return nil
	}
	for attempt := 0; attempt < 2; attempt++ {
		req, e := http.NewRequestWithContext(ctx, http.MethodGet, target, nil)
		if e != nil {
			return nil, e
		}
		req.Header.Set("User-Agent", "novel-translate-cli/1 source-evidence-research")
		resp, e := client.Do(req)
		if e != nil {
			return nil, e
		}
		body, e := io.ReadAll(io.LimitReader(resp.Body, 2<<20+1))
		resp.Body.Close()
		if e != nil {
			return nil, e
		}
		if len(body) > 2<<20 {
			return nil, errors.New("research page vượt 2 MiB")
		}
		if resp.StatusCode == 429 && attempt == 0 {
			delay := time.Second
			if n, e := strconv.Atoi(resp.Header.Get("Retry-After")); e == nil && n > 0 {
				delay = time.Duration(n) * time.Second
			}
			if delay > 30*time.Second {
				delay = 30 * time.Second
			}
			timer := time.NewTimer(delay)
			select {
			case <-ctx.Done():
				timer.Stop()
				return nil, ctx.Err()
			case <-timer.C:
			}
			continue
		}
		if resp.StatusCode != 200 {
			return nil, fmt.Errorf("research HTTP %d", resp.StatusCode)
		}
		return body, nil
	}
	return nil, errors.New("research rate limit chưa giải quyết")
}
func (c *Client) Search(ctx context.Context, s workspace.ResearchSource, term string) ([]Hit, error) {
	base, e := url.Parse(s.BaseURL)
	if e != nil {
		return nil, e
	}
	switch s.API {
	case "fetch":
		return []Hit{{URL: s.BaseURL, Title: s.ID}}, nil
	case "mediawiki":
		endpoint := *base
		endpoint.Path = strings.TrimSuffix(base.Path, "/") + "/api.php"
		q := endpoint.Query()
		q.Set("action", "query")
		q.Set("list", "search")
		q.Set("srsearch", term)
		q.Set("srlimit", "3")
		q.Set("format", "json")
		endpoint.RawQuery = q.Encode()
		b, e := c.get(ctx, s.BaseURL, endpoint.String())
		if e != nil {
			return nil, e
		}
		var result struct {
			Error *struct {
				Info string `json:"info"`
			} `json:"error"`
			Query struct {
				Search []struct {
					Title string `json:"title"`
				} `json:"search"`
			} `json:"query"`
		}
		if e = json.Unmarshal(b, &result); e != nil {
			return nil, e
		}
		if result.Error != nil {
			return nil, errors.New(result.Error.Info)
		}
		var out []Hit
		for _, hit := range result.Query.Search {
			u := *base
			u.Path = "/wiki/" + strings.ReplaceAll(hit.Title, " ", "_")
			u.RawQuery = ""
			out = append(out, Hit{URL: u.String(), Title: hit.Title})
		}
		return out, nil
	case "json":
		q := base.Query()
		q.Set("q", term)
		base.RawQuery = q.Encode()
		b, e := c.get(ctx, s.BaseURL, base.String())
		if e != nil {
			return nil, e
		}
		var result struct {
			Results []Hit `json:"results"`
		}
		if e = json.Unmarshal(b, &result); e != nil {
			return nil, e
		}
		for _, hit := range result.Results {
			if !sameOrigin(s.BaseURL, hit.URL) {
				return nil, errors.New("search trả URL không được phép")
			}
		}
		return result.Results, nil
	default:
		return nil, fmt.Errorf("research api chưa hỗ trợ: %s", s.API)
	}
}
func (c *Client) Fetch(ctx context.Context, s workspace.ResearchSource, hit Hit, term string) (Evidence, error) {
	id := domain.Digest(struct {
		S    workspace.ResearchSource
		URL  string
		Term string
	}{s, hit.URL, term})
	path := c.Store.Path("research", "evidence", id+".json")
	var saved Evidence
	if e := state.ReadJSON(path, &saved); e == nil {
		if saved.URL == hit.URL && domain.Hash([]byte(saved.Content)) == saved.ContentHash {
			return saved, nil
		}
		return saved, errors.New("research cache checksum không khớp")
	}
	b, e := c.get(ctx, s.BaseURL, hit.URL)
	if e != nil {
		return Evidence{}, e
	}
	ev := Evidence{ID: id, URL: hit.URL, Title: hit.Title, SourceID: s.ID, Edition: s.Edition, Kind: s.Kind, FetchedAt: time.Now().UTC().Format(time.RFC3339), ContentHash: domain.Hash(b), Content: string(b), Locator: "UTF-8 byte range 0:" + strconv.Itoa(len(b)), Term: term, Status: "reference-only; future knowledge excluded"}
	return ev, state.WriteJSON(path, ev)
}
func (c *Client) Research(ctx context.Context, term string, book domain.Book) ([]Evidence, error) {
	seenInSource := false
	for _, b := range book.Blocks {
		if term != "" && strings.Contains(b.Source, term) {
			seenInSource = true
			break
		}
	}
	if !seenInSource {
		return nil, errors.New("term phải đã xuất hiện trong nguồn nhập")
	}
	if len(c.Sources) == 0 {
		return nil, errors.New("chưa cấu hình research_sources")
	}
	var result []Evidence
	seen := map[string]bool{}
	var errs []error
	for _, s := range c.Sources {
		hits, e := c.Search(ctx, s, term)
		if e != nil {
			errs = append(errs, fmt.Errorf("%s: %w", s.ID, e))
			continue
		}
		for _, hit := range hits {
			if seen[hit.URL] {
				continue
			}
			seen[hit.URL] = true
			ev, e := c.Fetch(ctx, s, hit, term)
			if e != nil {
				errs = append(errs, e)
				continue
			}
			result = append(result, ev)
		}
	}
	return result, errors.Join(errs...)
}

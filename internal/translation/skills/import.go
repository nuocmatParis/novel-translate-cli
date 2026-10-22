package skills

import (
	"errors"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type ImportScope struct {
	FromVolume  int
	UntilVolume int
}
type Reference struct {
	ID        string           `json:"id"`
	Source    string           `json:"source"`
	Path      string           `json:"path"`
	Hash      string           `json:"hash"`
	Paragraph int              `json:"paragraph"`
	Body      string           `json:"body"`
	From      domain.Position  `json:"from"`
	Until     *domain.Position `json:"until,omitempty"`
	Status    string           `json:"status"`
}

var volumeMention = regexp.MustCompile(`(?i)(?:vol(?:ume)?\.?|tập)\s*(\d+)(?:\s*[–—-]\s*(\d+))?`)

func scopedParagraphs(body string, explicit ImportScope) []Reference {
	var out []Reference
	heading := ImportScope{}
	for i, para := range strings.Split(strings.ReplaceAll(body, "\r\n", "\n"), "\n\n") {
		trim := strings.TrimSpace(para)
		if trim == "" {
			continue
		}
		scope := heading
		mentions := volumeMention.FindAllStringSubmatch(para, -1)
		if strings.HasPrefix(trim, "#") {
			heading = ImportScope{}
			scope = heading
		}
		if len(mentions) > 0 {
			start, _ := strconv.Atoi(mentions[0][1])
			end := start + 1
			if mentions[0][2] != "" {
				n, _ := strconv.Atoi(mentions[0][2])
				end = n + 1
			}
			for _, m := range mentions[1:] {
				n, _ := strconv.Atoi(m[1])
				if n > start {
					start = n
				}
				if n >= end {
					end = n + 1
				}
				if m[2] != "" {
					n, _ = strconv.Atoi(m[2])
					if n >= end {
						end = n + 1
					}
				}
			}
			scope = ImportScope{start, end}
			if strings.HasPrefix(trim, "#") {
				heading = scope
			}
		}
		if explicit.FromVolume > 0 {
			if scope.FromVolume == 0 {
				scope = explicit
			} else {
				scope.FromVolume = max(scope.FromVolume, explicit.FromVolume)
				if explicit.UntilVolume > 0 {
					scope.UntilVolume = min(scope.UntilVolume, explicit.UntilVolume)
				}
			}
		}
		r := Reference{Paragraph: i + 1, Body: para, Status: "quarantined — cần scope", From: domain.Position{Volume: scope.FromVolume}}
		if scope.UntilVolume > 0 {
			r.Until = &domain.Position{Volume: scope.UntilVolume}
		}
		if scope.FromVolume > 0 && (scope.UntilVolume == 0 || scope.FromVolume < scope.UntilVolume) {
			r.Status = "scoped-reference"
		}
		out = append(out, r)
	}
	return out
}

// Import preserves the exact legacy files and creates source-located paragraphs.
// Explicit volume mentions are conservatively limited to those volumes. Unknown
// scope remains quarantined unless the caller supplies a range deliberately.
func Import(c workspace.Config, source string, scopes ...ImportScope) (string, error) {
	explicit := ImportScope{}
	if len(scopes) > 0 {
		explicit = scopes[0]
	}
	if explicit.FromVolume < 0 || explicit.UntilVolume < 0 || explicit.UntilVolume > 0 && explicit.UntilVolume <= explicit.FromVolume {
		return "", errors.New("import scope không hợp lệ")
	}
	source = c.ReadPath(source)
	info, e := os.Stat(source)
	if e != nil {
		return "", e
	}
	var files []string
	if info.IsDir() {
		e = filepath.WalkDir(source, func(p string, d os.DirEntry, e error) error {
			if e != nil {
				return e
			}
			if d.Type()&os.ModeSymlink != 0 {
				return errors.New("không import symlink")
			}
			if !d.IsDir() && strings.HasSuffix(strings.ToLower(p), ".md") {
				files = append(files, p)
			}
			return nil
		})
	} else {
		files = []string{source}
	}
	if e != nil {
		return "", e
	}
	if len(files) == 0 {
		return "", errors.New("không có Markdown để import")
	}
	var records []Reference
	for _, f := range files {
		b, e := os.ReadFile(f)
		if e != nil {
			return "", e
		}
		hash := domain.Hash(b)
		dest, e := c.WritePath("Skills/imported/" + hash + "/" + filepath.Base(f))
		if e != nil {
			return "", e
		}
		if e = state.Atomic(dest, b); e != nil {
			return "", e
		}
		for _, r := range scopedParagraphs(string(b), explicit) {
			r.Source = f
			r.Path = dest
			r.Hash = hash
			r.ID = domain.Digest(r)[:24]
			records = append(records, r)
		}
	}
	id := domain.Digest(records)[:20]
	store := state.Store{Root: c.Root}
	if e = state.WriteJSON(store.Path("skill_imports", id+".json"), records); e != nil {
		return "", e
	}
	var existing []Reference
	index := c.ReadPath("Skills/imported/index.json")
	if e = state.ReadJSON(index, &existing); e != nil && !errors.Is(e, os.ErrNotExist) {
		return "", e
	}
	seen := map[string]bool{}
	for _, r := range existing {
		seen[r.ID] = true
	}
	for _, r := range records {
		if !seen[r.ID] {
			existing = append(existing, r)
			seen[r.ID] = true
		}
	}
	return id, state.WriteJSON(index, existing)
}

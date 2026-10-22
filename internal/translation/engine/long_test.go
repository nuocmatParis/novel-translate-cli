package engine

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
	"path/filepath"
	"strings"
	"testing"
	"unicode/utf8"
)

type longModel struct {
	calls          map[string]int
	completed      int
	cancelAt       int
	reviewCalls    int
	cancelReviewAt int
}

func (*longModel) Identity(string) string { return "fake/long" }
func (*longModel) InputLimit(string) int  { return 24000 }
func (m *longModel) Call(ctx context.Context, role, system string, in any, c llmcontract.Contract, out any, validate func() error) error {
	p := in.(payload)
	b := p.Units[0]
	if m.calls == nil {
		m.calls = map[string]int{}
	}
	key := role + ":" + b.ID
	m.calls[key]++
	if role == "translation" {
		if len(b.Source) > 700 {
			return &llmcontract.Failure{Kind: llmcontract.FailureLength, Err: errors.New("scripted output truncation")}
		}
		m.completed++
		if m.completed == m.cancelAt {
			return context.Canceled
		}
		raw, _ := json.Marshal(map[string]any{"units": []any{map[string]string{"id": b.ID, "target": testutil.TranslateFixture(b.Source)}}, "continuity_candidates": []any{map[string]string{"block_id": b.ID, "quote": "彼は来た", "speaker": "a", "listener": "b", "value": "confirmed"}}, "rule_candidates": []any{}, "uncertainties": []any{}})
		json.Unmarshal(raw, out)
	} else {
		m.reviewCalls++
		if m.reviewCalls == m.cancelReviewAt {
			return context.Canceled
		}
		raw, _ := json.Marshal(map[string]any{"reviewed": []any{map[string]string{"id": b.ID, "target_hash": domain.Hash([]byte(p.Targets[0].Target))}}, "issues": []any{}, "ruby_decisions": testutil.RubyDecisions(b)})
		json.Unmarshal(raw, out)
	}
	return validate()
}
func TestAdaptiveLongBlockResumeAndReviewResume(t *testing.T) {
	m := &longModel{cancelAt: 2}
	r := Runner{Volume: workspace.Volume{ID: "v1", Order: 1}, Store: state.Store{Root: t.TempDir()}, Model: m}
	b := domain.Block{ID: "v1:long", Source: strings.Repeat("彼は来た。", 400), Position: domain.Position{Volume: 1, Chapter: 1, Block: 1}}
	b.SourceHash = domain.Hash([]byte(b.Source))
	p := payload{Mode: "translate", Units: []domain.Block{b}}
	if _, e := r.translate(context.Background(), b, p, "", domain.Dependencies{}, nil); !errors.Is(e, context.Canceled) {
		t.Fatal(e)
	}
	first := ""
	// Locate the persisted completed leaf rather than assuming an adaptive ID.
	files, _ := filepath.Glob(r.Store.Path("volumes", "v1", "jobs", "*.json"))
	var saved []domain.Child
	if len(files) != 1 {
		t.Fatal(files)
	}
	if e := state.ReadJSON(files[0], &saved); e != nil {
		t.Fatal(e)
	}
	for _, child := range saved {
		if child.Target != "" {
			first = "translation:" + child.ID
			break
		}
	}
	if first == "" {
		t.Fatal("no child saved before cancellation")
	}
	m.cancelAt = 0
	u, e := r.translate(context.Background(), b, p, "", domain.Dependencies{}, nil)
	if e != nil {
		t.Fatal(e)
	}
	if m.calls[first] != 1 || len(u.Children) < 4 || len(u.Events) != len(u.Children) {
		t.Fatal("completed leaf repeated or candidates lost", m.calls[first], len(u.Children), len(u.Events))
	}
	joined := ""
	for _, child := range u.Children {
		joined += child.Source
		if !utf8.ValidString(child.Source) {
			t.Fatal("split invalid UTF-8")
		}
	}
	if joined != b.Source {
		t.Fatal("source coverage lost")
	}
	m.cancelReviewAt = 2
	if _, e = r.review(context.Background(), b, u, p, ""); !errors.Is(e, context.Canceled) {
		t.Fatal(e)
	}
	firstReview := "translation_review:" + u.Children[0].ID
	m.cancelReviewAt = 0
	review, e := r.review(context.Background(), b, u, p, "")
	if e != nil || review.Decision != "accepted" || m.calls[firstReview] != 1 {
		t.Fatal("review resume repeated/lost child", review, e, m.calls[firstReview])
	}
}

func TestSplitProtectsInlineAndRuby(t *testing.T) {
	b := domain.Block{ID: "b", Source: strings.Repeat("あ", 35) + "⟦o⟧名⟦c⟧" + strings.Repeat("い", 70), Tokens: []domain.Token{{ID: "o", Kind: "open", Name: "strong"}, {ID: "c", Kind: "close", Name: "strong", Pair: "o"}}}
	start := strings.Index(b.Source, "名")
	b.Ruby = []domain.Ruby{{Base: "名", Reading: "な", Start: start, End: start + len("名")}}
	children, e := SplitBlock(b, 128)
	if e != nil {
		t.Fatal(e)
	}
	for _, child := range children {
		cb := b
		cb.Source = child.Source
		cb.Tokens = nil
		for _, token := range b.Tokens {
			if strings.Contains(cb.Source, "⟦"+token.ID+"⟧") {
				cb.Tokens = append(cb.Tokens, token)
			}
		}
		if _, e = epub.TokensHTML(cb, cb.Source); e != nil {
			t.Fatal("unbalanced child", child, e)
		}
	}
	b.Source = "⟦o⟧" + strings.Repeat("あ", 200) + "⟦c⟧"
	if _, e = SplitBlock(b, 128); !errors.Is(e, ErrUnresolved) {
		t.Fatal("indivisible over-budget markup not reported", e)
	}
}

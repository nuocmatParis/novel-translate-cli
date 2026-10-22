package engine

import (
	"context"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type scripted struct {
	Calls    int
	FailAt   int
	Omission bool
	Forge    bool
	Callback func()
	Counts   map[string]int
}

func TestLexicalRuleChangeKeepsUnrelatedRevisions(t *testing.T) {
	r, book := fixtureRunner(t, &scripted{})
	if e := r.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	g, e := r.Store.Load(r.Volume.ID)
	if e != nil {
		t.Fatal(e)
	}
	s, e := skills.Load(r.Config)
	if e != nil {
		t.Fatal(e)
	}
	s.Rules = []domain.Rule{{ID: "emphasis", Key: "強調", Value: "nhấn mạnh", Kind: "glossary", From: domain.Position{Volume: 1}, Priority: 100, Status: "confirmed", Version: 1}}
	p, e := ImpactPlan(r.Config, r.Volume, r.Store, book, g, s)
	if e != nil {
		t.Fatal(e)
	}
	if len(p.Units) != 1 {
		t.Fatal("lexical rule affected unrelated downstream blocks", p.Units)
	}
	selected := ""
	for _, b := range book.Blocks {
		if strings.Contains(b.Source, "強調") {
			selected = b.ID
		}
	}
	if p.Units[0].ID != selected {
		t.Fatal(p.Units)
	}
}

func TestManualSkillChangePlanRefreshesReviewWithoutReplacingText(t *testing.T) {
	s := &scripted{}
	r, book := fixtureRunner(t, s)
	if e := r.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	g, e := r.Store.Load("v1")
	if e != nil {
		t.Fatal(e)
	}
	var selected domain.Block
	for _, b := range book.Blocks {
		if b.Disposition == "translate" && len(b.Tokens) == 0 {
			selected = b
			break
		}
	}
	u := g.Units[selected.ID]
	u.Origin = "manual"
	u.Target = "Bản Việt do người dùng sửa."
	u.Review = nil
	u.RefreshDependencies = true
	g.Units[u.ID] = u
	if e = revision.CaptureViews(r.Config, r.Volume, book, &g); e != nil {
		t.Fatal(e)
	}
	if e = r.Store.Commit("v1", &g, g.ID); e != nil {
		t.Fatal(e)
	}
	if e = revision.Materialize(r.Config, r.Volume, book, g); e != nil {
		t.Fatal(e)
	}
	p := r.Config.ReadPath(r.Config.SeriesSkill)
	raw, _ := os.ReadFile(p)
	if e = os.WriteFile(p, append(raw, []byte("\nBám nguồn.\n")...), 0600); e != nil {
		t.Fatal(e)
	}
	snapshot, e := skills.Load(r.Config)
	if e != nil {
		t.Fatal(e)
	}
	impact, e := ImpactPlan(r.Config, r.Volume, r.Store, book, g, snapshot)
	if e != nil {
		t.Fatal(e)
	}
	if e = revision.Apply(&g, impact); e != nil {
		t.Fatal(e)
	}
	if e = revision.CaptureViews(r.Config, r.Volume, book, &g); e != nil {
		t.Fatal(e)
	}
	if e = r.Store.Commit("v1", &g, g.ID); e != nil {
		t.Fatal(e)
	}
	if e = revision.Materialize(r.Config, r.Volume, book, g); e != nil {
		t.Fatal(e)
	}
	before := s.Counts["translation:"+u.ID]
	if e = r.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	g, _ = r.Store.Load("v1")
	if g.Units[u.ID].Target != u.Target || !g.Units[u.ID].Accepted() {
		t.Fatal("manual revision lost or not reviewed")
	}
	if s.Counts["translation:"+u.ID] != before {
		t.Fatal("manual text translated over")
	}
}

func (s *scripted) Identity(role string) string { return "fake/" + role }
func (s *scripted) InputLimit(string) int       { return 24000 }
func (s *scripted) Call(ctx context.Context, role, system string, in any, contract llmcontract.Contract, out any, validate func() error) error {
	s.Calls++
	if s.Counts == nil {
		s.Counts = map[string]int{}
	}
	if s.FailAt == s.Calls {
		return context.Canceled
	}
	raw, _ := json.Marshal(in)
	var p payload
	if e := json.Unmarshal(raw, &p); e != nil {
		return e
	}
	b := p.Units[0]
	s.Counts[role+":"+b.ID]++
	var result any
	if role == "translation_review" {
		issues := []map[string]string{}
		if s.Omission && strings.Contains(b.Source, "雨が") {
			quote := "雨が降るが"
			if s.Forge {
				quote = "source span never present"
			}
			issues = append(issues, map[string]string{"id": "omitted-clause", "type": "omission", "severity": "blocking", "block_id": b.ID, "source_quote": quote, "target_quote": "", "explanation": "mất mệnh đề mưa"})
		}
		result = map[string]any{"reviewed": []map[string]string{{"id": b.ID, "target_hash": domain.Hash([]byte(p.Targets[0].Target))}}, "issues": issues, "ruby_decisions": testutil.RubyDecisions(b)}
	} else {
		target := testutil.TranslateFixture(b.Source)
		if s.Omission && strings.Contains(b.Source, "雨が") {
			target = "Cậu chạy."
		}
		result = map[string]any{"units": []map[string]string{{"id": b.ID, "target": target}}, "uncertainties": []any{}, "continuity_candidates": []any{}, "rule_candidates": []any{}}
	}
	raw, _ = json.Marshal(result)
	if e := json.Unmarshal(raw, out); e != nil {
		return e
	}
	if s.Callback != nil {
		s.Callback()
	}
	return validate()
}
func fixtureRunner(t *testing.T, s *scripted) (Runner, domain.Book) {
	t.Helper()
	c, e := workspace.Create(t.TempDir(), "test", "Test")
	if e != nil {
		t.Fatal(e)
	}
	if e = skills.InstallDefaults(c); e != nil {
		t.Fatal(e)
	}
	source := filepath.Join(c.Root, "input.epub")
	if e = os.WriteFile(source, testutil.EPUB(), 0600); e != nil {
		t.Fatal(e)
	}
	v := workspace.Volume{ID: "v1", Order: 1, Directory: "Tập 1", Source: "input.epub"}
	if e = c.Add(v); e != nil {
		t.Fatal(e)
	}
	book, e := Import(c, v)
	if e != nil {
		t.Fatal(e)
	}
	return Runner{Config: c, Volume: v, Store: state.Store{Root: c.Root}, Model: s}, book
}
func TestResumeAndReviewDigest(t *testing.T) {
	s := &scripted{FailAt: 2}
	r, book := fixtureRunner(t, s)
	if e := r.Run(context.Background(), false, ""); !errors.Is(e, context.Canceled) {
		t.Fatal(e)
	}
	g, e := r.Store.Load("v1")
	if e != nil {
		t.Fatal(e)
	}
	if len(g.Units) != 1 {
		t.Fatal("translation not persisted before review")
	}
	s.FailAt = 0
	if e = r.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	for id, u := range g.Units {
		if s.Counts["translation:"+id] != 1 {
			t.Fatal("resume translated committed unit twice")
		}
		_ = u
	}
	g, e = r.Store.Load("v1")
	if e != nil {
		t.Fatal(e)
	}
	eligible := 0
	for _, b := range book.Blocks {
		if b.Disposition == "translate" {
			eligible++
			if !g.Units[b.ID].Accepted() {
				t.Fatal("missing accepted unit", b.ID)
			}
		}
	}
	if len(g.Units) != eligible {
		t.Fatal("coverage")
	}
	calls := s.Calls
	if e = r.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	if s.Calls != calls {
		t.Fatal("repeat run made unnecessary requests")
	}
}
func TestMissingClauseStopsAfterBoundedRepair(t *testing.T) {
	s := &scripted{Omission: true}
	r, _ := fixtureRunner(t, s)
	r.Config.Run.MaxRepairRounds = 1
	e := r.Run(context.Background(), false, "")
	if !errors.Is(e, ErrUnresolved) {
		t.Fatal(e)
	}
	g, e := r.Store.Load("v1")
	if e != nil {
		t.Fatal(e)
	}
	found := false
	for _, u := range g.Units {
		if u.Review != nil && u.Review.Decision == "unresolved" {
			found = true
			if u.Accepted() {
				t.Fatal("blocking omission accepted")
			}
			if s.Counts["translation:"+u.ID] != 2 {
				t.Fatal("repair limit not honored")
			}
		}
	}
	if !found {
		t.Fatal("omission not retained")
	}
}
func TestManualEditDuringRequestPreserved(t *testing.T) {
	s := &scripted{}
	r, book := fixtureRunner(t, s)
	s.FailAt = 3
	if e := r.Run(context.Background(), false, ""); !errors.Is(e, context.Canceled) {
		t.Fatal(e)
	}
	g, e := r.Store.Load("v1")
	if e != nil {
		t.Fatal(e)
	}
	var edited string
	for _, ch := range book.Chapters {
		for _, id := range ch.BlockIDs {
			if u, ok := g.Units[id]; ok {
				p, e := revision.ChapterPath(r.Config, r.Volume, ch.ID)
				if e != nil {
					t.Fatal(e)
				}
				s.Callback = func() {
					b, _ := os.ReadFile(p)
					_ = os.WriteFile(p, []byte(strings.Replace(string(b), u.Target, "Bản sửa tay được giữ.", 1)), 0600)
					edited = p
				}
				break
			}
		}
		if s.Callback != nil {
			break
		}
	}
	s.FailAt = 0
	if e = r.Run(context.Background(), false, ""); e == nil {
		t.Fatal("concurrent edit committed")
	}
	b, e := os.ReadFile(edited)
	if e != nil || !strings.Contains(string(b), "Bản sửa tay được giữ.") {
		t.Fatal("manual edit overwritten", e)
	}
}
func TestInvalidEvidenceAndBatchPlan(t *testing.T) {
	b := domain.Block{ID: "b", Source: "雨が降るが、彼は走った。"}
	u := domain.Unit{ID: "b", Target: "Cậu chạy."}
	result := reviewed{}
	result.Reviewed = append(result.Reviewed, struct {
		ID         string `json:"id"`
		TargetHash string `json:"target_hash"`
	}{"b", domain.Hash([]byte(u.Target))})
	result.Issues = []domain.Issue{{ID: "i", BlockID: "b", Type: "omission", Severity: "blocking", SourceQuote: "fake", Explanation: "missing"}}
	if e := ValidateReview(b, u, result); e == nil {
		t.Fatal("forged evidence accepted")
	}
	chs := []domain.Chapter{}
	for i := 1; i <= 5; i++ {
		chs = append(chs, domain.Chapter{ID: string(rune('a' + i)), Order: i, Kind: "chapter"})
	}
	batches := PlanBatches(chs)
	if len(batches) != 3 || len(batches[0].Chapters) != 2 || len(batches[1].Chapters) != 2 || len(batches[2].Chapters) != 1 {
		t.Fatal(batches)
	}
	parts := Split("b", strings.Repeat("日本語。", 100), 128)
	all := ""
	last := 0
	for _, p := range parts {
		if p.Start != last || p.End-p.Start != len(p.Source) {
			t.Fatal("child gap")
		}
		last = p.End
		all += p.Source
	}
	if all != strings.Repeat("日本語。", 100) {
		t.Fatal("split lost bytes")
	}
}

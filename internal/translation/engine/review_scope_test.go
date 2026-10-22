package engine

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"

	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

// Echo application-supplied review identities instead of calculating hashes in
// the fake model. This catches requests that ask a real LLM to invent a digest.
type reviewScopeEcho struct {
	issues []domain.Issue
	scopes []struct {
		ID         string `json:"id"`
		TargetHash string `json:"target_hash"`
	}
}

func (*reviewScopeEcho) Identity(string) string { return "fake/scope-echo" }
func (*reviewScopeEcho) InputLimit(string) int  { return 24000 }
func (m *reviewScopeEcho) Call(_ context.Context, role, system string, in any, contract llmcontract.Contract, out any, validate func() error) error {
	if role != "translation_review" {
		return errors.New("saved translation must not be translated again")
	}
	data, err := json.Marshal(in)
	if err != nil {
		return err
	}
	var p struct {
		Units   []domain.Block `json:"owned_units"`
		Targets []domain.Unit  `json:"targets"`
		Scope   []struct {
			ID         string `json:"id"`
			TargetHash string `json:"target_hash"`
		} `json:"review_scope"`
	}
	if err := json.Unmarshal(data, &p); err != nil {
		return err
	}
	if len(p.Scope) != 1 || len(p.Units) != 1 || len(p.Targets) != 1 || p.Scope[0].ID != p.Units[0].ID || p.Scope[0].ID != p.Targets[0].ID || p.Scope[0].TargetHash == "" {
		return errors.New("review request must supply an exact id and target_hash to copy")
	}
	if !strings.Contains(system, "review_scope") {
		return errors.New("review prompt must explain how to copy review_scope")
	}
	m.scopes = append(m.scopes, p.Scope[0])
	issues := m.issues
	if issues == nil {
		issues = []domain.Issue{}
	}
	response := map[string]any{"reviewed": p.Scope, "issues": issues, "ruby_decisions": testutil.RubyDecisions(p.Units[0])}
	data, err = json.Marshal(response)
	if err != nil {
		return err
	}
	if err := llmcontract.ValidateJSON(contract.Schema, data); err != nil {
		return err
	}
	if err := json.Unmarshal(data, out); err != nil {
		return err
	}
	return validate()
}

func TestReviewSuppliesExactRevisionReference(t *testing.T) {
	m := &reviewScopeEcho{}
	r := Runner{Model: m}
	b := domain.Block{ID: "v1:dialogue", Source: "「雨だ」と彼は言った。"}
	b.SourceHash = domain.Hash([]byte(b.Source))
	u := domain.Unit{ID: b.ID, SourceHash: b.SourceHash, Target: "“Mưa rồi,” cậu nói.\n"}
	for _, target := range []string{u.Target, "“Mưa rồi!” cậu nói.\n"} {
		u.Target = target
		got, err := r.review(context.Background(), b, u, payload{}, "")
		if err != nil {
			t.Fatal(err)
		}
		if got.TargetHash != domain.Hash([]byte(target)) || got.Decision != "accepted" {
			t.Fatal("review not bound to current revision", got)
		}
		if last := m.scopes[len(m.scopes)-1]; last.TargetHash != got.TargetHash || last.ID != b.ID {
			t.Fatal("payload supplied a stale revision")
		}
	}
	if m.scopes[0].TargetHash == m.scopes[1].TargetHash {
		t.Fatal("edited target reused old review identity")
	}
}

func TestReviewStillRejectsMissingDuplicateOrStaleReferences(t *testing.T) {
	b := domain.Block{ID: "v1:current"}
	u := domain.Unit{ID: b.ID, Target: "Bản Việt hiện tại."}
	valid := reviewReference{ID: b.ID, TargetHash: domain.Hash([]byte(u.Target))}
	for _, refs := range [][]reviewReference{
		nil,
		{valid, valid},
		{{ID: "v1:other", TargetHash: valid.TargetHash}},
		{{ID: b.ID, TargetHash: domain.Hash([]byte("Bản Việt cũ."))}},
	} {
		if err := ValidateReview(b, u, reviewed{Reviewed: refs}); err == nil {
			t.Fatal("mismatched review accepted")
		}
	}
	if err := ValidateReview(b, u, reviewed{Reviewed: []reviewReference{valid}}); err != nil {
		t.Fatal(err)
	}
}

func TestReviewPreservesBlockingIssues(t *testing.T) {
	b := domain.Block{ID: "v1:weather", Source: "雨が降る。"}
	b.SourceHash = domain.Hash([]byte(b.Source))
	u := domain.Unit{ID: b.ID, SourceHash: b.SourceHash, Target: "Trời mưa."}
	for _, severity := range []string{"blocking", "warning"} {
		issue := domain.Issue{ID: "weather-word", BlockID: b.ID, Type: "glossary", Severity: severity, SourceQuote: "雨", TargetQuote: "mưa", Explanation: "Đối chiếu thuật ngữ thời tiết."}
		m := &reviewScopeEcho{issues: []domain.Issue{issue}}
		got, err := (Runner{Model: m}).review(context.Background(), b, u, payload{}, "")
		if err != nil {
			t.Fatal(err)
		}
		if len(got.Issues) != 1 || got.Issues[0] != issue {
			t.Fatal("review issue dropped or changed")
		}
		if severity == "blocking" && got.Decision != "unresolved" {
			t.Fatal("blocking issue was accepted")
		}
		if severity == "warning" && got.Decision != "accepted" {
			t.Fatal("warning unexpectedly blocked acceptance")
		}
	}
}

func TestReviewInvalidSeverityFeedbackListsAcceptedValues(t *testing.T) {
	b := domain.Block{ID: "v1:weather", Source: "雨が降る。"}
	u := domain.Unit{ID: b.ID, Target: "Trời mưa."}
	result := reviewed{Reviewed: []reviewReference{{ID: b.ID, TargetHash: domain.Hash([]byte(u.Target))}}, Issues: []domain.Issue{{ID: "issue", BlockID: b.ID, Type: "glossary", Severity: "major", Explanation: "Đối chiếu thuật ngữ."}}}
	err := ValidateReview(b, u, result)
	if err == nil || !strings.Contains(err.Error(), "blocking") || !strings.Contains(err.Error(), "warning") {
		t.Fatal("correction feedback does not explain permitted severities", err)
	}
}

func TestReviewResumeKeepsTranslationSavedBeforeFailure(t *testing.T) {
	r, book := fixtureRunner(t, &scripted{FailAt: 2})
	if err := r.Run(context.Background(), false, ""); !errors.Is(err, context.Canceled) {
		t.Fatal(err)
	}
	g, err := r.Store.Load(r.Volume.ID)
	if err != nil || len(g.Units) != 1 {
		t.Fatal("translation not saved before review failure", err)
	}
	var original domain.Unit
	for _, u := range g.Units {
		original = u
	}
	if original.Review != nil {
		t.Fatal("failed review was accepted")
	}
	r.Model = &reviewScopeEcho{}
	if err := r.Run(context.Background(), true, ""); err != nil {
		t.Fatal(err)
	}
	g, err = r.Store.Load(r.Volume.ID)
	if err != nil {
		t.Fatal(err)
	}
	got := g.Units[original.ID]
	if got.Target != original.Target || got.Revision != original.Revision || !got.Accepted() {
		t.Fatal("resume replaced translation or lost review", got)
	}
	if g.SourceHash != book.SourceHash {
		t.Fatal("resume changed source")
	}
}

func TestChildReviewReferencesUseChildIDsAndTargets(t *testing.T) {
	m := &reviewScopeEcho{}
	r := Runner{Model: m, Volume: workspace.Volume{ID: "v1"}, Store: state.Store{Root: t.TempDir()}}
	b := domain.Block{ID: "v1:parent", Source: "雨だ。帰ろう。"}
	b.SourceHash = domain.Hash([]byte(b.Source))
	u := domain.Unit{ID: b.ID, SourceHash: b.SourceHash, Target: "Mưa rồi.Về thôi.", Children: []domain.Child{
		{ID: "v1:parent-child-0001", Start: 0, End: len("雨だ。"), Source: "雨だ。", Target: "Mưa rồi."},
		{ID: "v1:parent-child-0002", Start: len("雨だ。"), End: len(b.Source), Source: "帰ろう。", Target: "Về thôi."},
	}}
	got, err := r.review(context.Background(), b, u, payload{}, "")
	if err != nil {
		t.Fatal(err)
	}
	if got.Decision != "accepted" || len(m.scopes) != 2 {
		t.Fatal("child review failed")
	}
	for i, child := range u.Children {
		if m.scopes[i].ID != child.ID || m.scopes[i].TargetHash != domain.Hash([]byte(child.Target)) {
			t.Fatal("child review copied parent reference")
		}
	}
}

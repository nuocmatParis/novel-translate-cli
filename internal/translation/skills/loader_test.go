package skills

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func TestLegacyImportScopesNeverLeakVol27PairsIntoVol1(t *testing.T) {
	c, e := workspace.Create(t.TempDir(), "test", "Test")
	if e != nil {
		t.Fatal(e)
	}
	if e = InstallDefaults(c); e != nil {
		t.Fatal(e)
	}
	p := filepath.Join(t.TempDir(), "legacy.md")
	text := "# Rules\n\n## Vol. 26–27\n\n| スバル → レム | anh-em |\n\n## Alias\n\nVol. 28: プリスカ = Prisca.\n\nUnknown future material."
	if e = os.WriteFile(p, []byte(text), 0600); e != nil {
		t.Fatal(e)
	}
	if _, e = Import(c, p); e != nil {
		t.Fatal(e)
	}
	s, e := Load(c)
	if e != nil {
		t.Fatal(e)
	}
	first, _, e := s.Context(domain.Position{Volume: 1}, "スバル レム プリスカ", 16000)
	if e != nil {
		t.Fatal(e)
	}
	if strings.Contains(first, "anh-em") || strings.Contains(first, "Prisca") {
		t.Fatal("future reference leaked")
	}
	current, used, e := s.Context(domain.Position{Volume: 27}, "スバル レム プリスカ", 16000)
	if e != nil {
		t.Fatal(e)
	}
	if !strings.Contains(current, "anh-em") || strings.Contains(current, "Prisca") || len(used) <= 4 {
		t.Fatal("scoped reference missing", current, used)
	}
	raw, _ := os.ReadFile(p)
	if string(raw) != text {
		t.Fatal("original modified")
	}
}

func TestScopesAndConflict(t *testing.T) {
	s := Snapshot{Rules: []domain.Rule{{ID: "common", Key: "suffix", Value: "remove", Priority: 1, Status: "confirmed"}, {ID: "series", Key: "suffix", Value: "preserve", Priority: 100, Status: "confirmed"}, {ID: "future", Key: "pair", Value: "anh-em", Priority: 100, Status: "confirmed", From: domain.Position{Volume: 26}}}}
	rules, conflict := s.Resolve(domain.Position{Volume: 1})
	if len(conflict) != 0 || len(rules) != 1 || rules[0].Value != "preserve" {
		t.Fatal(rules, conflict)
	}
	s.Rules = append(s.Rules, domain.Rule{ID: "conflict", Key: "suffix", Value: "other", Priority: 100, Status: "confirmed"})
	if _, conflict = s.Resolve(domain.Position{Volume: 1}); len(conflict) != 1 {
		t.Fatal("confirmed conflict not reported")
	}
}
func TestDefaultsAndReferenceBudget(t *testing.T) {
	c, e := workspace.Create(t.TempDir(), "test", "Test")
	if e != nil {
		t.Fatal(e)
	}
	if e = InstallDefaults(c); e != nil {
		t.Fatal(e)
	}
	s, e := Load(c)
	if e != nil {
		t.Fatal(e)
	}
	text, used, e := s.Context(domain.Position{Volume: 1}, "名前", 16000)
	if e != nil || len(used) != 4 || !strings.Contains(text, "Never omit") {
		t.Fatal(text, used, e)
	}
	s.Skills = append(s.Skills, Skill{ID: "big:reference", Body: strings.Repeat("unrelated entry\n\n", 100000)})
	text, _, e = s.Context(domain.Position{Volume: 1}, "名前", 16000)
	if e != nil || len(text) > 16000 {
		t.Fatal("unrelated huge reference overflow", e)
	}
}

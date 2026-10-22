package verify

import (
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"testing"
)

func TestHonorificNumberAndGlossaryAreReviewSignals(t *testing.T) {
	b := domain.Block{ID: "b", Source: "レムさんは１２人を見た。"}
	u := domain.Unit{Target: "Rem đã nhìn thấy mười hai người."}
	rules := []domain.Rule{{ID: "name-rem", Kind: "name", Key: "レム", Value: "Rem", Status: "confirmed"}}
	issues := Signals(b, u, rules)
	if len(issues) != 2 {
		t.Fatal(issues)
	}
	for _, issue := range issues {
		if issue.Severity != "warning" {
			t.Fatal("signal silently became semantic verdict", issue)
		}
	}
	u.Target = "Rem-san đã nhìn thấy 12 người."
	if issues = Signals(b, u, rules); len(issues) != 0 {
		t.Fatal(issues)
	}
}

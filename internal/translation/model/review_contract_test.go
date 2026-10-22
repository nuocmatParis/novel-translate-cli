package model

import (
	"encoding/json"
	"testing"

	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

func TestReviewContractBindsIDAndCurrentTargetHash(t *testing.T) {
	const id = "v1:block"
	hash := domain.Hash([]byte("Bản Việt hiện tại.\n"))
	c := ReviewContractFor(id, hash)
	if err := llmcontract.ValidateStrictReady(c.Schema); err != nil {
		t.Fatal(err)
	}
	for _, tc := range []struct {
		id, hash string
		valid    bool
	}{
		{id, hash, true},
		{"another-block", hash, false},
		{id, domain.Hash([]byte("Bản Việt cũ.")), false},
		{id, "", false},
	} {
		raw, err := json.Marshal(map[string]any{"reviewed": []map[string]string{{"id": tc.id, "target_hash": tc.hash}}, "issues": []any{}, "ruby_decisions": []any{}})
		if err != nil {
			t.Fatal(err)
		}
		if err := llmcontract.ValidateJSON(c.Schema, raw); (err == nil) != tc.valid {
			t.Fatalf("review identity validation: %v", err)
		}
	}
	other := ReviewContractFor("another-block", "another-hash")
	if other.Fingerprint() == c.Fingerprint() {
		t.Fatal("review contract did not bind each revision")
	}
	if ReviewContract().Fingerprint() == c.Fingerprint() {
		t.Fatal("scoped contract mutated generic contract")
	}
}

func TestReviewContractConstrainedLabels(t *testing.T) {
	const id = "v1:block"
	hash := domain.Hash([]byte("Bản Việt."))
	for _, contract := range []llmcontract.Contract{ReviewContract(), ReviewContractFor(id, hash)} {
		response := func(issueType, severity any) map[string]any {
			return map[string]any{
				"reviewed":       []map[string]string{{"id": id, "target_hash": hash}},
				"issues":         []map[string]any{{"id": "issue-1", "type": issueType, "severity": severity, "block_id": id, "source_quote": "雨", "target_quote": "mưa", "explanation": "Đối chiếu từ vựng."}},
				"ruby_decisions": []any{},
			}
		}
		check := func(v any, valid bool) {
			t.Helper()
			data, err := json.Marshal(v)
			if err != nil {
				t.Fatal(err)
			}
			if err := llmcontract.ValidateJSON(contract.Schema, data); (err == nil) != valid {
				t.Fatalf("invalid review labels accepted or valid labels rejected: %v", err)
			}
		}
		for _, severity := range []string{"blocking", "warning"} {
			check(response("mistranslation", severity), true)
		}
		for _, severity := range []any{"error", "critical", "major", "minor", "high", "info", "none", "BLOCKING", "", nil} {
			check(response("mistranslation", severity), false)
		}
		for _, kind := range []string{"omission", "addition", "mistranslation", "speaker/pronoun", "glossary", "honorific", "ruby", "number/name", "unresolved-source"} {
			check(response(kind, "blocking"), true)
		}
		for _, kind := range []any{"translation", "pronoun", "style", "", nil} {
			check(response(kind, "blocking"), false)
		}
		for _, decision := range []string{"phonetic", "semantic", "unresolved", "literal", "reading", ""} {
			v := response("ruby", "blocking")
			v["ruby_decisions"] = []map[string]any{{"index": 0, "base": "雨", "reading": "あめ", "decision": decision, "target_quote": "mưa", "reason": "Đối chiếu chú âm."}}
			check(v, decision == "phonetic" || decision == "semantic" || decision == "unresolved")
		}
	}
}

func TestScopedReviewIssuesCannotNameAnotherBlock(t *testing.T) {
	contract := ReviewContractFor("v1:current", "current-hash")
	data, err := json.Marshal(map[string]any{
		"reviewed":       []map[string]string{{"id": "v1:current", "target_hash": "current-hash"}},
		"issues":         []map[string]string{{"id": "i", "type": "omission", "severity": "blocking", "block_id": "v1:other", "source_quote": "雨", "target_quote": "", "explanation": "Thiếu mệnh đề."}},
		"ruby_decisions": []any{},
	})
	if err != nil {
		t.Fatal(err)
	}
	if err := llmcontract.ValidateJSON(contract.Schema, data); err == nil {
		t.Fatal("issue for another block accepted")
	}
}

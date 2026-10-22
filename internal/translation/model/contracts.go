package model

import (
	"sort"

	"github.com/voocel/ainovel-cli/internal/llmcontract"
)

func str() map[string]any { return map[string]any{"type": "string"} }
func enumStr(values ...string) map[string]any {
	return map[string]any{"type": "string", "enum": values}
}
func object(props map[string]any) map[string]any {
	keys := []string{}
	for k := range props {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return map[string]any{"type": "object", "properties": props, "required": keys, "additionalProperties": false}
}
func array(items map[string]any) map[string]any {
	return map[string]any{"type": "array", "items": items}
}
func TranslationContract() llmcontract.Contract {
	return llmcontract.Contract{Name: "jp_vi_translation", Description: "Vietnamese translation of owned source units", Schema: object(map[string]any{"units": array(object(map[string]any{"id": str(), "target": str()})), "uncertainties": array(object(map[string]any{"block_id": str(), "quote": str(), "explanation": str()})), "continuity_candidates": array(object(map[string]any{"block_id": str(), "quote": str(), "speaker": str(), "listener": str(), "value": str()})), "rule_candidates": array(object(map[string]any{"block_id": str(), "quote": str(), "key": str(), "value": str(), "kind": str()}))})}
}
func ReviewContract() llmcontract.Contract {
	return reviewContract(str(), str())
}

// ReviewContractFor constrains identity fields in both native structured output
// and prompt schemas. Semantic validation still checks count and all evidence.
func ReviewContractFor(id, targetHash string) llmcontract.Contract {
	return reviewContract(enumStr(id), enumStr(targetHash))
}

func reviewContract(id, targetHash map[string]any) llmcontract.Contract {
	severity := enumStr("blocking", "warning")
	severity["description"] = "blocking: must be resolved before export; warning: non-blocking observation. Never downgrade an error to permit export. If there are no issues, return issues: []."
	return llmcontract.Contract{
		Name:        "jp_vi_review",
		Description: "Clause-level source and target comparison",
		Schema: object(map[string]any{
			"reviewed": array(object(map[string]any{"id": id, "target_hash": targetHash})),
			"issues": array(object(map[string]any{
				"id":           str(),
				"type":         enumStr("omission", "addition", "mistranslation", "speaker/pronoun", "glossary", "honorific", "ruby", "number/name", "unresolved-source"),
				"severity":     severity,
				"block_id":     id,
				"source_quote": map[string]any{"type": "string", "description": "Verbatim substring of the owned Japanese source; required except for addition. Never paraphrase a quote."},
				"target_quote": map[string]any{"type": "string", "description": "Verbatim substring of the current Vietnamese target when present; required for addition. Never paraphrase a quote."},
				"explanation":  str(),
			})),
			"ruby_decisions": array(object(map[string]any{
				"index": map[string]any{"type": "integer", "description": "Zero-based index in the owned unit's ruby array."},
				"base":  str(), "reading": str(),
				"decision":     enumStr("phonetic", "semantic", "unresolved"),
				"target_quote": str(), "reason": str(),
			})),
		}),
	}
}

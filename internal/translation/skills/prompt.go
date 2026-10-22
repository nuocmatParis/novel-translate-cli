package skills

import (
	"encoding/json"
	"fmt"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"strings"
	"unicode"
)

type ReferenceData struct {
	Path    string `json:"path"`
	Hash    string `json:"hash"`
	Section int    `json:"section"`
	Text    string `json:"text"`
}

func referenceSections(body string) []string {
	var out []string
	for _, para := range strings.Split(body, "\n\n") {
		if strings.Contains(para, "|") {
			out = append(out, strings.Split(para, "\n")...)
		} else {
			out = append(out, para)
		}
	}
	return out
}
func (s Snapshot) Prompt(p domain.Position, source string, limit int) (string, []ReferenceData, []string, error) {
	var core strings.Builder
	var refs []ReferenceData
	var used []string
	for _, sk := range s.Skills {
		if strings.Contains(sk.ID, ":") || !Applies(sk.From, sk.Until, p) {
			continue
		}
		if core.Len()+len(sk.Body)+2 > limit {
			return "", nil, nil, fmt.Errorf("skill core vượt budget: %s", sk.ID)
		}
		if sk.Priority >= 100 {
			const precedence = "\nSeries-specific instructions follow. They override conflicting shared defaults within this series; retain source fidelity.\n"
			if core.Len()+len(precedence)+len(sk.Body)+2 > limit {
				return "", nil, nil, fmt.Errorf("skill core vượt budget: %s", sk.ID)
			}
			core.WriteString(precedence)
		}
		core.WriteString("\n" + sk.Body + "\n")
		used = append(used, sk.Path)
	}
	for _, sk := range s.Skills {
		if !strings.Contains(sk.ID, ":") || !Applies(sk.From, sk.Until, p) {
			continue
		}
		for i, section := range referenceSections(sk.Body) {
			if !referenceMatches(section, source) {
				continue
			}
			item := ReferenceData{Path: sk.Path, Hash: sk.Hash, Section: i + 1, Text: section}
			candidate := append(append([]ReferenceData(nil), refs...), item)
			raw, _ := json.Marshal(candidate)
			if core.Len()+len(raw) > limit {
				continue
			}
			refs = candidate
			used = append(used, fmt.Sprintf("%s#section-%d", sk.Path, i+1))
		}
	}
	return core.String(), refs, used, nil
}
func isSingleKanji(term string) bool {
	r := []rune(term)
	return len(r) == 1 && unicode.In(r[0], unicode.Han)
}

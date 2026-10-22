package continuity

import (
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"strings"
)

// Scoped name rules supply explicit aliases. IDs depend on the Japanese key,
// so a Vietnamese spelling revision does not silently create a new character.
func Entities(rules []domain.Rule) []domain.Entity {
	var out []domain.Entity
	for _, r := range rules {
		if r.Kind != "name" || r.Status != "confirmed" || r.Key == "" || r.Value == "" || len(r.Evidence) == 0 {
			continue
		}
		out = append(out, domain.Entity{ID: "entity:" + domain.Hash([]byte(r.Key))[:16], Aliases: []string{r.Key, r.Value}, Evidence: r.Evidence, RevealedAt: r.From, RuleID: r.ID})
	}
	return out
}
func CanonicalActor(entities []domain.Entity, actor string) string {
	for _, e := range entities {
		if actor == e.ID {
			return actor
		}
	}
	matched := ""
	for _, e := range entities {
		for _, alias := range e.Aliases {
			if strings.EqualFold(alias, actor) {
				if matched != "" && matched != e.ID {
					return actor
				}
				matched = e.ID
			}
		}
	}
	if matched != "" {
		return matched
	}
	return actor // Anonymous source-grounded IDs remain distinct; do not guess aliases.
}

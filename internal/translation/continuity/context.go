package continuity

import (
	"sort"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

// Context derives publication-safe history only from currently accepted revisions.
// A flashback gets an explicit story-time boundary; uncertain time is reported
// instead of substituting the most recent series state.
func Context(generations []domain.Generation, p domain.Position, storyTime *domain.Position) []domain.Event {
	var candidates []domain.Event
	for _, g := range generations {
		for _, u := range g.Units {
			if !u.Accepted() {
				continue
			}
			for _, e := range u.Events {
				if e.RevisionHash != domain.Hash([]byte(u.Target)) || !e.Position.Before(p) {
					continue
				}
				if storyTime != nil {
					if e.StoryTime == nil || storyTime.Before(*e.StoryTime) {
						continue
					}
				}
				if e.Uncertainty != "" {
					continue
				}
				candidates = append(candidates, e)
			}
		}
	}
	sort.Slice(candidates, func(i, j int) bool {
		a, b := candidates[i], candidates[j]
		if a.Position == b.Position {
			if a.SourceOffset != b.SourceOffset {
				return a.SourceOffset < b.SourceOffset
			}
			return a.ID < b.ID
		}
		return a.Position.Before(b.Position)
	})
	latest := map[string]domain.Event{}
	for _, e := range candidates {
		latest[e.Speaker+"→"+e.Listener] = e
	}
	var out []domain.Event
	for _, e := range latest {
		out = append(out, e)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].ID < out[j].ID })
	return out
}

// Events live inside immutable generations, so translation/review/context publish
// share a transaction and replay cannot duplicate an event.
func ValidateEvent(e domain.Event, b domain.Block) bool {
	return e.Speaker != "" && e.Listener != "" && e.Value != "" && e.Evidence.BlockID == b.ID && e.Evidence.Quote != "" && contains(b.Source, e.Evidence.Quote)
}
func contains(s, sub string) bool {
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			return true
		}
	}
	return false
}

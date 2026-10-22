package continuity

import (
	"testing"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

func accepted(id string, volume, chapter int, speaker, listener, value string) domain.Unit {
	u := domain.Unit{ID: id, Target: id, SourceHash: id}
	h := domain.Hash([]byte(id))
	u.Review = &domain.Review{SourceHash: id, TargetHash: h, Decision: "accepted"}
	u.Events = []domain.Event{{ID: id, Position: domain.Position{Volume: volume, Chapter: chapter}, Speaker: speaker, Listener: listener, Value: value, RevisionHash: h}}
	return u
}
func TestDirectedPairsAndNoFuture(t *testing.T) {
	g := domain.Generation{Units: map[string]domain.Unit{"v1": accepted("v1", 1, 2, "A", "B", "anh-em"), "reverse": accepted("reverse", 1, 2, "B", "A", "tôi-anh"), "future": accepted("future", 27, 1, "A", "B", "future-secret")}}
	events := Context([]domain.Generation{g}, domain.Position{Volume: 2}, nil)
	if len(events) != 2 {
		t.Fatal(events)
	}
	for _, e := range events {
		if e.Value == "future-secret" {
			t.Fatal("future leaked")
		}
	}
	if len(Context([]domain.Generation{g}, domain.Position{Volume: 1, Chapter: 1}, nil)) != 0 {
		t.Fatal("later chapter leaked")
	}
	u := g.Units["v1"]
	u.Target = "manual"
	g.Units["v1"] = u
	if len(Context([]domain.Generation{g}, domain.Position{Volume: 2}, nil)) != 1 {
		t.Fatal("stale review published")
	}
}

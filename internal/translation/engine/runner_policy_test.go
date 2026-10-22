package engine

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/voocel/ainovel-cli/internal/translation/progress"
)

type recordedObserver struct {
	events []progress.Event
}

func (r *recordedObserver) OnEvent(e progress.Event) {
	r.events = append(r.events, e)
}

func TestRunnerReviewOnlyDoesNotRepair(t *testing.T) {
	s := &scripted{Omission: true}
	r, book := fixtureRunner(t, s)
	obs := &recordedObserver{}
	r.Observer = obs

	// Run initially in review-only mode on un-translated book
	// ReviewOnly with no units will do nothing
	err := r.RunWithOptions(context.Background(), RunOptions{
		Policy: ReviewPolicy{ReviewOnly: true, AutoRepair: false},
	})
	if err != nil {
		t.Fatal(err)
	}

	// Now run normal translation so units exist
	s.Omission = false
	if err := r.Run(context.Background(), false, ""); err != nil {
		t.Fatal(err)
	}

	g, err := r.Store.Load(r.Volume.ID)
	if err != nil {
		t.Fatal(err)
	}
	var targetID string
	for _, b := range book.Blocks {
		if b.Disposition == "translate" && strings.Contains(b.Source, "雨が") {
			targetID = b.ID
			break
		}
	}
	if targetID == "" {
		t.Fatal("could not find block with 雨が")
	}
	if !g.Units[targetID].Accepted() {
		t.Fatal("expected unit to be accepted")
	}

	// Now force a re-review with omission issue in ReviewOnly mode
	s.Omission = true
	obs.events = nil
	err = r.RunWithOptions(context.Background(), RunOptions{
		Policy: ReviewPolicy{
			ReviewOnly: true,
			Force:      true,
			AutoRepair: false,
		},
		Scope: RunScope{UnitIDs: []string{targetID}},
	})
	if err != nil {
		t.Fatal("review only should complete without repair error, got:", err)
	}

	// Verify unit is no longer accepted due to blocking omission issue, but was NOT repaired (Revision stays 1)
	g, err = r.Store.Load(r.Volume.ID)
	if err != nil {
		t.Fatal(err)
	}
	unit := g.Units[targetID]
	if unit.Accepted() {
		t.Fatal("unit should NOT be accepted after blocking issue")
	}
	if unit.Revision != 1 {
		t.Fatalf("unit revision should remain 1, got %d", unit.Revision)
	}

	// Check that events were emitted
	hasReviewCommitted := false
	for _, ev := range obs.events {
		if ev.Kind == progress.EventReviewCommitted {
			hasReviewCommitted = true
		}
	}
	if !hasReviewCommitted {
		t.Fatal("expected EventReviewCommitted event")
	}
}

func TestRunnerPauseBoundary(t *testing.T) {
	s := &scripted{}
	r, _ := fixtureRunner(t, s)
	paused := false
	pauseNotified := false

	obs := &recordedObserver{}
	r.Observer = obs

	pauseRequested := true
	err := r.RunWithOptions(context.Background(), RunOptions{
		IsPaused: func() bool {
			return pauseRequested
		},
		PauseNotify: func() {
			pauseNotified = true
		},
	})
	if !errors.Is(err, ErrPaused) {
		t.Fatalf("expected ErrPaused, got %v", err)
	}
	if !pauseNotified {
		t.Fatal("expected PauseNotify callback")
	}
	_ = paused
}

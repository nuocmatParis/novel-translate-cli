package engine

import (
	"context"
	"encoding/json"
	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
	"os"
	"strings"
	"testing"
)

type historicalModel struct {
	base scripted
	seen map[string]payload
}

func (m *historicalModel) Identity(role string) string { return m.base.Identity(role) }
func (m *historicalModel) InputLimit(role string) int  { return m.base.InputLimit(role) }
func (m *historicalModel) Call(ctx context.Context, role, system string, in any, c llmcontract.Contract, out any, validate func() error) error {
	raw, _ := json.Marshal(in)
	var p payload
	json.Unmarshal(raw, &p)
	if m.seen == nil {
		m.seen = map[string]payload{}
	}
	m.seen[p.Units[0].ID] = p
	if e := m.base.Call(ctx, role, system, in, c, out, func() error { return nil }); e != nil {
		return e
	}
	if role == "translation" && p.Units[0].Position.Volume == 1 {
		result := out.(*translated)
		b := p.Units[0]
		if b.Source == "彼は来た。" {
			result.Continuity = []candidateEvent{{BlockID: b.ID, Quote: "彼は来た", Speaker: "a", Listener: "b", Value: "early"}, {BlockID: b.ID, Quote: "彼は来た", Speaker: "b", Listener: "a", Value: "reverse"}}
		}
		if strings.Contains(b.Source, "雨が") {
			result.Continuity = []candidateEvent{{BlockID: b.ID, Quote: "雨が", Speaker: "a", Listener: "b", Value: "later"}}
		}
	}
	return validate()
}

func TestCrossVolumeContextFreshnessAndFlashback(t *testing.T) {
	r, book := fixtureRunner(t, &scripted{})
	m := &historicalModel{}
	r.Model = m
	if e := r.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	v2 := workspace.Volume{ID: "v2", Order: 2, Previous: r.Volume.ID, Directory: "Tập 2", Source: r.Volume.Source}
	if e := r.Config.Add(v2); e != nil {
		t.Fatal(e)
	}
	book2, e := Import(r.Config, v2)
	if e != nil {
		t.Fatal(e)
	}
	r2 := r
	r2.Volume = v2
	if e = r2.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	first := book2.Blocks[0]
	g2, _ := r.Store.Load(v2.ID)
	events, e := ContextEvents(r.Config, v2, r.Store, g2, first.Position, nil)
	if e != nil {
		t.Fatal(e)
	}
	values := map[string]string{}
	for _, ev := range events {
		values[ev.Speaker+"→"+ev.Listener] = ev.Value
	}
	if values["a→b"] != "later" || values["b→a"] != "reverse" {
		t.Fatalf("directed previous state lost: %v", values)
	}
	var early domain.Position
	for _, b := range book.Blocks {
		if b.Source == "彼は来た。" {
			early = b.Position
		}
	}
	if e = SetScene(r.Config, v2, book2, first.ID, first.ID, &early, first.Source, "source establishes a flashback", false); e != nil {
		t.Fatal(e)
	}
	events, e = ContextEvents(r.Config, v2, r.Store, g2, first.Position, nil)
	if e != nil {
		t.Fatal(e)
	}
	for _, ev := range events {
		if ev.Speaker == "a" && ev.Value != "early" {
			t.Fatal("flashback used later relation", ev)
		}
	}
	if len(events) != 2 {
		t.Fatal("flashback lost earlier directed facts", events)
	}
	if e = SetScene(r.Config, v2, book2, first.ID, first.ID, nil, first.Source, "time remains uncertain", true); e != nil {
		t.Fatal(e)
	}
	events, e = ContextEvents(r.Config, v2, r.Store, g2, first.Position, nil)
	if e != nil || len(events) != 0 {
		t.Fatal("uncertain scene guessed latest relation", events, e)
	}
	// Register a future volume with a forged accepted event. Vol1 never walks it.
	v27 := workspace.Volume{ID: "v27", Order: 27, Previous: v2.ID, Directory: "Tập 27", Source: r.Volume.Source}
	if e = r.Config.Add(v27); e != nil {
		t.Fatal(e)
	}
	future := domain.Unit{ID: "future", SourceHash: "x", Target: "future", Review: &domain.Review{SourceHash: "x", TargetHash: domain.Hash([]byte("future")), Decision: "accepted"}, Events: []domain.Event{{ID: "spoiler", Speaker: "a", Listener: "b", Value: "spoiler", Position: domain.Position{Volume: 27}, RevisionHash: domain.Hash([]byte("future"))}}}
	fg := domain.Generation{Units: map[string]domain.Unit{future.ID: future}}
	if e = r.Store.Commit(v27.ID, &fg, ""); e != nil {
		t.Fatal(e)
	}
	g1, _ := r.Store.Load(r.Volume.ID)
	events, e = ContextEvents(r.Config, r.Volume, r.Store, g1, domain.Position{Volume: 1, Chapter: 999}, nil)
	if e != nil {
		t.Fatal(e)
	}
	for _, ev := range events {
		if ev.Value == "spoiler" {
			t.Fatal("future state leaked")
		}
	}
	// A convention change makes earlier accepted facts stale, even before sync.
	path := r.Config.ReadPath(r.Config.SeriesSkill)
	raw, _ := os.ReadFile(path)
	os.WriteFile(path, append(raw, []byte("\nNew source-fidelity convention.\n")...), 0600)
	events, e = ContextEvents(r.Config, v2, r.Store, g2, domain.Position{Volume: 2, Chapter: 999}, nil)
	if e != nil || len(events) != 0 {
		t.Fatal("stale facts inherited", events, e)
	}
	snapshot, e := skills.Load(r.Config)
	if e != nil {
		t.Fatal(e)
	}
	plan, e := ImpactPlan(r.Config, v2, r.Store, book2, g2, snapshot)
	if e != nil || len(plan.Units) == 0 {
		t.Fatal("previous/skill change was not planned", plan, e)
	}
	var scenes []Scene
	if e = state.ReadJSON(r.Store.Path("volumes", v2.ID, "scenes.json"), &scenes); e != nil {
		t.Fatal(e)
	}
}

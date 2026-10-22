package engine

import (
	"context"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"os"
	"strings"
	"testing"
)

func TestSourceRevisionArchivesManualAndRecoversSourcePointer(t *testing.T) {
	r, old := fixtureRunner(t, &scripted{})
	if e := r.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	g, e := r.Store.Load("v1")
	if e != nil {
		t.Fatal(e)
	}
	var changedID string
	for _, b := range old.Blocks {
		if strings.Contains(b.Source, "彼は来た") {
			changedID = b.ID
		}
	}
	u := g.Units[changedID]
	u.Origin = "manual"
	u.Target = "Bản sửa tay."
	u.Review = nil
	g.Units[u.ID] = u
	if e = revision.CaptureViews(r.Config, r.Volume, old, &g); e != nil {
		t.Fatal(e)
	}
	if e = r.Store.Commit("v1", &g, g.ID); e != nil {
		t.Fatal(e)
	}
	if e = revision.Materialize(r.Config, r.Volume, old, g); e != nil {
		t.Fatal(e)
	}
	f := testutil.Files()
	f["OPS/a.xhtml"] = strings.Replace(f["OPS/a.xhtml"], "彼は来た。⟦", "彼は帰った。", 1)
	if e = os.WriteFile(r.Config.ReadPath(r.Volume.Source), testutil.ZIP(f), 0600); e != nil {
		t.Fatal(e)
	}
	if _, _, e = Source(r.Config, r.Volume, true); e == nil {
		t.Fatal("changed source ignored")
	}
	plan, e := PlanSourceRevision(r.Config, r.Volume)
	if e != nil {
		t.Fatal(e)
	}
	if e = ApplySourceRevision(r.Config, r.Volume, plan); e != nil {
		t.Fatal(e)
	}
	g, e = r.Store.Load("v1")
	if e != nil {
		t.Fatal(e)
	}
	found := false
	for _, u := range g.Units {
		if u.Origin == "manual" && u.Target == "Bản sửa tay." {
			found = true
			if u.Accepted() {
				t.Fatal("source-changed manual accepted")
			}
		}
	}
	if !found {
		t.Fatal("manual lost")
	}
	// Crash replay: generation has the new source; convenience pointer remains old.
	if e = state.WriteJSON(r.Store.Path("volumes", "v1", "source.json"), SourcePointer{Hash: plan.OldHash, ManifestHash: domain.Hash([]byte("old"))}); e != nil {
		t.Fatal(e)
	}
	_, book, e := Source(r.Config, r.Volume, true)
	if e != nil || book.SourceHash != plan.NewHash {
		t.Fatal("source transaction replay", e)
	}
	if e = r.Run(context.Background(), false, ""); e != nil {
		t.Fatal(e)
	}
	g, _ = r.Store.Load("v1")
	for _, u := range g.Units {
		if u.Origin == "manual" && u.Target != "Bản sửa tay." {
			t.Fatal("manual overwritten on resume")
		}
	}
}

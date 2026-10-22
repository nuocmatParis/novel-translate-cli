package model

import (
	"context"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
	"os"
	"testing"
)

// Opt-in: can make paid requests. Never enabled by the default CI suite.
func TestLiveConfiguredProviderSmoke(t *testing.T) {
	path := os.Getenv("NOVEL_TRANSLATE_LIVE_PROVIDER")
	if path == "" {
		t.Skip("live acceptance pending — no explicitly supplied provider configuration")
	}
	c := workspace.Defaults()
	c.Root = t.TempDir()
	c.ProviderConfig = path
	c.Run.MaxRequests = 6
	a, e := New(c)
	if e != nil {
		t.Fatal(e)
	}
	source := domain.Block{ID: "smoke:1", Source: "雨が降っていたが、彼は走った。", Disposition: "translate"}
	source.SourceHash = domain.Hash([]byte(source.Source))
	var out struct {
		Units []struct {
			ID     string `json:"id"`
			Target string `json:"target"`
		} `json:"units"`
	}
	payload := map[string]any{"owned_units": []domain.Block{source}, "rules": []any{}, "continuity": []any{}, "read_only_previous": []any{}}
	if e = a.Call(context.Background(), "translation", "Translate the supplied Japanese source into Vietnamese. Preserve both the rain clause and running action; return only the requested unit ID. No speculative candidates.", payload, TranslationContract(), &out, func() error {
		if len(out.Units) != 1 || out.Units[0].ID != source.ID || out.Units[0].Target == "" || out.Units[0].Target == source.Source {
			return os.ErrInvalid
		}
		return nil
	}); e != nil {
		t.Fatal(e)
	}
	usage, e := ReadUsage(a.Store)
	if e != nil || usage.Requests == 0 {
		t.Fatal(usage, e)
	}
	t.Logf("provider smoke %s; %d metered requests; semantic review/real-book acceptance still required", a.Identity("translation"), usage.Requests)
}

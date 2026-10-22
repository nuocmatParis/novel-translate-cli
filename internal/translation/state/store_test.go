package state

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

func TestGenerationAndOrphanRecovery(t *testing.T) {
	s := Store{Root: t.TempDir()}
	g, e := s.Load("v1")
	if e != nil {
		t.Fatal(e)
	}
	g.Units["u"] = domain.Unit{ID: "u", Target: "first"}
	if e = s.Commit("v1", &g, ""); e != nil {
		t.Fatal(e)
	}
	first := g.ID
	if e = Atomic(s.Path("volumes", "v1", "generations", "orphan.json"), []byte("partial")); e != nil {
		t.Fatal(e)
	}
	loaded, e := s.Load("v1")
	if e != nil || loaded.ID != first {
		t.Fatal("orphan changed pointer", e)
	}
	g.Units["u"] = domain.Unit{ID: "u", Target: "second"}
	if e = s.Commit("v1", &g, first); e != nil {
		t.Fatal(e)
	}
	if e = s.Commit("v1", &loaded, first); e == nil {
		t.Fatal("stale generation overwrite")
	}
	if e = os.WriteFile(s.Path("volumes", "v1", "generations", g.ID+".json"), []byte("corrupt"), 0600); e != nil {
		t.Fatal(e)
	}
	if _, e = s.Load("v1"); e == nil {
		t.Fatal("corrupt generation accepted")
	}
}
func TestLockAndAtomicReplace(t *testing.T) {
	s := Store{Root: t.TempDir()}
	release, e := s.Lock(context.Background())
	if e != nil {
		t.Fatal(e)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()
	if _, e = s.Lock(ctx); e == nil {
		t.Fatal("second writer acquired lock")
	}
	release()
	p := s.Path("test.json")
	for _, value := range []string{"first", "second"} {
		if e = Atomic(p, []byte(value)); e != nil {
			t.Fatal(e)
		}
	}
	b, e := os.ReadFile(p)
	if e != nil || string(b) != "second" {
		t.Fatal(string(b), e)
	}
}

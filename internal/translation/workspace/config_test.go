package workspace

import (
	"os"
	"path/filepath"
	"testing"
)

func TestWorkspacePathsAndPrevious(t *testing.T) {
	c, e := Create(filepath.Join(t.TempDir(), "bộ truyện có dấu"), "series", "Bộ truyện")
	if e != nil {
		t.Fatal(e)
	}
	if e = c.Add(Volume{ID: "v1", Order: 1, Directory: "Tập 1", Source: "Tập 1/source.epub"}); e != nil {
		t.Fatal(e)
	}
	if e = c.Add(Volume{ID: "v2", Order: 2, Previous: "v1", Directory: "Tập 2", Source: "Tập 2/source.epub"}); e != nil {
		t.Fatal(e)
	}
	loaded, e := Load(c.Root)
	if e != nil {
		t.Fatal(e)
	}
	if loaded.ReadPath("Tập 2/source.epub") != filepath.Join(c.Root, "Tập 2", "source.epub") {
		t.Fatal("cwd-relative paths")
	}
	if e = c.Add(Volume{ID: "v3", Order: 2, Directory: "Tập 3", Source: "source.epub"}); e == nil {
		t.Fatal("duplicate order accepted")
	}
	if _, e = c.WritePath("../escape"); e == nil {
		t.Fatal("write escape")
	}
	c.Volumes[0].Previous = "v2"
	if e = c.Validate(); e == nil {
		t.Fatal("cycle accepted")
	}
}
func TestStrictYAML(t *testing.T) {
	c, e := Create(t.TempDir(), "s", "s")
	if e != nil {
		t.Fatal(e)
	}
	p := filepath.Join(c.Root, "workspace.yaml")
	b, e := os.ReadFile(p)
	if e != nil {
		t.Fatal(e)
	}
	if e = os.WriteFile(p, append(b, []byte("unexpected: value\n")...), 0600); e != nil {
		t.Fatal(e)
	}
	if _, e = Load(c.Root); e == nil {
		t.Fatal("unknown YAML field accepted")
	}
}

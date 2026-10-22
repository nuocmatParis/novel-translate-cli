package epub

import (
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"strings"
	"testing"
)

func TestRangeIncludesNotesAndPrunesInDocumentChapter(t *testing.T) {
	raw := testutil.EPUB()
	p, e := Read(raw, "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	var chapter string
	for _, ch := range p.Book.Chapters {
		if ch.Title == "第一章" {
			chapter = ch.ID
		}
	}
	if chapter == "" {
		t.Fatal("fixture chapter missing")
	}
	scoped, e := Read(raw, "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	if e = scoped.SelectScope([]string{chapter}); e != nil {
		t.Fatal(e)
	}
	if _, ok := scoped.Files["OPS/z.xhtml"]; ok {
		t.Fatal("front narrative retained")
	}
	if _, ok := scoped.Files["OPS/notes.xhtml"]; !ok {
		t.Fatal("referenced note lost")
	}
	if strings.Contains(scoped.Trees["OPS/end.xhtml"].AllText(), "雨が") {
		t.Fatal("out of range paragraph retained")
	}
	if strings.Contains(scoped.Trees["OPS/nav.xhtml"].XML(), "#c2") {
		t.Fatal("omitted nav destination retained")
	}
	g := domain.Generation{Units: map[string]domain.Unit{}}
	for _, b := range scoped.Book.Blocks {
		if b.Disposition != "translate" {
			continue
		}
		u := domain.Unit{ID: b.ID, Target: b.Source, SourceHash: b.SourceHash}
		u.Review = &domain.Review{SourceHash: b.SourceHash, TargetHash: domain.Hash([]byte(u.Target)), Decision: "accepted"}
		g.Units[b.ID] = u
	}
	opts := ExportOptions{Chapters: []string{chapter}}
	out, e := Write(raw, "v1", 1, g, opts)
	if e != nil {
		t.Fatal(e)
	}
	r, e := VerifyOutput(raw, out, "v1", 1, g, opts)
	if e != nil {
		t.Fatal(r, e)
	}
	if !r.Structure || r.Accepted != r.Eligible {
		t.Fatal(r)
	}
}
func TestRangeRejectsDanglingNarrativeReference(t *testing.T) {
	f := testutil.Files()
	f["OPS/a.xhtml"] = testutil.XHTML(`<h1 id="c1">第一章</h1><p><a href="end.xhtml#c2">第二章へ</a></p>`)
	p, e := Read(testutil.ZIP(f), "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	var id string
	for _, ch := range p.Book.Chapters {
		if ch.Title == "第一章" {
			id = ch.ID
		}
	}
	if e = p.SelectScope([]string{id}); e == nil {
		t.Fatal("broken narrative reference exported")
	}
}

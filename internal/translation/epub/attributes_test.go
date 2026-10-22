package epub

import (
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"testing"
)

func TestNestedTextAttributesSurviveParentTranslation(t *testing.T) {
	f := testutil.Files()
	f["OPS/a.xhtml"] = testutil.XHTML(`<h1 id="c1">第一章</h1><p id="b1" title="外">前<a href="notes.xhtml#n1" title="内">注</a><img src="image.svg" alt="色"/>後</p>`)
	raw := testutil.ZIP(f)
	p, e := Read(raw, "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	if p.Book.TextAttributes != 3 || p.Book.CoveredTextAttributes != 3 {
		t.Fatal("nested attribute coverage", p.Book.TextAttributes, p.Book.CoveredTextAttributes)
	}
	g := domain.Generation{Units: map[string]domain.Unit{}}
	for _, b := range p.Book.Blocks {
		if b.Disposition != "translate" {
			continue
		}
		target := b.Source
		if b.Kind == "attribute" {
			target = "Thuộc tính Việt: " + b.Source
		}
		u := domain.Unit{ID: b.ID, SourceHash: b.SourceHash, Target: target}
		u.Review = &domain.Review{SourceHash: b.SourceHash, TargetHash: domain.Hash([]byte(target)), Decision: "accepted"}
		g.Units[b.ID] = u
	}
	out, e := Write(raw, "v1", 1, g, ExportOptions{Preview: true})
	if e != nil {
		t.Fatal(e)
	}
	report, e := VerifyOutput(raw, out, "v1", 1, g)
	if e != nil {
		t.Fatal(report.Diagnostics, e)
	}
}

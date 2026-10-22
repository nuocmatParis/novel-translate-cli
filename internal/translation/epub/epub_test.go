package epub

import (
	"strings"
	"testing"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
)

func TestInventoryAndRoundTrip(t *testing.T) {
	raw := testutil.EPUB()
	p, e := Read(raw, "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	if p.Book.Documents[0].Path != "OPS/z.xhtml" {
		t.Fatal("reader sorted filenames instead of spine")
	}
	if p.Book.TextNodes != p.Book.CoveredTextNodes {
		t.Fatal("text-node coverage")
	}
	images, rubies := 0, 0
	for _, b := range p.Book.Blocks {
		if b.Kind == "image" {
			images++
		}
		rubies += len(b.Ruby)
	}
	if images != 1 || rubies != 1 {
		t.Fatalf("images=%d ruby=%d", images, rubies)
	}
	var c1 domain.Chapter
	for _, ch := range p.Book.Chapters {
		if ch.Title == "第一章" {
			c1 = ch
		}
	}
	if len(c1.BlockIDs) < 4 {
		t.Fatal("logical chapter did not span image/continuation documents")
	}
	p2, e := Read(raw, "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	if domain.Digest(p.Book) != domain.Digest(p2.Book) {
		t.Fatal("unstable IDs/inventory")
	}
	g := domain.Generation{Units: map[string]domain.Unit{}}
	for _, b := range p.Book.Blocks {
		if b.Disposition == "translate" {
			u := domain.Unit{ID: b.ID, SourceHash: b.SourceHash, Target: b.Source, Revision: 1}
			u.Review = &domain.Review{SourceHash: b.SourceHash, TargetHash: domain.Hash([]byte(u.Target)), Decision: "accepted"}
			g.Units[b.ID] = u
		}
	}
	output, e := Write(raw, "v1", 1, g, ExportOptions{Preview: true})
	if e != nil {
		t.Fatal(e)
	}
	r, e := VerifyOutput(raw, output, "v1", 1, g)
	if e != nil {
		t.Fatalf("round trip: %v %+v", e, r.Diagnostics)
	}
	// Swapping one image href is detected despite unchanged resource count.
	out, e := Read(output, "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	out.Files["OPS/picture.xhtml"] = []byte(strings.ReplaceAll(string(out.Files["OPS/picture.xhtml"]), "image.svg", "nav.xhtml"))
	files := map[string]string{}
	for name, b := range out.Files {
		files[name] = string(b)
	}
	if _, e = VerifyOutput(raw, testutil.ZIP(files), "v1", 1, g); e == nil {
		t.Fatal("image occurrence change was accepted")
	}
	// All IDs, target strings and images are still present, but text blocks move.
	out, e = Read(output, "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	tree := out.Trees["OPS/a.xhtml"]
	paragraphs := tree.Find("p")
	if len(paragraphs) < 2 {
		t.Fatal("fixture requires two paragraphs")
	}
	body := tree.Find("body")[0]
	i, j := -1, -1
	for k, n := range body.Children {
		if n == paragraphs[0] {
			i = k
		}
		if n == paragraphs[1] {
			j = k
		}
	}
	body.Children[i], body.Children[j] = body.Children[j], body.Children[i]
	for name, b := range out.Files {
		files[name] = string(b)
	}
	files["OPS/a.xhtml"] = string(Serialize(tree))
	if report, e := VerifyOutput(raw, testutil.ZIP(files), "v1", 1, g); e == nil {
		t.Fatal("reordered text accepted", report)
	}
}
func TestBadZIPAndTokens(t *testing.T) {
	for _, name := range []string{"../bad", "/absolute", "OPS/../../bad"} {
		f := testutil.Files()
		f[name] = "bad"
		if _, e := Read(testutil.ZIP(f), "v1", 1); e == nil {
			t.Fatalf("unsafe ZIP accepted %s", name)
		}
	}
	p, e := Read(testutil.EPUB(), "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	for _, b := range p.Book.Blocks {
		if len(b.Tokens) > 1 {
			if _, e = TokensHTML(b, strings.Replace(b.Source, "⟦t0⟧", "", 1)); e == nil {
				t.Fatal("missing token accepted")
			}
			if _, e = TokensHTML(b, b.Source+"⟦unknown⟧"); e == nil {
				t.Fatal("unknown token accepted")
			}
			break
		}
	}
}
func TestMissingResourcesAndSVGText(t *testing.T) {
	f := testutil.Files()
	delete(f, "OPS/image.svg")
	f["OPS/picture.xhtml"] = testutil.XHTML(`<svg xmlns="http://www.w3.org/2000/svg"><text>未処理</text></svg>`)
	p, e := Read(testutil.ZIP(f), "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	codes := map[string]bool{}
	for _, d := range p.Book.Diagnostics {
		if d.Blocking {
			codes[d.Code] = true
		}
	}
	if !codes["missing-resource"] || !codes["svg-text"] {
		t.Fatal(codes)
	}
}

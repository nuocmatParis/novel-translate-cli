package epub

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
)

func TestImageReplacementAndTranscriptExport(t *testing.T) {
	raw := testutil.EPUB()
	p, err := Read(raw, "v1", 1)
	if err != nil {
		t.Fatal(err)
	}

	var imageBlock domain.Block
	for _, b := range p.Book.Blocks {
		if b.Kind == "image" {
			imageBlock = b
			break
		}
	}
	if imageBlock.ID == "" {
		t.Fatal("expected at least one image block in test fixture")
	}

	g := domain.Generation{Units: map[string]domain.Unit{}}
	for _, b := range p.Book.Blocks {
		if b.Disposition == "translate" {
			u := domain.Unit{ID: b.ID, SourceHash: b.SourceHash, Target: b.Source, Revision: 1}
			u.Review = &domain.Review{SourceHash: b.SourceHash, TargetHash: domain.Hash([]byte(u.Target)), Decision: "accepted"}
			g.Units[b.ID] = u
		}
	}

	// 1. Test Image Transcript
	transcriptText := "Nhân vật chính: Subaru\nNữ chính: Emilia"
	optsTranscript := ExportOptions{
		Preview:          true,
		ImageTranscripts: map[string]string{imageBlock.ID: transcriptText},
	}
	outputTranscript, err := Write(raw, "v1", 1, g, optsTranscript)
	if err != nil {
		t.Fatalf("Write with transcript failed: %v", err)
	}
	rTranscript, err := VerifyOutput(raw, outputTranscript, "v1", 1, g, optsTranscript)
	if err != nil {
		t.Fatalf("VerifyOutput with transcript failed: %v %+v", err, rTranscript.Diagnostics)
	}

	// Check that the output EPUB XML actually contains the transcript
	outPkg, err := Read(outputTranscript, "v1", 1)
	if err != nil {
		t.Fatalf("failed to read output EPUB: %v", err)
	}
	docTree := outPkg.Trees[imageBlock.Document]
	if docTree == nil {
		t.Fatalf("document tree missing for %s", imageBlock.Document)
	}
	docXML := docTree.XML()
	if !strings.Contains(docXML, "translation-image-transcript") {
		t.Fatal("output document XML does not contain translation-image-transcript container")
	}
	if !strings.Contains(docXML, "Nhân vật chính: Subaru") || !strings.Contains(docXML, "Nữ chính: Emilia") {
		t.Fatal("output document XML does not contain transcript text lines")
	}

	// 2. Test Image Replacement
	replacementBytes := []byte("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 10 10\"><rect width=\"10\" height=\"10\" fill=\"red\"/></svg>")
	optsReplace := ExportOptions{
		Preview:           true,
		ImageReplacements: map[string][]byte{imageBlock.Resource: replacementBytes},
	}
	outputReplace, err := Write(raw, "v1", 1, g, optsReplace)
	if err != nil {
		t.Fatalf("Write with image replacement failed: %v", err)
	}
	rReplace, err := VerifyOutput(raw, outputReplace, "v1", 1, g, optsReplace)
	if err != nil {
		t.Fatalf("VerifyOutput with image replacement failed: %v %+v", err, rReplace.Diagnostics)
	}

	// Verify that the replacement image is inside the package files
	outPkgReplace, err := Read(outputReplace, "v1", 1)
	if err != nil {
		t.Fatalf("failed to read output EPUB: %v", err)
	}
	actualBytes := outPkgReplace.Files[imageBlock.Resource]
	if !bytes.Equal(actualBytes, replacementBytes) {
		t.Fatalf("expected resource %s to contain replacement bytes", imageBlock.Resource)
	}
}

func TestInlineImageTranscriptsArePreservedAndVerified(t *testing.T) {
	files := testutil.Files()
	files["OPS/a.xhtml"] = testutil.XHTML(`<h1 id="c1">第一章</h1><p id="b1">前<img src="image.svg"/>中<img src="image.svg"/>後。</p><ul><li>項目<img src="image.svg"/></li></ul>`)
	raw := testutil.ZIP(files)
	p, err := Read(raw, "v1", 1)
	if err != nil {
		t.Fatal(err)
	}
	g := domain.Generation{Units: map[string]domain.Unit{}}
	for _, b := range p.Book.Blocks {
		if b.Disposition == "translate" {
			g.Units[b.ID] = domain.Unit{ID: b.ID, SourceHash: b.SourceHash, Target: b.Source}
		}
	}
	transcripts := map[string]string{}
	var inlineIDs []string
	for _, image := range imageOccurrences(p) {
		if image.document == "OPS/a.xhtml" {
			inlineIDs = append(inlineIDs, image.id)
			transcripts[image.id] = fmt.Sprintf("Bản dịch ảnh số %d\nNội dung <đầy đủ> & chính xác.", len(inlineIDs))
		}
	}
	if len(inlineIDs) != 3 {
		t.Fatal("fixture must contain three inline images", inlineIDs)
	}
	opts := ExportOptions{Preview: true, ImageTranscripts: transcripts}
	out, err := Write(raw, "v1", 1, g, opts)
	if err != nil {
		t.Fatal(err)
	}
	if report, err := VerifyOutput(raw, out, "v1", 1, g, opts); err != nil {
		t.Fatal(err, report.Diagnostics)
	}
	output, err := Read(out, "v1", 1)
	if err != nil {
		t.Fatal(err)
	}
	var actualIDs []string
	output.Trees["OPS/a.xhtml"].Walk(func(n *Node) {
		if id := n.Get("data-source-image-transcript"); id != "" {
			actualIDs = append(actualIDs, id)
		}
	})
	if strings.Join(actualIDs, "|") != strings.Join(inlineIDs, "|") {
		t.Fatal("caption order changed", actualIDs, inlineIDs)
	}
	for _, replacement := range []string{"Nội dung khác", ""} {
		changed := map[string]string{}
		for path, data := range output.Files {
			changed[path] = string(data)
		}
		changed["OPS/a.xhtml"] = strings.ReplaceAll(changed["OPS/a.xhtml"], "Bản dịch ảnh số 1", replacement)
		if _, err := VerifyOutput(raw, testutil.ZIP(changed), "v1", 1, g, opts); err == nil {
			t.Fatal("altered caption passed readback")
		}
	}
	transcripts["missing:image"] = "Không được bỏ qua"
	if _, err := Write(raw, "v1", 1, g, opts); err == nil {
		t.Fatal("unknown image occurrence silently skipped")
	}
}

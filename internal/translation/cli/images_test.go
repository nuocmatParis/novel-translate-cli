package cli

import (
	"bytes"
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"github.com/voocel/ainovel-cli/internal/translation/verify"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func TestImageCLIEditRoundTrip(t *testing.T) {
	root := filepath.Join(t.TempDir(), "Ảnh có chữ")
	var out, errout bytes.Buffer
	options := Options{NewModel: func(workspace.Config) (model.Caller, error) { return &offlineModel{}, nil }}
	run := func(want int, args ...string) {
		t.Helper()
		out.Reset()
		errout.Reset()
		if code := Run(context.Background(), args, &out, &errout, options); code != want {
			t.Fatalf("%v: exit=%d want=%d\n%s\n%s", args, code, want, errout.String(), out.String())
		}
	}
	run(0, "init", root, "--series", "images")
	files := testutil.Files()
	files["OPS/a.xhtml"] = testutil.XHTML(`<h1 id="c1">第一章</h1><p id="b1">彼は来た。<img src="image.svg" alt=""/></p>`)
	source := filepath.Join(root, "source.epub")
	if err := os.WriteFile(source, testutil.ZIP(files), 0600); err != nil {
		t.Fatal(err)
	}
	run(0, "volume", "add", "--workspace", root, "--id", "v1", "--order", "1", "--source", source)
	run(0, "import", "--workspace", root, "--volume", "v1")
	var book domain.Book
	if err := json.Unmarshal(out.Bytes(), &book); err != nil {
		t.Fatal(err)
	}
	run(0, "image", "export", "--workspace", root, "--volume", "v1")
	var exported []struct {
		ID, File string
	}
	if err := json.Unmarshal(out.Bytes(), &exported); err != nil || len(exported) != 2 {
		t.Fatal(err, out.String())
	}
	if exported[0].File != exported[1].File {
		t.Fatal("shared resource exported inconsistently")
	}
	if err := os.WriteFile(exported[0].File, []byte("manual edits"), 0600); err != nil {
		t.Fatal(err)
	}
	run(1, "image", "export", "--workspace", root, "--volume", "v1")
	if !strings.Contains(errout.String(), "không ghi đè") {
		t.Fatal(errout.String())
	}
	run(1, "image", "export", "--workspace", root, "--volume", "v1", "--directory", "../escape")
	run(0, "translate", "--workspace", root, "--volume", "v1")
	for _, image := range verify.Images(book) {
		run(0, "image", "assess", "--workspace", root, "--volume", "v1", "--id", image.ID, "--disposition", "transcript", "--reason", "Đã đối chiếu", "--transcript", "Bản dịch ảnh: Xin chào!")
	}
	c, err := workspace.Load(root)
	if err != nil {
		t.Fatal(err)
	}
	if checker := os.Getenv("NOVEL_TRANSLATE_TEST_EPUBCHECK"); checker != "" {
		c.Output.EPUBCheck = checker
		if err := c.Save(); err != nil {
			t.Fatal(err)
		}
		run(0, "export", "--workspace", root, "--volume", "v1")
		if !strings.Contains(out.String(), "EPUBCheck passed") {
			t.Fatal(out.String())
		}
	}
	run(0, "export", "--workspace", root, "--volume", "v1", "--preview")
	preview, err := os.ReadFile(filepath.Join(root, "Output", "v1_VI.preview.epub"))
	if err != nil {
		t.Fatal(err)
	}
	p, err := epub.Read(preview, "v1", 1)
	if err != nil || !strings.Contains(string(p.Files["OPS/a.xhtml"]), "Bản dịch ảnh: Xin chào!") {
		t.Fatal("inline transcript missing from CLI export", err)
	}
	replacement := filepath.Join(root, "edited.svg")
	data := []byte(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="red"/></svg>`)
	if err := os.WriteFile(replacement, data, 0600); err != nil {
		t.Fatal(err)
	}
	for _, image := range verify.Images(book) {
		run(0, "image", "assess", "--workspace", root, "--volume", "v1", "--id", image.ID, "--disposition", "replace", "--reason", "Đã edit", "--file", replacement)
	}
	run(0, "verify", "--workspace", root, "--volume", "v1")
	if err := os.WriteFile(replacement, append(data, ' '), 0600); err != nil {
		t.Fatal(err)
	}
	run(2, "verify", "--workspace", root, "--volume", "v1")
	if !strings.Contains(out.String(), "image-replacement-invalid") {
		t.Fatal("changed approved image was not detected", out.String())
	}
	run(2, "export", "--workspace", root, "--volume", "v1")
}

func TestCommandHelpDoesNotNeedWorkspace(t *testing.T) {
	for _, args := range [][]string{{"init", "--help"}, {"export", "--help"}, {"image", "export", "--help"}, {"skill", "--help"}} {
		var out, errout bytes.Buffer
		if code := Run(context.Background(), args, &out, &errout, Options{}); code != 0 {
			t.Fatal(args, code, errout.String())
		}
		if !strings.Contains(out.String()+errout.String(), "workspace") {
			t.Fatal("missing help output", args)
		}
	}
}

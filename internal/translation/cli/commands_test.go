package cli

import (
	"bytes"
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"github.com/voocel/ainovel-cli/internal/translation/verify"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type offlineModel struct{ Calls int }

func (*offlineModel) Identity(role string) string { return "fake/" + role }
func (*offlineModel) InputLimit(string) int       { return 24000 }
func (m *offlineModel) Call(ctx context.Context, role, system string, payload any, c llmcontract.Contract, out any, validate func() error) error {
	m.Calls++
	b, _ := json.Marshal(payload)
	var p struct {
		Units   []domain.Block `json:"owned_units"`
		Targets []domain.Unit  `json:"targets"`
	}
	if e := json.Unmarshal(b, &p); e != nil {
		return e
	}
	var response any
	if role == "translation_review" {
		response = map[string]any{"reviewed": []map[string]string{{"id": p.Units[0].ID, "target_hash": domain.Hash([]byte(p.Targets[0].Target))}}, "issues": []any{}, "ruby_decisions": testutil.RubyDecisions(p.Units[0])}
	} else {
		response = map[string]any{"units": []map[string]string{{"id": p.Units[0].ID, "target": testutil.TranslateFixture(p.Units[0].Source)}}, "uncertainties": []any{}, "continuity_candidates": []any{}, "rule_candidates": []any{}}
	}
	b, _ = json.Marshal(response)
	if e := json.Unmarshal(b, out); e != nil {
		return e
	}
	return validate()
}
func TestOfflineCLIWorkflowAndPreview(t *testing.T) {
	root := filepath.Join(t.TempDir(), "Bộ có dấu và khoảng trắng")
	var out, errout bytes.Buffer
	m := &offlineModel{}
	options := Options{NewModel: func(workspace.Config) (model.Caller, error) { return m, nil }}
	run := func(args ...string) {
		t.Helper()
		out.Reset()
		errout.Reset()
		if code := Run(context.Background(), args, &out, &errout, options); code != 0 {
			if logs, _ := filepath.Glob(filepath.Join(root, ".translation_work", "exports", "*.txt")); len(logs) > 0 {
				for _, log := range logs {
					raw, _ := os.ReadFile(log)
					t.Logf("checker %s: %s", log, raw)
				}
			}
			t.Fatalf("%v exit=%d %s\n%s", args, code, errout.String(), out.String())
		}
	}
	run("init", root, "--series", "test", "--title", "Bộ thử")
	source := filepath.Join(root, "nguồn Nhật.epub")
	if e := os.WriteFile(source, testutil.EPUB(), 0600); e != nil {
		t.Fatal(e)
	}
	run("volume", "add", "--workspace", root, "--id", "v1", "--order", "1", "--source", source)
	run("import", "--workspace", root, "--volume", "v1")
	var imported domain.Book
	if e := json.Unmarshal(out.Bytes(), &imported); e != nil {
		t.Fatal(e)
	}
	for _, image := range verify.Images(imported) {
		run("image", "assess", "--workspace", root, "--volume", "v1", "--id", image.ID, "--disposition", "no-text", "--reason", "synthetic blue rectangle has no text")
	}
	run("status", "--workspace", root, "--json")
	run("translate", "--workspace", root, "--volume", "v1")
	calls := m.Calls
	run("resume", "--workspace", root, "--volume", "v1")
	if m.Calls != calls {
		t.Fatal("resume repeated valid work")
	}
	run("verify", "--workspace", root, "--volume", "v1")
	run("export", "--workspace", root, "--volume", "v1", "--preview")
	if _, e := os.Stat(filepath.Join(root, "Output", "v1_VI.preview.epub")); e != nil {
		t.Fatal(e)
	}
	var rangeID string
	for _, ch := range imported.Chapters {
		if ch.Title == "第一章" {
			rangeID = ch.ID
		}
	}
	run("export", "--workspace", root, "--volume", "v1", "--preview", "--from", rangeID, "--to", rangeID)
	var result struct {
		Report domain.Report `json:"report"`
	}
	if e := json.Unmarshal(out.Bytes(), &result); e != nil {
		t.Fatal(e)
	}
	if !result.Report.Structure {
		t.Fatal("range verifier failed", result.Report.Diagnostics)
	}
	rangePath := filepath.Join(root, "Output", "v1_VI_"+rangeID+"-"+rangeID+".preview.epub")
	rangeRaw, e := os.ReadFile(rangePath)
	if e != nil {
		t.Fatal(e)
	}
	p, e := epub.Read(rangeRaw, "v1", 1)
	if e != nil {
		t.Fatal(e)
	}
	if _, ok := p.Files["OPS/notes.xhtml"]; !ok {
		t.Fatal("range dropped note")
	}
	if _, ok := p.Files["OPS/z.xhtml"]; ok {
		t.Fatal("range retained front chapter")
	}
	out.Reset()
	errout.Reset()
	if code := Run(context.Background(), []string{"export", "--workspace", root, "--volume", "v1"}, &out, &errout, options); code != 2 {
		t.Fatalf("missing EPUBCheck must block final export, code=%d %s", code, errout.String())
	}
	if _, e := os.Stat(filepath.Join(root, "Output", "v1_VI.epub")); !os.IsNotExist(e) {
		t.Fatal("final EPUB was published without validator")
	}
	// Use the actual checker only when explicitly configured by the developer.
	if checker := os.Getenv("NOVEL_TRANSLATE_TEST_EPUBCHECK"); checker != "" {
		c, e := workspace.Load(root)
		if e != nil {
			t.Fatal(e)
		}
		c.Output.EPUBCheck = checker
		if fonts := os.Getenv("NOVEL_TRANSLATE_TEST_FONTS"); fonts != "" {
			c.Output.FontDirectory = fonts
		}
		if e = c.Save(); e != nil {
			t.Fatal(e)
		}
		run("export", "--workspace", root, "--volume", "v1")
		var final struct {
			Report domain.Report `json:"report"`
		}
		if e = json.Unmarshal(out.Bytes(), &final); e != nil {
			t.Fatal(e)
		}
		if !final.Report.Structure || final.Report.Accepted != final.Report.Eligible || !strings.Contains(final.Report.Format, "pass") {
			t.Fatal("actual final gates not satisfied", final.Report)
		}
		if artifact := os.Getenv("NOVEL_TRANSLATE_TEST_OUTPUT"); artifact != "" {
			if e = os.MkdirAll(artifact, 0700); e != nil {
				t.Fatal(e)
			}
			data, e := os.ReadFile(filepath.Join(root, "Output", "v1_VI.epub"))
			if e != nil {
				t.Fatal(e)
			}
			if e = os.WriteFile(filepath.Join(artifact, "scripted-final.epub"), data, 0600); e != nil {
				t.Fatal(e)
			}
			data, e = json.MarshalIndent(final.Report, "", "  ")
			if e != nil {
				t.Fatal(e)
			}
			if e = os.WriteFile(filepath.Join(artifact, "scripted-final.report.json"), data, 0600); e != nil {
				t.Fatal(e)
			}
		}
	}
	// Status must re-evaluate freshness, even before a retranslate plan is applied.
	c, e := workspace.Load(root)
	if e != nil {
		t.Fatal(e)
	}
	skillPath := c.ReadPath(c.SeriesSkill)
	raw, e := os.ReadFile(skillPath)
	if e != nil {
		t.Fatal(e)
	}
	if e = os.WriteFile(skillPath, append(raw, []byte("\nUpdated convention.\n")...), 0600); e != nil {
		t.Fatal(e)
	}
	run("status", "--workspace", root, "--json")
	var statusResult struct {
		Volumes []struct {
			Accepted int      `json:"accepted"`
			Stale    []string `json:"stale"`
		} `json:"volumes"`
	}
	if e = json.Unmarshal(out.Bytes(), &statusResult); e != nil {
		t.Fatal(e)
	}
	if len(statusResult.Volumes) != 1 || statusResult.Volumes[0].Accepted != 0 || len(statusResult.Volumes[0].Stale) == 0 {
		t.Fatal("status reported stale review as accepted", out.String())
	}
}

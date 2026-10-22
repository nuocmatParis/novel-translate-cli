package cli

import (
	"bytes"
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func setupTUI(t *testing.T) *translationTUI {
	t.Helper()
	root := filepath.Join(t.TempDir(), "Bộ thử có dấu")
	var out, errout bytes.Buffer
	if code := Run(context.Background(), []string{"init", root, "--series", "test", "--title", "Bộ thử"}, &out, &errout, Options{}); code != 0 {
		t.Fatal(code, errout.String())
	}
	c, err := workspace.Load(root)
	if err != nil {
		t.Fatal(err)
	}
	m := newTUI(context.Background(), Options{})
	m.config = &c
	return m
}

func TestTUIModelSetupPersistsMaskedKeyAndReusesIt(t *testing.T) {
	t.Setenv("NOVEL_TRANSLATE_API_KEY", "")
	m := setupTUI(t)
	envPath := filepath.Join(m.config.Root, ".env")
	if err := os.WriteFile(envPath, []byte("OTHER_SETTING='keep me'\n"), 0600); err != nil {
		t.Fatal(err)
	}
	const key = `fake-$key#with"quotes\and spaces`
	m.openForm("model")
	values := []string{"gemini", "fake-model", "", key, "", "chat", "32000"}
	for i, value := range values {
		m.form.fields[i].value = value
		m.form.fields[i].input.SetValue(value)
	}
	if strings.Contains(m.View(), key) {
		t.Fatal("key exposed in form")
	}
	// A failed asynchronous save must retain all entered fields for correction.
	m.form.fields[modelContextField].value = "invalid"
	m.submitForm()
	if done := finishTUITask(t, m); done.err == nil || m.form == nil || m.form.fields[3].value != key {
		t.Fatal("failed save lost form")
	}
	m.form.fields[modelContextField].value = "32000"
	m.submitForm()
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	if m.form != nil || m.submittedForm != nil {
		t.Fatal("successful form still retained")
	}
	path := providerPath(*m.config)
	raw, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(raw), key) || strings.Contains(strings.Join(m.logs, "\n"), key) {
		t.Fatal("key exposed outside .env")
	}
	unresolved, err := bootstrap.ReadConfigFile(path)
	if err != nil || unresolved.DefaultProviderConfig().APIKey != "" {
		t.Fatal("inline key persisted in JSON", err)
	}
	parsed, err := bootstrap.ReadDotEnv(envPath)
	if err != nil || parsed["NOVEL_TRANSLATE_API_KEY"] != key || parsed["OTHER_SETTING"] != "keep me" {
		t.Fatal("dotenv did not round trip or retain other values", err)
	}
	cfg, err := bootstrap.LoadConfigFile(path, m.config.Root)
	if err != nil || cfg.DefaultProviderConfig().APIKey != key {
		t.Fatal("saved setup cannot reload", err)
	}
	if cfg.DefaultProviderConfig().API != "" {
		t.Fatal("OpenAI-only api field leaked to Gemini")
	}
	m.openForm("model")
	if m.form.fields[3].value != "" {
		t.Fatal("saved key exposed when opening form")
	}
	m.submitForm()
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal("blank key should reuse saved key", done.err)
	}
}

func TestLocalSetupRollback(t *testing.T) {
	dir := t.TempDir()
	old, fresh := filepath.Join(dir, ".env"), filepath.Join(dir, "provider.json")
	if err := os.WriteFile(old, []byte("original"), 0600); err != nil {
		t.Fatal(err)
	}
	err := saveLocalFiles([]localWrite{{old, []byte("replacement")}, {fresh, []byte("new")}}, func() error { return errors.New("workspace commit failed") })
	if err == nil {
		t.Fatal("commit failure hidden")
	}
	data, err := os.ReadFile(old)
	if err != nil || string(data) != "original" {
		t.Fatal("original file not restored", err)
	}
	if _, err := os.Stat(fresh); !errors.Is(err, os.ErrNotExist) {
		t.Fatal("new file not rolled back", err)
	}
}

func TestTUISettingsAndInputCursor(t *testing.T) {
	m := setupTUI(t)
	m.openForm("settings")
	values := []string{"17", "90", "tools/checker.jar", "fonts"}
	for i, value := range values {
		m.form.fields[i].value = value
	}
	m.submitForm()
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	if m.config.Run.MaxRequests != 17 || m.config.Run.RequestTimeoutSeconds != 90 || m.config.Output.FontDirectory != "fonts" {
		t.Fatal("settings not saved")
	}
	m.openForm("create")
	m.form.fields[0].value = "abc"
	m.updateForm(tea.KeyMsg{Type: tea.KeyHome})
	m.updateForm(tuiKey("Z"))
	m.updateForm(tea.KeyMsg{Type: tea.KeyDelete})
	if m.form.fields[0].value != "Zbc" {
		t.Fatalf("cursor editing failed: %q", m.form.fields[0].value)
	}
	// Directory selection accepts spaces and non-ASCII paths without shell escaping.
	m.form.fields[0].value = m.config.Root
	cmd := m.openPicker()
	if cmd != nil {
		m.Update(cmd())
	}
	m.updatePicker(tuiKey(" "))
	if m.picker != nil || m.form.fields[0].value != m.config.Root {
		t.Fatal("directory not selected")
	}
}

func TestTUIShortcutDoesNotTypeIntoNewForm(t *testing.T) {
	m := setupTUI(t)
	for _, pair := range []struct{ key, kind, first string }{
		{"n", "create", "workspace/bo-truyen"},
		{"o", "open", ""},
		{"a", "add", "vol01"},
		{"m", "model", "openai"},
		{"c", "settings", "0"},
	} {
		m.form = nil
		m.Update(tuiKey(pair.key))
		if m.form == nil || m.form.kind != pair.kind || m.form.fields[0].value != pair.first {
			t.Fatalf("shortcut %s typed into new form", pair.key)
		}
	}
}

func TestTUILongPathInputScrollsToCursor(t *testing.T) {
	m := setupTUI(t)
	m.Update(tea.WindowSizeMsg{Width: 40, Height: 24})
	m.openForm("create")
	m.updateForm(tea.KeyMsg{Type: tea.KeyCtrlU})
	m.updateForm(tuiKey(strings.Repeat("a", 100) + "TAIL"))
	if !strings.Contains(m.View(), "TAIL") {
		t.Fatal("end of path hidden while typing")
	}
}

func TestDefaultCommandOpensHeadlessTUI(t *testing.T) {
	var out, errout bytes.Buffer
	code := Run(context.Background(), nil, &out, &errout, Options{Input: strings.NewReader("q")})
	if code != 0 || !strings.Contains(out.String(), "NOVEL TRANSLATE") {
		t.Fatal(code, errout.String())
	}
}

func TestTUIFilePickerSelectsEPUBWithSpaces(t *testing.T) {
	m := setupTUI(t)
	path := filepath.Join(t.TempDir(), "nguồn Nhật có dấu.epub")
	if err := os.WriteFile(path, []byte("picker fixture"), 0600); err != nil {
		t.Fatal(err)
	}
	m.openForm("add")
	m.form.index = 2
	m.form.fields[2].value = path
	cmd := m.openPicker()
	if cmd != nil {
		m.Update(cmd())
	}
	m.updatePicker(tea.KeyMsg{Type: tea.KeyEnter})
	if m.picker != nil || m.form.fields[2].value != path {
		t.Fatal("wrong selected path")
	}
}

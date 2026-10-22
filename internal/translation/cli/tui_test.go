package cli

import (
	"bytes"
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"github.com/voocel/ainovel-cli/internal/translation/verify"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func tuiKey(text string) tea.KeyMsg {
	return tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune(text)}
}

func finishTUITask(t *testing.T, m *translationTUI) tuiDoneMsg {
	t.Helper()
	deadline := time.NewTimer(10 * time.Second)
	defer deadline.Stop()
	for {
		select {
		case event := <-m.events:
			m.Update(event)
			if done, ok := event.(tuiDoneMsg); ok {
				return done
			}
		case <-deadline.C:
			if m.cancel != nil {
				m.cancel()
			}
			t.Fatal("TUI task did not finish")
		}
	}
}

func selectTUIAction(t *testing.T, m *translationTUI, command string) {
	t.Helper()
	for i, a := range m.actions() {
		if len(a.command) > 0 && a.command[0] == command {
			m.selected = i
			return
		}
	}
	t.Fatalf("no action %s", command)
}

func TestTUIWorkspaceFormsAndOfflineImport(t *testing.T) {
	m := newTUI(context.Background(), Options{NewModel: func(workspace.Config) (model.Caller, error) {
		t.Fatal("workspace/import/status must not create a model")
		return nil, nil
	}})
	if m.Init() != nil || !strings.Contains(m.View(), "Mở workspace") {
		t.Fatal("missing welcome screen")
	}
	root := filepath.Join(t.TempDir(), "Bộ truyện có dấu")
	m.openForm("create")
	m.form.fields[0].value = root
	m.form.fields[1].value = "sample"
	m.form.fields[2].value = "Bộ thử"
	m.submitForm()
	if done := finishTUITask(t, m); done.err != nil || done.snapshotErr != nil {
		t.Fatal(done)
	}
	if m.config == nil || m.config.Root != root {
		t.Fatal("created workspace was not opened")
	}
	source := filepath.Join(root, "nguồn Nhật.epub")
	if err := os.WriteFile(source, testutil.EPUB(), 0600); err != nil {
		t.Fatal(err)
	}
	for _, id := range []string{"vol01", "vol02"} {
		m.openForm("add")
		if m.form.fields[0].value != id {
			t.Fatalf("bad suggested volume: %#v", m.form.fields)
		}
		m.form.fields[2].value = `"` + source + `"`
		m.submitForm()
		if done := finishTUITask(t, m); done.err != nil {
			t.Fatal(done.err)
		}
	}
	if m.config.Volumes[1].Previous != "vol01" {
		t.Fatal("previous volume not linked")
	}
	m.Update(tea.KeyMsg{Type: tea.KeyRight})
	if m.volume != 1 {
		t.Fatal("could not select second volume")
	}
	selectTUIAction(t, m, "import")
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if done := finishTUITask(t, m); done.err != nil || done.snapshotErr != nil {
		t.Fatal(done)
	}
	if len(m.snapshot.Volumes) != 2 || !m.snapshot.Volumes[1].Imported || m.snapshot.Volumes[0].Imported {
		t.Fatal("import affected wrong volume", m.snapshot)
	}
	if !strings.Contains(m.View(), "vol02") {
		t.Fatal("selected volume missing from view")
	}
	v, _ := m.config.Volume("vol02")
	_, book, err := engine.Source(*m.config, v, true)
	if err != nil {
		t.Fatal(err)
	}
	images := verify.Images(book)
	if len(images) == 0 || m.snapshot.Volumes[1].ImageIssues == 0 {
		t.Fatal("missing pending image status")
	}
	m.openForm("image")
	m.form.fields[0].value = images[0].ID
	m.form.fields[1].value = "no-text"
	m.form.fields[2].value = "Synthetic rectangle has no text"
	m.submitForm()
	if done := finishTUITask(t, m); done.err != nil || m.snapshot.Volumes[1].ImageIssues != 0 {
		t.Fatal("TUI image decision failed", done.err, m.snapshot)
	}
	// Provider form saves only the path and never needs credentials to open.
	provider := filepath.Join(root, "provider.jsonc")
	if err := os.WriteFile(provider, []byte(`{"provider":"not-configured-yet"}`), 0600); err != nil {
		t.Fatal(err)
	}
	m.openForm("provider")
	m.form.fields[0].value = provider
	m.submitForm()
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	c, err := workspace.Load(root)
	if err != nil || c.ProviderConfig != provider {
		t.Fatal(c.ProviderConfig, err)
	}
}

type cancelTUIModel struct {
	started chan struct{}
}

func (*cancelTUIModel) Identity(string) string { return "test/cancel" }
func (*cancelTUIModel) InputLimit(string) int  { return 24000 }
func (m *cancelTUIModel) Call(ctx context.Context, _, _ string, _ any, _ llmcontract.Contract, _ any, _ func() error) error {
	close(m.started)
	<-ctx.Done()
	return ctx.Err()
}

func TestTUIConfirmsPaidWorkAndWaitsForCancellation(t *testing.T) {
	root := filepath.Join(t.TempDir(), "workspace")
	var out, errout bytes.Buffer
	run := func(args ...string) {
		t.Helper()
		if code := Run(context.Background(), args, &out, &errout, Options{}); code != 0 {
			t.Fatal(code, errout.String())
		}
	}
	run("init", root, "--series", "sample")
	source := filepath.Join(root, "source.epub")
	if err := os.WriteFile(source, testutil.EPUB(), 0600); err != nil {
		t.Fatal(err)
	}
	run("volume", "add", "--workspace", root, "--id", "vol01", "--order", "1", "--source", source)
	run("import", "--workspace", root, "--volume", "vol01")
	c, err := workspace.Load(root)
	if err != nil {
		t.Fatal(err)
	}
	caller := &cancelTUIModel{started: make(chan struct{})}
	modelCreated := false
	m := newTUI(context.Background(), Options{NewModel: func(workspace.Config) (model.Caller, error) {
		modelCreated = true
		return caller, nil
	}})
	m.config = &c
	m.Init()
	if done := finishTUITask(t, m); done.err != nil || modelCreated {
		t.Fatal("opening TUI called a model", done)
	}
	selectTUIAction(t, m, "resume")
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if m.pending == nil || m.busy || modelCreated {
		t.Fatal("model called before confirmation")
	}
	m.Update(tea.KeyMsg{Type: tea.KeyEsc})
	if m.pending != nil || modelCreated {
		t.Fatal("canceling confirmation called model")
	}
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	m.Update(tuiKey("y"))
	select {
	case <-caller.started:
	case <-time.After(5 * time.Second):
		t.Fatal("translation did not start")
	}
	_, cmd := m.Update(tuiKey("q"))
	if cmd != nil || !m.busy || !m.quitOnDone {
		t.Fatal("quit must wait for cancellation to finish")
	}
	done := finishTUITask(t, m)
	if done.err == nil || !strings.Contains(done.err.Error(), "context canceled") || m.busy {
		t.Fatal("cancellation not propagated", done.err)
	}
	// Cancellation releases the normal workspace writer lock.
	var checkOut bytes.Buffer
	if code := Run(context.Background(), []string{"sync", "--workspace", root, "--volume", "vol01"}, &checkOut, &errout, Options{}); code != 0 {
		t.Fatal("workspace still locked after cancellation", errout.String())
	}
}

func TestTUIHelpAndHeadlessQuit(t *testing.T) {
	for _, command := range []string{"tui", "--tui"} {
		var out, errout bytes.Buffer
		if code := Run(context.Background(), []string{command, "--help"}, &out, &errout, Options{}); code != 0 || !strings.Contains(errout.String(), "--workspace") {
			t.Fatal(code, errout.String())
		}
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	var out, errout bytes.Buffer
	if code := Run(ctx, []string{"tui"}, &out, &errout, Options{Input: strings.NewReader("q")}); code != 0 {
		t.Fatal(code, errout.String())
	}
	if !strings.Contains(Help, "tui [--workspace") {
		t.Fatal("TUI not discoverable in CLI help")
	}
}

func TestTUISanitizesTerminalTextAndRetainsUnresolvedResult(t *testing.T) {
	m := newTUI(context.Background(), Options{})
	m.addLog("source\x1b[2J\x1b[31mRED\x1b[0m\a")
	if strings.ContainsAny(strings.Join(m.logs, ""), "\x1b\a") {
		t.Fatal("terminal control sequence survived")
	}
	m.Update(tuiDoneMsg{err: errors.New("bad input\x1b[2J")})
	if strings.Contains(ansi.Strip(m.View()), "\x1b") {
		t.Fatal("provider error injected escape")
	}
	m.Update(tea.WindowSizeMsg{Width: 50, Height: 24})
	for _, line := range strings.Split(m.View(), "\n") {
		if ansi.StringWidth(line) > 50 {
			t.Fatal("view did not adapt to terminal width", line)
		}
	}
}

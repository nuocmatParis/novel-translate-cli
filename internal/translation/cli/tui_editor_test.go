package cli

import (
	"bytes"
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func translatedTUI(t *testing.T) *translationTUI {
	t.Helper()
	m := setupTUI(t)
	root := m.config.Root
	source := filepath.Join(root, "nguồn Nhật.epub")
	if err := os.WriteFile(source, testutil.EPUB(), 0600); err != nil {
		t.Fatal(err)
	}
	m.options.NewModel = func(workspace.Config) (model.Caller, error) { return &offlineModel{}, nil }
	var out, errout bytes.Buffer
	for _, args := range [][]string{
		{"volume", "add", "--workspace", root, "--id", "vol01", "--order", "1", "--source", source},
		{"import", "--workspace", root, "--volume", "vol01"},
		{"translate", "--workspace", root, "--volume", "vol01"},
	} {
		if code := Run(context.Background(), args, &out, &errout, m.options); code != 0 {
			t.Fatal(code, errout.String())
		}
	}
	c, err := workspace.Load(root)
	if err != nil {
		t.Fatal(err)
	}
	m.config = &c
	m.openEditor("translation")
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	// Select a plain-text unit while going through the same chapter/unit screens.
	for i := range m.editor.chapters {
		m.editor.chapter = i
		m.editor.selectChapter()
		for j, b := range m.editor.units {
			if b.Kind != "attribute" && len(b.Tokens) == 0 {
				m.editor.unit = j
				m.Update(tea.KeyMsg{Type: tea.KeyEnter})
				return m
			}
		}
	}
	t.Fatal("no plain unit in fixture")
	return nil
}

func TestTUIEditorSavesRevisionAndProtectsConcurrentChanges(t *testing.T) {
	m := translatedTUI(t)
	e := m.editor
	id := e.units[e.unit].ID
	before := e.generation.Units[id]
	const target = "Bản dịch đã sửa, giữ nguyên ánh mắt ấy."
	e.input.SetValue(target)
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlS})
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	store := state.Store{Root: m.config.Root}
	g, err := store.Load(e.volume.ID)
	if err != nil {
		t.Fatal(err)
	}
	u := g.Units[id]
	if u.Target != target || u.Origin != "manual" || u.Revision != before.Revision+1 || u.Review != nil || !u.RefreshDependencies || e.base != target {
		t.Fatal("manual revision or invalidation incorrect", u)
	}
	if err := revision.Check(*m.config, e.volume, e.book, g); err != nil {
		t.Fatal("chapter Markdown not synchronized", err)
	}
	// Simulate another command committing a generation while the editor is open.
	if err := store.Commit(e.volume.ID, &g, g.ID); err != nil {
		t.Fatal(err)
	}
	e.input.SetValue("Nội dung đang nhập phải được giữ lại.")
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlS})
	if done := finishTUITask(t, m); done.err == nil {
		t.Fatal("stale editor overwrote generation")
	}
	current, err := store.Load(e.volume.ID)
	if err != nil || current.Units[id].Target != target || e.input.Value() != "Nội dung đang nhập phải được giữ lại." {
		t.Fatal("conflict lost text", err)
	}
	m.Update(tea.KeyMsg{Type: tea.KeyEsc})
	if !e.discard || e.stage != "edit" {
		t.Fatal("unsaved edit discarded without confirmation")
	}
	m.Update(tuiKey("n"))
	if e.discard || e.stage != "edit" {
		t.Fatal("cancel discard failed")
	}
}

func TestTUISkillEditorSavesAndRejectsExternalEdits(t *testing.T) {
	m := setupTUI(t)
	m.openEditor("skill")
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	e := m.editor
	text := e.base + "\nGiữ hậu tố theo bản Nhật.\n"
	e.input.SetValue(text)
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlS})
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	path := m.config.ReadPath(m.config.SeriesSkill)
	data, err := os.ReadFile(path)
	if err != nil || string(data) != text {
		t.Fatal("skill not saved exactly", err)
	}
	external := append(data, []byte("\nExternal edit\n")...)
	if err := os.WriteFile(path, external, 0600); err != nil {
		t.Fatal(err)
	}
	e.input.SetValue(text + "\nLocal edit\n")
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlS})
	if done := finishTUITask(t, m); done.err == nil {
		t.Fatal("external skill edit overwritten")
	}
	data, err = os.ReadFile(path)
	if err != nil || !bytes.Equal(data, external) || !strings.Contains(e.input.Value(), "Local edit") {
		t.Fatal("conflict lost content", err)
	}
}

func TestTUIEditorFitsNarrowWindowAndRetainsLongInput(t *testing.T) {
	e := &tuiEditor{kind: "skill", stage: "edit", base: strings.Repeat("Một dòng quy ước.\n", 150)}
	e.initInput(40, 18)
	if e.input.Value() != e.base {
		t.Fatal("long text truncated when opening")
	}
	if e.input.Line() != 0 {
		t.Fatal("editor opened at end of document")
	}
	for _, line := range strings.Split(e.view(40, 18, "", false), "\n") {
		if ansi.StringWidth(line) > 40 {
			t.Fatalf("editor line wider than terminal: %q", line)
		}
	}
}

func TestTUIEditorWindowsLineBreaksAndNoOpSave(t *testing.T) {
	m := setupTUI(t)
	path := m.config.ReadPath(m.config.SeriesSkill)
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	windows := strings.ReplaceAll(string(data), "\n", "\r\n") + "\r\n\tQuy ước có tab.\r\n"
	if err := os.WriteFile(path, []byte(windows), 0600); err != nil {
		t.Fatal(err)
	}
	m.openEditor("skill")
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	want := strings.ReplaceAll(strings.ReplaceAll(windows, "\r\n", "\n"), "\t", "    ")
	if m.editor.input.Value() != want {
		t.Fatal("Windows line breaks doubled or text truncated")
	}
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlS})
	if m.busy {
		t.Fatal("unmodified input should not rewrite file")
	}
	current, err := os.ReadFile(path)
	if err != nil || string(current) != windows {
		t.Fatal("no-op save changed original bytes", err)
	}
	if _, err := prepareEditorText(strings.Repeat("line\n", 10000)); err == nil {
		t.Fatal("oversized input silently truncated")
	}
}

func TestTUIEditorRejectsUnsyncedChapter(t *testing.T) {
	m := translatedTUI(t)
	v := m.config.Volumes[0]
	_, book, err := engine.Source(*m.config, v, true)
	if err != nil {
		t.Fatal(err)
	}
	path, err := revision.ChapterPath(*m.config, v, book.Chapters[0].ID)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(path, []byte("External chapter edit"), 0600); err != nil {
		t.Fatal(err)
	}
	m.editor.input.SetValue("Một sửa đổi khác")
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlS})
	if done := finishTUITask(t, m); done.err == nil {
		t.Fatal("external chapter edit overwritten")
	}
	data, err := os.ReadFile(path)
	if err != nil || string(data) != "External chapter edit" {
		t.Fatal("chapter edit lost", err)
	}
}

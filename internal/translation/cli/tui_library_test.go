package cli

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
)

func TestTUISharedLibrarySelectionAndEditor(t *testing.T) {
	home := t.TempDir()
	t.Setenv("HOME", home)
	t.Setenv("USERPROFILE", home)
	m := setupTUI(t)
	m.Update(tuiKey("l"))
	if m.form == nil || m.form.kind != "library" {
		t.Fatal("shared library shortcut missing")
	}
	directory := filepath.Join(t.TempDir(), "Skill chung có dấu")
	m.form.fields[0].value = directory
	m.submitForm()
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	if path, err := skills.SharedDirectory(); err != nil || path != directory {
		t.Fatal(path, err)
	}
	privatePath := m.config.ReadPath(m.config.SeriesSkill)
	private, err := os.ReadFile(privatePath)
	if err != nil {
		t.Fatal(err)
	}
	m.openEditor("shared")
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	if m.editor == nil || m.editor.stage != "skills" || len(m.editor.sharedSkills) < 3 {
		t.Fatal("missing shared skill list")
	}
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	e := m.editor
	text := e.base + "\nShared translation instruction.\n"
	e.input.SetValue(text)
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlS})
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	actual, err := os.ReadFile(e.sharedSkills[e.chapter].Path)
	if err != nil || string(actual) != text {
		t.Fatal("shared skill not saved", err)
	}
	actual, err = os.ReadFile(privatePath)
	if err != nil || string(actual) != string(private) {
		t.Fatal("shared edit overwrote series convention", err)
	}
	// A concurrent file edit is kept and the local text remains available.
	path := e.sharedSkills[e.chapter].Path
	if err := os.WriteFile(path, []byte(text+"\nExternal edit\n"), 0600); err != nil {
		t.Fatal(err)
	}
	e.input.SetValue(text + "\nLocal edit\n")
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlS})
	if done := finishTUITask(t, m); done.err == nil {
		t.Fatal("concurrent shared edit overwritten")
	}
	if !strings.Contains(e.input.Value(), "Local edit") {
		t.Fatal("unsaved common text lost")
	}
	m.Update(tea.KeyMsg{Type: tea.KeyEsc})
	m.Update(tuiKey("y"))
	if m.editor == nil || e.stage != "skills" {
		t.Fatal("shared editor did not return to its list")
	}
	m.Update(tea.KeyMsg{Type: tea.KeyEsc})
	if m.editor != nil {
		t.Fatal("shared list did not close")
	}
}

func TestTUIEnabledSkillsRejectsUnknownAndDuplicateIDs(t *testing.T) {
	m := setupTUI(t)
	for _, value := range []string{"missing-module", "translation-review, translation-review"} {
		m.openForm("enabled-skills")
		m.form.fields[0].value = value
		m.submitForm()
		if done := finishTUITask(t, m); done.err == nil || m.form == nil {
			t.Fatal("invalid skill selection accepted")
		}
	}
	m.form.fields[0].value = "jp-vn-translation, translation-review"
	m.submitForm()
	if done := finishTUITask(t, m); done.err != nil {
		t.Fatal(done.err)
	}
	if len(m.config.EnabledSkills) != 2 {
		t.Fatal("skill selection not saved")
	}
}

func TestTUISharedDirectoryRejectsEmptyAndWorkspaceLocalPaths(t *testing.T) {
	m := setupTUI(t)
	before, err := skills.SharedDirectory()
	if err != nil {
		t.Fatal(err)
	}
	for _, value := range []string{"", filepath.Join(m.config.Root, "Skills", "common")} {
		m.openForm("library")
		m.form.fields[0].value = value
		m.submitForm()
		if done := finishTUITask(t, m); done.err == nil || m.form == nil {
			t.Fatal("invalid common directory accepted")
		}
		if after, err := skills.SharedDirectory(); err != nil || after != before {
			t.Fatal("invalid form changed common directory", err)
		}
	}
}

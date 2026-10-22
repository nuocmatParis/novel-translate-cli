package tui

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/charmbracelet/bubbles/filepicker"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type pathPickerState struct {
	model     filepicker.Model
	directory bool
}

func (a *App) openPathPicker() (bool, tea.Cmd) {
	var value string
	var directory bool
	var allowedTypes []string
	switch a.CurrentScreen {
	case "welcome":
		if !a.WelcomeScreen.Opening {
			return false, nil
		}
		value, directory = a.WelcomeScreen.PathInput.Value(), true
	case "content":
		s := a.ContentScreen
		if !s.AddingVolume || s.AddField != 2 || len(s.AddInputs) <= s.AddField {
			return false, nil
		}
		value, allowedTypes = s.AddInputs[s.AddField].Value(), []string{".epub", ".EPUB"}
	case "settings":
		s := a.SettingsScreen
		if s.ActiveTab != 2 || s.FocusedField < 0 || s.FocusedField >= len(s.Inputs) {
			return false, nil
		}
		directory = s.FocusedField == 0 || s.FocusedField == 2
		value = s.Inputs[s.FocusedField].Value()
	case "verify":
		s := a.VerifyScreen
		if !s.AssessingImage || s.FocusedField != 3 || len(s.Inputs) <= s.FocusedField {
			return false, nil
		}
		value = s.Inputs[s.FocusedField].Value()
		allowedTypes = []string{".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg"}
	case "skills":
		s := a.SkillsScreen
		if !s.EditingLibrary {
			return false, nil
		}
		value, directory = s.LibraryInput.Value(), true
	default:
		return false, nil
	}

	picker := filepicker.New()
	picker.ShowPermissions = false
	picker.ShowSize = true
	picker.ShowHidden = true
	picker.AllowedTypes = allowedTypes
	picker.Height = max(3, a.Height-8)
	picker.AutoHeight = false
	picker.FileAllowed = !directory
	picker.DirAllowed = false

	path := strings.Trim(strings.TrimSpace(value), "\"'")
	if path == "" {
		if a.Service != nil && a.Service.Config != nil {
			path = a.Service.Config.Root
		} else {
			path, _ = os.Getwd()
		}
	} else if !filepath.IsAbs(path) {
		if a.Service != nil && a.Service.Config != nil && a.CurrentScreen != "welcome" {
			path = a.Service.Config.ReadPath(path)
		} else {
			path, _ = filepath.Abs(path)
		}
	}
	for {
		if info, err := os.Stat(path); err == nil {
			if !info.IsDir() {
				path = filepath.Dir(path)
			}
			break
		}
		next := filepath.Dir(path)
		if next == path {
			path, _ = os.Getwd()
			break
		}
		path = next
	}
	picker.CurrentDirectory = path
	a.PathPicker = &pathPickerState{model: picker, directory: directory}
	return true, picker.Init()
}

func (a *App) updatePathPicker(msg tea.Msg) (tea.Model, tea.Cmd) {
	state := a.PathPicker
	if state == nil {
		return a, nil
	}
	finish := func(path string) tea.Cmd {
		if path != "" {
			switch a.CurrentScreen {
			case "welcome":
				a.WelcomeScreen.PathInput.SetValue(path)
				a.WelcomeScreen.PathInput.CursorEnd()
				a.WelcomeScreen.PathInput.Focus()
			case "content":
				field := a.ContentScreen.AddField
				a.ContentScreen.AddInputs[field].SetValue(path)
				a.ContentScreen.AddInputs[field].CursorEnd()
				a.ContentScreen.AddInputs[field].Focus()
			case "settings":
				field := a.SettingsScreen.FocusedField
				a.SettingsScreen.Inputs[field].SetValue(path)
				a.SettingsScreen.Inputs[field].CursorEnd()
				a.SettingsScreen.Inputs[field].Focus()
			case "verify":
				field := a.VerifyScreen.FocusedField
				a.VerifyScreen.Inputs[field].SetValue(path)
				a.VerifyScreen.Inputs[field].CursorEnd()
				a.VerifyScreen.Inputs[field].Focus()
			case "skills":
				a.SkillsScreen.LibraryInput.SetValue(path)
				a.SkillsScreen.LibraryInput.CursorEnd()
				a.SkillsScreen.LibraryInput.Focus()
			}
			a.Notice = "Đã chọn: " + path
		}
		a.PathPicker = nil
		return nil
	}

	if key, ok := msg.(tea.KeyMsg); ok {
		switch key.String() {
		case "esc", "ctrl+c":
			a.PathPicker = nil
			return a, nil
		case " ":
			if state.directory {
				return a, finish(state.model.CurrentDirectory)
			}
		}
	}
	if size, ok := msg.(tea.WindowSizeMsg); ok {
		state.model.Height = max(3, size.Height-8)
	}
	var cmd tea.Cmd
	state.model, cmd = state.model.Update(msg)
	if selected, path := state.model.DidSelectFile(msg); selected && !state.directory {
		return a, finish(path)
	}
	return a, cmd
}

func (s *pathPickerState) View(th theme.Theme, width, height int) string {
	lines := []string{
		"Thư mục hiện tại: " + s.model.CurrentDirectory,
		"",
	}
	lines = append(lines, strings.Split(s.model.View(), "\n")...)
	keys := "↑/↓ chọn · Enter mở thư mục/chọn file · Esc hủy"
	if s.directory {
		keys += " · Space chọn thư mục đang mở"
	}
	lines = append(lines, "", keys)
	for i := range lines {
		lines[i] = ansi.Truncate(lines[i], max(1, width-4), "…")
	}
	return components.RenderPanel(th, components.Panel{
		Title:  "Chọn file hoặc thư mục",
		Width:  width,
		Height: height,
		Lines:  lines,
	})
}

package cli

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/charmbracelet/bubbles/filepicker"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/x/ansi"
)

type tuiPicker struct {
	model     filepicker.Model
	directory bool
}

func (m *translationTUI) openPicker() tea.Cmd {
	field := &m.form.fields[m.form.index]
	if !field.browse {
		m.notice = "Ô này không phải đường dẫn file/thư mục."
		return nil
	}
	p := filepicker.New()
	p.ShowPermissions, p.ShowSize, p.ShowHidden = false, true, true
	p.AllowedTypes = field.extensions
	p.Height = max(3, m.height-7)
	p.AutoHeight = false
	path := strings.Trim(strings.TrimSpace(field.value), "\"'")
	if path == "" {
		if m.config != nil {
			path = m.config.Root
		} else {
			path, _ = os.Getwd()
		}
	} else if !filepath.IsAbs(path) {
		if m.config != nil && m.form.kind != "open" && m.form.kind != "create" && m.form.kind != "add" {
			path = m.config.ReadPath(path)
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
	p.CurrentDirectory = path
	p.FileAllowed = !field.directory
	// Enter opens folders; Space selects the current folder in directory mode.
	p.DirAllowed = false
	m.picker = &tuiPicker{model: p, directory: field.directory}
	return p.Init()
}

func (m *translationTUI) updatePicker(msg tea.Msg) tea.Cmd {
	p := m.picker
	finish := func(path string) tea.Cmd {
		field := &m.form.fields[m.form.index]
		field.value = path
		field.input.SetValue(path)
		field.input.CursorEnd()
		m.picker = nil
		return field.input.Focus()
	}
	if key, ok := msg.(tea.KeyMsg); ok {
		switch key.String() {
		case "esc", "ctrl+c":
			m.picker = nil
			return m.form.fields[m.form.index].input.Focus()
		case " ":
			if p.directory {
				return finish(p.model.CurrentDirectory)
			}
		}
	}
	if size, ok := msg.(tea.WindowSizeMsg); ok {
		p.model.Height = max(3, size.Height-7)
	}
	var cmd tea.Cmd
	p.model, cmd = p.model.Update(msg)
	if selected, path := p.model.DidSelectFile(msg); selected && !p.directory {
		return finish(path)
	}
	return cmd
}

func (p *tuiPicker) view(width, height int) string {
	text := "CHỌN FILE / THƯ MỤC\n" + p.model.CurrentDirectory + "\n\n" + p.model.View() + "\n↑/↓ chọn · Enter mở thư mục/chọn file · ← thư mục cha · Esc hủy"
	if p.directory {
		text += "\nSpace: chọn thư mục đang mở (có thể gõ thêm tên thư mục mới sau đó)"
	}
	lines := strings.Split(text, "\n")
	for i := range lines {
		lines[i] = ansi.Truncate(lines[i], max(1, width-2), "…")
	}
	return strings.Join(lines, "\n")
}

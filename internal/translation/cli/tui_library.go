package cli

import (
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func (m *translationTUI) openSharedEditor() tea.Cmd {
	c := *m.config
	return m.startOperation("Mở thư viện skill chung", c.Root, func(ctx context.Context, out, progress io.Writer) error {
		directory, err := skills.EnsureShared()
		if err != nil {
			return err
		}
		list, err := skills.SharedSkills()
		if err != nil {
			return err
		}
		if len(list) == 0 {
			return errors.New("thư viện skill chung chưa có file")
		}
		e := &tuiEditor{kind: "shared", stage: "skills", config: c, sharedDirectory: directory, sharedSkills: list}
		select {
		case progress.(tuiProgress).events <- tuiEditorLoadedMsg{e}:
		case <-ctx.Done():
			return ctx.Err()
		}
		return nil
	})
}

func (m *translationTUI) selectSharedSkill() tea.Cmd {
	e := m.editor
	data, err := os.ReadFile(e.sharedSkills[e.chapter].Path)
	if err != nil {
		m.notice = err.Error()
		return nil
	}
	text, err := prepareEditorText(string(data))
	if err != nil {
		m.notice = err.Error()
		return nil
	}
	e.stage, e.base, e.fileHash = "edit", text, domain.Hash(data)
	m.notice = "Skill chung áp dụng cho các bộ đang bật skill này; quy ước riêng vẫn ưu tiên."
	return e.initInput(m.width, m.height)
}

func (m *translationTUI) saveSharedEditor() tea.Cmd {
	e := m.editor
	directory, id, text, hash := e.sharedDirectory, e.sharedSkills[e.chapter].ID, e.input.Value(), e.fileHash
	return m.startOperation("Lưu skill chung", m.config.Root, func(ctx context.Context, out, progress io.Writer) error {
		if err := skills.SaveSharedSkill(ctx, directory, id, hash, text); err != nil {
			return err
		}
		select {
		case progress.(tuiProgress).events <- tuiEditorSavedMsg{text: text, hash: domain.Hash([]byte(text))}:
		case <-ctx.Done():
			return ctx.Err()
		}
		fmt.Fprintln(out, "Đã lưu skill chung. Các bản dịch bị ảnh hưởng cần đối chiếu bằng retranslate plan/apply.")
		return nil
	})
}

func (m *translationTUI) saveLibraryForm(values []string) func(context.Context, io.Writer, io.Writer) error {
	root := m.config.Root
	return func(ctx context.Context, out, progress io.Writer) error {
		value := strings.Trim(strings.TrimSpace(values[0]), "\"'")
		if strings.TrimSpace(value) == "" {
			return errors.New("cần thư mục skill chung")
		}
		directory, err := filepath.Abs(value)
		if err != nil {
			return err
		}
		if rel, err := filepath.Rel(root, directory); err == nil && rel != ".." && !strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
			return errors.New("hãy chọn thư mục skill chung bên ngoài workspace của bộ truyện")
		}
		if err := skills.SetSharedDirectory(ctx, directory); err != nil {
			return err
		}
		fmt.Fprintln(out, "Đã chọn thư viện skill chung cho mọi workspace dùng mặc định. Quy ước riêng vẫn nằm trong từng workspace.")
		return nil
	}
}

func (m *translationTUI) saveEnabledSkills(values []string) func(context.Context, io.Writer, io.Writer) error {
	root := m.config.Root
	return func(ctx context.Context, out, progress io.Writer) error {
		unlock, err := (state.Store{Root: root}).Lock(ctx)
		if err != nil {
			return err
		}
		defer unlock()
		c, err := workspace.Load(root)
		if err != nil {
			return err
		}
		c.EnabledSkills = nil
		for _, id := range strings.Split(values[0], ",") {
			c.EnabledSkills = append(c.EnabledSkills, strings.TrimSpace(id))
		}
		if err := c.Validate(); err != nil {
			return err
		}
		if _, err := skills.Load(c); err != nil {
			return err
		}
		if err := c.Save(); err != nil {
			return err
		}
		fmt.Fprintln(out, "Đã lưu danh sách skill chung dùng cho bộ truyện này.")
		return nil
	}
}

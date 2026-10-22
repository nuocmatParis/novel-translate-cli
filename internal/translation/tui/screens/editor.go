package screens

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/textarea"
	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type EditorScreen struct {
	Service      *app.AppService
	VolumeID     string
	Units        []domain.Block
	SelectedUnit int
	CurrentUnit  *domain.Unit
	History      []app.UnitRevision
	HistoryError string
	Note         app.UnitNote
	NoteError    string
	ActiveTab    int // 0: Compare JP-VI, 1: Vietnamese, 2: Japanese & Ruby, 3: History, 4: Notes
	TextArea     textarea.Model
	NoteArea     textarea.Model
	FocusedPane  int // 0: Tree, 1: Editor/Tab, 2: Actions
	Message      string
	Error        string
	IsDirty      bool
	NoteDirty    bool
}

func NewEditorScreen(svc *app.AppService, volumeID string) *EditorScreen {
	ta := textarea.New()
	ta.Placeholder = "Nhập nội dung bản dịch tiếng Việt..."
	ta.ShowLineNumbers = true
	noteArea := textarea.New()
	noteArea.Placeholder = "Ghi chú riêng cho khối này; không đưa vào target hoặc prompt..."
	noteArea.ShowLineNumbers = true

	es := &EditorScreen{
		Service:  svc,
		VolumeID: volumeID,
		TextArea: ta,
		NoteArea: noteArea,
	}
	return es
}

func (s *EditorScreen) Tabs() []string {
	return []string{
		"So sánh JP-VI",
		"Bản dịch (Sửa)",
		"Nguồn & Ruby",
		"Lịch sử revision",
		"Ghi chú",
	}
}

func (s *EditorScreen) LoadBlocks() {
	if s.Service == nil || s.Service.Config == nil || s.VolumeID == "" {
		return
	}
	v, err := s.Service.Config.Volume(s.VolumeID)
	if err != nil {
		return
	}
	_, book, err := engine.Source(*s.Service.Config, v, false)
	if err != nil {
		return
	}
	var eligible []domain.Block
	for _, b := range book.Blocks {
		if b.Disposition == "translate" {
			eligible = append(eligible, b)
		}
	}
	s.Units = eligible
	s.SelectUnit(s.SelectedUnit)
}

func (s *EditorScreen) SelectUnit(idx int) {
	if idx < 0 || idx >= len(s.Units) {
		return
	}
	s.SelectedUnit = idx
	block := s.Units[idx]
	note, noteErr := s.Service.QueryUnitNote(s.VolumeID, block.ID)
	s.Note = note
	s.NoteError = ""
	if noteErr != nil {
		s.NoteError = noteErr.Error()
	}
	s.NoteArea.SetValue(note.Text)
	s.NoteDirty = false

	g, err := s.Service.Store.Load(s.VolumeID)
	if err == nil {
		if u, ok := g.Units[block.ID]; ok {
			s.CurrentUnit = &u
			s.TextArea.SetValue(u.Target)
			s.IsDirty = false
			if s.ActiveTab == 3 {
				s.LoadHistory()
			}
			return
		}
	}
	s.CurrentUnit = nil
	s.TextArea.SetValue("")
	s.IsDirty = false
	if s.ActiveTab == 3 {
		s.LoadHistory()
	}
}

func (s *EditorScreen) LoadHistory() {
	s.History = nil
	s.HistoryError = ""
	if s.Service == nil || s.SelectedUnit < 0 || s.SelectedUnit >= len(s.Units) {
		return
	}
	history, err := s.Service.QueryUnitHistory(s.VolumeID, s.Units[s.SelectedUnit].ID)
	if err != nil {
		s.HistoryError = err.Error()
		return
	}
	s.History = history
}

func (s *EditorScreen) Save() error {
	if s.SelectedUnit < 0 || s.SelectedUnit >= len(s.Units) {
		return nil
	}
	if !s.IsDirty {
		s.Message = "Không có bản sửa mới để lưu."
		return nil
	}
	block := s.Units[s.SelectedUnit]
	newText := s.TextArea.Value()
	err := s.Service.SaveManualTranslation(s.VolumeID, block.ID, newText)
	if err != nil {
		s.Error = err.Error()
		return err
	}
	s.Message = "Đã lưu bản sửa tay."
	s.Error = ""
	s.IsDirty = false
	// Reload the saved translation without discarding an independent note draft.
	note := s.Note
	noteError := s.NoteError
	noteText := s.NoteArea.Value()
	noteDirty := s.NoteDirty
	s.SelectUnit(s.SelectedUnit)
	if noteDirty {
		s.Note = note
		s.NoteError = noteError
		s.NoteArea.SetValue(noteText)
		s.NoteDirty = true
	}
	return nil
}

func (s *EditorScreen) SaveNote() error {
	if s.SelectedUnit < 0 || s.SelectedUnit >= len(s.Units) {
		return fmt.Errorf("chọn khối trước khi lưu ghi chú")
	}
	if !s.NoteDirty {
		s.Message = "Không có ghi chú mới để lưu."
		return nil
	}
	block := s.Units[s.SelectedUnit]
	if err := s.Service.SaveUnitNote(s.VolumeID, block.ID, s.NoteArea.Value()); err != nil {
		s.NoteError = err.Error()
		return err
	}
	note, err := s.Service.QueryUnitNote(s.VolumeID, block.ID)
	if err != nil {
		s.NoteError = err.Error()
		return err
	}
	s.Note = note
	s.NoteDirty = false
	s.NoteError = ""
	s.Message = "Đã lưu ghi chú riêng cho khối nguồn hiện tại."
	return nil
}

func (s *EditorScreen) HasUnsavedChanges() bool {
	return s.IsDirty || s.NoteDirty
}

func (s *EditorScreen) DiscardDrafts() {
	if s.SelectedUnit >= 0 && s.SelectedUnit < len(s.Units) {
		s.SelectUnit(s.SelectedUnit)
	} else {
		s.IsDirty = false
		s.NoteDirty = false
	}
}

func (s *EditorScreen) View(theme theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)

	treeW := max(16, w/4)
	editW := max(34, w/2)
	actW := max(16, w-treeW-editW-2)
	if treeW+editW+actW+2 > w {
		editW = max(28, w-treeW-actW-2)
	}

	// Left: Block tree
	var treeLines []string
	for i, b := range s.Units {
		prefix := "  "
		if s.FocusedPane == 0 && i == s.SelectedUnit {
			prefix = "› "
		}
		label := fmt.Sprintf("%s%s (%s)", prefix, b.ID, b.Chapter)
		if s.FocusedPane == 0 && i == s.SelectedUnit {
			treeLines = append(treeLines, theme.TableRowSelected.Render(label))
		} else {
			treeLines = append(treeLines, theme.Text.Render(label))
		}
	}

	treePanel := components.RenderPanel(theme, components.Panel{
		Title:   "Cây khối nguồn",
		Focused: focused && s.FocusedPane == 0,
		Width:   treeW,
		Height:  h,
		Lines:   treeLines,
	})

	// Middle: Tabs + Editor
	var midLines []string
	tabs := s.Tabs()
	var tabBtns []string
	for i, tab := range tabs {
		if i == s.ActiveTab {
			tabBtns = append(tabBtns, theme.ButtonPrimary.Render(tab))
		} else {
			tabBtns = append(tabBtns, theme.ButtonSecondary.Render(tab))
		}
	}
	midLines = append(midLines, strings.Join(tabBtns, " "))
	midLines = append(midLines, "")

	if len(s.Units) > 0 && s.SelectedUnit < len(s.Units) {
		currentBlock := s.Units[s.SelectedUnit]
		switch s.ActiveTab {
		case 0: // So sánh JP - VI
			midLines = append(midLines, theme.PanelTitle.Render("Nguồn JP:"))
			midLines = append(midLines, theme.Text.Render(currentBlock.Source))
			midLines = append(midLines, "")
			midLines = append(midLines, theme.PanelTitle.Render("Bản dịch VI:"))
			if s.CurrentUnit != nil {
				midLines = append(midLines, theme.Text.Render(s.CurrentUnit.Target))
			} else {
				midLines = append(midLines, theme.TextMuted.Render("Chưa có bản dịch cho khối này"))
			}
		case 1: // Sửa trực tiếp
			s.TextArea.SetWidth(editW - 4)
			s.TextArea.SetHeight(max(5, h-7))
			midLines = append(midLines, s.TextArea.View())
		case 2: // Nguồn & Ruby
			midLines = append(midLines, theme.PanelTitle.Render("Nguồn đầy đủ:"))
			midLines = append(midLines, currentBlock.Source)
			midLines = append(midLines, "")
			midLines = append(midLines, theme.PanelTitle.Render(fmt.Sprintf("Ruby (%d):", len(currentBlock.Ruby))))
			for _, r := range currentBlock.Ruby {
				midLines = append(midLines, fmt.Sprintf("  Base: %s | Reading: %s", r.Base, r.Reading))
			}
		case 3: // Lịch sử
			if s.HistoryError != "" {
				midLines = append(midLines, theme.TextError.Render(cleanDisplayText(s.HistoryError)))
			}
			if len(s.History) == 0 {
				midLines = append(midLines, theme.TextMuted.Render("Khối này chưa có revision được lưu."))
			}
			for _, revision := range s.History {
				midLines = append(midLines, theme.PanelTitleFocus.Render(fmt.Sprintf("r%d · %s · %s", revision.Revision, revision.Origin, revision.CreatedAt)))
				midLines = append(midLines, theme.Text.Render(revision.Target))
				if revision.Review != nil {
					midLines = append(midLines, theme.TextMuted.Render(fmt.Sprintf("Reviewer: %s · %s · %d issue · %d ruby", revision.Review.Model, revision.Review.Decision, len(revision.Review.Issues), len(revision.Review.RubyDecisions))))
					if revision.Review.DecisionReason != "" {
						midLines = append(midLines, theme.TextMuted.Render("Lý do quyết định: "+revision.Review.DecisionReason))
					}
				}
				midLines = append(midLines, "")
			}
		case 4: // Ghi chú
			if s.Note.Stale {
				midLines = append(midLines, theme.TextWarning.Render("Ghi chú thuộc hash nguồn cũ; hãy đối chiếu trước khi dùng."))
			} else if s.Note.Text == "" {
				midLines = append(midLines, theme.TextMuted.Render("Chưa có ghi chú cho khối này."))
			} else {
				midLines = append(midLines, theme.TextMuted.Render("Ghi chú riêng · không đi vào bản dịch hoặc prompt."))
			}
			s.NoteArea.SetWidth(editW - 4)
			s.NoteArea.SetHeight(max(5, h-9))
			midLines = append(midLines, s.NoteArea.View())
			midLines = append(midLines, theme.TextMuted.Render("Ctrl+S lưu ghi chú riêng"))
			if s.NoteError != "" {
				midLines = append(midLines, theme.TextError.Render(cleanDisplayText(s.NoteError)))
			}
		}
	}

	midPanel := components.RenderPanel(theme, components.Panel{
		Title:   editorTitle(s.HasUnsavedChanges()),
		Focused: focused && s.FocusedPane == 1,
		Width:   editW,
		Height:  h,
		Lines:   midLines,
	})

	// Right: Actions
	var actLines []string
	actLines = append(actLines, theme.PanelTitleFocus.Render("THAO TÁC"))
	if s.ActiveTab == 4 {
		actLines = append(actLines, theme.ButtonPrimary.Render("[Ctrl+S] Lưu ghi chú"))
	} else {
		actLines = append(actLines, theme.ButtonPrimary.Render("[Ctrl+S] Lưu bản sửa"))
	}
	actLines = append(actLines, "")
	actLines = append(actLines, theme.ButtonSecondary.Render("[r] Review lại khối"))
	actLines = append(actLines, theme.ButtonSecondary.Render("[v] Xem reviewer"))
	actLines = append(actLines, "")
	if s.Message != "" {
		actLines = append(actLines, theme.TextSuccess.Render(s.Message))
	}
	if s.Error != "" {
		actLines = append(actLines, theme.TextError.Render(s.Error))
	}

	actPanel := components.RenderPanel(theme, components.Panel{
		Title:   "Hành động",
		Focused: focused && s.FocusedPane == 2,
		Width:   actW,
		Height:  h,
		Lines:   actLines,
	})

	return lipgloss.JoinHorizontal(lipgloss.Top, treePanel, " ", midPanel, " ", actPanel)
}

func editorTitle(dirty bool) string {
	if dirty {
		return "Nội dung bản dịch · draft chưa lưu"
	}
	return "Nội dung bản dịch"
}

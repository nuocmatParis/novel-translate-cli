package screens

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type ContentScreen struct {
	Service         *app.AppService
	Volumes         []app.VolumeSummary
	Chapters        []app.ChapterSummary
	ScopePlan       app.ScopePlan
	SelectedVolume  int
	SelectedChapter int
	CheckedChapters map[string]bool
	FocusedPane     int // 0: Volumes, 1: Chapters, 2: Scope/Actions
	SelectedMode    int // 0: continue volume, 1: selected chapters, 2: review volume
	AddingVolume    bool
	AddInputs       []textinput.Model
	AddField        int
	AddError        string
}

func NewContentScreen(svc *app.AppService) *ContentScreen {
	return &ContentScreen{
		Service:         svc,
		CheckedChapters: make(map[string]bool),
	}
}

func (s *ContentScreen) Refresh() {
	if s.Service == nil || s.Service.Config == nil {
		return
	}
	dash, _ := s.Service.QueryDashboard()
	s.Volumes = dash.Volumes

	if len(s.Volumes) > 0 {
		if s.SelectedVolume >= len(s.Volumes) {
			s.SelectedVolume = 0
		}
		volID := s.Volumes[s.SelectedVolume].ID
		chapters, _ := s.Service.QueryChapters(volID)
		s.Chapters = chapters

		var checked []string
		for _, ch := range s.Chapters {
			if s.CheckedChapters[ch.ID] {
				checked = append(checked, ch.ID)
			}
		}
		if s.SelectedMode == 1 && len(checked) == 0 {
			s.ScopePlan = app.ScopePlan{Chapters: []string{}}
		} else {
			plan, _ := s.Service.QueryScopePlan(volID, checked)
			s.ScopePlan = plan
		}
	}
}

func (s *ContentScreen) ToggleCurrentChapter() {
	if s.SelectedChapter >= 0 && s.SelectedChapter < len(s.Chapters) {
		chID := s.Chapters[s.SelectedChapter].ID
		s.CheckedChapters[chID] = !s.CheckedChapters[chID]
		s.Refresh()
	}
}

func (s *ContentScreen) StartAddVolume() {
	order := 1
	previous := ""
	if s.Service != nil && s.Service.Config != nil {
		order = len(s.Service.Config.Volumes) + 1
		if len(s.Service.Config.Volumes) > 0 {
			previous = s.Service.Config.Volumes[len(s.Service.Config.Volumes)-1].ID
		}
	}
	values := []string{fmt.Sprintf("vol%02d", order), strconv.Itoa(order), "", previous}
	s.AddInputs = make([]textinput.Model, len(values))
	for i, value := range values {
		s.AddInputs[i] = textinput.New()
		s.AddInputs[i].SetValue(value)
		s.AddInputs[i].CharLimit = 4096
	}
	s.AddField = 0
	s.AddInputs[0].Focus()
	s.AddError = ""
	s.AddingVolume = true
}

func (s *ContentScreen) CancelAddVolume() {
	s.AddingVolume = false
	s.AddError = ""
	for i := range s.AddInputs {
		s.AddInputs[i].Blur()
	}
}

func (s *ContentScreen) SaveAddedVolume() (workspace.Volume, error) {
	if s.Service == nil || s.Service.Config == nil {
		return workspace.Volume{}, fmt.Errorf("chưa mở workspace")
	}
	if len(s.AddInputs) != 4 {
		return workspace.Volume{}, fmt.Errorf("biểu mẫu thêm tập chưa sẵn sàng")
	}
	id := strings.TrimSpace(s.AddInputs[0].Value())
	order, err := strconv.Atoi(strings.TrimSpace(s.AddInputs[1].Value()))
	if err != nil || order < 1 {
		return workspace.Volume{}, fmt.Errorf("thứ tự tập phải là số nguyên dương")
	}
	source := strings.TrimSpace(s.AddInputs[2].Value())
	if source == "" {
		return workspace.Volume{}, fmt.Errorf("đường dẫn EPUB nguồn không được để trống")
	}
	previous := strings.TrimSpace(s.AddInputs[3].Value())
	if err := s.Service.AddVolume(id, source, order, previous, id); err != nil {
		return workspace.Volume{}, err
	}
	volume, err := s.Service.Config.Volume(id)
	if err != nil {
		return workspace.Volume{}, err
	}
	s.CancelAddVolume()
	s.Refresh()
	return volume, nil
}

func (s *ContentScreen) RunModes() []string {
	return []string{
		"Tiếp tục dịch toàn tập",
		"Chạy chương đã chọn",
		"Review lại cả tập",
	}
}

func (s *ContentScreen) View(theme theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)
	if s.AddingVolume {
		labels := []string{"ID tập:", "Thứ tự tập:", "Đường dẫn EPUB tiếng Nhật:", "Tập trước (có thể để trống):"}
		lines := []string{theme.TextBold.Render("ĐĂNG KÝ EPUB NGUỒN"), theme.TextMuted.Render("Chỉ thêm nguồn vào workspace; thao tác này không gọi model."), ""}
		for i, label := range labels {
			prefix := "  "
			style := theme.Text
			if i == s.AddField {
				prefix = "› "
				style = theme.TextAccent
			}
			lines = append(lines, style.Render(prefix+label))
			s.AddInputs[i].Width = max(1, w-8)
			lines = append(lines, "    "+s.AddInputs[i].View())
		}
		lines = append(lines, "", theme.TextMuted.Render("Tab: trường tiếp · Ctrl+O: chọn EPUB · Ctrl+S/Enter: thêm tập · Esc: hủy"))
		if s.AddError != "" {
			lines = append(lines, "", theme.TextError.Render(cleanDisplayText(s.AddError)))
		}
		return components.RenderPanel(theme, components.Panel{Title: "Thêm tập", Focused: focused, Width: w, Height: h, Lines: lines})
	}

	// 3 panels division
	vW := max(18, w/5)
	cW := max(24, (w*2)/5)
	sW := max(22, w-vW-cW-4)

	// 1. Volume list
	var volLines []string
	for i, v := range s.Volumes {
		prefix := "  "
		if s.FocusedPane == 0 && i == s.SelectedVolume {
			prefix = "› "
		}
		title := fmt.Sprintf("%s%s (%d/%d)", prefix, v.ID, v.Accepted, v.Eligible)
		if s.FocusedPane == 0 && i == s.SelectedVolume {
			volLines = append(volLines, theme.TableRowSelected.Render(title))
		} else {
			volLines = append(volLines, theme.Text.Render(title))
		}
	}
	vPanel := components.RenderPanel(theme, components.Panel{
		Title:   "1. Tập",
		Focused: focused && s.FocusedPane == 0,
		Width:   vW,
		Height:  h,
		Lines:   volLines,
	})

	// 2. Chapter list with checkboxes
	var chLines []string
	for i, ch := range s.Chapters {
		prefix := "  "
		if s.FocusedPane == 1 && i == s.SelectedChapter {
			prefix = "› "
		}
		checked := "[ ]"
		if s.CheckedChapters[ch.ID] {
			checked = "[x]"
		}
		row := fmt.Sprintf("%s%s %s (%d/%d)", prefix, checked, ch.Title, ch.AcceptedCount, ch.EligibleCount)
		if ch.BlockingCount > 0 {
			row += fmt.Sprintf(" [%d chặn]", ch.BlockingCount)
		}
		if s.FocusedPane == 1 && i == s.SelectedChapter {
			chLines = append(chLines, theme.TableRowSelected.Render(row))
		} else {
			chLines = append(chLines, theme.Text.Render(row))
		}
	}
	cPanel := components.RenderPanel(theme, components.Panel{
		Title:   "2. Chương (Space chọn range liên tiếp)",
		Focused: focused && s.FocusedPane == 1,
		Width:   cW,
		Height:  h,
		Lines:   chLines,
	})

	// 3. Scope & Actions
	var scopeLines []string
	scopeLines = append(scopeLines, theme.PanelTitleFocus.Render("CHẾ ĐỘ CHẠY"))
	modes := s.RunModes()
	for i, m := range modes {
		if s.FocusedPane == 2 && i == s.SelectedMode {
			scopeLines = append(scopeLines, theme.ButtonPrimary.Render(m))
		} else {
			scopeLines = append(scopeLines, theme.ButtonSecondary.Render(m))
		}
	}
	scopeLines = append(scopeLines, "")

	scopeLines = append(scopeLines, theme.PanelTitle.Render("DỰ KIẾN PHẠM VI"))
	if s.SelectedMode == 2 {
		scopeLines = append(scopeLines, theme.Text.Render("Review lại các revision đã lưu trong tập."))
	} else {
		scopeLines = append(scopeLines, fmt.Sprintf("Tổng khối trong phạm vi: %d", s.ScopePlan.TotalUnits))
		scopeLines = append(scopeLines, fmt.Sprintf("Sẽ dịch mới: %d", s.ScopePlan.UnitsToTranslate))
		scopeLines = append(scopeLines, fmt.Sprintf("Sẽ review: %d", s.ScopePlan.UnitsToReview))
		scopeLines = append(scopeLines, fmt.Sprintf("Bỏ qua vì đã đạt: %d", s.ScopePlan.UnitsSkippedAccepted))
		if s.ScopePlan.UnitsNeedingAttention > 0 {
			scopeLines = append(scopeLines, theme.TextError.Render(fmt.Sprintf("Cần xử lý trước: %d khối chặn", s.ScopePlan.UnitsNeedingAttention)))
		}
	}
	if s.SelectedVolume >= 0 && s.SelectedVolume < len(s.Volumes) && !s.Volumes[s.SelectedVolume].Imported {
		scopeLines = append(scopeLines, "", theme.TextWarning.Render("Tập chưa import snapshot nguồn."))
		scopeLines = append(scopeLines, theme.ButtonPrimary.Render("[ i ] Import EPUB nguồn"))
	}
	if s.ScopePlan.HasContextGap {
		reason := s.ScopePlan.ContextGapReason
		if reason == "" {
			reason = "Có khoảng trống ngữ cảnh trước phạm vi này."
		}
		scopeLines = append(scopeLines, "", theme.TextWarning.Render("Lưu ý: "+reason))
	}
	if s.SelectedMode == 2 {
		scopeLines = append(scopeLines, "", theme.TextMuted.Render("Chỉ review bản dịch đã lưu; không dịch block thiếu target và không tự repair."))
	}

	scopeLines = append(scopeLines, "")
	if s.SelectedVolume >= 0 && s.SelectedVolume < len(s.Volumes) && !s.Volumes[s.SelectedVolume].Imported {
		scopeLines = append(scopeLines, theme.ButtonDisabled.Render("[ Bắt đầu: khóa đến khi import ]"))
	} else {
		scopeLines = append(scopeLines, theme.ButtonPrimary.Render("[ Bắt đầu chạy phạm vi (Enter) ]"))
	}

	sPanel := components.RenderPanel(theme, components.Panel{
		Title:   "3. Thiết lập phạm vi",
		Focused: focused && s.FocusedPane == 2,
		Width:   sW,
		Height:  h,
		Lines:   scopeLines,
	})

	return lipgloss.JoinHorizontal(lipgloss.Top, vPanel, " ", cPanel, " ", sPanel)
}

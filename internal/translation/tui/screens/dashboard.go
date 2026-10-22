package screens

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type DashboardScreen struct {
	Service        *app.AppService
	Projection     app.DashboardProjection
	SelectedAction int
	SelectedVolume int
	FocusedPane    int // 0: cover, details and actions; 1: volume list
	ActiveVolumeID string
	ActiveJobLabel string
	ActiveJobPhase string
	Cover          app.VolumeCover
	CoverPreview   string
	CoverMessage   string
	CoverLoading   bool
	RefreshError   string
}

func NewDashboardScreen(svc *app.AppService) *DashboardScreen {
	s := &DashboardScreen{Service: svc}
	s.Refresh()
	return s
}

func (s *DashboardScreen) Refresh() {
	if s.Service == nil {
		return
	}
	selectedID := ""
	if s.SelectedVolume >= 0 && s.SelectedVolume < len(s.Projection.Volumes) {
		selectedID = s.Projection.Volumes[s.SelectedVolume].ID
	}
	projection, err := s.Service.QueryDashboard()
	if err != nil {
		s.RefreshError = err.Error()
		return
	}
	s.Projection = projection
	s.RefreshError = ""
	if selectedID != "" {
		for i, volume := range projection.Volumes {
			if volume.ID == selectedID {
				s.SelectedVolume = i
				return
			}
		}
	}
	if len(projection.Volumes) == 0 {
		s.SelectedVolume = 0
	} else if s.SelectedVolume < 0 || s.SelectedVolume >= len(projection.Volumes) {
		s.SelectedVolume = 0
	}
}

func (s *DashboardScreen) Actions() []string {
	return []string{
		"Tiếp tục dịch",
		"Chọn phạm vi",
		"Mở bản dịch",
		"Xem lỗi review",
		"Hình ảnh & xuất",
		"Báo cáo",
	}
}

func (s *DashboardScreen) SelectedVolumeSummary() (app.VolumeSummary, bool) {
	if len(s.Projection.Volumes) == 0 {
		return app.VolumeSummary{}, false
	}
	if s.ActiveVolumeID != "" {
		for _, volume := range s.Projection.Volumes {
			if volume.ID == s.ActiveVolumeID {
				return volume, true
			}
		}
	}
	if s.SelectedVolume < 0 || s.SelectedVolume >= len(s.Projection.Volumes) {
		return app.VolumeSummary{}, false
	}
	return s.Projection.Volumes[s.SelectedVolume], true
}

func (s *DashboardScreen) View(th theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)

	mainW := max(40, w/2)
	volW := max(30, w-mainW-1)
	if mainW+volW+1 > w {
		mainW = w - volW - 1
	}
	if mainW < 36 {
		mainW = 36
		volW = max(24, w-mainW-1)
	}

	mainLines := s.bookCardLines(th, mainW-4, h-2)
	mainPanel := components.RenderPanel(th, components.Panel{
		Title:   "Bộ truyện & tập đang mở",
		Focused: focused && s.FocusedPane == 0,
		Width:   mainW,
		Height:  h,
		Lines:   mainLines,
	})

	volumeLines := s.volumeLines(th, volW-4, h-2)
	volumePanel := components.RenderPanel(th, components.Panel{
		Title:   "Tiến độ từng tập",
		Focused: focused && s.FocusedPane == 1,
		Width:   volW,
		Height:  h,
		Lines:   volumeLines,
	})

	return lipgloss.JoinHorizontal(lipgloss.Top, mainPanel, " ", volumePanel)
}

func (s *DashboardScreen) bookCardLines(th theme.Theme, width, height int) []string {
	var lines []string
	series := cleanDisplayText(s.Projection.SeriesTitle)
	if series == "" {
		series = "Chưa mở workspace"
	}
	lines = append(lines, th.TextBold.Render(series))

	volume, ok := s.SelectedVolumeSummary()
	if !ok {
		lines = append(lines, "", th.TextMuted.Render("Workspace chưa có tập nào."), th.TextMuted.Render("Dùng [a] để đăng ký EPUB nguồn."))
		lines = append(lines, "")
		lines = append(lines, s.actionLines(th, width)...)
		return lines
	}

	title := cleanDisplayText(s.Cover.DisplayTitle())
	titleSource := s.Cover.TitleSource
	if title == "" {
		title = cleanDisplayText(volume.BookTitle)
		if title != "" {
			titleSource = "metadata EPUB"
		}
	}
	if title == "" {
		title = cleanDisplayText(filepath.Base(volume.SourcePath))
		titleSource = "tên file"
	}
	if title == "." || title == "" {
		title = "Chưa có tên tập"
		titleSource = "chưa xác định"
	}

	coverRows := min(9, max(5, height/2))
	coverColumns := min(17, max(8, width/3))
	coverLines := s.coverLines(th, coverColumns, coverRows)
	detailLines := []string{
		th.TextAccent.Render(fmt.Sprintf("Tập %02d · %s", volume.Order, cleanDisplayText(volume.ID))),
		th.TextBold.Render(ansi.Truncate(title, max(12, width-coverColumns-3), "…")),
		th.TextMuted.Render("Tên lấy từ: " + titleSource),
		"",
		th.TextMuted.Render("Nguồn EPUB:"),
		th.Text.Render(ansi.Truncate(cleanDisplayText(volume.SourcePath), max(12, width-coverColumns-3), "…")),
		fmt.Sprintf("%d chương · %d ảnh", volume.ChaptersCount, volume.Images),
	}
	if s.ActiveJobLabel != "" && s.ActiveVolumeID == volume.ID {
		phase := cleanDisplayText(s.ActiveJobPhase)
		if phase == "" {
			phase = "đang chạy"
		}
		detailLines = append(detailLines, "", th.TextAccent.Render(s.ActiveJobLabel), th.TextMuted.Render("Giai đoạn: "+phase))
	} else if !volume.Imported {
		detailLines = append(detailLines, "", th.TextWarning.Render("Chưa import nguồn EPUB"))
	} else if volume.FreshnessError != "" {
		detailLines = append(detailLines, "", th.TextError.Render("Nguồn/quy ước cần đối chiếu lại"))
	} else {
		detailLines = append(detailLines, "", th.TextSuccess.Render(fmt.Sprintf("Đã đối chiếu: %d / %d", volume.Accepted, volume.Eligible)))
	}
	if s.Cover.SnapshotStale {
		detailLines = append(detailLines, th.TextWarning.Render("Bìa thuộc snapshot đã import; file nguồn đã đổi."))
	}

	coverBlock := strings.Join(coverLines, "\n")
	detailBlock := strings.Join(detailLines, "\n")
	card := lipgloss.JoinHorizontal(lipgloss.Top,
		lipgloss.NewStyle().Width(coverColumns).Render(coverBlock),
		" ",
		lipgloss.NewStyle().Width(max(12, width-coverColumns-2)).Render(detailBlock),
	)
	lines = append(lines, strings.Split(card, "\n")...)
	if volume.LastSaveTime != "" {
		lines = append(lines, th.TextMuted.Render("Lần lưu gần nhất: "+volume.LastSaveTime))
	}
	if s.Projection.Totals.TotalEligible > 0 {
		lines = append(lines, th.TextMuted.Render(fmt.Sprintf("Workspace: %d / %d khối đã đối chiếu · %d stale", s.Projection.Totals.TotalAccepted, s.Projection.Totals.TotalEligible, s.Projection.Totals.TotalStale)))
	}
	if s.Projection.BlockingIssuesCount+s.Projection.ImageIssuesCount+s.Projection.RuleConflictCount > 0 {
		lines = append(lines, th.TextWarning.Render(fmt.Sprintf("Cần xử lý: %d issue · %d ảnh · %d quy ước", s.Projection.BlockingIssuesCount, s.Projection.ImageIssuesCount, s.Projection.RuleConflictCount)))
	}
	if s.RefreshError != "" {
		lines = append(lines, th.TextError.Render("Không tải được dashboard: "+cleanDisplayText(s.RefreshError)))
	}
	lines = append(lines, "")
	lines = append(lines, s.actionLines(th, width)...)
	return lines
}

func (s *DashboardScreen) coverLines(th theme.Theme, columns, rows int) []string {
	if s.CoverLoading {
		return []string{th.TextMuted.Render("Đang tải"), th.TextMuted.Render("bìa…")}
	}
	if s.CoverPreview != "" {
		return strings.Split(s.CoverPreview, "\n")
	}
	label := "Chưa có ảnh bìa"
	if s.CoverMessage != "" {
		label = cleanDisplayText(s.CoverMessage)
	}
	lines := make([]string, max(2, min(rows, 5)))
	for i := range lines {
		lines[i] = strings.Repeat("·", columns)
	}
	if len(lines) > 0 {
		lines[0] = th.TextMuted.Render(ansi.Truncate(label, columns, "…"))
	}
	return lines
}

func (s *DashboardScreen) actionLines(th theme.Theme, width int) []string {
	labels := s.Actions()
	var lines []string
	for i := 0; i < len(labels); i += 2 {
		left := fmt.Sprintf("%d %s", i+1, labels[i])
		right := ""
		if i+1 < len(labels) {
			right = fmt.Sprintf("%d %s", i+2, labels[i+1])
		}
		leftW := max(12, (width-1)/2)
		style := th.TextMuted
		if s.FocusedPane == 0 && s.SelectedAction == i {
			style = th.TextAccent.Bold(true)
		}
		leftCell := style.Width(leftW).Render(ansi.Truncate(left, leftW, "…"))
		rightW := max(1, width-leftW-1)
		rightStyle := th.TextMuted
		if s.FocusedPane == 0 && s.SelectedAction == i+1 {
			rightStyle = th.TextAccent.Bold(true)
		}
		rightCell := rightStyle.Width(rightW).Render(ansi.Truncate(right, rightW, "…"))
		lines = append(lines, leftCell+" "+rightCell)
	}
	if width >= 34 {
		lines = append(lines, th.TextMuted.Render("Enter mở · Tab chuyển vùng · ? trợ giúp"))
	}
	return lines
}

func (s *DashboardScreen) volumeLines(th theme.Theme, width, height int) []string {
	volumes := s.Projection.Volumes
	if len(volumes) == 0 {
		return []string{th.TextMuted.Render("Chưa có tập nào."), th.TextMuted.Render("Đăng ký EPUB bằng [a].")}
	}
	visible := max(1, height/4)
	start := max(0, min(s.SelectedVolume-visible/2, len(volumes)-visible))
	var lines []string
	for i := start; i < min(len(volumes), start+visible); i++ {
		v := volumes[i]
		prefix := "  "
		style := th.TextBold
		if s.FocusedPane == 1 && i == s.SelectedVolume {
			prefix = "› "
			style = th.TextAccent.Bold(true)
		}
		label := fmt.Sprintf("%sTập %02d · %s", prefix, v.Order, cleanDisplayText(v.ID))
		if v.BookTitle != "" {
			label += " · " + cleanDisplayText(v.BookTitle)
		}
		lines = append(lines, style.Render(ansi.Truncate(label, width, "…")))
		lines = append(lines, components.RenderProgressBar(th, v.Accepted, v.Eligible, max(8, width-2)))
		state := "Chưa dịch"
		switch {
		case !v.Imported:
			state = "Chưa import EPUB"
		case v.FreshnessError != "":
			state = "Cần đối chiếu lại"
		case v.Eligible > 0 && v.Accepted == v.Eligible:
			state = "Đã đối chiếu"
		case v.Translated > 0:
			state = "Đang dịch / chờ review"
		}
		if s.ActiveVolumeID == v.ID && s.ActiveJobLabel != "" {
			state = s.ActiveJobLabel
		}
		lines = append(lines, th.TextMuted.Render(ansi.Truncate("  "+state, width, "…")), "")
	}
	if len(volumes) > visible {
		lines = append(lines, th.TextMuted.Render(fmt.Sprintf("↑/↓ chọn tập · %d tập", len(volumes))))
	}
	return lines
}

package screens

import (
	"fmt"

	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/progress"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type JobScreen struct {
	JobManager   *app.JobManager
	LastEvent    *progress.Event
	AutoFollow   bool
	SpinnerIdx   int
	FocusedPane  int
	CurrentBlock *domain.Block
	CurrentUnit  *domain.Unit
}

var spinnerFrames = []string{"|", "/", "-", "\\"}

func NewJobScreen(jm *app.JobManager) *JobScreen {
	return &JobScreen{
		JobManager: jm,
		AutoFollow: true,
	}
}

func (s *JobScreen) Tick() {
	s.SpinnerIdx = (s.SpinnerIdx + 1) % len(spinnerFrames)
}

func (s *JobScreen) View(theme theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)

	status := app.JobStatus{State: app.StateIdle, Label: "Không có tác vụ đang chạy"}
	if s.JobManager != nil {
		status = s.JobManager.Status()
	}

	leftW := max(22, w/3)
	midW := max(24, w/3)
	rightW := max(20, w-leftW-midW-4)

	// Left: Progress & Phases
	var leftLines []string
	leftLines = append(leftLines, theme.TextBold.Render("Tác vụ: "+status.Label))
	stateBadge := string(status.State)
	if status.State == app.StateRunning {
		spinner := spinnerFrames[s.SpinnerIdx]
		leftLines = append(leftLines, theme.TextAccent.Render(fmt.Sprintf("Trạng thái: Đang chạy %s", spinner)))
	} else if status.State == app.StatePaused {
		leftLines = append(leftLines, theme.TextWarning.Render("Trạng thái: ĐÃ TẠM DỪNG"))
	} else {
		leftLines = append(leftLines, theme.TextMuted.Render("Trạng thái: "+stateBadge))
	}

	leftLines = append(leftLines, "")
	leftLines = append(leftLines, theme.PanelTitle.Render("GIAI ĐOẠN HIỆN TẠI"))
	leftLines = append(leftLines, "Phase: "+status.Phase)
	leftLines = append(leftLines, "Khối đang chạy: "+status.ActiveUnit)
	if s.LastEvent != nil && s.LastEvent.Role != "" {
		leftLines = append(leftLines, "Role: "+s.LastEvent.Role)
	}
	leftLines = append(leftLines, "Thông báo: "+status.Message)
	leftLines = append(leftLines, "")
	leftLines = append(leftLines, fmt.Sprintf("Thời gian trôi qua: %v", status.Elapsed.Truncate(100000000)))

	leftLines = append(leftLines, "")
	leftLines = append(leftLines, theme.PanelTitle.Render("ĐIỀU KHIỂN TÁC VỤ"))
	if status.Kind == "translate" {
		if status.State == app.StatePaused {
			leftLines = append(leftLines, theme.ButtonPrimary.Render("[p] Tiếp tục chạy"))
		} else if status.State == app.StateRunning {
			leftLines = append(leftLines, theme.ButtonSecondary.Render("[p] Tạm dừng tại ranh giới"))
		}
	} else if status.State == app.StateRunning {
		leftLines = append(leftLines, theme.TextMuted.Render("Tác vụ dừng bằng Ctrl+C; hiện không có pause boundary."))
	}
	leftLines = append(leftLines, theme.ButtonSecondary.Render("[Ctrl+C] Dừng tác vụ"))

	leftPanel := components.RenderPanel(theme, components.Panel{
		Title:   "Tiến độ tác vụ",
		Focused: focused && s.FocusedPane == 0,
		Width:   leftW,
		Height:  h,
		Lines:   leftLines,
	})

	// Middle: JP Source & VI target
	var midLines []string
	midLines = append(midLines, theme.PanelTitleFocus.Render("NGUỒN NHẬT (JP)"))
	if s.LastEvent != nil && s.LastEvent.UnitID != "" {
		midLines = append(midLines, theme.Text.Render(fmt.Sprintf("Đang xử lý khối: %s", s.LastEvent.UnitID)))
		if s.CurrentBlock != nil && s.CurrentBlock.ID == s.LastEvent.UnitID {
			midLines = append(midLines, theme.Text.Render(s.CurrentBlock.Source))
		}
	} else {
		midLines = append(midLines, theme.TextMuted.Render("Đang chờ thông tin khối..."))
	}

	midLines = append(midLines, "")
	midLines = append(midLines, theme.PanelTitleFocus.Render("BẢN DỊCH VIỆT (VI)"))
	if s.CurrentUnit != nil {
		midLines = append(midLines, theme.Text.Render(s.CurrentUnit.Target))
	} else if status.State == app.StateRunning && status.Kind == "translate" {
		spinner := spinnerFrames[s.SpinnerIdx]
		midLines = append(midLines, theme.TextMuted.Render(fmt.Sprintf("Đang chờ phản hồi từ model %s…", spinner)))
	} else if status.State == app.StateRunning {
		midLines = append(midLines, theme.TextMuted.Render(status.Message))
	} else {
		midLines = append(midLines, theme.TextMuted.Render("Chưa có bản lưu mới trong đợt này."))
	}

	midPanel := components.RenderPanel(theme, components.Panel{
		Title:   "Nội dung đang xử lý",
		Focused: focused && s.FocusedPane == 1,
		Width:   midW,
		Height:  h,
		Lines:   midLines,
	})

	// Right: Model / Rule info / Log events
	var rightLines []string
	rightLines = append(rightLines, theme.PanelTitle.Render("THÔNG TIN MODEL & QUY TẮC"))
	rightLines = append(rightLines, "Model: "+status.ActiveModel)
	if s.LastEvent != nil && s.LastEvent.Attempt > 0 {
		rightLines = append(rightLines, fmt.Sprintf("Attempt: %d", s.LastEvent.Attempt))
	}
	rightLines = append(rightLines, "")
	rightLines = append(rightLines, theme.PanelTitle.Render("NHẬT KÝ SỰ KIỆN GẦN NHẤT"))
	if s.JobManager != nil {
		logs := s.JobManager.Logs()
		start := max(0, len(logs)-(h-8))
		for _, l := range logs[start:] {
			rightLines = append(rightLines, theme.TextMuted.Render(l))
		}
	}

	rightPanel := components.RenderPanel(theme, components.Panel{
		Title:   "Nhật ký & Model",
		Focused: focused && s.FocusedPane == 2,
		Width:   rightW,
		Height:  h,
		Lines:   rightLines,
	})

	return lipgloss.JoinHorizontal(lipgloss.Top, leftPanel, " ", midPanel, " ", rightPanel)
}

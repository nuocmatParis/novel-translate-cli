package components

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

func RenderBadge(th theme.Theme, status string) string {
	switch status {
	case "Đã đối chiếu":
		return th.BadgeAccepted.Render(" ĐÃ ĐỐI CHIẾU ")
	case "Người dùng giữ":
		return th.BadgeUserKept.Render(" NGƯỜI DÙNG GIỮ ")
	case "Cần xử lý":
		return th.BadgeBlocking.Render(" CẦN XỬ LÝ ")
	case "Chờ review":
		return th.BadgePending.Render(" CHỜ REVIEW ")
	case "Cần đối chiếu lại":
		return th.BadgeStale.Render(" CẦN ĐỐI CHIẾU LẠI ")
	case "Có lưu ý":
		return th.BadgeWarning.Render(" CÓ LƯU Ý ")
	case "Đang dịch":
		return th.BadgeUserKept.Render(" ĐANG DỊCH ")
	case "Đang review":
		return th.BadgeUserKept.Render(" ĐANG REVIEW ")
	case "Đang sửa":
		return th.BadgeUserKept.Render(" ĐANG SỬA ")
	default:
		return th.BadgePending.Render(" " + status + " ")
	}
}

func RenderProgressBar(th theme.Theme, completed, total int, width int) string {
	if total <= 0 {
		return th.TextMuted.Render("không có nội dung cần dịch")
	}

	percent := float64(completed) / float64(total) * 100
	if percent > 100 {
		percent = 100
	}

	label := fmt.Sprintf(" %d/%d (%.1f%%)", completed, total, percent)
	barWidth := max(5, width-len(label))

	filledWidth := int(float64(barWidth) * float64(completed) / float64(total))
	if filledWidth > barWidth {
		filledWidth = barWidth
	}
	emptyWidth := barWidth - filledWidth

	filled := lipgloss.NewStyle().Foreground(lipgloss.Color(theme.ColorAccent)).Render(strings.Repeat("█", filledWidth))
	empty := lipgloss.NewStyle().Foreground(lipgloss.Color(theme.ColorBorder)).Render(strings.Repeat("░", emptyWidth))

	return filled + empty + th.TextMuted.Render(label)
}

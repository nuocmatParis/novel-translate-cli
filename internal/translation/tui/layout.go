package tui

import (
	"fmt"
	"time"

	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type TerminalSizeTier int

const (
	SizeCompact TerminalSizeTier = iota // < 80x24
	SizeNormal                          // 80-109 x 24+
	SizeMedium                          // 110-139 x 32+
	SizeWide                            // >= 140x42
)

type Layout struct {
	Width         int
	Height        int
	Tier          TerminalSizeTier
	SidebarWidth  int
	ContentWidth  int
	ContentHeight int
}

func CalculateLayout(width, height int) Layout {
	if width < 80 || height < 24 {
		return Layout{
			Width:         width,
			Height:        height,
			Tier:          SizeCompact,
			SidebarWidth:  0,
			ContentWidth:  max(1, width-2),
			ContentHeight: max(1, height-4),
		}
	}

	tier := SizeNormal
	sbWidth := 0
	if width >= 140 && height >= 42 {
		tier = SizeWide
		sbWidth = 26
	} else if width >= 110 && height >= 32 {
		tier = SizeMedium
		sbWidth = 22
	}

	contentWidth := width
	if sbWidth > 0 {
		contentWidth = max(1, width-sbWidth-2)
	}
	contentHeight := max(1, height-4) // 2 lines header + 2 lines footer

	return Layout{
		Width:         width,
		Height:        height,
		Tier:          tier,
		SidebarWidth:  sbWidth,
		ContentWidth:  contentWidth,
		ContentHeight: contentHeight,
	}
}

func RenderHeader(th theme.Theme, width int, appName, seriesTitle, screenName, jobStatus string) string {
	w := max(1, width)
	left := fmt.Sprintf("%s · %s", appName, screenName)
	if seriesTitle != "" {
		left = fmt.Sprintf("%s · %s · %s", appName, seriesTitle, screenName)
	}

	clock := time.Now().Format("15:04:05")
	right := clock
	if jobStatus != "" {
		right = fmt.Sprintf("[%s] %s", jobStatus, clock)
	}

	avail := max(1, w-ansi.StringWidth(right)-2)
	truncatedLeft := ansi.Truncate(left, avail, "…")
	gap := max(1, w-ansi.StringWidth(truncatedLeft)-ansi.StringWidth(right)-2)

	line1 := fmt.Sprintf(" %s%s%s ", truncatedLeft, lipgloss.NewStyle().Width(gap).Render(""), right)
	border := lipgloss.NewStyle().
		Foreground(lipgloss.Color(theme.ColorBorder)).
		Render(lipgloss.NewStyle().Width(w).Render(repeatRune('─', w)))

	return th.Header.Width(w).Render(line1) + "\n" + border
}

func RenderFooter(th theme.Theme, width int, notice string, errorStr string, keys string) string {
	w := max(1, width)
	var line1 string
	if errorStr != "" {
		line1 = th.TextError.Render(" Lỗi: " + ansi.Truncate(errorStr, w-8, "…"))
	} else if notice != "" {
		line1 = th.FooterNotice.Render(" " + ansi.Truncate(notice, w-2, "…"))
	} else {
		line1 = th.FooterNotice.Render(" Sẵn sàng")
	}

	line2 := " " + ansi.Truncate(keys, w-2, "…")
	styledLine2 := th.FooterKeys.Render(line2)

	return line1 + "\n" + styledLine2
}

func repeatRune(r rune, n int) string {
	if n <= 0 {
		return ""
	}
	b := make([]rune, n)
	for i := range b {
		b[i] = r
	}
	return string(b)
}

package components

import (
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

func HighlightQuote(th theme.Theme, fullText, quote string) string {
	if quote == "" || !strings.Contains(fullText, quote) {
		return th.Text.Render(fullText)
	}

	parts := strings.Split(fullText, quote)
	highlighted := th.QuoteHighlight.Render(quote)

	var res []string
	for i, part := range parts {
		res = append(res, th.Text.Render(part))
		if i < len(parts)-1 {
			res = append(res, highlighted)
		}
	}
	return strings.Join(res, "")
}

func RenderSideBySide(th theme.Theme, leftTitle, leftText, rightTitle, rightText string, width, height int) string {
	w := max(10, (width-4)/2)
	h := max(3, height)

	leftPanel := RenderPanel(th, Panel{
		Title:  leftTitle,
		Width:  w,
		Height: h,
		Lines:  strings.Split(leftText, "\n"),
	})

	rightPanel := RenderPanel(th, Panel{
		Title:  rightTitle,
		Width:  w,
		Height: h,
		Lines:  strings.Split(rightText, "\n"),
	})

	return lipgloss.JoinHorizontal(lipgloss.Top, leftPanel, "  ", rightPanel)
}

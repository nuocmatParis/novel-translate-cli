package components

import (
	"strings"

	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type Panel struct {
	Title   string
	Focused bool
	Width   int
	Height  int
	Scroll  int
	Lines   []string
}

func RenderPanel(th theme.Theme, p Panel) string {
	w := max(10, p.Width)
	h := max(3, p.Height)

	style := th.Panel
	titleStyle := th.PanelTitle
	if p.Focused {
		style = th.PanelFocused
		titleStyle = th.PanelTitleFocus
	}

	titleText := ""
	if p.Title != "" {
		titleText = " " + p.Title + " "
	}
	renderedTitle := titleStyle.Render(titleText)

	// Inner dimensions
	innerW := max(1, w-2)
	innerH := max(1, h-2)

	visibleLines := make([]string, innerH)
	start := min(p.Scroll, max(0, len(p.Lines)-innerH))
	if start < 0 {
		start = 0
	}

	for i := 0; i < innerH; i++ {
		lineIdx := start + i
		if lineIdx < len(p.Lines) {
			visibleLines[i] = ansi.Truncate(p.Lines[lineIdx], innerW, "…")
		} else {
			visibleLines[i] = ""
		}
	}

	content := strings.Join(visibleLines, "\n")
	box := style.Width(innerW).Height(innerH).Render(content)

	// Inject title into top border if present
	if titleText != "" && len(box) > 0 {
		// Box top border line replacement
		boxLines := strings.Split(box, "\n")
		if len(boxLines) > 0 {
			top := boxLines[0]
			topR := []rune(top)
			titleR := []rune(renderedTitle)
			if len(topR) > len(titleR)+4 {
				boxLines[0] = string(topR[:2]) + renderedTitle + string(topR[2+len([]rune(titleText)):])
				box = strings.Join(boxLines, "\n")
			}
		}
	}

	return box
}

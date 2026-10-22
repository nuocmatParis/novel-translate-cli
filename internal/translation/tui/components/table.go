package components

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type Column struct {
	Title string
	Width int
}

type Table struct {
	Columns  []Column
	Rows     [][]string
	Selected int
	Scroll   int
}

func RenderTable(th theme.Theme, t Table, width, height int, focused bool) string {
	w := max(1, width)
	h := max(1, height)

	var headerCells []string
	totalColWidth := 0
	for _, col := range t.Columns {
		cw := max(1, col.Width)
		totalColWidth += cw
		cell := ansi.Truncate(col.Title, cw, "…")
		headerCells = append(headerCells, lipgloss.NewStyle().Width(cw).Bold(true).Foreground(lipgloss.Color(theme.ColorMuted)).Render(cell))
	}
	headerLine := strings.Join(headerCells, " ")

	divider := lipgloss.NewStyle().Foreground(lipgloss.Color(theme.ColorBorder)).Render(strings.Repeat("─", min(w, totalColWidth+len(t.Columns))))

	dataHeight := max(1, h-2)
	start := max(0, min(t.Selected-dataHeight/2, len(t.Rows)-dataHeight))
	if start < 0 {
		start = 0
	}

	var rowLines []string
	for i := 0; i < dataHeight; i++ {
		rowIdx := start + i
		if rowIdx >= len(t.Rows) {
			break
		}
		row := t.Rows[rowIdx]
		var cells []string
		for colIdx, col := range t.Columns {
			val := ""
			if colIdx < len(row) {
				val = row[colIdx]
			}
			cw := max(1, col.Width)
			cell := ansi.Truncate(val, cw, "…")
			cells = append(cells, lipgloss.NewStyle().Width(cw).Render(cell))
		}
		rowContent := strings.Join(cells, " ")
		if rowIdx == t.Selected {
			if focused {
				rowContent = th.TableRowSelected.Width(w).Render("› " + rowContent)
			} else {
				rowContent = lipgloss.NewStyle().Background(lipgloss.Color("#0E2331")).Width(w).Render("  " + rowContent)
			}
		} else {
			rowContent = "  " + rowContent
		}
		rowLines = append(rowLines, ansi.Truncate(rowContent, w, "…"))
	}

	lines := []string{headerLine, divider}
	lines = append(lines, rowLines...)
	return strings.Join(lines, "\n")
}

func SimpleRow(values ...any) []string {
	res := make([]string, len(values))
	for i, v := range values {
		res[i] = fmt.Sprintf("%v", v)
	}
	return res
}

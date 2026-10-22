package screens

import (
	"fmt"
	"sort"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type ContinuityScreen struct {
	Service     *app.AppService
	VolumeID    string
	Events      []domain.Event
	SelectedIdx int
	FocusedPane int
}

func NewContinuityScreen(svc *app.AppService, volumeID string) *ContinuityScreen {
	cs := &ContinuityScreen{
		Service:  svc,
		VolumeID: volumeID,
	}
	return cs
}

func (s *ContinuityScreen) Refresh() {
	if s.Service == nil || s.VolumeID == "" {
		return
	}
	g, err := s.Service.Store.Load(s.VolumeID)
	if err == nil {
		var evs []domain.Event
		for _, u := range g.Units {
			evs = append(evs, u.Events...)
		}
		sort.SliceStable(evs, func(i, j int) bool {
			return evs[i].Position.Before(evs[j].Position)
		})
		s.Events = evs
		s.SelectedIdx = min(max(0, s.SelectedIdx), max(0, len(s.Events)-1))
	}
}

func (s *ContinuityScreen) View(theme theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)

	leftW := max(30, (w*3)/5)
	rightW := max(20, w-leftW-2)

	// Left: Table of directed pronoun / honorific pairs
	var tableLines []string
	tableLines = append(tableLines, theme.TextMuted.Render("Người nói       → Người nghe      | Cách xưng hô       | Vị trí"))
	tableLines = append(tableLines, strings.Repeat("─", leftW-4))

	if len(s.Events) == 0 {
		tableLines = append(tableLines, theme.TextMuted.Render("Chưa có sự kiện ngữ cảnh / cặp xưng hô nào được ghi nhận."))
	} else {
		for i, ev := range s.Events {
			prefix := "  "
			if s.FocusedPane == 0 && i == s.SelectedIdx {
				prefix = "› "
			}
			row := fmt.Sprintf("%s%-15s → %-15s | %-18s | Ch.%d:Bl.%d",
				prefix, ev.Speaker, ev.Listener, ev.Value, ev.Position.Chapter, ev.Position.Block)
			if s.FocusedPane == 0 && i == s.SelectedIdx {
				tableLines = append(tableLines, theme.TableRowSelected.Render(row))
			} else {
				tableLines = append(tableLines, theme.Text.Render(row))
			}
		}
	}

	leftPanel := components.RenderPanel(theme, components.Panel{
		Title:   "Cặp xưng hô & Ngữ cảnh nhân vật (A → B khác B → A)",
		Focused: focused && s.FocusedPane == 0,
		Width:   leftW,
		Height:  h,
		Scroll:  max(0, s.SelectedIdx-(h-4)/2),
		Lines:   tableLines,
	})

	// Right: Detail & Evidence
	var rightLines []string
	rightLines = append(rightLines, theme.PanelTitle.Render("BẰNG CHỨNG NGUỒN (EVIDENCE)"))
	if len(s.Events) > 0 && s.SelectedIdx < len(s.Events) {
		ev := s.Events[s.SelectedIdx]
		rightLines = append(rightLines, fmt.Sprintf("Người nói: %s", ev.Speaker))
		rightLines = append(rightLines, fmt.Sprintf("Người nghe: %s", ev.Listener))
		rightLines = append(rightLines, fmt.Sprintf("Cách gọi: %s", ev.Value))
		rightLines = append(rightLines, fmt.Sprintf("Khối phát sinh: %s", ev.UnitID))
		rightLines = append(rightLines, "")
		rightLines = append(rightLines, theme.PanelTitle.Render("TRÍCH ĐOẠN"))
		rightLines = append(rightLines, theme.QuoteHighlight.Render(ev.Evidence.Quote))
	} else {
		rightLines = append(rightLines, theme.TextMuted.Render("Chưa chọn cặp xưng hô"))
	}

	rightPanel := components.RenderPanel(theme, components.Panel{
		Title:   "Chi tiết & Bằng chứng",
		Focused: focused && s.FocusedPane == 1,
		Width:   rightW,
		Height:  h,
		Lines:   rightLines,
	})

	return lipgloss.JoinHorizontal(lipgloss.Top, leftPanel, " ", rightPanel)
}

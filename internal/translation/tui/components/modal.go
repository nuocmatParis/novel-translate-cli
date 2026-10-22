package components

import (
	"strings"

	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type ModalType int

const (
	ModalConfirm ModalType = iota
	ModalInput
	ModalHelp
)

type Modal struct {
	Type        ModalType
	Title       string
	Message     string
	Input       textinput.Model
	ConfirmText string
	CancelText  string
	Paid        bool
	CostEst     string
	Active      bool
	HelpKeys    []theme.KeyBinding
}

func RenderModal(th theme.Theme, m Modal, screenWidth, screenHeight int) string {
	boxWidth := min(70, max(40, screenWidth-10))

	var lines []string
	titleStyle := th.PanelTitleFocus
	lines = append(lines, titleStyle.Render(m.Title), "")

	if m.Paid {
		lines = append(lines, th.TextWarning.Render("Tác vụ này gọi API model và có thể tốn phí."))
		if m.CostEst != "" {
			lines = append(lines, th.TextMuted.Render("Ước lượng: "+m.CostEst))
		}
		lines = append(lines, "")
	}

	if m.Message != "" {
		for _, line := range strings.Split(m.Message, "\n") {
			lines = append(lines, th.Text.Render(ansi.Truncate(line, boxWidth-4, "…")))
		}
		lines = append(lines, "")
	}

	if m.Type == ModalInput {
		lines = append(lines, th.Panel.Render(m.Input.View()), "")
	} else if m.Type == ModalHelp {
		for _, b := range m.HelpKeys {
			k := lipgloss.NewStyle().Width(14).Bold(true).Foreground(lipgloss.Color(theme.ColorAccent)).Render(b.Key)
			desc := th.Text.Render(b.Description)
			lines = append(lines, k+" "+desc)
		}
		lines = append(lines, "")
	}

	// Action buttons
	confirmBtn := m.ConfirmText
	if confirmBtn == "" {
		confirmBtn = "Đồng ý (Enter)"
	}
	cancelBtn := m.CancelText
	if cancelBtn == "" {
		cancelBtn = "Hủy (Esc)"
	}

	btnLine := th.ButtonPrimary.Render(confirmBtn) + "  " + th.ButtonSecondary.Render(cancelBtn)
	lines = append(lines, btnLine)

	inner := strings.Join(lines, "\n")
	contentBox := lipgloss.NewStyle().
		Background(lipgloss.Color(theme.ColorPanel)).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(theme.ColorAccent)).
		Padding(1, 2).
		Width(boxWidth).
		Render(inner)

	return lipgloss.Place(
		screenWidth,
		screenHeight,
		lipgloss.Center,
		lipgloss.Center,
		contentBox,
	)
}

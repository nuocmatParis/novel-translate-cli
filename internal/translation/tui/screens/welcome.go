package screens

import (
	"fmt"
	"path/filepath"

	"github.com/charmbracelet/bubbles/textinput"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type WelcomeScreen struct {
	Service       *app.AppService
	SelectedIndex int // 0: create, 1: open by path, 2+: recent workspaces
	Workspaces    []app.WorkspaceSummary
	Creating      bool
	Opening       bool
	TitleInput    textinput.Model
	PathInput     textinput.Model
	Error         string
}

func NewWelcomeScreen(svc *app.AppService) *WelcomeScreen {
	titleInput := textinput.New()
	titleInput.Placeholder = "Tên đầy đủ bộ truyện..."
	titleInput.Prompt = ""
	pathInput := textinput.New()
	pathInput.Placeholder = "Đường dẫn thư mục có workspace.yaml..."
	pathInput.Prompt = ""
	pathInput.CharLimit = 4096
	s := &WelcomeScreen{Service: svc, TitleInput: titleInput, PathInput: pathInput}
	s.Refresh()
	if len(s.Workspaces) > 0 {
		s.SelectedIndex = 2
	}
	return s
}

func (s *WelcomeScreen) Refresh() {
	selectedRoot := ""
	if s.SelectedIndex > 1 && s.SelectedIndex-2 < len(s.Workspaces) {
		selectedRoot = s.Workspaces[s.SelectedIndex-2].Root
	}
	if s.Service != nil {
		var err error
		s.Workspaces, err = s.Service.AvailableWorkspaces()
		s.Error = ""
		if err != nil {
			s.Error = err.Error()
		}
	}
	s.SelectedIndex = min(s.SelectedIndex, len(s.Workspaces)+1)
	for i, item := range s.Workspaces {
		if item.Root == selectedRoot {
			s.SelectedIndex = i + 2
			break
		}
	}
}

func (s *WelcomeScreen) View(th theme.Theme, width, height int, focused bool) string {
	w := max(10, width)
	h := max(3, height)
	lines := []string{
		th.TextBold.Render("NOVEL TRANSLATE  /  Nhật → Việt"),
		th.TextMuted.Render("Quản lý workspace và dịch EPUB bằng bàn phím."),
		"",
	}

	switch {
	case s.Creating:
		lines = append(lines,
			th.PanelTitleFocus.Render("TẠO WORKSPACE MỚI"),
			th.TextMuted.Render("Nhập tên bộ truyện; thư mục và ID được tạo tự động."),
			"",
		)
		s.TitleInput.Width = max(1, w-6)
		lines = append(lines, th.TextAccent.Render("Tên bộ truyện:"), "  "+s.TitleInput.View(), "", th.TextMuted.Render("Enter: Tạo · Esc: Hủy"))
	case s.Opening:
		lines = append(lines,
			th.PanelTitleFocus.Render("MỞ WORKSPACE TỪ ĐƯỜNG DẪN"),
			th.TextMuted.Render("Nhập thư mục chứa workspace.yaml:"),
			"",
		)
		s.PathInput.Width = max(1, w-6)
		lines = append(lines, "  "+s.PathInput.View(), "", th.TextMuted.Render("Ctrl+O: chọn thư mục · Enter: Mở · Esc: Quay lại"))
	default:
		create := "  + Tạo workspace mới"
		if s.SelectedIndex == 0 {
			create = th.TextAccent.Render("› + Tạo workspace mới")
		}
		open := "  + Mở workspace từ đường dẫn"
		if s.SelectedIndex == 1 {
			open = th.TextAccent.Render("› + Mở workspace từ đường dẫn")
		}
		lines = append(lines, create, open, "", th.TextBold.Render(fmt.Sprintf("Workspace gần đây (%d):", len(s.Workspaces))))
		if len(s.Workspaces) == 0 {
			lines = append(lines, th.TextMuted.Render("Chưa có workspace gần đây."))
		} else {
			visible := max(1, h-13)
			selectedWorkspace := s.SelectedIndex - 2
			start := max(0, min(selectedWorkspace-visible/2, len(s.Workspaces)-visible))
			for i := start; i < min(len(s.Workspaces), start+visible); i++ {
				item := s.Workspaces[i]
				label := fmt.Sprintf("%s · %d tập · %s", cleanDisplayText(item.Title), item.VolumeCount, cleanDisplayText(filepath.Base(item.Root)))
				if i+2 == s.SelectedIndex {
					lines = append(lines, th.TableRowSelected.Render("› "+label))
				} else {
					lines = append(lines, th.Text.Render("  "+label))
				}
			}
			if s.SelectedIndex > 1 && s.SelectedIndex-2 < len(s.Workspaces) {
				lines = append(lines, "", th.TextMuted.Render("Thư mục: "+cleanDisplayText(s.Workspaces[s.SelectedIndex-2].Root)))
			}
		}
		lines = append(lines, "", th.TextMuted.Render("↑/↓ chọn · Enter mở · o đường dẫn · n tạo · r làm mới"))
	}

	if s.Error != "" {
		lines = append(lines, "", th.TextError.Render("Lỗi: "+cleanDisplayText(s.Error)))
	}
	return components.RenderPanel(th, components.Panel{
		Title:   "Khởi đầu",
		Focused: focused,
		Width:   w,
		Height:  h,
		Lines:   lines,
	})
}

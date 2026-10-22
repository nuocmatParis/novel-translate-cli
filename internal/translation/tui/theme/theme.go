package theme

import (
	"strings"
	"unicode"

	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
)

// Theme colors from Section 4.1 of docs/tui-implementation-plan.md
const (
	ColorBg          = "#07141B"
	ColorPanel       = "#0B1B26"
	ColorBorder      = "#344A5E"
	ColorBorderFocus = "#1677FF"
	ColorFg          = "#DCE6EE"
	ColorMuted       = "#95A8B9"
	ColorAccent      = "#1677FF"
	ColorSuccess     = "#78BE8F"
	ColorWarning     = "#E9BC69"
	ColorError       = "#ED8796"
)

type Theme struct {
	AppTitle lipgloss.Style
	Header   lipgloss.Style

	FooterNotice lipgloss.Style
	FooterKeys   lipgloss.Style

	Sidebar       lipgloss.Style
	SidebarActive lipgloss.Style
	SidebarItem   lipgloss.Style

	Panel           lipgloss.Style
	PanelFocused    lipgloss.Style
	PanelTitle      lipgloss.Style
	PanelTitleFocus lipgloss.Style

	Text        lipgloss.Style
	TextMuted   lipgloss.Style
	TextBold    lipgloss.Style
	TextSuccess lipgloss.Style
	TextWarning lipgloss.Style
	TextError   lipgloss.Style
	TextAccent  lipgloss.Style

	ButtonPrimary   lipgloss.Style
	ButtonSecondary lipgloss.Style
	ButtonDisabled  lipgloss.Style

	TableRowSelected lipgloss.Style
	BadgeAccepted    lipgloss.Style
	BadgeWarning     lipgloss.Style
	BadgeBlocking    lipgloss.Style
	BadgeUserKept    lipgloss.Style
	BadgeStale       lipgloss.Style
	BadgePending     lipgloss.Style

	QuoteHighlight lipgloss.Style
}

func DefaultTheme() Theme {
	panel := lipgloss.Color(ColorPanel)
	border := lipgloss.Color(ColorBorder)
	borderFocus := lipgloss.Color(ColorBorderFocus)
	fg := lipgloss.Color(ColorFg)
	muted := lipgloss.Color(ColorMuted)
	accent := lipgloss.Color(ColorAccent)
	success := lipgloss.Color(ColorSuccess)
	warning := lipgloss.Color(ColorWarning)
	err := lipgloss.Color(ColorError)

	return Theme{
		AppTitle: lipgloss.NewStyle().
			Foreground(fg).
			Bold(true),

		Header: lipgloss.NewStyle().
			Background(panel).
			Foreground(fg).
			Padding(0, 1),

		FooterNotice: lipgloss.NewStyle().
			Foreground(muted),

		FooterKeys: lipgloss.NewStyle().
			Foreground(accent).
			Bold(true),

		Sidebar: lipgloss.NewStyle().
			Background(panel).
			Border(lipgloss.NormalBorder(), false, true, false, false).
			BorderForeground(border),

		SidebarActive: lipgloss.NewStyle().
			Background(accent).
			Foreground(lipgloss.Color("#FFFFFF")).
			Bold(true),

		SidebarItem: lipgloss.NewStyle().
			Foreground(fg).
			Padding(0, 1),

		Panel: lipgloss.NewStyle().
			Background(panel).
			Border(lipgloss.RoundedBorder()).
			BorderForeground(border),

		PanelFocused: lipgloss.NewStyle().
			Background(panel).
			Border(lipgloss.RoundedBorder()).
			BorderForeground(borderFocus),

		PanelTitle: lipgloss.NewStyle().
			Foreground(muted).
			Bold(true),

		PanelTitleFocus: lipgloss.NewStyle().
			Foreground(accent).
			Bold(true),

		Text:        lipgloss.NewStyle().Foreground(fg),
		TextMuted:   lipgloss.NewStyle().Foreground(muted),
		TextBold:    lipgloss.NewStyle().Foreground(fg).Bold(true),
		TextSuccess: lipgloss.NewStyle().Foreground(success),
		TextWarning: lipgloss.NewStyle().Foreground(warning),
		TextError:   lipgloss.NewStyle().Foreground(err),
		TextAccent:  lipgloss.NewStyle().Foreground(accent),

		ButtonPrimary: lipgloss.NewStyle().
			Background(accent).
			Foreground(lipgloss.Color("#FFFFFF")).
			Padding(0, 1).
			Bold(true),

		ButtonSecondary: lipgloss.NewStyle().
			Background(panel).
			Foreground(fg).
			Border(lipgloss.NormalBorder()).
			BorderForeground(border).
			Padding(0, 1),

		ButtonDisabled: lipgloss.NewStyle().
			Foreground(muted).
			Padding(0, 1),

		TableRowSelected: lipgloss.NewStyle().
			Background(lipgloss.Color("#132A3D")).
			Foreground(lipgloss.Color("#FFFFFF")).
			Bold(true),

		BadgeAccepted: lipgloss.NewStyle().
			Background(success).
			Foreground(lipgloss.Color("#000000")).
			Padding(0, 1),

		BadgeWarning: lipgloss.NewStyle().
			Background(warning).
			Foreground(lipgloss.Color("#000000")).
			Padding(0, 1),

		BadgeBlocking: lipgloss.NewStyle().
			Background(err).
			Foreground(lipgloss.Color("#FFFFFF")).
			Bold(true).
			Padding(0, 1),

		BadgeUserKept: lipgloss.NewStyle().
			Background(accent).
			Foreground(lipgloss.Color("#FFFFFF")).
			Padding(0, 1),

		BadgeStale: lipgloss.NewStyle().
			Background(lipgloss.Color("#553333")).
			Foreground(warning).
			Padding(0, 1),

		BadgePending: lipgloss.NewStyle().
			Background(lipgloss.Color("#223344")).
			Foreground(muted).
			Padding(0, 1),

		QuoteHighlight: lipgloss.NewStyle().
			Background(lipgloss.Color("#4A3B18")).
			Foreground(warning).
			Bold(true),
	}
}

// CleanText sanitizes control characters and ANSI escape sequences
func CleanText(s string) string {
	return strings.Map(func(r rune) rune {
		if unicode.IsControl(r) && r != '\n' && r != '\t' {
			return -1
		}
		return r
	}, ansi.Strip(s))
}

type KeyBinding struct {
	Key         string
	Description string
}

func CommonKeys() []KeyBinding {
	return []KeyBinding{
		{"Tab", "Đổi vùng focus"},
		{"↑/↓, j/k", "Chọn dòng"},
		{"←/→", "Đổi tab / nhánh cây"},
		{"Enter", "Mở / Thực hiện"},
		{"Space", "Chọn / bỏ chọn"},
		{"Ctrl+O", "Chọn file / thư mục ở trường đường dẫn"},
		{"PgUp/PgDn", "Cuộn trang"},
		{"Ctrl+S", "Lưu thay đổi"},
		{"Esc", "Quay lại / Đóng"},
		{"p", "Tạm dừng / Tiếp tục"},
		{"Ctrl+C", "Dừng tác vụ"},
		{"q", "Thoát"},
		{"?", "Trợ giúp"},
	}
}

func ScreenHelp(screenID string) []KeyBinding {
	switch screenID {
	case "welcome":
		return []KeyBinding{
			{"↑/↓, PgUp/PgDn", "Chọn bộ truyện trong danh sách workspace"},
			{"Enter", "Mở bộ đã chọn hoặc tạo bộ mới từ tên"},
			{"n", "Tạo workspace mới: chỉ cần tên bộ truyện"},
			{"r / o", "Đọc lại danh sách workspace"},
			{"Esc", "Hủy tạo mới / quay lại bộ đang mở"},
		}
	case "dashboard":
		return []KeyBinding{
			{"Tab", "Chuyển giữa Sidebar / Thao tác / Danh sách tập"},
			{"Enter", "Mở mục đang chọn"},
			{"q", "Thoát ứng dụng"},
			{"?", "Mở bảng trợ giúp"},
		}
	case "content":
		return []KeyBinding{
			{"Tab", "Chuyển giữa Tập / Chương / Chi tiết phạm vi"},
			{"Space", "Đánh dấu / Bỏ dấu chương để chạy"},
			{"Enter", "Xem chi tiết / Bắt đầu chạy phạm vi"},
			{"Esc", "Quay lại Dashboard"},
		}
	case "job":
		return []KeyBinding{
			{"p", "Pause/resume ở ranh giới khối của dịch/review"},
			{"Ctrl+C", "Dừng khẩn cấp và giải phóng khóa writer"},
			{"Tab", "Chuyển giữa xem tiến độ và log"},
			{"PgUp/PgDn", "Cuộn log sự kiện"},
			{"Esc", "Quay lại màn hình trước (tác vụ vẫn chạy nền)"},
		}
	case "editor":
		return []KeyBinding{
			{"Tab", "Chuyển giữa Cây khối / Nội dung / Thao tác"},
			{"Ctrl+S", "Lưu bản dịch hoặc ghi chú theo tab đang mở"},
			{"←/→", "Chuyển tab: So sánh / Bản Việt / Nguồn-Ruby / Lịch sử / Ghi chú"},
			{"↑/↓", "Chọn khối nguồn (yêu cầu lưu bản nháp trước)"},
			{"Esc", "Thoát trình sửa (có xác nhận nếu chưa lưu)"},
		}
	case "review":
		return []KeyBinding{
			{"Tab", "Chuyển giữa Hàng đợi / Chi tiết issue / Thao tác"},
			{"↑/↓", "Chọn issue cần xem xét"},
			{"Enter", "Xem bằng chứng & đối chiếu"},
			{"k", "Giữ bản dịch này (mở ô nhập lý do)"},
			{"r", "Yêu cầu review lại khối này"},
			{"e", "Mở trình sửa tay cho khối này"},
			{"Esc", "Quay lại Dashboard"},
		}
	case "skills":
		return []KeyBinding{
			{"Tab", "Chuyển giữa danh mục, danh sách và chi tiết"},
			{"←/→", "Chuyển loại skill/rule/research"},
			{"Space", "Bật/tắt skill chung đang chọn"},
			{"Ctrl+S", "Lưu danh sách skill đã bật"},
			{"e", "Sửa skill chung hoặc skill riêng đang chọn"},
			{"l", "Chọn thư mục thư viện skill chung"},
			{"c / x", "Xác nhận / từ chối candidate bằng lý do"},
			{"Esc", "Quay lại Dashboard"},
		}
	case "verify":
		return []KeyBinding{
			{"↑/↓", "Chọn cổng kiểm tra hoặc occurrence ảnh"},
			{"a", "Đánh giá occurrence ảnh đang chọn"},
			{"i", "Import EPUB nếu chưa có snapshot"},
			{"p", "Tạo EPUB preview và chạy readback/EPUBCheck"},
			{"f", "Xuất Final khi các cổng trước đã đạt"},
			{"Tab", "Chuyển giữa checklist, findings và thao tác"},
		}
	case "settings":
		return []KeyBinding{
			{"Tab / Shift+Tab", "Chuyển trường"},
			{"Ctrl+←/→", "Chuyển nhóm cấu hình"},
			{"Ctrl+S / Enter", "Lưu nhóm đang mở"},
			{"Esc", "Quay lại Dashboard"},
		}
	default:
		return CommonKeys()
	}
}

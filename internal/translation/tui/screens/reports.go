package screens

import (
	"fmt"

	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type ReportsScreen struct {
	Service *app.AppService
	Usage   model.UsageSummary
}

func NewReportsScreen(svc *app.AppService) *ReportsScreen {
	rs := &ReportsScreen{
		Service: svc,
	}
	rs.Refresh()
	return rs
}

func (s *ReportsScreen) Refresh() {
	if s.Service != nil {
		u, err := model.ReadUsage(s.Service.Store)
		if err == nil {
			s.Usage = u
		}
	}
}

func (s *ReportsScreen) View(theme theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)

	var lines []string
	lines = append(lines, theme.PanelTitleFocus.Render("BÁO CÁO SỬ DỤNG TOKEN & CHI PHÍ"))
	lines = append(lines, "")

	costStr := fmt.Sprintf("$%.4f USD", s.Usage.KnownCostUSD)
	if s.Usage.UnknownCostRequests > 0 {
		costStr += fmt.Sprintf(" (+ %d requests chưa xác định đơn giá)", s.Usage.UnknownCostRequests)
	}

	lines = append(lines, fmt.Sprintf("Tổng chi phí ghi nhận: %s", theme.TextBold.Render(costStr)))
	lines = append(lines, fmt.Sprintf("Tổng input tokens:     %d", s.Usage.InputTokens))
	lines = append(lines, fmt.Sprintf("Tổng output tokens:    %d", s.Usage.OutputTokens))
	lines = append(lines, fmt.Sprintf("Tổng số lượt gọi API:  %d", s.Usage.Requests))
	lines = append(lines, "")

	lines = append(lines, theme.PanelTitle.Render("LƯU Ý DỮ LIỆU CŨ & ĐỘ CHÍNH XÁC"))
	lines = append(lines, theme.TextMuted.Render("Các request không có bảng giá hoặc dữ liệu nhật ký cũ chưa liên kết"))
	lines = append(lines, theme.TextMuted.Render("được đánh dấu là 'chưa phân loại/chưa xác định' thay vì 0 USD."))

	return components.RenderPanel(theme, components.Panel{
		Title:   "Báo cáo & Tiêu thụ API",
		Focused: focused,
		Width:   w,
		Height:  h,
		Lines:   lines,
	})
}

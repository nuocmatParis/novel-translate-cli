package screens

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type VerifyScreen struct {
	Service        *app.AppService
	VolumeID       string
	Projection     app.VerifyProjection
	Error          string
	SelectedGate   int
	SelectedImage  int
	SelectedAction int // 0 preview, 1 final, 2 import when needed
	FocusedPane    int
	AssessingImage bool
	Inputs         []textinput.Model
	FocusedField   int
	FormError      string
}

func NewVerifyScreen(svc *app.AppService, volumeID string) *VerifyScreen {
	return &VerifyScreen{Service: svc, VolumeID: volumeID}
}

func (s *VerifyScreen) Refresh() {
	if s.Service == nil || s.VolumeID == "" {
		return
	}
	p, err := s.Service.QueryVerify(s.VolumeID)
	if err != nil {
		s.Error = err.Error()
		s.Projection = app.VerifyProjection{VolumeID: s.VolumeID, SourceError: err.Error()}
		return
	}
	s.Projection = p
	s.Error = ""
	if s.SelectedGate >= len(p.Gates) {
		s.SelectedGate = max(0, len(p.Gates)-1)
	}
	if s.SelectedImage >= len(p.Images) {
		s.SelectedImage = max(0, len(p.Images)-1)
	}
}

func (s *VerifyScreen) CanExportFinal() (bool, string) {
	if !s.Projection.Imported {
		return false, "Cần import EPUB trước."
	}
	if s.Projection.Report.Eligible == 0 {
		return false, "Không có khối eligible để xác nhận hoàn tất."
	}
	if s.Service == nil || s.Service.Config == nil || strings.TrimSpace(s.Service.Config.Output.EPUBCheck) == "" {
		return false, "Cấu hình EPUBCheck trước khi xuất Final."
	}
	for _, gate := range s.Projection.Gates {
		switch gate.Name {
		case "Bóc tách văn bản", "Đầy đủ bản dịch", "Freshness", "Hình ảnh", "Resources":
			if gate.Status != "Đạt" && gate.Status != "Không áp dụng" {
				return false, gate.Name + ": " + gate.Status
			}
		}
	}
	return true, "Sẽ chạy readback và EPUBCheck trước khi ghi file Final."
}

func (s *VerifyScreen) Actions() []string {
	return []string{"Xuất EPUB xem trước", "Xuất EPUB hoàn chỉnh", "Đánh giá occurrence ảnh", "Import snapshot nguồn"}
}

func (s *VerifyScreen) StartAssessment() error {
	if s.SelectedImage < 0 || s.SelectedImage >= len(s.Projection.Images) {
		return fmt.Errorf("chọn occurrence ảnh trước")
	}
	image := s.Projection.Images[s.SelectedImage]
	values := []string{image.ID, "", "", "", ""}
	placeholders := []string{"ID occurrence", "no-text / keep-original / replace / transcript", "Lý do hoặc bằng chứng kiểm tra", "Đường dẫn ảnh thay thế trong workspace", "Transcript tiếng Việt nếu có"}
	s.Inputs = make([]textinput.Model, len(values))
	for i := range values {
		ti := textinput.New()
		ti.Placeholder = placeholders[i]
		ti.SetValue(values[i])
		ti.CharLimit = 8192
		s.Inputs[i] = ti
	}
	s.FocusedField = 1
	s.Inputs[1].Focus()
	s.AssessingImage = true
	s.FormError = ""
	return nil
}

func (s *VerifyScreen) CancelAssessment() {
	s.AssessingImage = false
	s.FormError = ""
	for i := range s.Inputs {
		s.Inputs[i].Blur()
	}
}

func (s *VerifyScreen) SaveAssessment() error {
	if len(s.Inputs) != 5 {
		return fmt.Errorf("biểu mẫu đánh giá ảnh chưa sẵn sàng")
	}
	values := make([]string, len(s.Inputs))
	for i := range s.Inputs {
		values[i] = strings.TrimSpace(s.Inputs[i].Value())
	}
	if err := s.Service.SaveImageAssessment(s.VolumeID, values[0], values[1], values[2], values[3], values[4]); err != nil {
		s.FormError = err.Error()
		return err
	}
	s.CancelAssessment()
	s.Refresh()
	return nil
}

func (s *VerifyScreen) View(th theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)
	if s.AssessingImage {
		labels := []string{"Occurrence ID:", "Disposition:", "Lý do/bằng chứng:", "Ảnh thay thế:", "Transcript:"}
		lines := []string{th.TextBold.Render("ĐÁNH GIÁ NỘI DUNG ẢNH"), th.TextMuted.Render("Quyết định gắn với hash source/resource hiện tại."), ""}
		for i, label := range labels {
			prefix := "  "
			if i == s.FocusedField {
				prefix = "› "
			}
			lines = append(lines, prefix+label)
			s.Inputs[i].Width = max(1, w-8)
			lines = append(lines, "    "+s.Inputs[i].View())
		}
		lines = append(lines, "", th.TextMuted.Render("Tab: trường tiếp · Ctrl+O: chọn ảnh thay thế · Ctrl+S/Enter: lưu · Esc: hủy"))
		if s.FormError != "" {
			lines = append(lines, "", th.TextError.Render(cleanDisplayText(s.FormError)))
		}
		return components.RenderPanel(th, components.Panel{Title: "Đánh giá ảnh", Focused: focused, Width: w, Height: h, Lines: lines})
	}
	leftW := max(23, w/3)
	midW := max(24, w/3)
	rightW := max(20, w-leftW-midW-4)
	if leftW+midW+rightW+2 > w {
		midW = max(20, w-leftW-rightW-2)
	}

	var gateLines []string
	gateLines = append(gateLines, th.TextMuted.Render("Trạng thái lấy từ snapshot và report thật."), "")
	if len(s.Projection.Gates) == 0 {
		gateLines = append(gateLines, th.TextMuted.Render("Chưa có dữ liệu kiểm tra."))
	}
	for i, gate := range s.Projection.Gates {
		style := th.Text
		prefix := "  "
		switch gate.Status {
		case "Đạt":
			style = th.TextSuccess
		case "Có vấn đề", "Cần chạy lại":
			style = th.TextError
		case "Chưa chạy", "Không áp dụng":
			style = th.TextMuted
		}
		if s.FocusedPane == 0 && i == s.SelectedGate {
			prefix = "› "
			style = th.TextAccent.Bold(true)
		}
		gateLines = append(gateLines, style.Render(prefix+gate.Name+": "+gate.Status))
		if gate.Detail != "" && (s.FocusedPane != 0 || i == s.SelectedGate) {
			gateLines = append(gateLines, th.TextMuted.Render("  "+gate.Detail))
		}
	}
	leftPanel := components.RenderPanel(th, components.Panel{
		Title:   "Cổng kiểm tra",
		Focused: focused && s.FocusedPane == 0,
		Width:   leftW,
		Height:  h,
		Scroll:  max(0, s.SelectedGate-3),
		Lines:   gateLines,
	})

	var findingLines []string
	findingLines = append(findingLines, th.PanelTitle.Render(fmt.Sprintf("HÌNH ẢNH (%d)", len(s.Projection.Images))))
	if len(s.Projection.Images) == 0 {
		findingLines = append(findingLines, th.TextMuted.Render("Chưa có ảnh hoặc EPUB chưa được import."))
	} else {
		for i, image := range s.Projection.Images {
			prefix := "  "
			style := th.Text
			if s.FocusedPane == 1 && i == s.SelectedImage {
				prefix = "› "
				style = th.TextAccent.Bold(true)
			}
			findingLines = append(findingLines, style.Render(fmt.Sprintf("%s%s · %s", prefix, image.ID, image.Resource)))
		}
	}
	findingLines = append(findingLines, "", th.PanelTitle.Render(fmt.Sprintf("DIAGNOSTIC (%d)", len(s.Projection.Diagnostics))))
	if len(s.Projection.Diagnostics) == 0 {
		findingLines = append(findingLines, th.TextMuted.Render("Không có diagnostic được ghi nhận."))
	} else {
		for _, diagnostic := range s.Projection.Diagnostics {
			style := th.TextWarning
			if diagnostic.Blocking {
				style = th.TextError
			}
			findingLines = append(findingLines, style.Render(diagnostic.Code+": "+diagnostic.Detail))
		}
	}
	midPanel := components.RenderPanel(th, components.Panel{
		Title:   "Ảnh và nguyên nhân",
		Focused: focused && s.FocusedPane == 1,
		Width:   midW,
		Height:  h,
		Scroll:  max(0, s.SelectedImage-2),
		Lines:   findingLines,
	})

	var actionLines []string
	if !s.Projection.Imported {
		actionLines = append(actionLines, th.TextWarning.Render("Chưa có source snapshot."))
		actionLines = append(actionLines, th.TextMuted.Render(cleanDisplayText(s.Projection.SourceError)))
	}
	for i, action := range s.Actions() {
		style := th.ButtonSecondary
		if i == s.SelectedAction && s.FocusedPane == 2 {
			style = th.ButtonPrimary
		}
		if i == 3 && s.Projection.Imported {
			style = th.ButtonDisabled
		}
		actionLines = append(actionLines, style.Render(fmt.Sprintf("[%s] %s", []string{"p", "f", "a", "i"}[i], action)))
	}
	if ok, reason := s.CanExportFinal(); !ok {
		actionLines = append(actionLines, "", th.TextWarning.Render("Final đang khóa: "+reason))
	} else {
		actionLines = append(actionLines, "", th.TextMuted.Render(reason))
	}
	if s.Projection.LatestExport != nil {
		actionLines = append(actionLines, "", th.PanelTitle.Render("LẦN XUẤT GẦN NHẤT"))
		freshness := "đã stale"
		if s.Projection.LatestExportFresh {
			freshness = "đang khớp source/generation"
		}
		actionLines = append(actionLines, th.TextMuted.Render(freshness))
		actionLines = append(actionLines, th.TextMuted.Render(s.Projection.LatestExportPath))
		actionLines = append(actionLines, th.TextMuted.Render(s.Projection.LatestExport.Format))
	}
	if s.Error != "" {
		actionLines = append(actionLines, "", th.TextError.Render(cleanDisplayText(s.Error)))
	}
	rightPanel := components.RenderPanel(th, components.Panel{
		Title:   "Phạm vi xuất",
		Focused: focused && s.FocusedPane == 2,
		Width:   rightW,
		Height:  h,
		Lines:   actionLines,
	})
	return lipgloss.JoinHorizontal(lipgloss.Top, leftPanel, " ", midPanel, " ", rightPanel)
}

func (s *VerifyScreen) Summary() string {
	if s.Projection.LatestExport == nil {
		return "Chưa có file xuất gần nhất."
	}
	state := "đã stale"
	if s.Projection.LatestExportFresh {
		state = "còn fresh"
	}
	return strings.TrimSpace(fmt.Sprintf("%s · %s", s.Projection.LatestExportPath, state))
}

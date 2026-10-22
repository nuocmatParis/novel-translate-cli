package screens

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type ReviewScreen struct {
	Service        *app.AppService
	VolumeID       string
	Queue          []app.ReviewQueueItem
	SelectedIndex  int
	FocusedPane    int    // 0: Queue, 1: Content/Evidence, 2: Details/Actions
	FilterSeverity string // "", "blocking", "warning"
	FilterStatus   string // "", "Cần xử lý", "Chờ review", "Đã đối chiếu"
	EnteringReason bool
	ReasonInput    textinput.Model
	ActionIndex    int
	Message        string
	Error          string
}

func NewReviewScreen(svc *app.AppService, volumeID string) *ReviewScreen {
	ti := textinput.New()
	ti.Placeholder = "Nhập lý do bắt buộc để giữ bản dịch..."

	rs := &ReviewScreen{
		Service:     svc,
		VolumeID:    volumeID,
		ReasonInput: ti,
	}
	return rs
}

func (s *ReviewScreen) Refresh() {
	if s.Service == nil || s.VolumeID == "" {
		return
	}
	items, err := s.Service.QueryReviewQueue(s.VolumeID)
	if err == nil {
		s.Queue = items
		s.SelectedIndex = min(max(0, s.SelectedIndex), max(0, len(s.FilteredQueue())-1))
	}
}

func (s *ReviewScreen) FilteredQueue() []app.ReviewQueueItem {
	var res []app.ReviewQueueItem
	for _, item := range s.Queue {
		if s.FilterSeverity == "blocking" && item.BlockingCount == 0 {
			continue
		}
		if s.FilterSeverity == "warning" && item.WarningCount == 0 {
			continue
		}
		if s.FilterStatus != "" && item.Status != s.FilterStatus {
			continue
		}
		res = append(res, item)
	}
	return res
}

func (s *ReviewScreen) SelectedItem() *app.ReviewQueueItem {
	filtered := s.FilteredQueue()
	if s.SelectedIndex >= 0 && s.SelectedIndex < len(filtered) {
		return &filtered[s.SelectedIndex]
	}
	return nil
}

func (s *ReviewScreen) QueueCounts() (pending, reviewing, needRepair, accepted int) {
	for _, it := range s.Queue {
		switch it.Status {
		case "Chờ review", "Chưa dịch":
			pending++
		case "Đang review":
			reviewing++
		case "Cần xử lý":
			needRepair++
		case "Cần đối chiếu lại":
			// Stale is shown as its own primary state below.
		case "Đã đối chiếu", "Người dùng giữ":
			accepted++
		default:
			pending++
		}
	}
	return
}

func (s *ReviewScreen) SubmitKeepReason() error {
	reason := strings.TrimSpace(s.ReasonInput.Value())
	if reason == "" {
		s.Error = "Lý do giữ bản dịch là bắt buộc"
		return fmt.Errorf("%s", s.Error)
	}
	item := s.SelectedItem()
	if item == nil {
		return fmt.Errorf("không có khối được chọn")
	}

	err := s.Service.KeepRevision(s.VolumeID, item.UnitID, reason)
	if err != nil {
		s.Error = err.Error()
		return err
	}
	s.EnteringReason = false
	s.ReasonInput.SetValue("")
	s.Message = "Đã lưu quyết định: Người dùng quyết định giữ bản dịch này"
	s.Error = ""
	s.Refresh()
	return nil
}

func (s *ReviewScreen) View(theme theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)

	qW := max(18, w/4)
	cW := max(28, w/2)
	dW := max(16, w-qW-cW-2)
	if qW+cW+dW+2 > w {
		cW = max(24, w-qW-dW-2)
	}

	pendingCount, reviewingCount, repairCount, acceptedCount := s.QueueCounts()
	staleCount := 0
	for _, item := range s.Queue {
		if item.Status == "Cần đối chiếu lại" {
			staleCount++
		}
	}

	// 1. Left: Queue list & counts
	var qLines []string
	qLines = append(qLines, theme.PanelTitle.Render("HÀNG ĐỢI REVIEW"))
	qLines = append(qLines, fmt.Sprintf("Chờ review: %d", pendingCount))
	qLines = append(qLines, fmt.Sprintf("Đang review: %d", reviewingCount))
	qLines = append(qLines, theme.TextError.Render(fmt.Sprintf("Cần sửa: %d", repairCount)))
	qLines = append(qLines, theme.TextWarning.Render(fmt.Sprintf("Cần đối chiếu lại: %d", staleCount)))
	qLines = append(qLines, theme.TextSuccess.Render(fmt.Sprintf("Đã đối chiếu: %d", acceptedCount)))
	filter := "Tất cả"
	switch {
	case s.FilterSeverity == "blocking":
		filter = "Issue blocking"
	case s.FilterSeverity == "warning":
		filter = "Issue warning"
	case s.FilterStatus != "":
		filter = s.FilterStatus
	}
	qLines = append(qLines, theme.TextMuted.Render("Bộ lọc: "+filter+" · 0 tất cả / 1 chặn / 2 lưu ý / 3 stale / 4 chờ"))
	qLines = append(qLines, strings.Repeat("─", qW-4))

	filtered := s.FilteredQueue()
	for i, item := range filtered {
		prefix := "  "
		if s.FocusedPane == 0 && i == s.SelectedIndex {
			prefix = "› "
		}
		itemLabel := fmt.Sprintf("%s%s (r%d)", prefix, cleanDisplayText(item.UnitID), item.Revision)
		if item.BlockingCount > 0 {
			itemLabel += theme.TextError.Render(fmt.Sprintf(" [%d chặn]", item.BlockingCount))
		} else if item.WarningCount > 0 {
			itemLabel += theme.TextWarning.Render(fmt.Sprintf(" [%d lưu ý]", item.WarningCount))
		}

		if s.FocusedPane == 0 && i == s.SelectedIndex {
			qLines = append(qLines, theme.TableRowSelected.Render(itemLabel))
		} else {
			qLines = append(qLines, theme.Text.Render(itemLabel))
		}
	}

	queuePanel := components.RenderPanel(theme, components.Panel{
		Title:   "Hàng đợi review",
		Focused: focused && s.FocusedPane == 0,
		Width:   qW,
		Height:  h,
		Scroll:  max(0, s.SelectedIndex+6-(h-4)/2),
		Lines:   qLines,
	})

	// 2. Middle: Content, Evidence, and Diff
	var cLines []string
	sel := s.SelectedItem()
	if sel != nil {
		cLines = append(cLines, theme.PanelTitleFocus.Render(fmt.Sprintf("NGUỒN JP / BẢN VIỆT r%d", sel.Revision)))
		cLines = append(cLines, theme.TextBold.Render("Nguồn JP:"))

		sourceText := cleanDisplayText(sel.Source)
		targetText := cleanDisplayText(sel.Target)
		var primeIssue *domain.Issue
		if sel.PrimaryIssue != nil {
			primeIssue = sel.PrimaryIssue
		}

		if primeIssue != nil && primeIssue.SourceQuote != "" {
			cLines = append(cLines, components.HighlightQuote(theme, sourceText, primeIssue.SourceQuote))
		} else {
			cLines = append(cLines, theme.Text.Render(sourceText))
		}

		cLines = append(cLines, "")
		cLines = append(cLines, theme.TextBold.Render("Bản dịch Việt:"))
		if primeIssue != nil && primeIssue.TargetQuote != "" {
			cLines = append(cLines, components.HighlightQuote(theme, targetText, primeIssue.TargetQuote))
		} else {
			cLines = append(cLines, theme.Text.Render(targetText))
		}

		cLines = append(cLines, "")
		cLines = append(cLines, theme.PanelTitle.Render("BẰNG CHỨNG & GIẢI THÍCH REVIEWER"))
		if len(sel.Issues) > 0 {
			for i, issue := range sel.Issues {
				severity := theme.TextWarning
				if issue.Severity == "blocking" {
					severity = theme.TextError
				}
				cLines = append(cLines, "", severity.Render(fmt.Sprintf("Issue %d · %s · %s · %s", i+1, cleanDisplayText(issue.ID), cleanDisplayText(issue.Type), cleanDisplayText(issue.Severity))))
				if issue.SourceQuote != "" {
					cLines = append(cLines, theme.TextWarning.Render("Bằng chứng JP: ")+theme.QuoteHighlight.Render(cleanDisplayText(issue.SourceQuote)))
				}
				if issue.TargetQuote != "" {
					cLines = append(cLines, theme.TextWarning.Render("Bằng chứng VI: ")+theme.QuoteHighlight.Render(cleanDisplayText(issue.TargetQuote)))
				} else {
					cLines = append(cLines, theme.TextMuted.Render("Bản dịch: Không có phần tương ứng trong target."))
				}
				if issue.Explanation != "" {
					cLines = append(cLines, theme.Text.Render("Giải thích: "+cleanDisplayText(issue.Explanation)))
				}
				if issue.Resolution != "" {
					cLines = append(cLines, theme.TextSuccess.Render("Đã ghi nhận cách xử lý: "+cleanDisplayText(issue.Resolution)))
				}
			}
		} else {
			cLines = append(cLines, theme.TextMuted.Render("Khối này không có lỗi chặn hoặc lưu ý nào."))
		}
		if len(sel.RubyDecisions) > 0 {
			cLines = append(cLines, "", theme.PanelTitle.Render("QUYẾT ĐỊNH RUBY"))
			for _, ruby := range sel.RubyDecisions {
				cLines = append(cLines, fmt.Sprintf("%s【%s】 · %s", cleanDisplayText(ruby.Base), cleanDisplayText(ruby.Reading), cleanDisplayText(ruby.Decision)))
				if ruby.TargetQuote != "" {
					cLines = append(cLines, theme.TextMuted.Render("Bản dịch: "+cleanDisplayText(ruby.TargetQuote)))
				}
				if ruby.Reason != "" {
					cLines = append(cLines, theme.TextMuted.Render("Lý do: "+cleanDisplayText(ruby.Reason)))
				}
			}
		}
		if len(sel.Signals) > 0 {
			cLines = append(cLines, "", theme.PanelTitle.Render("TÍN HIỆU CƠ HỌC · KHÔNG PHẢI KẾT LUẬN"))
			for _, signal := range sel.Signals {
				cLines = append(cLines, theme.TextMuted.Render(cleanDisplayText(signal.Type+": "+signal.Explanation)))
			}
		}
		if len(sel.Children) > 0 {
			cLines = append(cLines, "", theme.PanelTitle.Render(fmt.Sprintf("REVIEW CHIA KHỐI (%d)", len(sel.Children))))
			for _, child := range sel.Children {
				decision, modelName, issueCount := "chưa có kết quả", "", 0
				if child.Review != nil {
					decision = child.Review.Decision
					modelName = child.Review.Model
					issueCount = len(child.Review.Issues)
				}
				cLines = append(cLines, fmt.Sprintf("%s · span %d–%d · %s · %d issue", child.ID, child.Start, child.End, decision, issueCount))
				if modelName != "" {
					cLines = append(cLines, theme.TextMuted.Render("Model: "+modelName))
				}
				if child.Source != "" {
					cLines = append(cLines, theme.TextMuted.Render("JP: "+child.Source))
				}
				if child.Target != "" {
					cLines = append(cLines, theme.TextMuted.Render("VI: "+child.Target))
				}
			}
		}

		cLines = append(cLines, "")
		if s.EnteringReason {
			cLines = append(cLines, theme.PanelTitleFocus.Render("NHẬP LÝ DO GIỮ BẢN DỊCH:"))
			cLines = append(cLines, s.ReasonInput.View())
			cLines = append(cLines, theme.ButtonPrimary.Render("Xác nhận lưu (Enter)")+"  "+theme.ButtonSecondary.Render("Hủy (Esc)"))
		} else {
			actions := []string{"[ Sửa tay ]", "[ Review lại ]", "[ Giữ bản này ]"}
			var actBtns []string
			for i, act := range actions {
				if s.FocusedPane == 1 && i == s.ActionIndex {
					actBtns = append(actBtns, theme.ButtonPrimary.Render(act))
				} else {
					actBtns = append(actBtns, theme.ButtonSecondary.Render(act))
				}
			}
			cLines = append(cLines, strings.Join(actBtns, " "))
			if s.FocusedPane == 1 {
				cLines = append(cLines, theme.TextMuted.Render("←/→ chọn thao tác · Enter thực hiện"))
			}
		}
	} else {
		cLines = append(cLines, theme.TextMuted.Render("Chọn một khối từ hàng đợi bên trái để xem chi tiết."))
	}

	contentPanel := components.RenderPanel(theme, components.Panel{
		Title:   "Đối chiếu JP-VI & Bằng chứng",
		Focused: focused && s.FocusedPane == 1,
		Width:   cW,
		Height:  h,
		Lines:   cLines,
	})

	// 3. Right: Issue details & metadata
	var dLines []string
	dLines = append(dLines, theme.PanelTitle.Render("CHI TIẾT ĐỐI CHIẾU"))
	if sel != nil {
		dLines = append(dLines, "Trạng thái: "+components.RenderBadge(theme, sel.Status))
		dLines = append(dLines, fmt.Sprintf("Khối: %s", sel.UnitID))
		dLines = append(dLines, fmt.Sprintf("Chương: %s", sel.ChapterTitle))
		dLines = append(dLines, fmt.Sprintf("Revision: r%d", sel.Revision))
		dLines = append(dLines, fmt.Sprintf("Tổng issue: %d (%d chặn, %d lưu ý)", sel.IssuesCount, sel.BlockingCount, sel.WarningCount))
		dLines = append(dLines, fmt.Sprintf("Ruby decisions: %d", sel.RubyCount))
		if sel.Model != "" {
			dLines = append(dLines, "Reviewer model: "+cleanDisplayText(sel.Model))
		}
		if sel.Decision != "" {
			dLines = append(dLines, "Reviewer decision: "+sel.Decision)
		}
		if sel.DecisionReason != "" {
			dLines = append(dLines, "Lý do quyết định: "+sel.DecisionReason)
		}
		if sel.StaleReason != "" {
			dLines = append(dLines, "", theme.TextError.Render("Stale: "+sel.StaleReason))
		}

		if sel.PrimaryIssue != nil {
			dLines = append(dLines, "")
			dLines = append(dLines, theme.PanelTitle.Render("ISSUE ĐANG CHỌN"))
			dLines = append(dLines, "Loại: "+sel.PrimaryIssue.Type)
			sevStyle := theme.TextWarning
			if sel.PrimaryIssue.Severity == "blocking" {
				sevStyle = theme.TextError
			}
			dLines = append(dLines, "Mức độ: "+sevStyle.Render(sel.PrimaryIssue.Severity))
		}
	} else {
		dLines = append(dLines, theme.TextMuted.Render("Chưa chọn khối"))
	}

	if s.Message != "" {
		dLines = append(dLines, "", theme.TextSuccess.Render(s.Message))
	}
	if s.Error != "" {
		dLines = append(dLines, "", theme.TextError.Render(s.Error))
	}

	detailsPanel := components.RenderPanel(theme, components.Panel{
		Title:   "Thông tin Issue",
		Focused: focused && s.FocusedPane == 2,
		Width:   dW,
		Height:  h,
		Lines:   dLines,
	})

	return lipgloss.JoinHorizontal(lipgloss.Top, queuePanel, " ", contentPanel, " ", detailsPanel)
}

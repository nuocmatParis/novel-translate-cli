package screens

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/charmbracelet/bubbles/textarea"
	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/research"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type SkillsScreen struct {
	Service         *app.AppService
	ActiveTab       int // 0: Shared, 1: Series, 2: Rules, 3: Candidates, 4: Research
	Snapshot        skills.Snapshot
	SharedSkills    []skills.Skill
	SharedLibrary   []skills.Skill
	SeriesSkill     []skills.Skill
	Rules           []domain.Rule
	Research        []research.Evidence
	LoadedShared    map[string]bool
	EnabledSkills   map[string]bool
	EnabledDirty    bool
	LibraryPath     string
	Editing         bool
	EditDirty       bool
	EditSkill       skills.Skill
	EditHash        string
	EditDirectory   string
	Editor          textarea.Model
	EditingLibrary  bool
	LibraryInput    textinput.Model
	Resolving       bool
	ResolveID       string
	ResolveDecision string
	ReasonInput     textinput.Model
	SelectedRule    int
	FocusedPane     int // 0: Tabs/Nav, 1: List, 2: Detail
	Error           string
}

func NewSkillsScreen(svc *app.AppService) *SkillsScreen {
	return &SkillsScreen{Service: svc, LoadedShared: map[string]bool{}, EnabledSkills: map[string]bool{}}
}

func (s *SkillsScreen) Tabs() []string {
	return []string{"Skill chung", "Skill riêng", "Rules", "Ứng viên", "Research đã lưu"}
}

func (s *SkillsScreen) Refresh() {
	if s.Service == nil || s.Service.Config == nil {
		return
	}
	s.Error = ""
	snapshot, err := s.Service.QuerySkillSnapshot()
	if err != nil {
		s.Error = err.Error()
		return
	}
	s.Snapshot = snapshot
	s.Rules = snapshot.Rules
	if !s.EnabledDirty {
		s.EnabledSkills = map[string]bool{}
		for _, id := range s.Service.Config.EnabledSkills {
			s.EnabledSkills[id] = true
		}
	}
	s.LoadedShared = map[string]bool{}
	s.SeriesSkill = nil
	seriesPath := filepath.Clean(s.Service.Config.ReadPath(s.Service.Config.SeriesSkill))
	for _, skill := range snapshot.Skills {
		if filepath.Clean(skill.Path) == seriesPath {
			s.SeriesSkill = append(s.SeriesSkill, skill)
		} else if !strings.Contains(skill.ID, ":") {
			s.LoadedShared[skill.ID] = true
		}
	}
	s.LibraryPath, s.SharedLibrary, err = s.Service.QuerySharedSkillLibrary()
	if err != nil {
		s.Error = err.Error()
	}
	s.SharedSkills = s.SharedLibrary
	s.Research, err = s.Service.QueryResearchEvidence()
	if err != nil {
		if s.Error != "" {
			s.Error += " · "
		}
		s.Error += err.Error()
	}
	s.SelectedRule = min(max(0, s.SelectedRule), max(0, s.ItemCount()-1))
}

func (s *SkillsScreen) ToggleSelectedSkill() {
	if s.ActiveTab != 0 || s.SelectedRule < 0 || s.SelectedRule >= len(s.SharedLibrary) {
		return
	}
	id := s.SharedLibrary[s.SelectedRule].ID
	s.EnabledSkills[id] = !s.EnabledSkills[id]
	s.EnabledDirty = true
}

func (s *SkillsScreen) SaveEnabledSelection() error {
	var ids []string
	for id, enabled := range s.EnabledSkills {
		if enabled {
			ids = append(ids, id)
		}
	}
	sort.Strings(ids)
	if err := s.Service.SaveEnabledSkills(ids); err != nil {
		s.Error = err.Error()
		return err
	}
	s.EnabledDirty = false
	s.Error = ""
	s.Refresh()
	return nil
}

func (s *SkillsScreen) StartLibraryEdit() {
	s.EditingLibrary = true
	s.LibraryInput = textinput.New()
	s.LibraryInput.Placeholder = "Đường dẫn thư mục skill chung"
	s.LibraryInput.CharLimit = 4096
	s.LibraryInput.SetValue(s.LibraryPath)
	s.LibraryInput.CursorEnd()
	s.LibraryInput.Focus()
}

func (s *SkillsScreen) SaveLibraryDirectory() error {
	if s.EnabledDirty {
		return fmt.Errorf("lưu danh sách skill bật/tắt trước khi đổi thư viện")
	}
	if err := s.Service.SaveSharedSkillDirectory(s.LibraryInput.Value()); err != nil {
		s.Error = err.Error()
		return err
	}
	s.EditingLibrary = false
	s.LibraryInput.Blur()
	s.Error = ""
	s.Refresh()
	return nil
}

func (s *SkillsScreen) CancelLibraryEdit() {
	s.EditingLibrary = false
	s.LibraryInput.Blur()
}

func (s *SkillsScreen) StartResolve(decision string) error {
	if s.ActiveTab != 3 {
		return fmt.Errorf("chuyển sang mục Ứng viên trước")
	}
	items := s.FilteredRules()
	if s.SelectedRule < 0 || s.SelectedRule >= len(items) {
		return fmt.Errorf("chọn ứng viên trước khi xử lý")
	}
	s.ResolveID = items[s.SelectedRule].ID
	s.ResolveDecision = decision
	s.ReasonInput = textinput.New()
	s.ReasonInput.Placeholder = "Lý do xác nhận hoặc từ chối bắt buộc..."
	s.ReasonInput.CharLimit = 4096
	s.ReasonInput.Focus()
	s.Resolving = true
	s.Error = ""
	return nil
}

func (s *SkillsScreen) SaveResolution() error {
	reason := strings.TrimSpace(s.ReasonInput.Value())
	if reason == "" {
		return fmt.Errorf("lý do xác nhận/từ chối là bắt buộc")
	}
	if err := s.Service.ResolveSkillCandidate(context.Background(), s.ResolveID, s.ResolveDecision, reason); err != nil {
		s.Error = err.Error()
		return err
	}
	s.Resolving = false
	s.ReasonInput.Blur()
	s.Error = ""
	s.Refresh()
	return nil
}

func (s *SkillsScreen) CancelResolution() {
	s.Resolving = false
	s.ReasonInput.Blur()
}

func (s *SkillsScreen) StartSkillEdit() error {
	var items []skills.Skill
	switch s.ActiveTab {
	case 0:
		items = s.SharedLibrary
	case 1:
		items = s.SeriesSkill
	default:
		return fmt.Errorf("chỉ có thể sửa skill chung hoặc skill riêng")
	}
	if s.SelectedRule < 0 || s.SelectedRule >= len(items) {
		return fmt.Errorf("chọn skill trước khi sửa")
	}
	doc := items[s.SelectedRule]
	data, err := os.ReadFile(doc.Path)
	if err != nil {
		return err
	}
	s.EditSkill = doc
	s.EditHash = domain.Hash(data)
	s.EditDirectory = s.LibraryPath
	s.Editor = textarea.New()
	s.Editor.CharLimit = 0
	s.Editor.ShowLineNumbers = true
	s.Editor.SetValue(strings.ReplaceAll(strings.ReplaceAll(string(data), "\r\n", "\n"), "\r", "\n"))
	s.Editor.Focus()
	s.Editing = true
	s.EditDirty = false
	s.Error = ""
	return nil
}

func (s *SkillsScreen) SaveSkillEdit() error {
	if !s.Editing {
		return nil
	}
	if !s.EditDirty {
		s.CancelSkillEdit()
		return nil
	}
	text := s.Editor.Value()
	var err error
	if s.ActiveTab == 0 {
		err = s.Service.SaveSharedSkill(context.Background(), s.EditDirectory, s.EditSkill.ID, s.EditHash, text)
	} else {
		err = s.Service.SaveSeriesSkill(context.Background(), s.EditSkill.ID, s.EditHash, text)
	}
	if err != nil {
		s.Error = err.Error()
		return err
	}
	s.Editing = false
	s.EditDirty = false
	s.Error = ""
	s.Refresh()
	return nil
}

func (s *SkillsScreen) CancelSkillEdit() {
	s.Editing = false
	s.EditDirty = false
	s.Editor.Blur()
}

func (s *SkillsScreen) FilteredRules() []domain.Rule {
	if s.ActiveTab == 3 {
		var result []domain.Rule
		for _, rule := range s.Rules {
			if rule.Status == "candidate" {
				result = append(result, rule)
			}
		}
		return result
	}
	if s.ActiveTab == 2 {
		return s.Rules
	}
	return nil
}

func (s *SkillsScreen) ItemCount() int {
	switch s.ActiveTab {
	case 0:
		return len(s.SharedSkills)
	case 1:
		return len(s.SeriesSkill)
	case 2, 3:
		return len(s.FilteredRules())
	case 4:
		return len(s.Research)
	default:
		return 0
	}
}

func (s *SkillsScreen) View(th theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)
	if s.Resolving {
		s.ReasonInput.Width = max(1, w-8)
		decision := "XÁC NHẬN ỨNG VIÊN"
		if s.ResolveDecision == "reject" {
			decision = "TỪ CHỐI ỨNG VIÊN"
		}
		lines := []string{
			th.TextBold.Render(decision + " · " + s.ResolveID),
			th.TextMuted.Render("Ghi lý do để lưu lịch sử quyết định. Thay đổi rules có thể cần retranslate plan/apply."),
			"",
			"Lý do:",
			"  " + s.ReasonInput.View(),
			"",
			th.TextMuted.Render("Ctrl+S/Enter: lưu · Esc: hủy"),
		}
		if s.Error != "" {
			lines = append(lines, "", th.TextError.Render(cleanDisplayText(s.Error)))
		}
		return components.RenderPanel(th, components.Panel{Title: "Quyết định ứng viên", Focused: focused, Width: w, Height: h, Lines: lines})
	}
	if s.EditingLibrary {
		s.LibraryInput.Width = max(1, w-8)
		lines := []string{
			th.TextBold.Render("THƯ MỤC SKILL CHUNG"),
			th.TextMuted.Render("Thư viện này dùng chung cho các workspace đang dùng cấu hình mặc định."),
			th.TextMuted.Render("Thư mục phải nằm ngoài workspace hiện tại; skill mặc định sẽ được cài nếu còn thiếu."),
			"",
			"Đường dẫn thư mục:",
			"  " + s.LibraryInput.View(),
			"",
			th.TextMuted.Render("Ctrl+O: chọn thư mục · Ctrl+S/Enter: lưu · Esc: hủy"),
		}
		if s.Error != "" {
			lines = append(lines, "", th.TextError.Render(cleanDisplayText(s.Error)))
		}
		return components.RenderPanel(th, components.Panel{Title: "Cấu hình thư viện", Focused: focused, Width: w, Height: h, Lines: lines})
	}
	if s.Editing {
		s.Editor.SetWidth(max(1, w-6))
		s.Editor.SetHeight(max(3, h-7))
		lines := []string{
			th.TextBold.Render("SỬA SKILL · " + s.EditSkill.ID),
			th.TextMuted.Render(s.EditSkill.Path),
			"",
			s.Editor.View(),
			"",
			th.TextMuted.Render("Ctrl+S: lưu (kiểm tra hash trước khi ghi) · Esc: hủy sửa · Ctrl+O không áp dụng"),
		}
		if s.Error != "" {
			lines = append(lines, "", th.TextError.Render(cleanDisplayText(s.Error)))
		}
		return components.RenderPanel(th, components.Panel{Title: "Trình sửa skill", Focused: focused, Width: w, Height: h, Lines: lines})
	}
	tabW := max(17, w/5)
	listW := max(30, w/2)
	detailW := max(18, w-tabW-listW-2)
	if tabW+listW+detailW+2 > w {
		listW = max(24, w-tabW-detailW-2)
	}

	var tabLines []string
	for i, tab := range s.Tabs() {
		prefix := "  "
		if i == s.ActiveTab {
			prefix = "› "
		}
		style := th.SidebarItem
		if i == s.ActiveTab {
			style = th.SidebarActive
		}
		tabLines = append(tabLines, style.Render(prefix+tab))
	}
	tabLines = append(tabLines, "", th.TextMuted.Render("←/→ đổi mục"))
	tabPanel := components.RenderPanel(th, components.Panel{
		Title:   "Thư viện",
		Focused: focused && s.FocusedPane == 0,
		Width:   tabW,
		Height:  h,
		Lines:   tabLines,
	})

	var listLines []string
	switch s.ActiveTab {
	case 0:
		listLines = append(listLines, th.TextMuted.Render(fmt.Sprintf("Skill chung · %d module · Space bật/tắt · Ctrl+S lưu", len(s.SharedSkills))))
		for i, skill := range s.SharedSkills {
			prefix := "  "
			style := th.Text
			if i == s.SelectedRule {
				prefix = "› "
				if s.FocusedPane == 1 {
					style = th.TextAccent.Bold(true)
				}
			}
			enabled := "[ ]"
			if s.EnabledSkills[skill.ID] {
				enabled = "[x]"
			}
			state := ""
			if s.LoadedShared[skill.ID] {
				state = " · đang nạp"
			}
			listLines = append(listLines, style.Render(fmt.Sprintf("%s%s %s · priority %d%s", prefix, enabled, skill.ID, skill.Priority, state)))
			listLines = append(listLines, th.TextMuted.Render("  "+skill.Path))
		}
	case 1:
		listLines = append(listLines, th.TextMuted.Render("Skill riêng của workspace"))
		for i, skill := range s.SeriesSkill {
			prefix := "  "
			style := th.Text
			if i == s.SelectedRule {
				prefix = "› "
				if s.FocusedPane == 1 {
					style = th.TextAccent.Bold(true)
				}
			}
			listLines = append(listLines, style.Render(prefix+skill.ID))
			listLines = append(listLines, th.TextMuted.Render("  "+skill.Path))
		}
	case 2, 3:
		filtered := s.FilteredRules()
		listLines = append(listLines, th.TextMuted.Render("Key | Value | Loại | Trạng thái"))
		listLines = append(listLines, strings.Repeat("─", listW-4))
		for i, rule := range filtered {
			prefix := "  "
			style := th.Text
			if i == s.SelectedRule {
				prefix = "› "
				if s.FocusedPane == 1 {
					style = th.TextAccent.Bold(true)
				}
			}
			row := fmt.Sprintf("%s%s | %s | %s | %s", prefix, rule.Key, rule.Value, rule.Kind, rule.Status)
			listLines = append(listLines, style.Render(row))
		}
		if s.ActiveTab == 3 {
			listLines = append(listLines, "", th.TextMuted.Render("c: xác nhận · x: từ chối · cần ghi lý do"))
		}
	case 4:
		listLines = append(listLines, th.TextMuted.Render(fmt.Sprintf("Evidence đã lưu · %d mục · không gọi research khi mở màn", len(s.Research))))
		for i, evidence := range s.Research {
			prefix := "  "
			style := th.Text
			if i == s.SelectedRule {
				prefix = "› "
				if s.FocusedPane == 1 {
					style = th.TextAccent.Bold(true)
				}
			}
			listLines = append(listLines, style.Render(prefix+evidence.Term+" · "+evidence.Title))
			listLines = append(listLines, th.TextMuted.Render("  "+evidence.Status+" · "+evidence.SourceID))
		}
	}
	if s.ActiveTab == 0 || s.ActiveTab == 1 {
		listLines = append(listLines, "", th.TextMuted.Render("e: sửa skill · l: chọn thư mục chung"))
	}
	if s.EnabledDirty {
		listLines = append(listLines, th.TextWarning.Render("Danh sách bật/tắt chưa lưu · Ctrl+S để áp dụng"))
	}
	if s.ItemCount() == 0 {
		listLines = append(listLines, "", th.TextMuted.Render("Không có dữ liệu trong mục này."))
	}
	if s.Error != "" {
		listLines = append(listLines, "", th.TextError.Render(cleanDisplayText(s.Error)))
	}
	listPanel := components.RenderPanel(th, components.Panel{
		Title:   s.Tabs()[s.ActiveTab],
		Focused: focused && s.FocusedPane == 1,
		Width:   listW,
		Height:  h,
		Scroll:  max(0, s.SelectedRule+2-(h-4)/2),
		Lines:   listLines,
	})

	var detailLines []string
	detailLines = append(detailLines, th.PanelTitle.Render("CHI TIẾT & BẰNG CHỨNG"))
	switch s.ActiveTab {
	case 0, 1:
		documents := s.SharedSkills
		if s.ActiveTab == 1 {
			documents = s.SeriesSkill
		}
		if s.SelectedRule >= 0 && s.SelectedRule < len(documents) {
			doc := documents[s.SelectedRule]
			detailLines = append(detailLines, "ID: "+doc.ID, fmt.Sprintf("Priority: %d", doc.Priority), "Hash: "+doc.Hash)
			if s.ActiveTab == 0 {
				detailLines = append(detailLines, "Được bật: "+fmt.Sprint(s.EnabledSkills[doc.ID]), "Đang nạp: "+fmt.Sprint(s.LoadedShared[doc.ID]))
			}
			detailLines = append(detailLines, "", th.Text.Render(doc.Body))
		}
	case 2, 3:
		filtered := s.FilteredRules()
		if s.SelectedRule >= 0 && s.SelectedRule < len(filtered) {
			rule := filtered[s.SelectedRule]
			detailLines = append(detailLines, "ID: "+rule.ID, "Key: "+rule.Key, "Value: "+rule.Value, "Loại: "+rule.Kind, fmt.Sprintf("Ưu tiên: %d", rule.Priority), "Trạng thái: "+rule.Status)
			if rule.DecisionReason != "" {
				detailLines = append(detailLines, "Lý do: "+rule.DecisionReason)
			}
			for _, evidence := range rule.Evidence {
				detailLines = append(detailLines, "", "Khối: "+evidence.BlockID, th.QuoteHighlight.Render(evidence.Quote))
			}
		}
	case 4:
		if s.SelectedRule >= 0 && s.SelectedRule < len(s.Research) {
			evidence := s.Research[s.SelectedRule]
			detailLines = append(detailLines, "Title: "+evidence.Title, "Term: "+evidence.Term, "Status: "+evidence.Status, "Source: "+evidence.SourceID, "Edition: "+evidence.Edition, "Locator: "+evidence.Locator, "URL: "+evidence.URL, "Fetched: "+evidence.FetchedAt, "", th.Text.Render(evidence.Content))
		}
	}
	if len(detailLines) == 1 {
		detailLines = append(detailLines, th.TextMuted.Render("Chọn một mục để xem nội dung."))
	}
	detailPanel := components.RenderPanel(th, components.Panel{
		Title:   "Xem nội dung",
		Focused: focused && s.FocusedPane == 2,
		Width:   detailW,
		Height:  h,
		Lines:   detailLines,
	})
	return lipgloss.JoinHorizontal(lipgloss.Top, tabPanel, " ", listPanel, " ", detailPanel)
}

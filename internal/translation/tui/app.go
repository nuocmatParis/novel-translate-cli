package tui

import (
	"context"
	"fmt"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/progress"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/screens"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type App struct {
	Service               *app.AppService
	JobManager            *app.JobManager
	JobContext            context.Context
	Theme                 theme.Theme
	Width                 int
	Height                int
	CurrentScreen         string
	SelectedVolumeID      string
	SidebarIndex          int
	FocusSidebar          bool
	ActiveModal           *components.Modal
	PathPicker            *pathPickerState
	Notice                string
	Error                 string
	PendingDraftExit      string
	DraftExitWaiting      bool
	PendingSkillDraftExit string
	Events                chan progress.Event
	CoverCache            map[string]string
	CoverToken            uint64
	PreviousBusy          bool
	ExitAfterStop         bool

	WelcomeScreen    *screens.WelcomeScreen
	DashboardScreen  *screens.DashboardScreen
	ContentScreen    *screens.ContentScreen
	JobScreen        *screens.JobScreen
	EditorScreen     *screens.EditorScreen
	ReviewScreen     *screens.ReviewScreen
	SkillsScreen     *screens.SkillsScreen
	ContinuityScreen *screens.ContinuityScreen
	VerifyScreen     *screens.VerifyScreen
	SettingsScreen   *screens.SettingsScreen
	ReportsScreen    *screens.ReportsScreen
}

func (a *App) SetJobContext(ctx context.Context) {
	if ctx == nil {
		ctx = context.Background()
	}
	a.JobContext = ctx
}

func (a *App) jobContext() context.Context {
	if a.JobContext == nil {
		return context.Background()
	}
	return a.JobContext
}

type MenuItem struct {
	ID    string
	Title string
}

func MenuItems() []MenuItem {
	return []MenuItem{
		{"welcome", "Chọn bộ truyện"},
		{"dashboard", "Dashboard"},
		{"content", "Bộ truyện & nội dung"},
		{"job", "Tác vụ đang chạy"},
		{"editor", "Bản dịch"},
		{"review", "Reviewer"},
		{"skills", "Skills & thuật ngữ"},
		{"continuity", "Ngữ cảnh & trí nhớ"},
		{"verify", "Kiểm tra & EPUB"},
		{"settings", "Cấu hình"},
		{"reports", "Báo cáo & usage"},
		{"help", "Trợ giúp"},
	}
}

type tickMsg time.Time

func tickCmd() tea.Cmd {
	return tea.Tick(150*time.Millisecond, func(t time.Time) tea.Msg {
		return tickMsg(t)
	})
}

func NewApp(service *app.AppService) *App {
	th := theme.DefaultTheme()
	jm := app.NewJobManager(service)

	appInstance := &App{
		Service:       service,
		JobManager:    jm,
		Theme:         th,
		Width:         100,
		Height:        30,
		CurrentScreen: "welcome",
		FocusSidebar:  false,
		Notice:        "Chọn workspace để bắt đầu. Mở giao diện không gọi API.",
		Events:        make(chan progress.Event, 50),
		CoverCache:    make(map[string]string),
	}

	jm.Broadcaster().Subscribe(progress.ObserverFunc(func(e progress.Event) {
		select {
		case appInstance.Events <- e:
		default:
		}
	}))

	if service != nil && service.Config != nil {
		appInstance.CurrentScreen = "dashboard"
		appInstance.Notice = "Workspace đã mở: " + service.Config.Series.Title
	}
	appInstance.initScreens()

	return appInstance
}

func (a *App) initScreens() {
	volID := ""
	if a.Service != nil && a.Service.Config != nil && len(a.Service.Config.Volumes) > 0 {
		volID = a.SelectedVolumeID
		found := false
		for _, v := range a.Service.Config.Volumes {
			if v.ID == volID {
				found = true
				break
			}
		}
		if !found {
			volID = a.Service.Config.Volumes[0].ID
		}
		a.SelectedVolumeID = volID
	}

	a.WelcomeScreen = screens.NewWelcomeScreen(a.Service)
	a.DashboardScreen = screens.NewDashboardScreen(a.Service)
	a.ContentScreen = screens.NewContentScreen(a.Service)
	a.JobScreen = screens.NewJobScreen(a.JobManager)
	a.EditorScreen = screens.NewEditorScreen(a.Service, volID)
	a.ReviewScreen = screens.NewReviewScreen(a.Service, volID)
	a.SkillsScreen = screens.NewSkillsScreen(a.Service)
	a.ContinuityScreen = screens.NewContinuityScreen(a.Service, volID)
	a.VerifyScreen = screens.NewVerifyScreen(a.Service, volID)
	a.SettingsScreen = screens.NewSettingsScreen(a.Service)
	a.ReportsScreen = screens.NewReportsScreen(a.Service)
	if a.DashboardScreen != nil {
		for i, volume := range a.DashboardScreen.Projection.Volumes {
			if volume.ID == volID {
				a.DashboardScreen.SelectedVolume = i
				break
			}
		}
	}
	if a.SelectedVolumeID == "" && volID != "" {
		a.SelectedVolumeID = volID
	}
	for i, item := range MenuItems() {
		if item.ID == a.CurrentScreen {
			a.SidebarIndex = i
			break
		}
	}
}

// SetSelectedVolumeID is used by the CLI adapter before the terminal loop
// starts. Interactive changes go through selectVolume so the cover can reload.
func (a *App) SetSelectedVolumeID(volumeID string) error {
	if a.Service == nil || a.Service.Config == nil {
		return fmt.Errorf("chưa mở workspace")
	}
	if _, err := a.Service.Config.Volume(volumeID); err != nil {
		return err
	}
	a.SelectedVolumeID = volumeID
	for i, volume := range a.Service.Config.Volumes {
		if volume.ID == volumeID {
			if a.DashboardScreen != nil {
				a.DashboardScreen.SelectedVolume = i
			}
			if a.ContentScreen != nil {
				a.ContentScreen.SelectedVolume = i
			}
			break
		}
	}
	return nil
}

// OpenScreen exposes the TUI router to the CLI's startup flag adapter.
func (a *App) OpenScreen(id string) tea.Cmd { return a.switchScreen(id) }

func (a *App) focusedPane() int {
	if a.FocusSidebar {
		return -1
	}
	switch a.CurrentScreen {
	case "dashboard":
		return a.DashboardScreen.FocusedPane
	case "content":
		return a.ContentScreen.FocusedPane
	case "job":
		return a.JobScreen.FocusedPane
	case "editor":
		return a.EditorScreen.FocusedPane
	case "review":
		return a.ReviewScreen.FocusedPane
	case "skills":
		return a.SkillsScreen.FocusedPane
	case "continuity":
		return a.ContinuityScreen.FocusedPane
	case "verify":
		return a.VerifyScreen.FocusedPane
	default:
		return 0
	}
}

func (a *App) setFocusedPane(index int) {
	switch a.CurrentScreen {
	case "dashboard":
		a.DashboardScreen.FocusedPane = index
	case "content":
		a.ContentScreen.FocusedPane = index
	case "job":
		a.JobScreen.FocusedPane = index
	case "editor":
		a.EditorScreen.FocusedPane = index
	case "review":
		a.ReviewScreen.FocusedPane = index
	case "skills":
		a.SkillsScreen.FocusedPane = index
	case "continuity":
		a.ContinuityScreen.FocusedPane = index
	case "verify":
		a.VerifyScreen.FocusedPane = index
	}
}

func (a *App) focusPaneCount() int {
	switch a.CurrentScreen {
	case "dashboard":
		return 2
	case "content", "job", "editor", "review", "skills", "verify":
		return 3
	case "continuity":
		return 2
	default:
		return 1
	}
}

func (a *App) cycleFocus(backwards bool) {
	layout := CalculateLayout(a.Width, a.Height)
	hasSidebar := layout.SidebarWidth > 0
	count := a.focusPaneCount()
	current := a.focusedPane()
	if !hasSidebar {
		if backwards {
			a.setFocusedPane((current - 1 + count) % count)
		} else {
			a.setFocusedPane((current + 1) % count)
		}
		return
	}
	if a.FocusSidebar {
		a.FocusSidebar = false
		if backwards {
			a.setFocusedPane(count - 1)
		} else {
			a.setFocusedPane(0)
		}
		return
	}
	if backwards {
		if current > 0 {
			a.setFocusedPane(current - 1)
		} else {
			a.FocusSidebar = true
		}
		return
	}
	if current+1 < count {
		a.setFocusedPane(current + 1)
	} else {
		a.FocusSidebar = true
	}
}

func (a *App) switchScreen(id string) tea.Cmd {
	if a.CurrentScreen == "editor" && id != "editor" && a.EditorScreen != nil && a.EditorScreen.HasUnsavedChanges() {
		a.prepareDraftExit(id)
		return nil
	}
	if a.CurrentScreen == "skills" && id != "skills" && a.SkillsScreen != nil && a.SkillsScreen.Editing && a.SkillsScreen.EditDirty {
		a.prepareSkillDraftExit(id)
		return nil
	}
	valid := false
	for i, item := range MenuItems() {
		if item.ID == id {
			a.SidebarIndex = i
			valid = true
			break
		}
	}
	if !valid {
		return nil
	}
	a.CurrentScreen = id
	a.FocusSidebar = false
	a.setFocusedPane(0)
	switch id {
	case "welcome":
		a.WelcomeScreen.Refresh()
	case "dashboard":
		a.DashboardScreen.Refresh()
		return a.requestDashboardCover()
	case "content":
		if a.Service != nil && a.Service.Config != nil {
			for i, volume := range a.Service.Config.Volumes {
				if volume.ID == a.SelectedVolumeID {
					a.ContentScreen.SelectedVolume = i
					break
				}
			}
		}
		a.ContentScreen.Refresh()
	case "editor":
		if a.SelectedVolumeID != "" {
			a.EditorScreen.VolumeID = a.SelectedVolumeID
			a.EditorScreen.LoadBlocks()
		}
	case "review":
		if a.SelectedVolumeID != "" {
			a.ReviewScreen.VolumeID = a.SelectedVolumeID
			a.ReviewScreen.Refresh()
		}
	case "continuity":
		if a.SelectedVolumeID != "" {
			a.ContinuityScreen.VolumeID = a.SelectedVolumeID
			a.ContinuityScreen.Refresh()
		}
	case "verify":
		if a.SelectedVolumeID != "" {
			a.VerifyScreen.VolumeID = a.SelectedVolumeID
			a.VerifyScreen.Refresh()
		}
	case "skills":
		a.SkillsScreen.Refresh()
	case "reports":
		a.ReportsScreen.Refresh()
	}
	return nil
}

func (a *App) Init() tea.Cmd {
	return tea.Batch(tickCmd(), a.requestDashboardCover())
}

func (a *App) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch m := msg.(type) {
	case tea.WindowSizeMsg:
		a.Width = m.Width
		a.Height = m.Height
		if a.PathPicker != nil {
			a.PathPicker.model.Height = max(3, m.Height-8)
		}
		if CalculateLayout(a.Width, a.Height).SidebarWidth == 0 {
			a.FocusSidebar = false
		}
		return a, a.requestDashboardCover()

	case coverDetailsMsg:
		if m.token != a.CoverToken || a.DashboardScreen == nil {
			return a, nil
		}
		a.DashboardScreen.Cover = m.volume
		a.DashboardScreen.CoverPreview = ""
		if m.err != nil {
			a.DashboardScreen.CoverLoading = false
			a.DashboardScreen.CoverMessage = theme.CleanText(m.err.Error())
			return a, nil
		}
		if !m.volume.CoverAvailable {
			a.DashboardScreen.CoverLoading = false
			a.DashboardScreen.CoverMessage = "EPUB không khai báo ảnh bìa."
			return a, nil
		}
		key := fmt.Sprintf("%s|%s|%s|%s|%d|%d|%t", m.root, m.volume.VolumeID, m.volume.SourceHash, m.volume.ResourceHash, m.columns, m.rows, m.useColor)
		if preview, ok := a.CoverCache[key]; ok {
			a.DashboardScreen.CoverPreview = preview
			a.DashboardScreen.CoverLoading = false
			return a, nil
		}
		return a, renderCoverCmd(m.volume, m.columns, m.rows, m.useColor, m.token, key)

	case coverLoadedMsg:
		if m.token != a.CoverToken || a.DashboardScreen == nil {
			return a, nil
		}
		a.DashboardScreen.Cover = m.volume
		a.DashboardScreen.CoverLoading = false
		a.DashboardScreen.CoverMessage = theme.CleanText(m.message)
		a.DashboardScreen.CoverPreview = m.preview
		if m.err == nil && m.key != "" && m.preview != "" {
			if len(a.CoverCache) >= 64 {
				for cachedKey := range a.CoverCache {
					delete(a.CoverCache, cachedKey)
					break
				}
			}
			a.CoverCache[m.key] = m.preview
		}
		return a, nil

	case tickMsg:
		if a.JobScreen != nil {
			a.JobScreen.Tick()
		}
		// Drain progress events
		for {
			select {
			case ev := <-a.Events:
				if a.JobScreen != nil {
					a.JobScreen.LastEvent = &ev
					switch data := ev.Data.(type) {
					case domain.Block:
						block := data
						a.JobScreen.CurrentBlock = &block
						if ev.Kind == progress.EventUnitStarted {
							a.JobScreen.CurrentUnit = nil
						}
					case domain.Unit:
						unit := data
						a.JobScreen.CurrentUnit = &unit
					}
				}
				if a.DashboardScreen != nil && ev.VolumeID != "" {
					a.DashboardScreen.ActiveVolumeID = ev.VolumeID
				}
				if ev.Message != "" {
					a.Notice = theme.CleanText(ev.Message)
				}
				if ev.Error != "" {
					a.Error = theme.CleanText(ev.Error)
				}
			default:
				busy := a.JobManager != nil && a.JobManager.IsBusy()
				var next tea.Cmd
				if a.DashboardScreen != nil {
					if busy {
						status := a.JobManager.Status()
						a.DashboardScreen.ActiveVolumeID = status.VolumeID
						a.DashboardScreen.ActiveJobLabel = status.Label
						a.DashboardScreen.ActiveJobPhase = status.Phase
					} else if a.PreviousBusy {
						a.DashboardScreen.ActiveVolumeID = ""
						a.DashboardScreen.ActiveJobLabel = ""
						a.DashboardScreen.ActiveJobPhase = ""
						a.DashboardScreen.Refresh()
						switch a.CurrentScreen {
						case "content":
							a.ContentScreen.Refresh()
						case "editor":
							if a.EditorScreen.HasUnsavedChanges() {
								a.Notice = "Tác vụ đã xong; bản nháp editor vẫn được giữ để bạn lưu hoặc đối chiếu."
							} else {
								a.EditorScreen.LoadBlocks()
							}
						case "review":
							a.ReviewScreen.Refresh()
						case "continuity":
							a.ContinuityScreen.Refresh()
						case "verify":
							a.VerifyScreen.Refresh()
						case "reports":
							a.ReportsScreen.Refresh()
						}
						if a.CurrentScreen == "dashboard" {
							next = a.requestDashboardCover()
						}
					}
				}
				a.PreviousBusy = busy
				if !busy && a.DraftExitWaiting {
					a.DraftExitWaiting = false
					a.showDraftDecision()
				}
				if a.ExitAfterStop && !busy {
					return a, tea.Quit
				}
				return a, tea.Batch(tickCmd(), next)
			}
		}

	case tea.KeyMsg:
		return a.handleKeyPress(m)
	}
	if a.CurrentScreen == "welcome" && (a.WelcomeScreen.Creating || a.WelcomeScreen.Opening) {
		var cmd tea.Cmd
		if a.WelcomeScreen.Creating {
			a.WelcomeScreen.TitleInput, cmd = a.WelcomeScreen.TitleInput.Update(msg)
		} else {
			a.WelcomeScreen.PathInput, cmd = a.WelcomeScreen.PathInput.Update(msg)
		}
		return a, cmd
	}

	return a, nil
}

func (a *App) requestDashboardCover() tea.Cmd {
	if a.Service == nil || a.Service.Config == nil || a.DashboardScreen == nil || len(a.Service.Config.Volumes) == 0 {
		a.CoverToken++
		if a.DashboardScreen != nil {
			a.DashboardScreen.Cover = app.VolumeCover{}
			a.DashboardScreen.CoverPreview = ""
			a.DashboardScreen.CoverMessage = ""
			a.DashboardScreen.CoverLoading = false
		}
		return nil
	}
	volumeID := a.SelectedVolumeID
	if a.JobManager != nil && a.JobManager.IsBusy() {
		status := a.JobManager.Status()
		a.DashboardScreen.ActiveVolumeID = status.VolumeID
		a.DashboardScreen.ActiveJobLabel = status.Label
		a.DashboardScreen.ActiveJobPhase = status.Phase
		if status.VolumeID != "" {
			volumeID = status.VolumeID
		}
	} else {
		a.DashboardScreen.ActiveVolumeID = ""
		a.DashboardScreen.ActiveJobLabel = ""
		a.DashboardScreen.ActiveJobPhase = ""
	}
	if volumeID == "" {
		volumeID = a.Service.Config.Volumes[0].ID
	}
	layout := CalculateLayout(max(80, a.Width), max(24, a.Height))
	columns := min(17, max(8, (layout.ContentWidth/2-4)/3))
	rows := min(9, max(5, (layout.ContentHeight-2)/2))
	a.CoverToken++
	token := a.CoverToken
	a.DashboardScreen.Cover = app.VolumeCover{}
	a.DashboardScreen.CoverPreview = ""
	a.DashboardScreen.CoverLoading = true
	a.DashboardScreen.CoverMessage = ""
	serviceSnapshot := *a.Service
	configSnapshot := *a.Service.Config
	serviceSnapshot.Config = &configSnapshot
	return loadCoverDetailsCmd(&serviceSnapshot, volumeID, columns, rows, token)
}

func (a *App) selectVolume(volumeID string) tea.Cmd {
	if a.Service == nil || a.Service.Config == nil {
		return nil
	}
	if _, err := a.Service.Config.Volume(volumeID); err != nil {
		a.Error = err.Error()
		return nil
	}
	volumeChanged := a.SelectedVolumeID != volumeID
	a.SelectedVolumeID = volumeID
	if a.DashboardScreen != nil {
		for i, volume := range a.DashboardScreen.Projection.Volumes {
			if volume.ID == volumeID {
				a.DashboardScreen.SelectedVolume = i
				break
			}
		}
	}
	if a.ContentScreen != nil {
		if volumeChanged {
			a.ContentScreen.CheckedChapters = make(map[string]bool)
			a.ContentScreen.SelectedChapter = 0
		}
		for i, volume := range a.Service.Config.Volumes {
			if volume.ID == volumeID {
				a.ContentScreen.SelectedVolume = i
				break
			}
		}
	}
	if a.EditorScreen != nil {
		a.EditorScreen.VolumeID = volumeID
	}
	if a.ReviewScreen != nil {
		a.ReviewScreen.VolumeID = volumeID
	}
	if a.ContinuityScreen != nil {
		a.ContinuityScreen.VolumeID = volumeID
	}
	if a.VerifyScreen != nil {
		a.VerifyScreen.VolumeID = volumeID
	}
	if a.CurrentScreen == "dashboard" {
		return a.requestDashboardCover()
	}
	return nil
}

func (a *App) View() string {
	layout := CalculateLayout(a.Width, a.Height)

	if layout.Tier == SizeCompact {
		message := "KÍCH THƯỚC TERMINAL QUÁ NHỎ\nMở rộng terminal tối thiểu 80 × 24 để xem nội dung.\n\nCtrl+C: Dừng tác vụ · q: Thoát · ?: Trợ giúp"
		if a.JobManager != nil && a.JobManager.IsBusy() {
			message += "\nTác vụ vẫn chạy; Ctrl+C yêu cầu dừng an toàn."
		}
		return lipgloss.Place(
			max(1, a.Width),
			max(1, a.Height),
			lipgloss.Center,
			lipgloss.Center,
			a.Theme.TextWarning.Render(message),
		)
	}

	appName := "NOVEL TRANSLATE"
	seriesTitle := ""
	if a.Service != nil && a.Service.Config != nil {
		seriesTitle = a.Service.Config.Series.Title
	}

	jobStatus := ""
	if a.JobManager != nil {
		status := a.JobManager.Status()
		if status.State == app.StateRunning {
			switch status.Kind {
			case "import":
				jobStatus = "Import"
			case "export":
				jobStatus = "Xuất EPUB"
			default:
				jobStatus = "Dịch / Review"
			}
		} else if status.State == app.StatePaused {
			jobStatus = "Tạm dừng"
		}
	}

	menuItems := MenuItems()
	screenName := a.CurrentScreen
	for _, it := range menuItems {
		if it.ID == a.CurrentScreen {
			screenName = it.Title
			break
		}
	}

	header := RenderHeader(a.Theme, a.Width, appName, seriesTitle, screenName, jobStatus)

	var sidebar string
	if layout.SidebarWidth > 0 {
		var sbLines []string
		sbLines = append(sbLines, a.Theme.PanelTitle.Render("ĐIỀU HƯỚNG"))
		for i, it := range menuItems {
			prefix := "  "
			if it.ID == a.CurrentScreen {
				prefix = "› "
				sbLines = append(sbLines, a.Theme.SidebarActive.Width(layout.SidebarWidth-2).Render(prefix+it.Title))
			} else if a.FocusSidebar && i == a.SidebarIndex {
				prefix = "› "
				sbLines = append(sbLines, a.Theme.TableRowSelected.Width(layout.SidebarWidth-2).Render(prefix+it.Title))
			} else {
				sbLines = append(sbLines, a.Theme.SidebarItem.Render(prefix+it.Title))
			}
		}
		sidebar = components.RenderPanel(a.Theme, components.Panel{
			Title:   "Menu",
			Focused: a.FocusSidebar,
			Width:   layout.SidebarWidth,
			Height:  layout.ContentHeight,
			Lines:   sbLines,
		})
	}

	// Content
	content := ""
	if a.PathPicker != nil {
		content = a.PathPicker.View(a.Theme, layout.ContentWidth, layout.ContentHeight)
	} else {
		switch a.CurrentScreen {
		case "welcome":
			content = a.WelcomeScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "dashboard":
			content = a.DashboardScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "content":
			content = a.ContentScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "job":
			content = a.JobScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "editor":
			content = a.EditorScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "review":
			content = a.ReviewScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "skills":
			content = a.SkillsScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "continuity":
			content = a.ContinuityScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "verify":
			content = a.VerifyScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "settings":
			content = a.SettingsScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "reports":
			content = a.ReportsScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		case "help":
			var lines []string
			lines = append(lines, a.Theme.TextBold.Render("TRỢ GIÚP ĐIỀU KHIỂN"), "")
			for _, binding := range theme.CommonKeys() {
				lines = append(lines, a.Theme.TextAccent.Render(binding.Key)+"  "+binding.Description)
			}
			lines = append(lines, "", a.Theme.TextMuted.Render("Mở giao diện không gọi model. Chỉ nút Bắt đầu trong phạm vi mới chạy dịch hoặc review."))
			content = components.RenderPanel(a.Theme, components.Panel{Title: "Trợ giúp", Focused: !a.FocusSidebar, Width: layout.ContentWidth, Height: layout.ContentHeight, Lines: lines})
		default:
			content = a.DashboardScreen.View(a.Theme, layout.ContentWidth, layout.ContentHeight, !a.FocusSidebar)
		}
	}

	body := content
	if sidebar != "" {
		body = lipgloss.JoinHorizontal(lipgloss.Top, sidebar, " ", content)
	}

	// Available shortcuts for footer
	keys := "Tab: Đổi panel · Enter: Mở/Thực hiện · ?: Trợ giúp · q: Thoát"
	if a.PathPicker != nil {
		keys = "↑/↓: Chọn · Enter: Mở/chọn · Esc: Hủy"
		if a.PathPicker.directory {
			keys = "↑/↓: Chọn · Enter: Mở thư mục · Space: Chọn thư mục · Esc: Hủy"
		}
	} else if a.CurrentScreen == "welcome" && a.WelcomeScreen.Creating {
		keys = "Nhập tên bộ truyện · Enter: Tạo workspace · Esc: Hủy"
	} else if a.CurrentScreen == "welcome" && a.WelcomeScreen.Opening {
		keys = "Nhập đường dẫn workspace · Enter: Mở · Esc: Hủy"
	} else if a.FocusSidebar {
		keys = "↑/↓: Chọn màn · Enter: Chuyển màn · Tab: Vào nội dung · q: Thoát"
	} else if a.CurrentScreen == "welcome" {
		keys = "↑/↓: Chọn · Enter: Mở mục · o: Mở đường dẫn · n: Tạo mới · r: Làm mới"
	} else if a.CurrentScreen == "review" {
		keys = "↑/↓: Chọn · 0–4: Lọc · ←/→/Enter: Thao tác · k: Giữ · r: Review lại · e: Sửa tay"
	} else if a.CurrentScreen == "content" {
		keys = "↑/↓: Chọn · Space: Chọn chương liên tiếp · i: Import · Enter: Chạy scope · a: Thêm tập"
	} else if a.CurrentScreen == "skills" {
		keys = "←/→: Danh mục · Space: Bật/tắt · Ctrl+S: Lưu · e: Sửa · l: Thư mục · c/x: Candidate · Tab: Vùng"
	} else if a.CurrentScreen == "job" {
		if a.JobManager != nil && a.JobManager.Status().Kind == "translate" {
			keys = "p: Tạm dừng/Tiếp tục · Ctrl+C: Dừng tác vụ · f: Theo dõi block · Esc: Dashboard"
		} else {
			keys = "Ctrl+C: Dừng tác vụ · Esc: Dashboard"
		}
	} else if a.CurrentScreen == "verify" {
		keys = "↑/↓: Chọn · a: Đánh giá ảnh · p: Preview · f: Final · i: Import · Enter: Chọn · Tab: Đổi panel"
	} else if layout.SidebarWidth == 0 {
		keys += " · Alt+←/→: Đổi màn"
	}

	footer := RenderFooter(a.Theme, a.Width, a.Notice, a.Error, keys)

	fullView := header + "\n" + body + "\n" + footer

	if a.ActiveModal != nil {
		fullView = components.RenderModal(a.Theme, *a.ActiveModal, a.Width, a.Height)
	}

	return fullView
}

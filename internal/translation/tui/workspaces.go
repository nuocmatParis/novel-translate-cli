package tui

import tea "github.com/charmbracelet/bubbletea"

func (a *App) handleWelcomeKey(key tea.KeyMsg) (tea.Model, tea.Cmd) {
	ws := a.WelcomeScreen
	if ws.Opening {
		switch key.String() {
		case "esc":
			ws.Opening = false
			ws.Error = ""
			ws.PathInput.Blur()
			return a, nil
		case "enter":
			if a.JobManager.IsBusy() {
				ws.Error = "đợi tác vụ hiện tại kết thúc trước khi đổi bộ truyện"
				return a, nil
			}
			cfg, err := a.Service.LoadWorkspace(ws.PathInput.Value())
			if err != nil {
				ws.Error = err.Error()
				return a, nil
			}
			ws.Opening = false
			ws.PathInput.Blur()
			a.SelectedVolumeID = ""
			a.CurrentScreen = "dashboard"
			a.initScreens()
			a.Notice = "Đã mở workspace: " + cfg.Series.Title
			return a, a.requestDashboardCover()
		default:
			before := ws.PathInput.Value()
			var cmd tea.Cmd
			ws.PathInput, cmd = ws.PathInput.Update(key)
			if before != ws.PathInput.Value() {
				ws.Error = ""
			}
			return a, cmd
		}
	}
	if ws.Creating {
		switch key.String() {
		case "esc":
			ws.Creating = false
			ws.Error = ""
			ws.TitleInput.Blur()
			return a, nil
		case "enter":
			if a.JobManager.IsBusy() {
				ws.Error = "đợi tác vụ hiện tại kết thúc trước khi đổi bộ truyện"
				return a, nil
			}
			cfg, err := a.Service.CreateWorkspace(ws.TitleInput.Value())
			if err != nil {
				ws.Error = err.Error()
				return a, nil
			}
			ws.Creating = false
			ws.TitleInput.Blur()
			a.SelectedVolumeID = ""
			a.CurrentScreen = "dashboard"
			a.initScreens()
			a.Notice = "Đã tạo workspace mới: " + cfg.Series.Title
			return a, a.requestDashboardCover()
		case "tab", "shift+tab":
			return a, nil
		default:
			before := ws.TitleInput.Value()
			var cmd tea.Cmd
			ws.TitleInput, cmd = ws.TitleInput.Update(key)
			if before != ws.TitleInput.Value() {
				ws.Error = ""
			}
			return a, cmd
		}
	}
	switch key.String() {
	case "up", "k":
		ws.SelectedIndex = max(0, ws.SelectedIndex-1)
	case "down", "j":
		ws.SelectedIndex = min(len(ws.Workspaces)+1, ws.SelectedIndex+1)
	case "pgup":
		ws.SelectedIndex = max(0, ws.SelectedIndex-8)
	case "pgdown":
		ws.SelectedIndex = min(len(ws.Workspaces)+1, ws.SelectedIndex+8)
	case "home":
		ws.SelectedIndex = 0
	case "end":
		ws.SelectedIndex = len(ws.Workspaces) + 1
	case "r":
		ws.Refresh()
	case "o":
		ws.Opening = true
		ws.Error = ""
		return a, ws.PathInput.Focus()
	case "n":
		ws.Creating = true
		ws.Error = ""
		return a, ws.TitleInput.Focus()
	case "enter":
		if ws.SelectedIndex == 0 {
			ws.Creating = true
			ws.Error = ""
			return a, ws.TitleInput.Focus()
		}
		if ws.SelectedIndex == 1 {
			ws.Opening = true
			ws.Error = ""
			return a, ws.PathInput.Focus()
		}
		workspaceIndex := ws.SelectedIndex - 2
		if workspaceIndex < 0 || workspaceIndex >= len(ws.Workspaces) {
			return a, nil
		}
		if a.JobManager.IsBusy() {
			ws.Error = "đợi tác vụ hiện tại kết thúc trước khi đổi bộ truyện"
			return a, nil
		}
		item := ws.Workspaces[workspaceIndex]
		cfg, err := a.Service.LoadWorkspace(item.Root)
		if err != nil {
			ws.Error = err.Error()
			return a, nil
		}
		a.SelectedVolumeID = ""
		a.CurrentScreen = "dashboard"
		a.initScreens()
		a.Notice = "Đã mở workspace: " + cfg.Series.Title
		return a, a.requestDashboardCover()
	case "esc":
		if a.Service.Config != nil {
			return a, a.switchScreen("dashboard")
		}
	}
	return a, nil
}

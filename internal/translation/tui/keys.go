package tui

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

func (a *App) textEntryFocused() bool {
	switch a.CurrentScreen {
	case "welcome":
		return a.WelcomeScreen.Creating || a.WelcomeScreen.Opening
	case "review":
		return a.ReviewScreen.EnteringReason
	case "editor":
		return (a.EditorScreen.ActiveTab == 1 || a.EditorScreen.ActiveTab == 4) && a.EditorScreen.FocusedPane == 1
	case "settings":
		return len(a.SettingsScreen.Inputs) > 0
	case "verify":
		return a.VerifyScreen.AssessingImage
	case "content":
		return a.ContentScreen.AddingVolume
	case "skills":
		return a.SkillsScreen.Editing || a.SkillsScreen.EditingLibrary || a.SkillsScreen.Resolving
	}
	return false
}

func (a *App) openDashboardAction(index int) tea.Cmd {
	if index < 0 || index >= len(a.DashboardScreen.Actions()) {
		return nil
	}
	if index < 2 {
		a.ContentScreen.SelectedMode = index
	}
	ids := []string{"content", "content", "editor", "review", "verify", "reports"}
	return a.switchScreen(ids[index])
}

func (a *App) runReviewAction(index int) (tea.Model, tea.Cmd) {
	rs := a.ReviewScreen
	item := rs.SelectedItem()
	if item == nil {
		return a, nil
	}
	switch index {
	case 0:
		a.SelectedVolumeID = rs.VolumeID
		cmd := a.switchScreen("editor")
		for i, unit := range a.EditorScreen.Units {
			if unit.ID == item.UnitID {
				a.EditorScreen.SelectUnit(i)
				break
			}
		}
		a.EditorScreen.ActiveTab = 1
		return a, cmd
	case 1:
		scope := engine.RunScope{UnitIDs: []string{item.UnitID}}
		if err := a.JobManager.StartReview(a.jobContext(), rs.VolumeID, scope, true); err != nil {
			a.Error = err.Error()
			return a, nil
		}
		return a, a.switchScreen("job")
	case 2:
		rs.EnteringReason = true
		rs.ReasonInput.Focus()
	}
	return a, nil
}

func (a *App) prepareDraftExit(destination string) {
	a.PendingDraftExit = destination
	if a.JobManager != nil && a.JobManager.IsBusy() {
		a.DraftExitWaiting = true
		a.ActiveModal = &components.Modal{
			Type:        components.ModalConfirm,
			Title:       "DỪNG TÁC VỤ TRƯỚC KHI RỜI EDITOR",
			Message:     "Có bản nháp chưa lưu. Dừng tác vụ xong, bạn sẽ chọn lưu, bỏ thay đổi hoặc tiếp tục sửa.",
			ConfirmText: "Dừng tác vụ (Enter)",
			CancelText:  "Tiếp tục tác vụ (Esc)",
		}
		return
	}
	a.showDraftDecision()
}

func (a *App) showDraftDecision() {
	a.ActiveModal = &components.Modal{
		Type:        components.ModalConfirm,
		Title:       "BẢN NHÁP CHƯA LƯU",
		Message:     "s/Enter: lưu rồi rời màn · d: bỏ thay đổi và rời màn · k/Esc: giữ bản nháp và tiếp tục sửa.",
		ConfirmText: "s / Enter: Lưu",
		CancelText:  "d: Bỏ · Esc: Tiếp tục sửa",
	}
}

func (a *App) finishDraftExit(save bool) (tea.Model, tea.Cmd) {
	if save {
		if a.JobManager != nil && a.JobManager.IsBusy() {
			a.Error = "đợi tác vụ ghi workspace kết thúc trước khi lưu bản nháp"
			return a, nil
		}
		if a.EditorScreen.IsDirty {
			if err := a.EditorScreen.Save(); err != nil {
				a.Error = err.Error()
				return a, nil
			}
		}
		if a.EditorScreen.NoteDirty {
			if err := a.EditorScreen.SaveNote(); err != nil {
				a.Error = err.Error()
				return a, nil
			}
		}
		a.Notice = a.EditorScreen.Message
	} else {
		a.EditorScreen.DiscardDrafts()
		a.Notice = "Đã bỏ bản nháp chưa lưu."
	}
	destination := a.PendingDraftExit
	a.PendingDraftExit = ""
	a.DraftExitWaiting = false
	a.ActiveModal = nil
	if destination == "quit" {
		return a, tea.Quit
	}
	return a, a.switchScreen(destination)
}

func (a *App) prepareSkillDraftExit(destination string) {
	a.PendingSkillDraftExit = destination
	a.ActiveModal = &components.Modal{
		Type:        components.ModalConfirm,
		Title:       "SKILL CÓ THAY ĐỔI CHƯA LƯU",
		Message:     "s/Enter: lưu rồi rời · d: bỏ thay đổi · k/Esc: tiếp tục sửa.",
		ConfirmText: "s / Enter: Lưu",
		CancelText:  "d: Bỏ · Esc: Tiếp tục sửa",
	}
}

func (a *App) finishSkillDraftExit(save bool) (tea.Model, tea.Cmd) {
	if save {
		if a.JobManager != nil && a.JobManager.IsBusy() {
			a.Error = "đợi tác vụ ghi workspace kết thúc trước khi lưu skill"
			return a, nil
		}
		if err := a.SkillsScreen.SaveSkillEdit(); err != nil {
			a.Error = err.Error()
			return a, nil
		}
		a.Notice = "Đã lưu skill."
	} else {
		a.SkillsScreen.CancelSkillEdit()
		a.Notice = "Đã bỏ thay đổi skill chưa lưu."
	}
	destination := a.PendingSkillDraftExit
	a.PendingSkillDraftExit = ""
	a.ActiveModal = nil
	if destination == "quit" {
		return a, tea.Quit
	}
	if destination == "cancel-edit" {
		return a, nil
	}
	return a, a.switchScreen(destination)
}

func (a *App) handleKeyPress(key tea.KeyMsg) (tea.Model, tea.Cmd) {
	k := key.String()
	if a.ActiveModal != nil {
		if a.PendingDraftExit != "" {
			if a.DraftExitWaiting {
				switch k {
				case "esc":
					a.PendingDraftExit = ""
					a.DraftExitWaiting = false
					a.ActiveModal = nil
					return a, nil
				case "enter":
					a.JobManager.Cancel()
					a.ActiveModal = nil
					a.Notice = "Đang dừng tác vụ; bản nháp vẫn được giữ trong editor…"
					return a, nil
				}
				return a, nil
			}
			switch k {
			case "esc", "k":
				a.PendingDraftExit = ""
				a.ActiveModal = nil
				a.Notice = "Bản nháp vẫn được giữ trong editor."
				return a, nil
			case "d":
				return a.finishDraftExit(false)
			case "s", "enter":
				return a.finishDraftExit(true)
			default:
				return a, nil
			}
		}
		if a.PendingSkillDraftExit != "" {
			switch k {
			case "esc", "k":
				a.PendingSkillDraftExit = ""
				a.ActiveModal = nil
				a.Notice = "Bản nháp skill vẫn được giữ để tiếp tục sửa."
				return a, nil
			case "d":
				return a.finishSkillDraftExit(false)
			case "s", "enter":
				return a.finishSkillDraftExit(true)
			default:
				return a, nil
			}
		}
		switch k {
		case "esc":
			a.ActiveModal = nil
			a.ExitAfterStop = false
			return a, nil
		case "enter":
			if a.ActiveModal.Type == components.ModalConfirm && a.JobManager != nil && a.JobManager.IsBusy() {
				a.JobManager.Cancel()
				a.ExitAfterStop = true
				a.Notice = "Đang dừng writer và chờ nhả khóa trước khi thoát…"
				a.ActiveModal = nil
				return a, nil
			}
			a.ActiveModal = nil
			return a, nil
		}
		return a, nil
	}
	if a.PathPicker != nil {
		if k == "ctrl+c" && a.JobManager != nil && a.JobManager.IsBusy() {
			a.PathPicker = nil
			a.JobManager.Cancel()
			a.Notice = "Đang dừng tác vụ…"
			return a, nil
		}
		return a.updatePathPicker(key)
	}
	if k == "ctrl+o" {
		if opened, cmd := a.openPathPicker(); opened {
			return a, cmd
		}
	}

	// Text fields own ordinary characters, including q and ?, while the user is
	// entering a title, a decision reason, a translation, or a setting value.
	if a.textEntryFocused() {
		if k == "ctrl+c" {
			if a.JobManager != nil && a.JobManager.IsBusy() {
				a.JobManager.Cancel()
				a.Notice = "Đang dừng tác vụ…"
				return a, nil
			}
			if a.CurrentScreen == "editor" && a.EditorScreen.HasUnsavedChanges() {
				a.prepareDraftExit("quit")
				return a, nil
			}
			if a.CurrentScreen == "skills" && a.SkillsScreen.Editing && a.SkillsScreen.EditDirty {
				a.prepareSkillDraftExit("quit")
				return a, nil
			}
			return a, tea.Quit
		}
		switch a.CurrentScreen {
		case "welcome":
			return a.handleWelcomeKey(key)
		case "review":
			switch k {
			case "esc":
				a.ReviewScreen.EnteringReason = false
				a.ReviewScreen.ReasonInput.Blur()
				return a, nil
			case "enter":
				if a.JobManager.IsBusy() {
					a.Error = "không thể ghi quyết định trong khi tác vụ đang ghi workspace"
					return a, nil
				}
				if err := a.ReviewScreen.SubmitKeepReason(); err != nil {
					a.Error = err.Error()
				} else {
					a.Notice = a.ReviewScreen.Message
				}
				return a, nil
			default:
				var cmd tea.Cmd
				a.ReviewScreen.ReasonInput, cmd = a.ReviewScreen.ReasonInput.Update(key)
				return a, cmd
			}
		case "editor":
			if k == "esc" {
				if a.EditorScreen.HasUnsavedChanges() {
					a.prepareDraftExit("dashboard")
					return a, nil
				}
				return a, a.switchScreen("dashboard")
			}
			if k == "ctrl+s" {
				if a.JobManager.IsBusy() {
					a.Error = "không thể lưu bản sửa khi tác vụ đang ghi workspace"
					return a, nil
				}
				var err error
				if a.EditorScreen.ActiveTab == 4 {
					err = a.EditorScreen.SaveNote()
				} else {
					err = a.EditorScreen.Save()
				}
				if err != nil {
					a.Error = err.Error()
				} else {
					a.Notice = a.EditorScreen.Message
				}
				return a, nil
			}
			var cmd tea.Cmd
			if a.EditorScreen.ActiveTab == 4 {
				before := a.EditorScreen.NoteArea.Value()
				a.EditorScreen.NoteArea, cmd = a.EditorScreen.NoteArea.Update(key)
				if before != a.EditorScreen.NoteArea.Value() {
					a.EditorScreen.NoteDirty = true
				}
			} else {
				before := a.EditorScreen.TextArea.Value()
				a.EditorScreen.TextArea, cmd = a.EditorScreen.TextArea.Update(key)
				if before != a.EditorScreen.TextArea.Value() {
					a.EditorScreen.IsDirty = true
				}
			}
			return a, cmd
		case "settings":
			if k == "ctrl+left" || k == "ctrl+right" {
				delta := 1
				if k == "ctrl+left" {
					delta = -1
				}
				a.SettingsScreen.SetTab(a.SettingsScreen.ActiveTab + delta)
				return a, nil
			}
			if k == "esc" {
				return a, a.switchScreen("dashboard")
			}
			if k == "tab" || k == "shift+tab" {
				st := a.SettingsScreen
				if len(st.Inputs) > 0 {
					delta := 1
					if k == "shift+tab" {
						delta = -1
					}
					st.Inputs[st.FocusedField].Blur()
					st.FocusedField = (st.FocusedField + delta + len(st.Inputs)) % len(st.Inputs)
					st.Inputs[st.FocusedField].Focus()
				}
				return a, nil
			}
			if k == "ctrl+s" || k == "enter" {
				if a.JobManager.IsBusy() {
					a.Error = "không thể đổi cấu hình khi tác vụ đang ghi workspace"
					return a, nil
				}
				if err := a.SettingsScreen.Save(); err != nil {
					a.Error = err.Error()
				} else {
					a.Notice = a.SettingsScreen.Message
				}
				return a, nil
			}
			idx := a.SettingsScreen.FocusedField
			if idx >= 0 && idx < len(a.SettingsScreen.Inputs) {
				var cmd tea.Cmd
				a.SettingsScreen.Inputs[idx], cmd = a.SettingsScreen.Inputs[idx].Update(key)
				return a, cmd
			}
		case "content":
			cs := a.ContentScreen
			switch k {
			case "esc":
				cs.CancelAddVolume()
				return a, nil
			case "tab", "shift+tab":
				delta := 1
				if k == "shift+tab" {
					delta = -1
				}
				cs.AddInputs[cs.AddField].Blur()
				cs.AddField = (cs.AddField + delta + len(cs.AddInputs)) % len(cs.AddInputs)
				cs.AddInputs[cs.AddField].Focus()
				return a, nil
			case "ctrl+s", "enter":
				if a.JobManager.IsBusy() {
					cs.AddError = "không thể thêm tập trong khi tác vụ đang ghi workspace"
					return a, nil
				}
				volume, err := cs.SaveAddedVolume()
				if err != nil {
					cs.AddError = err.Error()
					return a, nil
				}
				a.SelectedVolumeID = volume.ID
				a.DashboardScreen.Refresh()
				a.Notice = "Đã đăng ký tập " + volume.ID + ". Import nguồn trước khi dịch."
				return a, a.selectVolume(volume.ID)
			default:
				idx := cs.AddField
				cs.AddInputs[idx], _ = cs.AddInputs[idx].Update(key)
				cs.AddError = ""
				return a, nil
			}
		case "verify":
			vs := a.VerifyScreen
			switch k {
			case "esc":
				vs.CancelAssessment()
				return a, nil
			case "tab", "shift+tab":
				delta := 1
				if k == "shift+tab" {
					delta = -1
				}
				vs.Inputs[vs.FocusedField].Blur()
				vs.FocusedField = (vs.FocusedField + delta + len(vs.Inputs)) % len(vs.Inputs)
				vs.Inputs[vs.FocusedField].Focus()
				return a, nil
			case "ctrl+s", "enter":
				if a.JobManager.IsBusy() {
					vs.FormError = "không thể ghi quyết định ảnh khi tác vụ đang chạy"
					return a, nil
				}
				if err := vs.SaveAssessment(); err != nil {
					a.Error = err.Error()
					return a, nil
				}
				a.Error = ""
				a.Notice = "Đã lưu quyết định ảnh. Cổng hình ảnh đã được tính lại."
				return a, nil
			default:
				idx := vs.FocusedField
				vs.Inputs[idx], _ = vs.Inputs[idx].Update(key)
				vs.FormError = ""
				return a, nil
			}
		case "skills":
			ss := a.SkillsScreen
			if ss.Resolving {
				switch k {
				case "esc":
					ss.CancelResolution()
					return a, nil
				case "ctrl+s", "enter":
					if a.JobManager.IsBusy() {
						ss.Error = "không thể ghi quyết định khi tác vụ đang chạy"
						return a, nil
					}
					if err := ss.SaveResolution(); err != nil {
						a.Error = err.Error()
					} else {
						a.Error = ""
						a.Notice = "Đã lưu quyết định rule candidate."
					}
					return a, nil
				default:
					var cmd tea.Cmd
					ss.ReasonInput, cmd = ss.ReasonInput.Update(key)
					ss.Error = ""
					return a, cmd
				}
			}
			if ss.EditingLibrary {
				switch k {
				case "esc":
					ss.CancelLibraryEdit()
					return a, nil
				case "ctrl+s", "enter":
					if a.JobManager.IsBusy() {
						ss.Error = "không thể đổi thư viện khi tác vụ đang ghi workspace"
						return a, nil
					}
					if err := ss.SaveLibraryDirectory(); err != nil {
						a.Error = err.Error()
					} else {
						a.Error = ""
						a.Notice = "Đã cập nhật thư viện skill chung."
					}
					return a, nil
				default:
					var cmd tea.Cmd
					ss.LibraryInput, cmd = ss.LibraryInput.Update(key)
					ss.Error = ""
					return a, cmd
				}
			}
			if ss.Editing {
				switch k {
				case "esc":
					if ss.EditDirty {
						a.prepareSkillDraftExit("cancel-edit")
					} else {
						ss.CancelSkillEdit()
					}
					return a, nil
				case "ctrl+s":
					if a.JobManager.IsBusy() {
						ss.Error = "không thể lưu skill khi tác vụ đang ghi workspace"
						return a, nil
					}
					if err := ss.SaveSkillEdit(); err != nil {
						a.Error = err.Error()
					} else {
						a.Error = ""
						a.Notice = "Đã lưu skill; các khối bị ảnh hưởng cần đối chiếu lại."
					}
					return a, nil
				default:
					before := ss.Editor.Value()
					var cmd tea.Cmd
					ss.Editor, cmd = ss.Editor.Update(key)
					if before != ss.Editor.Value() {
						ss.EditDirty = true
					}
					return a, cmd
				}
			}
		}
	}

	switch k {
	case "ctrl+c":
		if a.JobManager != nil && a.JobManager.IsBusy() {
			a.JobManager.Cancel()
			a.Notice = "Đang dừng tác vụ…"
			return a, nil
		}
		if a.CurrentScreen == "editor" && a.EditorScreen.HasUnsavedChanges() {
			a.prepareDraftExit("quit")
			return a, nil
		}
		if a.CurrentScreen == "skills" && a.SkillsScreen.Editing && a.SkillsScreen.EditDirty {
			a.prepareSkillDraftExit("quit")
			return a, nil
		}
		return a, tea.Quit
	case "q":
		if a.JobManager != nil && a.JobManager.IsBusy() {
			if a.CurrentScreen == "editor" && a.EditorScreen.HasUnsavedChanges() {
				a.prepareDraftExit("quit")
				return a, nil
			}
			a.ActiveModal = &components.Modal{
				Type:        components.ModalConfirm,
				Title:       "DỪNG TÁC VỤ VÀ THOÁT",
				Message:     "Tác vụ sẽ được hủy qua context. Writer dừng và nhả khóa trước khi ứng dụng thoát.",
				ConfirmText: "Dừng & thoát (Enter)",
				CancelText:  "Tiếp tục (Esc)",
			}
			return a, nil
		}
		if a.CurrentScreen == "editor" && a.EditorScreen.HasUnsavedChanges() {
			a.prepareDraftExit("quit")
			return a, nil
		}
		if a.CurrentScreen == "skills" && a.SkillsScreen.Editing && a.SkillsScreen.EditDirty {
			a.prepareSkillDraftExit("quit")
			return a, nil
		}
		return a, tea.Quit
	case "?":
		a.ActiveModal = &components.Modal{Type: components.ModalHelp, Title: "TRỢ GIÚP · " + a.CurrentScreen, HelpKeys: theme.ScreenHelp(a.CurrentScreen)}
		return a, nil
	case "tab", "shift+tab":
		if a.CurrentScreen == "settings" {
			st := a.SettingsScreen
			if len(st.Inputs) > 0 {
				delta := 1
				if k == "shift+tab" {
					delta = -1
				}
				st.Inputs[st.FocusedField].Blur()
				st.FocusedField = (st.FocusedField + delta + len(st.Inputs)) % len(st.Inputs)
				st.Inputs[st.FocusedField].Focus()
			}
			return a, nil
		}
		a.cycleFocus(k == "shift+tab")
		return a, nil
	case "alt+left", "alt+right":
		items := MenuItems()
		delta := 1
		if k == "alt+left" {
			delta = -1
		}
		index := a.SidebarIndex + delta
		if index < 0 {
			index = len(items) - 1
		} else if index >= len(items) {
			index = 0
		}
		return a, a.switchScreen(items[index].ID)
	}

	if a.FocusSidebar {
		menu := MenuItems()
		switch k {
		case "up", "k":
			a.SidebarIndex = max(0, a.SidebarIndex-1)
		case "down", "j":
			a.SidebarIndex = min(len(menu)-1, a.SidebarIndex+1)
		case "enter":
			return a, a.switchScreen(menu[a.SidebarIndex].ID)
		}
		return a, nil
	}

	switch a.CurrentScreen {
	case "welcome":
		return a.handleWelcomeKey(key)

	case "dashboard":
		ds := a.DashboardScreen
		switch k {
		case "o":
			return a, a.switchScreen("welcome")
		case "left", "h":
			if ds.FocusedPane == 0 && ds.SelectedAction > 0 {
				ds.SelectedAction--
			}
		case "right", "l":
			if ds.FocusedPane == 0 && ds.SelectedAction+1 < len(ds.Actions()) {
				ds.SelectedAction++
			}
		case "up", "pgup", "home":
			if ds.FocusedPane == 1 && ds.SelectedVolume > 0 {
				step := 1
				if k == "pgup" {
					step = 8
				} else if k == "home" {
					step = ds.SelectedVolume
				}
				ds.SelectedVolume = max(0, ds.SelectedVolume-step)
				if id := ds.Projection.Volumes[ds.SelectedVolume].ID; id != a.SelectedVolumeID {
					return a, a.selectVolume(id)
				}
			}
		case "down", "j", "pgdown", "end":
			if ds.FocusedPane == 1 && ds.SelectedVolume+1 < len(ds.Projection.Volumes) {
				step := 1
				if k == "pgdown" {
					step = 8
				} else if k == "end" {
					step = len(ds.Projection.Volumes)
				}
				ds.SelectedVolume = min(len(ds.Projection.Volumes)-1, ds.SelectedVolume+step)
				if id := ds.Projection.Volumes[ds.SelectedVolume].ID; id != a.SelectedVolumeID {
					return a, a.selectVolume(id)
				}
			}
		case "enter":
			if ds.FocusedPane == 1 {
				return a, a.switchScreen("content")
			}
			return a, a.openDashboardAction(ds.SelectedAction)
		case "a":
			a.ContentScreen.StartAddVolume()
			return a, a.switchScreen("content")
		default:
			if len(k) == 1 && k[0] >= '1' && k[0] <= '6' {
				ds.SelectedAction = int(k[0] - '1')
				return a, a.openDashboardAction(ds.SelectedAction)
			}
		}

	case "content":
		cs := a.ContentScreen
		switch k {
		case "i":
			if len(cs.Volumes) == 0 {
				a.Error = "workspace chưa có tập EPUB"
				return a, nil
			}
			volumeID := cs.Volumes[cs.SelectedVolume].ID
			if cs.Volumes[cs.SelectedVolume].Imported {
				a.Notice = "Snapshot tập này đã import."
				return a, nil
			}
			if err := a.JobManager.StartImport(a.jobContext(), volumeID); err != nil {
				a.Error = err.Error()
				return a, nil
			}
			a.Error = ""
			a.Notice = "Đang import EPUB nguồn…"
			return a, a.switchScreen("job")
		case "space":
			if cs.FocusedPane == 1 {
				cs.ToggleCurrentChapter()
			}
		case "up", "k", "pgup", "home":
			switch cs.FocusedPane {
			case 0:
				if cs.SelectedVolume > 0 {
					step := 1
					if k == "pgup" {
						step = 8
					} else if k == "home" {
						step = cs.SelectedVolume
					}
					cs.SelectedVolume = max(0, cs.SelectedVolume-step)
					cs.Refresh()
					return a, a.selectVolume(cs.Volumes[cs.SelectedVolume].ID)
				}
			case 1:
				step := 1
				if k == "pgup" {
					step = 8
				} else if k == "home" {
					step = cs.SelectedChapter
				}
				cs.SelectedChapter = max(0, cs.SelectedChapter-step)
			case 2:
				cs.SelectedMode = max(0, cs.SelectedMode-1)
			}
		case "down", "j", "pgdown", "end":
			switch cs.FocusedPane {
			case 0:
				if cs.SelectedVolume+1 < len(cs.Volumes) {
					step := 1
					if k == "pgdown" {
						step = 8
					} else if k == "end" {
						step = len(cs.Volumes)
					}
					cs.SelectedVolume = min(len(cs.Volumes)-1, cs.SelectedVolume+step)
					cs.Refresh()
					return a, a.selectVolume(cs.Volumes[cs.SelectedVolume].ID)
				}
			case 1:
				step := 1
				if k == "pgdown" {
					step = 8
				} else if k == "end" {
					step = len(cs.Chapters)
				}
				cs.SelectedChapter = min(max(0, len(cs.Chapters)-1), cs.SelectedChapter+step)
			case 2:
				cs.SelectedMode = min(len(cs.RunModes())-1, cs.SelectedMode+1)
			}
		case "enter":
			if cs.FocusedPane == 0 && len(cs.Volumes) > 0 {
				return a, a.selectVolume(cs.Volumes[cs.SelectedVolume].ID)
			}
			if cs.FocusedPane != 2 {
				break
			}
			if len(cs.Volumes) == 0 {
				a.Error = "workspace chưa có tập EPUB"
				return a, nil
			}
			var chapters []string
			firstSelected, lastSelected := -1, -1
			for _, chapter := range cs.Chapters {
				if cs.CheckedChapters[chapter.ID] {
					chapters = append(chapters, chapter.ID)
				}
			}
			if cs.SelectedMode == 1 && len(chapters) == 0 {
				a.Error = "chọn ít nhất một chương trước khi chạy phạm vi"
				return a, nil
			}
			if cs.SelectedMode == 1 {
				for i, chapter := range cs.Chapters {
					if !cs.CheckedChapters[chapter.ID] {
						continue
					}
					if firstSelected < 0 {
						firstSelected = i
					}
					lastSelected = i
				}
				if lastSelected-firstSelected+1 != len(chapters) {
					a.Error = "phạm vi phải là các chương liên tiếp; bỏ chọn khoảng trống để tiếp tục"
					return a, nil
				}
			}
			volumeID := cs.Volumes[cs.SelectedVolume].ID
			if !cs.Volumes[cs.SelectedVolume].Imported {
				a.Error = "Tập chưa import. Nhấn i để tạo snapshot nguồn trước khi dịch."
				return a, nil
			}
			scope := engine.RunScope{}
			if cs.SelectedMode == 1 {
				scope.Chapters = chapters
			}
			reviewOnly := cs.SelectedMode == 2
			policy := engine.ReviewPolicy{ReviewOnly: reviewOnly, Force: reviewOnly, AutoRepair: !reviewOnly}
			if err := a.JobManager.StartTranslate(a.jobContext(), volumeID, scope, policy); err != nil {
				a.Error = err.Error()
				return a, nil
			}
			a.Error = ""
			a.Notice = "Đã bắt đầu " + cs.RunModes()[cs.SelectedMode] + " cho " + volumeID
			return a, a.switchScreen("job")
		case "esc":
			return a, a.switchScreen("dashboard")
		case "a":
			cs.StartAddVolume()
		}

	case "review":
		rs := a.ReviewScreen
		switch k {
		case "0":
			rs.FilterSeverity, rs.FilterStatus = "", ""
			rs.SelectedIndex = 0
		case "1":
			rs.FilterSeverity, rs.FilterStatus = "blocking", ""
			rs.SelectedIndex = 0
		case "2":
			rs.FilterSeverity, rs.FilterStatus = "warning", ""
			rs.SelectedIndex = 0
		case "3":
			rs.FilterSeverity, rs.FilterStatus = "", "Cần đối chiếu lại"
			rs.SelectedIndex = 0
		case "4":
			rs.FilterSeverity, rs.FilterStatus = "", "Chờ review"
			rs.SelectedIndex = 0
		case "up", "pgup", "home":
			if rs.FocusedPane == 0 {
				step := 1
				if k == "pgup" {
					step = 8
				} else if k == "home" {
					step = rs.SelectedIndex
				}
				rs.SelectedIndex = max(0, rs.SelectedIndex-step)
			}
		case "down", "j", "pgdown", "end":
			if rs.FocusedPane == 0 {
				step := 1
				if k == "pgdown" {
					step = 8
				} else if k == "end" {
					step = len(rs.FilteredQueue())
				}
				rs.SelectedIndex = min(max(0, len(rs.FilteredQueue())-1), rs.SelectedIndex+step)
			}
		case "k":
			return a.runReviewAction(2)
		case "e":
			return a.runReviewAction(0)
		case "r":
			return a.runReviewAction(1)
		case "left", "h":
			if rs.FocusedPane == 1 {
				rs.ActionIndex = max(0, rs.ActionIndex-1)
			}
		case "right", "l":
			if rs.FocusedPane == 1 {
				rs.ActionIndex = min(2, rs.ActionIndex+1)
			}
		case "enter":
			if rs.FocusedPane == 0 {
				rs.FocusedPane = 1
			} else if rs.FocusedPane == 1 {
				return a.runReviewAction(rs.ActionIndex)
			}
		case "esc":
			return a, a.switchScreen("dashboard")
		}

	case "editor":
		es := a.EditorScreen
		switch k {
		case "ctrl+s":
			if a.JobManager.IsBusy() {
				a.Error = "không thể lưu bản sửa khi tác vụ đang ghi workspace"
				return a, nil
			}
			var err error
			if es.ActiveTab == 4 {
				err = es.SaveNote()
			} else {
				err = es.Save()
			}
			if err != nil {
				a.Error = err.Error()
			} else {
				a.Notice = es.Message
			}
		case "left", "h":
			if es.FocusedPane != 1 && es.ActiveTab > 0 {
				es.ActiveTab--
				if es.ActiveTab == 3 {
					es.LoadHistory()
				}
			}
		case "right", "l":
			if es.FocusedPane != 1 && es.ActiveTab+1 < len(es.Tabs()) {
				es.ActiveTab++
				if es.ActiveTab == 3 {
					es.LoadHistory()
				}
			}
		case "up", "k", "pgup", "home":
			if es.FocusedPane == 0 && es.HasUnsavedChanges() {
				a.Error = "lưu bản nháp trước khi chuyển sang khối khác"
			} else if es.FocusedPane == 0 && es.SelectedUnit > 0 {
				step := 1
				if k == "pgup" {
					step = 8
				} else if k == "home" {
					step = es.SelectedUnit
				}
				es.SelectUnit(max(0, es.SelectedUnit-step))
			}
		case "down", "j", "pgdown", "end":
			if es.FocusedPane == 0 && es.HasUnsavedChanges() {
				a.Error = "lưu bản nháp trước khi chuyển sang khối khác"
			} else if es.FocusedPane == 0 && es.SelectedUnit+1 < len(es.Units) {
				step := 1
				if k == "pgdown" {
					step = 8
				} else if k == "end" {
					step = len(es.Units)
				}
				es.SelectUnit(min(len(es.Units)-1, es.SelectedUnit+step))
			}
		case "r":
			if es.HasUnsavedChanges() {
				a.Error = "lưu bản nháp trước khi yêu cầu review khối này"
				return a, nil
			}
			if es.SelectedUnit >= 0 && es.SelectedUnit < len(es.Units) {
				item := es.Units[es.SelectedUnit]
				scope := engine.RunScope{UnitIDs: []string{item.ID}}
				if err := a.JobManager.StartReview(a.jobContext(), es.VolumeID, scope, true); err != nil {
					a.Error = err.Error()
					return a, nil
				}
				return a, a.switchScreen("job")
			}
		case "v":
			return a, a.switchScreen("review")
		case "esc":
			return a, a.switchScreen("dashboard")
		}

	case "job":
		switch k {
		case "p":
			if a.JobManager.Status().Kind != "translate" {
				a.Notice = "Pause chỉ áp dụng cho tác vụ dịch/review tại ranh giới khối."
				return a, nil
			}
			if a.JobManager.Status().State == app.StatePaused {
				a.JobManager.Resume()
				a.Notice = "Tiếp tục chạy tác vụ"
			} else {
				a.JobManager.Pause()
				a.Notice = "Yêu cầu tạm dừng tại ranh giới request hợp lệ…"
			}
		case "f":
			a.JobScreen.AutoFollow = !a.JobScreen.AutoFollow
		case "esc":
			return a, a.switchScreen("dashboard")
		}

	case "settings":
		st := a.SettingsScreen
		switch k {
		case "ctrl+s", "enter":
			if a.JobManager.IsBusy() {
				a.Error = "không thể đổi cấu hình khi tác vụ đang ghi workspace"
				return a, nil
			}
			if err := st.Save(); err != nil {
				a.Error = err.Error()
			} else {
				a.Notice = st.Message
			}
		case "up":
			st.FocusedField = max(0, st.FocusedField-1)
		case "down":
			st.FocusedField = min(len(st.Inputs)-1, st.FocusedField+1)
		case "esc":
			return a, a.switchScreen("dashboard")
		}
		for i := range st.Inputs {
			if i == st.FocusedField {
				st.Inputs[i].Focus()
			} else {
				st.Inputs[i].Blur()
			}
		}

	case "verify":
		vs := a.VerifyScreen
		switch k {
		case "up", "k", "pgup", "home":
			switch vs.FocusedPane {
			case 0:
				step := 1
				if k == "pgup" {
					step = 8
				} else if k == "home" {
					step = vs.SelectedGate
				}
				vs.SelectedGate = max(0, vs.SelectedGate-step)
			case 1:
				step := 1
				if k == "pgup" {
					step = 8
				} else if k == "home" {
					step = vs.SelectedImage
				}
				vs.SelectedImage = max(0, vs.SelectedImage-step)
			case 2:
				vs.SelectedAction = max(0, vs.SelectedAction-1)
			}
		case "down", "j", "pgdown", "end":
			switch vs.FocusedPane {
			case 0:
				step := 1
				if k == "pgdown" {
					step = 8
				} else if k == "end" {
					step = len(vs.Projection.Gates)
				}
				vs.SelectedGate = min(max(0, len(vs.Projection.Gates)-1), vs.SelectedGate+step)
			case 1:
				step := 1
				if k == "pgdown" {
					step = 8
				} else if k == "end" {
					step = len(vs.Projection.Images)
				}
				vs.SelectedImage = min(max(0, len(vs.Projection.Images)-1), vs.SelectedImage+step)
			case 2:
				vs.SelectedAction = min(len(vs.Actions())-1, vs.SelectedAction+1)
			}
		case "left", "h", "right", "l":
			if vs.FocusedPane == 2 {
				delta := 1
				if k == "left" || k == "h" {
					delta = -1
				}
				vs.SelectedAction = (vs.SelectedAction + delta + len(vs.Actions())) % len(vs.Actions())
			}
		case "a", "i", "p", "f", "enter":
			if k == "a" {
				if a.JobManager.IsBusy() {
					a.Error = "không thể ghi quyết định ảnh khi tác vụ đang chạy"
					return a, nil
				}
				if err := vs.StartAssessment(); err != nil {
					a.Error = err.Error()
					return a, nil
				}
				return a, nil
			}
			if k == "enter" && vs.FocusedPane != 2 {
				return a, nil
			}
			if a.JobManager.IsBusy() {
				a.Error = "đang có tác vụ ghi workspace"
				return a, nil
			}
			action := vs.SelectedAction
			if k == "i" {
				action = 3
			} else if k == "a" {
				action = 2
			} else if k == "p" {
				action = 0
			} else if k == "f" {
				action = 1
			}
			if action == 2 {
				if err := vs.StartAssessment(); err != nil {
					a.Error = err.Error()
					return a, nil
				}
				return a, nil
			}
			if action == 3 {
				if vs.Projection.Imported {
					a.Notice = "Snapshot đã import; không cần import lại."
					return a, nil
				}
				if err := a.JobManager.StartImport(a.jobContext(), vs.VolumeID); err != nil {
					a.Error = err.Error()
					return a, nil
				}
				a.Error = ""
				a.Notice = "Đang import EPUB nguồn…"
				return a, a.switchScreen("job")
			}
			if !vs.Projection.Imported {
				a.Error = "Cần import EPUB trước khi xuất."
				return a, nil
			}
			preview := action == 0
			if !preview {
				if ok, reason := vs.CanExportFinal(); !ok {
					a.Error = "Final chưa sẵn sàng: " + reason
					return a, nil
				}
			}
			if err := a.JobManager.StartExport(a.jobContext(), vs.VolumeID, preview); err != nil {
				a.Error = err.Error()
				return a, nil
			}
			a.Error = ""
			a.Notice = "Đang kiểm tra và xuất EPUB…"
			return a, a.switchScreen("job")
		case "esc":
			return a, a.switchScreen("dashboard")
		}

	case "skills":
		ss := a.SkillsScreen
		switch k {
		case "c", "x":
			decision := "confirm"
			if k == "x" {
				decision = "reject"
			}
			if err := ss.StartResolve(decision); err != nil {
				a.Error = err.Error()
				return a, nil
			}
			if a.JobManager.IsBusy() {
				ss.CancelResolution()
				a.Error = "không thể ghi quyết định khi tác vụ đang chạy"
			}
		case "space":
			if ss.FocusedPane == 1 && ss.ActiveTab == 0 {
				ss.ToggleSelectedSkill()
			}
		case "ctrl+s":
			if ss.EnabledDirty {
				if a.JobManager.IsBusy() {
					a.Error = "không thể đổi skill khi tác vụ đang ghi workspace"
					return a, nil
				}
				if err := ss.SaveEnabledSelection(); err != nil {
					a.Error = err.Error()
				} else {
					a.Error = ""
					a.Notice = "Đã lưu danh sách skill chung cho workspace này."
				}
			}
		case "e":
			if ss.ActiveTab == 0 || ss.ActiveTab == 1 {
				if err := ss.StartSkillEdit(); err != nil {
					a.Error = err.Error()
					return a, nil
				}
			}
		case "l":
			ss.StartLibraryEdit()
		case "left", "h":
			if ss.FocusedPane == 0 && ss.ActiveTab > 0 {
				ss.ActiveTab--
				ss.SelectedRule = 0
				ss.Refresh()
			}
		case "right":
			if ss.FocusedPane == 0 && ss.ActiveTab+1 < len(ss.Tabs()) {
				ss.ActiveTab++
				ss.SelectedRule = 0
				ss.Refresh()
			}
		case "up", "k", "pgup", "home":
			if ss.FocusedPane == 1 {
				step := 1
				if k == "pgup" {
					step = 8
				} else if k == "home" {
					step = ss.SelectedRule
				}
				ss.SelectedRule = max(0, ss.SelectedRule-step)
			}
		case "down", "j", "pgdown", "end":
			if ss.FocusedPane == 1 {
				step := 1
				if k == "pgdown" {
					step = 8
				} else if k == "end" {
					step = ss.ItemCount()
				}
				ss.SelectedRule = min(max(0, ss.ItemCount()-1), ss.SelectedRule+step)
			}
		case "esc":
			return a, a.switchScreen("dashboard")
		}

	case "continuity":
		cs := a.ContinuityScreen
		switch k {
		case "up", "k", "pgup", "home":
			if cs.FocusedPane == 0 {
				step := 1
				if k == "pgup" {
					step = 8
				} else if k == "home" {
					step = cs.SelectedIdx
				}
				cs.SelectedIdx = max(0, cs.SelectedIdx-step)
			}
		case "down", "j", "pgdown", "end":
			if cs.FocusedPane == 0 {
				step := 1
				if k == "pgdown" {
					step = 8
				} else if k == "end" {
					step = len(cs.Events)
				}
				cs.SelectedIdx = min(max(0, len(cs.Events)-1), cs.SelectedIdx+step)
			}
		case "esc":
			return a, a.switchScreen("dashboard")
		}

	default:
		if k == "esc" {
			return a, a.switchScreen("dashboard")
		}
	}
	return a, nil
}

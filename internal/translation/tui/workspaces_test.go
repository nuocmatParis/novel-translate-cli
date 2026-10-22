package tui_test

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/tui"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func workspaceService(t *testing.T) *app.AppService {
	t.Helper()
	dir := t.TempDir()
	for _, name := range []string{"APPDATA", "XDG_CONFIG_HOME", "USERPROFILE", "HOME"} {
		t.Setenv(name, dir)
	}
	return app.NewService(app.ServiceOptions{
		WorkspaceDirectory: filepath.Join(dir, "workspace"),
		NewModel: func(workspace.Config) (model.Caller, error) {
			t.Fatal("workspace flow must not call a model")
			return nil, nil
		},
	})
}

func typeWorkspaceName(a *tui.App, text string) {
	for _, r := range text {
		a.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune{r}})
	}
}

func TestWelcomeSelectsWorkspaceFromMetadata(t *testing.T) {
	svc := workspaceService(t)
	cfg, err := workspace.Create(filepath.Join(svc.WorkspaceDirectory, "old-folder-name"), "story", "Tên bộ lấy từ YAML")
	if err != nil {
		t.Fatal(err)
	}
	if err := cfg.Add(workspace.Volume{ID: "vol01", Order: 1, Directory: "vol01", Source: "source.epub"}); err != nil {
		t.Fatal(err)
	}
	a := tui.NewApp(svc)
	view := a.View()
	if !strings.Contains(view, cfg.Series.Title) || !strings.Contains(view, "1 tập") || strings.Contains(view, "Nhập đường dẫn") {
		t.Fatal("welcome does not show selectable workspace metadata", view)
	}
	if a.WelcomeScreen.SelectedIndex != 2 {
		t.Fatal("existing workspace should be selected by default")
	}
	a.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if a.CurrentScreen != "dashboard" || svc.Config == nil || svc.Config.Root != cfg.Root {
		t.Fatal("Enter did not open the selected workspace")
	}
	typeWorkspaceName(a, "o")
	if a.CurrentScreen != "welcome" || len(a.WelcomeScreen.Workspaces) != 1 {
		t.Fatal("workspace picker cannot be reopened from the dashboard")
	}
}

func TestWelcomeCreatesWorkspaceFromNameAndKeepsTypingShortcuts(t *testing.T) {
	svc := workspaceService(t)
	a := tui.NewApp(svc)
	typeWorkspaceName(a, "n")
	if !a.WelcomeScreen.Creating {
		t.Fatal("new workspace shortcut did not focus title input")
	}
	view := a.View()
	if strings.Contains(view, "Mã bộ truyện (ID):") || strings.Contains(view, "Đường dẫn workspace:") || strings.Contains(view, "Thư mục workspace:") {
		t.Fatal("creation still asks for path or ID", view)
	}
	a.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if a.WelcomeScreen.Error == "" || !a.WelcomeScreen.Creating || svc.Config != nil {
		t.Fatal("blank title was accepted or creation form lost")
	}
	typeWorkspaceName(a, " null ")
	a.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if a.WelcomeScreen.Error == "" || svc.Config != nil {
		t.Fatal("null title accepted")
	}
	a.Update(tea.KeyMsg{Type: tea.KeyCtrlU})
	const title = "Quyển truyện q? n / Dị giới"
	typeWorkspaceName(a, title)
	a.Update(tea.KeyMsg{Type: tea.KeyTab})
	if a.WelcomeScreen.TitleInput.Value() != title || a.FocusSidebar || a.ActiveModal != nil || !a.WelcomeScreen.Creating {
		t.Fatal("title typing was intercepted by navigation shortcuts")
	}
	a.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if svc.Config == nil || a.CurrentScreen != "dashboard" || svc.Config.Series.Title != title || filepath.Dir(svc.Config.Root) != svc.WorkspaceDirectory {
		t.Fatal("name-only creation did not open a valid workspace", a.WelcomeScreen.Error)
	}
	if _, err := os.Stat(svc.Config.ReadPath(svc.Config.SeriesSkill)); err != nil {
		t.Fatal("workspace default conventions were not installed", err)
	}
	reopened := tui.NewApp(app.NewService(app.ServiceOptions{WorkspaceDirectory: svc.WorkspaceDirectory}))
	if len(reopened.WelcomeScreen.Workspaces) != 1 || reopened.WelcomeScreen.Workspaces[0].Title != title {
		t.Fatal("new workspace was not found after reopening")
	}
}

func TestWelcomeCancelRefreshAndLongList(t *testing.T) {
	svc := workspaceService(t)
	for i := 0; i < 25; i++ {
		id := fmt.Sprintf("story-%02d", i)
		if _, err := workspace.Create(filepath.Join(svc.WorkspaceDirectory, id), id, fmt.Sprintf("Bộ truyện %02d", i)); err != nil {
			t.Fatal(err)
		}
	}
	a := tui.NewApp(svc)
	a.Update(tea.WindowSizeMsg{Width: 80, Height: 24})
	for range 4 {
		a.Update(tea.KeyMsg{Type: tea.KeyPgDown})
	}
	if a.WelcomeScreen.SelectedIndex != len(a.WelcomeScreen.Workspaces)+1 || !strings.Contains(a.View(), "Bộ truyện 24") {
		t.Fatal("workspace list did not scroll to the selected row")
	}
	typeWorkspaceName(a, "n")
	typeWorkspaceName(a, "Bộ chưa lưu")
	a.Update(tea.KeyMsg{Type: tea.KeyEsc})
	if a.WelcomeScreen.Creating || svc.Config != nil {
		t.Fatal("cancel created a workspace")
	}
	selected := a.WelcomeScreen.Workspaces[24]
	if err := os.Remove(filepath.Join(selected.Root, "workspace.yaml")); err != nil {
		t.Fatal(err)
	}
	a.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if a.CurrentScreen != "welcome" || a.WelcomeScreen.Error == "" || svc.Config != nil {
		t.Fatal("opening a deleted workspace silently succeeded")
	}
	typeWorkspaceName(a, "r")
	if len(a.WelcomeScreen.Workspaces) != 24 || a.WelcomeScreen.SelectedIndex > len(a.WelcomeScreen.Workspaces)+1 {
		t.Fatal("refresh did not remove a deleted workspace")
	}
}

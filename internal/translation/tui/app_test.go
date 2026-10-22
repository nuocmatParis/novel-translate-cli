package tui_test

import (
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/tui"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func TestTerminalSizeTiers(t *testing.T) {
	compact := tui.CalculateLayout(70, 20)
	if compact.Tier != tui.SizeCompact {
		t.Errorf("expected compact tier for 70x20, got %v", compact.Tier)
	}

	normal := tui.CalculateLayout(90, 25)
	if normal.Tier != tui.SizeNormal {
		t.Errorf("expected normal tier for 90x25, got %v", normal.Tier)
	}

	medium := tui.CalculateLayout(120, 35)
	if medium.Tier != tui.SizeMedium {
		t.Errorf("expected medium tier for 120x35, got %v", medium.Tier)
	}

	wide := tui.CalculateLayout(150, 45)
	if wide.Tier != tui.SizeWide {
		t.Errorf("expected wide tier for 150x45, got %v", wide.Tier)
	}
}

func TestCleanText(t *testing.T) {
	dirty := "\x1b[31mTiếng Việt có dấu\x1b[0m\x00\x07 và 日本語"
	cleaned := theme.CleanText(dirty)
	if strings.Contains(cleaned, "\x1b") || strings.Contains(cleaned, "\x00") || strings.Contains(cleaned, "\x07") {
		t.Errorf("CleanText failed to strip ANSI/control chars: %q", cleaned)
	}
	if !strings.Contains(cleaned, "Tiếng Việt có dấu") || !strings.Contains(cleaned, "日本語") {
		t.Errorf("CleanText corrupted text: %q", cleaned)
	}
}

func TestAppNavigationAndScreenSwitching(t *testing.T) {
	svc := app.NewService(app.ServiceOptions{})
	appInstance := tui.NewApp(svc)

	// Default without workspace is welcome screen
	if appInstance.CurrentScreen != "welcome" {
		t.Fatalf("expected welcome screen, got %s", appInstance.CurrentScreen)
	}

	view := appInstance.View()
	if !strings.Contains(view, "NOVEL TRANSLATE") {
		t.Errorf("welcome view should render app name, got: %s", view)
	}

	// Switch screens
	screens := []string{"dashboard", "content", "job", "editor", "review", "skills", "continuity", "verify", "settings", "reports"}
	for _, scr := range screens {
		appInstance.CurrentScreen = scr
		v := appInstance.View()
		if len(v) == 0 {
			t.Errorf("screen %s rendered empty view", scr)
		}
	}
}

func TestAppKeyRouting(t *testing.T) {
	svc := app.NewService(app.ServiceOptions{})
	appInstance := tui.NewApp(svc)
	appInstance.Update(tea.WindowSizeMsg{Width: 120, Height: 35})

	// Focus sidebar with tab
	appInstance.Update(tea.KeyMsg{Type: tea.KeyTab})
	if !appInstance.FocusSidebar {
		t.Errorf("expected FocusSidebar to be true after tab")
	}

	// Navigate down in menu
	appInstance.Update(tea.KeyMsg{Type: tea.KeyDown})
	if appInstance.SidebarIndex != 1 {
		t.Errorf("expected SidebarIndex 1, got %d", appInstance.SidebarIndex)
	}

	// Press ? to open help modal
	appInstance.FocusSidebar = false
	appInstance.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune{'?'}})
	if appInstance.ActiveModal == nil {
		t.Fatalf("expected ActiveModal to open on '?'")
	}

	// Press Esc to dismiss modal
	appInstance.Update(tea.KeyMsg{Type: tea.KeyEscape})
	if appInstance.ActiveModal != nil {
		t.Fatalf("expected ActiveModal to close on Esc")
	}
}

func TestReviewScreenQueueKeyKeep(t *testing.T) {
	dir := t.TempDir()
	cfg, err := workspace.Create(dir, "rezero", "Re:Zero kara Hajimeru Isekai Seikatsu")
	if err != nil {
		t.Fatal(err)
	}

	svc := app.NewService(app.ServiceOptions{})
	svc.Config = &cfg

	appInstance := tui.NewApp(svc)
	appInstance.CurrentScreen = "review"
	appInstance.ReviewScreen.Queue = []app.ReviewQueueItem{{UnitID: "unit:one"}}

	// Press 'k' to enter reason for keeping
	appInstance.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune{'k'}})
	if !appInstance.ReviewScreen.EnteringReason {
		t.Errorf("expected EnteringReason true on 'k'")
	}

	// Press Esc to cancel
	appInstance.Update(tea.KeyMsg{Type: tea.KeyEscape})
	if appInstance.ReviewScreen.EnteringReason {
		t.Errorf("expected EnteringReason false on Esc")
	}
}

func TestWorkspace1DashboardView(t *testing.T) {
	svc := app.NewService(app.ServiceOptions{})
	_, err := svc.LoadWorkspace("../../../workspace/1")
	if err != nil {
		t.Skipf("workspace/1 not found: %v", err)
	}

	appInstance := tui.NewApp(svc)
	appInstance.Update(tea.WindowSizeMsg{Width: 120, Height: 35})
	if appInstance.CurrentScreen != "dashboard" {
		t.Fatalf("expected dashboard screen when workspace loaded, got %s", appInstance.CurrentScreen)
	}

	view := appInstance.View()
	t.Logf("Workspace 1 Dashboard Render:\n%s", view)

	if !strings.Contains(view, "NOVEL TRANSLATE") {
		t.Errorf("expected NOVEL TRANSLATE header in view")
	}
	if !strings.Contains(view, "re:0") {
		t.Errorf("expected series title re:0 in view")
	}
	if !strings.Contains(view, "ĐIỀU HƯỚNG") {
		t.Errorf("expected sidebar ĐIỀU HƯỚNG in view")
	}
	if !strings.Contains(view, "Dashboard") {
		t.Errorf("expected Dashboard menu in view")
	}
	if !strings.Contains(view, "Reviewer") {
		t.Errorf("expected Reviewer menu in view")
	}
}

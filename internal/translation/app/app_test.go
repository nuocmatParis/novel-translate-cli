package app

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
)

func TestAppServiceQueriesAndReview(t *testing.T) {
	root := t.TempDir()
	svc := NewService(ServiceOptions{})

	cfg, err := svc.InitWorkspace(root, "test-series", "Test Series Title")
	if err != nil {
		t.Fatal(err)
	}
	if cfg.Series.ID != "test-series" || cfg.Series.Title != "Test Series Title" {
		t.Fatal("bad series metadata", cfg.Series)
	}

	source := filepath.Join(root, "source.epub")
	if err := os.WriteFile(source, testutil.EPUB(), 0600); err != nil {
		t.Fatal(err)
	}

	if err := svc.AddVolume("vol01", source, 1, "", ""); err != nil {
		t.Fatal(err)
	}

	v, err := svc.Config.Volume("vol01")
	if err != nil {
		t.Fatal(err)
	}

	if _, err := engine.Import(*svc.Config, v); err != nil {
		t.Fatal(err)
	}

	dash, err := svc.QueryDashboard()
	if err != nil {
		t.Fatal(err)
	}
	if dash.SeriesID != "test-series" || len(dash.Volumes) != 1 {
		t.Fatal("bad dashboard", dash)
	}
	if dash.Volumes[0].Eligible == 0 {
		t.Fatal("expected eligible units in volume")
	}

	chapters, err := svc.QueryChapters("vol01")
	if err != nil {
		t.Fatal(err)
	}
	if len(chapters) == 0 {
		t.Fatal("expected chapters")
	}

	// KeepRevision requires reason
	_, book, _ := engine.Source(*svc.Config, v, false)
	var firstID string
	for _, b := range book.Blocks {
		if b.Disposition == "translate" {
			firstID = b.ID
			break
		}
	}

	// Save manual translation
	if err := svc.SaveManualTranslation("vol01", firstID, "Bản dịch thủ công đầu tiên."); err != nil {
		t.Fatal(err)
	}

	// Verify manual decision
	if err := svc.KeepRevision("vol01", firstID, ""); err == nil {
		t.Fatal("expected error when keeping revision without reason")
	}
	if err := svc.KeepRevision("vol01", firstID, "Đã đọc và xác nhận dịch chuẩn ý nghĩa"); err != nil {
		t.Fatal(err)
	}

	queue, err := svc.QueryReviewQueue("vol01")
	if err != nil {
		t.Fatal(err)
	}
	found := false
	for _, item := range queue {
		if item.UnitID == firstID {
			found = true
			if item.Status != "Người dùng giữ" {
				t.Fatalf("expected status 'Người dùng giữ', got %q (stale: %q)", item.Status, item.StaleReason)
			}
		}
	}
	if !found {
		t.Fatal("did not find kept unit in review queue")
	}

	// Scope plan
	plan, err := svc.QueryScopePlan("vol01", []string{chapters[0].ID})
	if err != nil {
		t.Fatal(err)
	}
	if plan.TotalUnits == 0 {
		t.Fatal("scope plan should have units")
	}
}

func TestJobManagerLifecycle(t *testing.T) {
	svc := NewService(ServiceOptions{})
	jm := NewJobManager(svc)

	if jm.IsBusy() {
		t.Fatal("should start idle")
	}

	jm.Pause()
	status := jm.Status()
	if !jm.isPaused() {
		t.Fatal("expected isPaused to be true")
	}

	jm.Resume()
	if jm.isPaused() {
		t.Fatal("expected isPaused to be false")
	}

	jm.AddLog("test log 1")
	jm.AddLog("test log 2")
	logs := jm.Logs()
	if len(logs) != 2 || logs[0] != "test log 1" {
		t.Fatalf("unexpected logs: %v", logs)
	}
	_ = status
}

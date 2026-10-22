package app

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func workspaceServiceForTest(t *testing.T) *AppService {
	t.Helper()
	dir := t.TempDir()
	for _, name := range []string{"APPDATA", "XDG_CONFIG_HOME", "USERPROFILE", "HOME"} {
		t.Setenv(name, dir)
	}
	return NewService(ServiceOptions{
		WorkspaceDirectory: filepath.Join(dir, "workspace"),
		NewModel: func(workspace.Config) (model.Caller, error) {
			t.Fatal("workspace selection and creation must not call a model")
			return nil, nil
		},
	})
}

func TestAvailableWorkspacesReadsMetadataWithoutChangingSelection(t *testing.T) {
	svc := workspaceServiceForTest(t)
	root := svc.WorkspaceDirectory
	first, err := workspace.Create(filepath.Join(root, "old-folder"), "series1", "Tên thật từ workspace")
	if err != nil {
		t.Fatal(err)
	}
	if err := first.Add(workspace.Volume{ID: "vol01", Order: 1, Directory: "vol01", Source: "nguồn Nhật.epub"}); err != nil {
		t.Fatal(err)
	}
	second, err := workspace.Create(filepath.Join(root, "nested", "another-folder"), "series2", "Bộ truyện khác")
	if err != nil {
		t.Fatal(err)
	}
	external, err := workspace.Create(filepath.Join(t.TempDir(), "external"), "external", "Bộ nằm ngoài thư mục mặc định")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := svc.LoadWorkspace(external.Root); err != nil {
		t.Fatal(err)
	}
	svc.AddRecentWorkspace(first.Root) // The same directory is also discovered by scanning.
	bad := filepath.Join(root, "invalid", "workspace.yaml")
	if err := os.MkdirAll(filepath.Dir(bad), 0755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(bad, []byte("broken: ["), 0600); err != nil {
		t.Fatal(err)
	}
	before := svc.Config
	listed, err := svc.AvailableWorkspaces()
	if err != nil {
		t.Fatal(err)
	}
	if len(listed) != 3 || svc.Config != before {
		t.Fatal("missing, duplicate, invalid, or state-changing workspace results", listed)
	}
	want := map[string]string{first.Root: first.Series.Title, second.Root: second.Series.Title, external.Root: external.Series.Title}
	for _, item := range listed {
		if item.Title != want[item.Root] || item.SeriesID == "" {
			t.Fatal("workspace metadata was replaced with a folder name", item)
		}
		if item.Root == first.Root && item.VolumeCount != 1 {
			t.Fatal("volume count not loaded", item)
		}
	}

	// A metadata edit made on disk must appear on refresh without opening it.
	second.Series.Title = "Tên vừa sửa trên đĩa"
	if err := second.Save(); err != nil {
		t.Fatal(err)
	}
	listed, err = svc.AvailableWorkspaces()
	if err != nil {
		t.Fatal(err)
	}
	found := false
	for _, item := range listed {
		if item.Root == second.Root && item.Title == second.Series.Title {
			found = true
		}
	}
	if !found {
		t.Fatal("refresh kept stale metadata")
	}
}

func TestCreateWorkspaceRequiresOnlyTitleAndKeepsExistingData(t *testing.T) {
	svc := workspaceServiceForTest(t)
	for _, empty := range []string{"", " \t\n", "null", " NULL "} {
		if _, err := svc.CreateWorkspace(empty); err == nil {
			t.Fatalf("accepted empty/null title %q", empty)
		}
	}
	if _, err := os.Stat(svc.WorkspaceDirectory); !os.IsNotExist(err) {
		t.Fatal("invalid title created a workspace directory", err)
	}
	first, err := svc.CreateWorkspace("  Đường đến dị giới: Tập / 1?  ")
	if err != nil {
		t.Fatal(err)
	}
	if first.Series.Title != "Đường đến dị giới: Tập / 1?" || first.Series.ID != "duong-den-di-gioi-tap-1" || filepath.Dir(first.Root) != svc.WorkspaceDirectory {
		t.Fatal("wrong automatically derived workspace", first.Series, first.Root)
	}
	if _, err := skills.Load(*first); err != nil {
		t.Fatal("new workspace is missing default skills", err)
	}
	original, err := os.ReadFile(filepath.Join(first.Root, "workspace.yaml"))
	if err != nil {
		t.Fatal(err)
	}
	second, err := svc.CreateWorkspace(first.Series.Title)
	if err != nil {
		t.Fatal(err)
	}
	if first.Root == second.Root || second.Series.ID != first.Series.ID+"-2" {
		t.Fatal("duplicate title reused an existing directory")
	}
	current, err := os.ReadFile(filepath.Join(first.Root, "workspace.yaml"))
	if err != nil || string(current) != string(original) {
		t.Fatal("existing workspace was overwritten", err)
	}
	for _, title := range []string{"CON", "日本語の物語", "../", "🦊"} {
		cfg, err := svc.CreateWorkspace(title)
		if err != nil || !workspace.ValidID(cfg.Series.ID) || filepath.Dir(cfg.Root) != svc.WorkspaceDirectory || cfg.Series.Title != title {
			t.Fatal("valid display title could not create a safe workspace", title, cfg, err)
		}
	}
	listed, err := svc.AvailableWorkspaces()
	if err != nil || len(listed) != 6 {
		t.Fatal("created workspaces are not selectable", listed, err)
	}
}

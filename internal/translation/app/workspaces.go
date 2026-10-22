package app

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"unicode"

	"github.com/voocel/ainovel-cli/internal/translation/workspace"
	"golang.org/x/text/unicode/norm"
)

type WorkspaceSummary struct {
	Root        string
	SeriesID    string
	Title       string
	VolumeCount int
}

// Read metadata only. Listing workspaces must not open a model, install skills,
// or change the currently selected workspace or its recent-history entry.
func (s *AppService) AvailableWorkspaces() ([]WorkspaceSummary, error) {
	var result []WorkspaceSummary
	seen := map[string]bool{}
	add := func(path string) {
		cfg, err := workspace.Load(path)
		if err != nil {
			return
		}
		root := cfg.Root
		if canonical, err := filepath.EvalSymlinks(root); err == nil {
			root = canonical
		}
		key := root
		if runtime.GOOS == "windows" {
			key = strings.ToLower(key)
		}
		if seen[key] {
			return
		}
		seen[key] = true
		title := strings.TrimSpace(cfg.Series.Title)
		if title == "" {
			title = cfg.Series.ID
		}
		result = append(result, WorkspaceSummary{Root: root, SeriesID: cfg.Series.ID, Title: title, VolumeCount: len(cfg.Volumes)})
	}
	if s.Config != nil {
		add(s.Config.Root)
	}
	for _, path := range s.RecentWorkspaces() {
		add(path)
	}
	err := filepath.WalkDir(s.WorkspaceDirectory, func(path string, entry fs.DirEntry, walkErr error) error {
		if walkErr != nil {
			if path == s.WorkspaceDirectory && !errors.Is(walkErr, os.ErrNotExist) {
				return walkErr
			}
			return nil
		}
		if !entry.IsDir() {
			return nil
		}
		if _, err := os.Stat(filepath.Join(path, "workspace.yaml")); err == nil {
			add(path)
			return fs.SkipDir
		}
		return nil
	})
	sort.Slice(result, func(i, j int) bool {
		a, b := strings.ToLower(result[i].Title), strings.ToLower(result[j].Title)
		if a == b {
			return result[i].Root < result[j].Root
		}
		return a < b
	})
	return result, err
}

// A user supplies the title, while filesystem names and valid ASCII IDs are
// derived internally. Punctuation and non-Latin titles are valid display names.
func workspaceID(title string) string {
	var slug strings.Builder
	separator := false
	for _, r := range norm.NFD.String(strings.ToLower(title)) {
		if unicode.Is(unicode.Mn, r) {
			continue
		}
		if r == 'đ' {
			r = 'd'
		}
		if r >= 'a' && r <= 'z' || r >= '0' && r <= '9' {
			if separator && slug.Len() > 0 {
				slug.WriteByte('-')
			}
			slug.WriteRune(r)
			separator = false
			if slug.Len() >= 60 {
				break
			}
		} else {
			separator = true
		}
	}
	id := slug.String()
	if id == "" {
		id = "bo-truyen"
	}
	if !workspace.ValidID(id) {
		id = "bo-truyen-" + id
	}
	return id
}

func (s *AppService) CreateWorkspace(title string) (*workspace.Config, error) {
	title = strings.TrimSpace(title)
	if title == "" || strings.EqualFold(title, "null") {
		return nil, errors.New("tên bộ truyện không được để trống hoặc null")
	}
	if err := os.MkdirAll(s.WorkspaceDirectory, 0755); err != nil {
		return nil, err
	}
	baseID := workspaceID(title)
	for suffix := 1; ; suffix++ {
		id := baseID
		if suffix > 1 {
			id = fmt.Sprintf("%s-%d", baseID, suffix)
		}
		root := filepath.Join(s.WorkspaceDirectory, id)
		// Reserve a new directory, never reuse or overwrite an existing series.
		if err := os.Mkdir(root, 0755); errors.Is(err, os.ErrExist) {
			continue
		} else if err != nil {
			return nil, err
		}
		return s.InitWorkspace(root, id, title)
	}
}

package skills

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	translationassets "github.com/voocel/ainovel-cli/assets/translation"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func sharedSettingsPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, ".novel-translate", "skills.json"), nil
}

func DefaultSharedDirectory() (string, error) {
	p, err := sharedSettingsPath()
	if err != nil {
		return "", err
	}
	return filepath.Join(filepath.Dir(p), "skills"), nil
}

// SharedDirectory is per user, independent of the current series or terminal.
func SharedDirectory() (string, error) {
	p, err := sharedSettingsPath()
	if err != nil {
		return "", err
	}
	data, err := os.ReadFile(p)
	if errors.Is(err, os.ErrNotExist) {
		return DefaultSharedDirectory()
	}
	if err != nil {
		return "", err
	}
	var cfg struct {
		Directory string `json:"directory"`
	}
	if err := json.Unmarshal(data, &cfg); err != nil {
		return "", fmt.Errorf("cấu hình thư viện skill chung không hợp lệ: %s", p)
	}
	if !filepath.IsAbs(cfg.Directory) {
		return "", errors.New("thư viện skill chung phải là đường dẫn tuyệt đối")
	}
	return filepath.Clean(cfg.Directory), nil
}

func LockShared(ctx context.Context) (func(), error) {
	p, err := sharedSettingsPath()
	if err != nil {
		return nil, err
	}
	return (state.Store{Root: filepath.Dir(p)}).Lock(ctx)
}

func skillFiles(directory, id string) ([]string, error) {
	if !workspace.ValidID(id) {
		return nil, fmt.Errorf("skill ID không hợp lệ: %s", id)
	}
	var found []string
	for _, rel := range []string{filepath.Join(id, "SKILL.md"), id + ".md"} {
		p := filepath.Join(directory, rel)
		info, err := os.Stat(p)
		if errors.Is(err, os.ErrNotExist) {
			continue
		}
		if err != nil {
			return nil, err
		}
		if info.IsDir() {
			return nil, fmt.Errorf("skill phải là file: %s", p)
		}
		found = append(found, p)
	}
	return found, nil
}

// Caller holds the shared lock. Existing skills are never overwritten.
func installShared(directory string) error {
	if err := os.MkdirAll(directory, 0755); err != nil {
		return err
	}
	for _, id := range workspace.Defaults().EnabledSkills {
		paths, err := skillFiles(directory, id)
		if err != nil {
			return err
		}
		if len(paths) > 1 {
			return fmt.Errorf("skill chung %s có nhiều định nghĩa", id)
		}
		if len(paths) == 1 {
			continue
		}
		data, err := translationassets.Files.ReadFile("skills/" + id + ".md")
		if err != nil {
			return err
		}
		p, err := (workspace.Config{Root: directory}).WritePath(filepath.Join(id, "SKILL.md"))
		if err != nil {
			return err
		}
		if err := state.Atomic(p, data); err != nil {
			return err
		}
	}
	return nil
}

func EnsureShared() (string, error) {
	directory, err := SharedDirectory()
	if err != nil {
		return "", err
	}
	complete := true
	for _, id := range workspace.Defaults().EnabledSkills {
		paths, err := skillFiles(directory, id)
		if err != nil {
			return "", err
		}
		if len(paths) > 1 {
			return "", fmt.Errorf("skill chung %s có nhiều định nghĩa", id)
		}
		if len(paths) == 0 {
			complete = false
		}
	}
	if complete {
		return directory, nil
	}
	unlock, err := LockShared(context.Background())
	if err != nil {
		return "", err
	}
	defer unlock()
	// Re-read the setting after locking; another instance may have changed it.
	directory, err = SharedDirectory()
	if err != nil {
		return "", err
	}
	return directory, installShared(directory)
}

// SetSharedDirectory affects all workspaces using the default shared library.
func SetSharedDirectory(ctx context.Context, directory string) error {
	if strings.TrimSpace(directory) == "" {
		return errors.New("cần thư mục skill chung")
	}
	directory, err := filepath.Abs(directory)
	if err != nil {
		return err
	}
	unlock, err := LockShared(ctx)
	if err != nil {
		return err
	}
	defer unlock()
	if err := installShared(directory); err != nil {
		return err
	}
	// Validate the seeded defaults before publishing the shared setting.
	for _, id := range workspace.Defaults().EnabledSkills {
		paths, err := skillFiles(directory, id)
		if err != nil {
			return err
		}
		if len(paths) != 1 {
			return fmt.Errorf("skill chung %s bị mất hoặc có nhiều định nghĩa", id)
		}
		data, err := os.ReadFile(paths[0])
		if err != nil {
			return err
		}
		sk, err := Parse(paths[0], data)
		if err != nil {
			return err
		}
		if sk.ID != id {
			return fmt.Errorf("skill ID không khớp: %s", id)
		}
	}
	p, err := sharedSettingsPath()
	if err != nil {
		return err
	}
	return state.WriteJSON(p, struct {
		Directory string `json:"directory"`
	}{directory})
}

func SharedSkills() ([]Skill, error) {
	directory, err := EnsureShared()
	if err != nil {
		return nil, err
	}
	entries, err := os.ReadDir(directory)
	if err != nil {
		return nil, err
	}
	ids := map[string]bool{}
	for _, entry := range entries {
		id := entry.Name()
		if !entry.IsDir() {
			if filepath.Ext(id) != ".md" {
				continue
			}
			id = strings.TrimSuffix(id, ".md")
		}
		if workspace.ValidID(id) {
			ids[id] = true
		}
	}
	var out []Skill
	for id := range ids {
		paths, err := skillFiles(directory, id)
		if err != nil {
			return nil, err
		}
		if len(paths) == 0 {
			continue
		}
		if len(paths) > 1 {
			return nil, fmt.Errorf("skill chung %s có nhiều định nghĩa", id)
		}
		data, err := os.ReadFile(paths[0])
		if err != nil {
			return nil, err
		}
		sk, err := Parse(paths[0], data)
		if err != nil {
			return nil, err
		}
		if sk.ID != id {
			return nil, fmt.Errorf("skill ID không khớp: %s", id)
		}
		out = append(out, sk)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].ID < out[j].ID })
	return out, nil
}

// SaveSharedSkill checks both library selection and file revision under one lock.
func SaveSharedSkill(ctx context.Context, directory, id, hash, text string) error {
	unlock, err := LockShared(ctx)
	if err != nil {
		return err
	}
	defer unlock()
	current, err := SharedDirectory()
	if err != nil {
		return err
	}
	if current != directory {
		return errors.New("thư viện skill chung đã đổi; giữ nội dung đang nhập và mở lại để đối chiếu")
	}
	paths, err := skillFiles(directory, id)
	if err != nil {
		return err
	}
	if len(paths) != 1 {
		return errors.New("file skill chung bị mất hoặc có nhiều định nghĩa")
	}
	rel, err := filepath.Rel(directory, paths[0])
	if err != nil {
		return err
	}
	p, err := (workspace.Config{Root: directory}).WritePath(rel)
	if err != nil {
		return err
	}
	data, err := os.ReadFile(p)
	if err != nil {
		return err
	}
	if domain.Hash(data) != hash {
		return errors.New("skill chung đã sửa ở nơi khác; nội dung đang nhập vẫn được giữ")
	}
	sk, err := Parse(p, []byte(text))
	if err != nil {
		return err
	}
	if sk.ID != id {
		return errors.New("không đổi skill ID trong trình sửa")
	}
	return state.Atomic(p, []byte(text))
}

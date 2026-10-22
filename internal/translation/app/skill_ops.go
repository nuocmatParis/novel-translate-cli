package app

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func (s *AppService) QuerySharedSkillLibrary() (string, []skills.Skill, error) {
	directory, err := skills.SharedDirectory()
	if err != nil {
		return "", nil, err
	}
	items, err := skills.SharedSkills()
	return directory, items, err
}

func (s *AppService) SaveSharedSkillDirectory(directory string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	directory = strings.Trim(strings.TrimSpace(directory), "\"'")
	if directory == "" {
		return errors.New("cần thư mục skill chung")
	}
	abs, err := filepath.Abs(directory)
	if err != nil {
		return err
	}
	if rel, err := filepath.Rel(s.Config.Root, abs); err == nil && rel != ".." && !strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
		return errors.New("thư mục skill chung phải ở ngoài workspace bộ truyện")
	}
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()
	return skills.SetSharedDirectory(context.Background(), abs)
}

func (s *AppService) SaveEnabledSkills(ids []string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	root := s.Config.Root
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()

	c, err := workspace.Load(root)
	if err != nil {
		return err
	}
	c.EnabledSkills = nil
	for _, id := range ids {
		id = strings.TrimSpace(id)
		if id != "" {
			c.EnabledSkills = append(c.EnabledSkills, id)
		}
	}
	if err := c.Validate(); err != nil {
		return err
	}
	if _, err := skills.Load(c); err != nil {
		return err
	}
	if err := c.Save(); err != nil {
		return err
	}
	s.Config = &c
	return nil
}

func (s *AppService) SaveSharedSkill(ctx context.Context, directory, id, expectedHash, text string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	unlock, err := s.Store.Lock(ctx)
	if err != nil {
		return err
	}
	defer unlock()
	return skills.SaveSharedSkill(ctx, directory, id, expectedHash, text)
}

func (s *AppService) SaveSeriesSkill(ctx context.Context, id, expectedHash, text string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	root := s.Config.Root
	unlock, err := s.Store.Lock(ctx)
	if err != nil {
		return err
	}
	defer unlock()

	c, err := workspace.Load(root)
	if err != nil {
		return err
	}
	if s.Config != nil && domain.Digest(c) != domain.Digest(*s.Config) {
		return errors.New("cấu hình workspace đã thay đổi; mở lại skill trước khi lưu")
	}
	path, err := c.WritePath(c.SeriesSkill)
	if err != nil {
		return err
	}
	current, err := os.ReadFile(path)
	if err != nil {
		return err
	}
	if domain.Hash(current) != expectedHash {
		return errors.New("quy ước đã được sửa ở nơi khác; nội dung đang nhập vẫn được giữ để đối chiếu")
	}
	parsed, err := skills.Parse(path, []byte(text))
	if err != nil {
		return fmt.Errorf("quy ước chưa hợp lệ: %w", err)
	}
	if parsed.ID != id {
		return errors.New("không đổi skill ID trong trình sửa")
	}
	if err := state.Atomic(path, []byte(text)); err != nil {
		return err
	}
	s.Config = &c
	return nil
}

func (s *AppService) ResolveSkillCandidate(ctx context.Context, id, decision, reason string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	unlock, err := s.Store.Lock(ctx)
	if err != nil {
		return err
	}
	defer unlock()
	c, err := workspace.Load(s.Config.Root)
	if err != nil {
		return err
	}
	_, err = skills.ResolveCandidate(c, id, decision, reason)
	return err
}

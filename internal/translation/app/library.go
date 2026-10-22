package app

import (
	"errors"
	"os"
	"path/filepath"
	"sort"

	"github.com/voocel/ainovel-cli/internal/translation/research"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
)

func (s *AppService) QuerySkillSnapshot() (skills.Snapshot, error) {
	if s.Config == nil {
		return skills.Snapshot{}, errors.New("chưa mở workspace")
	}
	return skills.Load(*s.Config)
}

func (s *AppService) QueryResearchEvidence() ([]research.Evidence, error) {
	if s.Config == nil {
		return nil, errors.New("chưa mở workspace")
	}
	files, err := filepath.Glob(s.Store.Path("research", "evidence", "*.json"))
	if err != nil {
		return nil, err
	}
	evidence := make([]research.Evidence, 0, len(files))
	for _, file := range files {
		var item research.Evidence
		if err := state.ReadJSON(file, &item); err != nil {
			if errors.Is(err, os.ErrNotExist) {
				continue
			}
			return evidence, err
		}
		evidence = append(evidence, item)
	}
	sort.Slice(evidence, func(i, j int) bool {
		if evidence[i].FetchedAt == evidence[j].FetchedAt {
			return evidence[i].ID < evidence[j].ID
		}
		return evidence[i].FetchedAt > evidence[j].FetchedAt
	})
	return evidence, nil
}

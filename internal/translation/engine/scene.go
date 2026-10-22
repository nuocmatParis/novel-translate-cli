package engine

import (
	"errors"
	"os"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type Scene struct {
	ID          string           `json:"id"`
	SourceHash  string           `json:"source_hash"`
	From        domain.Position  `json:"from"`
	Until       domain.Position  `json:"until"`
	StoryTime   *domain.Position `json:"story_time,omitempty"`
	Uncertainty string           `json:"uncertainty,omitempty"`
	Evidence    domain.Evidence  `json:"evidence"`
	Reason      string           `json:"reason"`
}

func SceneAt(c workspace.Config, v workspace.Volume, p domain.Position) (*Scene, error) {
	var scenes []Scene
	if e := state.ReadJSON(state.Store{Root: c.Root}.Path("volumes", v.ID, "scenes.json"), &scenes); e != nil && !errors.Is(e, os.ErrNotExist) {
		return nil, e
	}
	var selected *Scene
	for _, s := range scenes {
		if !p.Before(s.From) && p.Before(s.Until) {
			if selected != nil {
				return nil, errors.New("temporal scene scopes chồng nhau")
			}
			copy := s
			selected = &copy
		}
	}
	return selected, nil
}
func SetScene(c workspace.Config, v workspace.Volume, book domain.Book, start, end string, storyTime *domain.Position, quote, reason string, uncertain bool) error {
	var first, last domain.Block
	for _, b := range book.Blocks {
		if b.ID == start {
			first = b
		}
		if b.ID == end {
			last = b
		}
	}
	if first.ID == "" || last.ID == "" || last.Position.Before(first.Position) || quote == "" || !strings.Contains(first.Source, quote) || strings.TrimSpace(reason) == "" {
		return errors.New("scene range/source quote/reason không hợp lệ")
	}
	if !uncertain && (storyTime == nil || storyTime.Volume < 1 || first.Position.Before(*storyTime)) {
		return errors.New("hồi tưởng cần story_time đã xác nhận không sau vị trí xuất bản")
	}
	s := Scene{SourceHash: book.SourceHash, From: first.Position, Until: last.Position, StoryTime: storyTime, Evidence: domain.Evidence{BlockID: first.ID, Quote: quote}, Reason: reason}
	s.Until.Block++
	if uncertain {
		s.StoryTime = nil
		s.Uncertainty = reason
	}
	s.ID = domain.Digest(s)[:24]
	store := state.Store{Root: c.Root}
	path := store.Path("volumes", v.ID, "scenes.json")
	var all []Scene
	if e := state.ReadJSON(path, &all); e != nil && !errors.Is(e, os.ErrNotExist) {
		return e
	}
	for i, old := range all {
		if old.From == s.From && old.Until == s.Until {
			all[i] = s
			return state.WriteJSON(path, all)
		}
		if s.From.Before(old.Until) && old.From.Before(s.Until) {
			return errors.New("scene range chồng nhau; xóa scope cũ trước")
		}
	}
	return state.WriteJSON(path, append(all, s))
}
func ContextDigest(c workspace.Config, v workspace.Volume, events []domain.Event, p domain.Position) (string, error) {
	s, e := SceneAt(c, v, p)
	if e != nil {
		return "", e
	}
	if s == nil {
		return domain.Digest(events), nil
	}
	return domain.Digest(struct {
		Events []domain.Event
		Scene  *Scene
	}{events, s}), nil
}

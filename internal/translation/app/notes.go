package app

import (
	"context"
	"errors"
	"os"
	"strings"
	"time"

	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/state"
)

type UnitNote struct {
	UnitID     string `json:"unit_id"`
	SourceHash string `json:"source_hash"`
	Text       string `json:"text"`
	UpdatedAt  string `json:"updated_at"`
	Stale      bool   `json:"stale"`
}

func (s *AppService) QueryUnitNote(volumeID, unitID string) (UnitNote, error) {
	if s.Config == nil {
		return UnitNote{}, errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return UnitNote{}, err
	}
	_, book, err := engine.Source(*s.Config, v, false)
	if err != nil {
		return UnitNote{}, err
	}
	var sourceHash string
	for _, block := range book.Blocks {
		if block.ID == unitID {
			sourceHash = block.SourceHash
			break
		}
	}
	if sourceHash == "" {
		return UnitNote{}, errors.New("không tìm thấy khối nguồn: " + unitID)
	}
	var notes map[string]UnitNote
	path := s.Store.Path("volumes", volumeID, "notes.json")
	if err := state.ReadJSON(path, &notes); err != nil && !errors.Is(err, os.ErrNotExist) {
		return UnitNote{}, err
	}
	note := notes[unitID]
	note.Stale = note.SourceHash != "" && note.SourceHash != sourceHash
	return note, nil
}

func (s *AppService) SaveUnitNote(volumeID, unitID, text string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return err
	}
	_, book, err := engine.Source(*s.Config, v, true)
	if err != nil {
		return err
	}
	var sourceHash string
	for _, block := range book.Blocks {
		if block.ID == unitID {
			sourceHash = block.SourceHash
			break
		}
	}
	if sourceHash == "" {
		return errors.New("không tìm thấy khối nguồn: " + unitID)
	}
	path := s.Store.Path("volumes", volumeID, "notes.json")
	var notes map[string]UnitNote
	if err := state.ReadJSON(path, &notes); err != nil && !errors.Is(err, os.ErrNotExist) {
		return err
	}
	if notes == nil {
		notes = map[string]UnitNote{}
	}
	if strings.TrimSpace(text) == "" {
		delete(notes, unitID)
	} else {
		notes[unitID] = UnitNote{UnitID: unitID, SourceHash: sourceHash, Text: text, UpdatedAt: time.Now().UTC().Format(time.RFC3339Nano)}
	}
	return state.WriteJSON(path, notes)
}

package app

import (
	"errors"
	"os"
	"path/filepath"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
)

type VolumeCover struct {
	VolumeID       string
	Order          int
	Title          string
	TitleSource    string
	SourcePath     string
	SourceHash     string
	ResourcePath   string
	ResourceHash   string
	MediaType      string
	Image          []byte
	Imported       bool
	SnapshotStale  bool
	CoverAvailable bool
}

// QueryVolumeCover loads only the selected volume's declared cover. It reads
// the imported source snapshot when present, so the title and image stay
// paired with the content the workspace currently knows about.
func (s *AppService) QueryVolumeCover(volumeID string) (VolumeCover, error) {
	if s.Config == nil {
		return VolumeCover{}, errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return VolumeCover{}, err
	}

	result := VolumeCover{
		VolumeID:   v.ID,
		Order:      v.Order,
		SourcePath: v.Source,
	}
	raw, book, err := engine.Source(*s.Config, v, false)
	if err == nil {
		result.Imported = true
	} else {
		_, pointerErr := os.Stat(s.Store.Path("volumes", v.ID, "source.json"))
		if pointerErr == nil || !errors.Is(pointerErr, os.ErrNotExist) {
			return result, err
		}
		// Before import there is no source snapshot yet. Reading the registered
		// EPUB here is read-only and lets the dashboard show its real metadata.
		raw, err = os.ReadFile(s.Config.ReadPath(v.Source))
		if err != nil {
			result.Title = filepath.Base(v.Source)
			result.TitleSource = "tên file (EPUB chưa đọc được)"
			return result, err
		}
		book = domain.Book{}
	}

	result.SourceHash = domain.Hash(raw)
	parsed, err := epub.Read(raw, v.ID, v.Order)
	if err != nil {
		result.Title = filepath.Base(v.Source)
		result.TitleSource = "tên file (metadata EPUB không đọc được)"
		return result, err
	}
	if book.Title != "" {
		result.Title = book.Title
	} else {
		result.Title = parsed.Book.Title
	}
	if result.Title != "" {
		result.TitleSource = "metadata EPUB"
	} else {
		result.Title = filepath.Base(v.Source)
		result.TitleSource = "tên file"
	}
	if resource, data, ok := parsed.Cover(); ok {
		result.ResourcePath = resource.Path
		result.ResourceHash = resource.Hash
		result.MediaType = resource.MediaType
		result.Image = data
		result.CoverAvailable = true
	}

	// If the registered source changed after import, show that the visible
	// cover is from the imported snapshot instead of silently relabeling it.
	if result.Imported {
		if live, readErr := os.ReadFile(s.Config.ReadPath(v.Source)); readErr == nil {
			result.SnapshotStale = domain.Hash(live) != result.SourceHash
		}
	}
	return result, nil
}

func (v VolumeCover) DisplayTitle() string {
	return strings.TrimSpace(v.Title)
}

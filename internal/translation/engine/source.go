package engine

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type SourcePointer struct {
	Hash         string `json:"hash"`
	ManifestHash string `json:"manifest_hash"`
	ManifestFile string `json:"manifest_file,omitempty"`
}

func Import(c workspace.Config, v workspace.Volume) (domain.Book, error) {
	s := state.Store{Root: c.Root}
	source, e := os.ReadFile(c.ReadPath(v.Source))
	if e != nil {
		return domain.Book{}, e
	}
	p, e := epub.Read(source, v.ID, v.Order)
	if e != nil {
		return domain.Book{}, e
	}
	var current SourcePointer
	e = state.ReadJSON(s.Path("volumes", v.ID, "source.json"), &current)
	if e == nil && current.Hash != p.Book.SourceHash {
		return domain.Book{}, errors.New("source đã thay đổi; đăng ký revision nguồn mới trước khi dùng mapping cũ")
	}
	if e != nil && !errors.Is(e, os.ErrNotExist) {
		return domain.Book{}, e
	}
	if current.Hash != "" {
		_, old, e := Source(c, v, false)
		if e != nil {
			return domain.Book{}, e
		}
		g, e := s.Load(v.ID)
		if e != nil {
			return domain.Book{}, e
		}
		if domain.Digest(old) != domain.Digest(p.Book) && len(g.Units) > 0 {
			return domain.Book{}, errors.New("parser snapshot đổi; chạy source plan/apply để giữ mapping cũ")
		}
	}
	ptr, e := saveSourceSnapshot(s, v, source, p.Book)
	if e != nil {
		return domain.Book{}, e
	}
	if e = state.WriteJSON(s.Path("volumes", v.ID, "source.json"), ptr); e != nil {
		return domain.Book{}, e
	}
	return p.Book, nil
}
func Source(c workspace.Config, v workspace.Volume, checkOriginal bool) ([]byte, domain.Book, error) {
	s := state.Store{Root: c.Root}
	var ptr SourcePointer
	if e := state.ReadJSON(s.Path("volumes", v.ID, "source.json"), &ptr); e != nil {
		return nil, domain.Book{}, fmt.Errorf("cần import tập trước: %w", e)
	}
	if len(ptr.Hash) != 64 {
		return nil, domain.Book{}, errors.New("source pointer sai")
	}
	g, e := s.Load(v.ID)
	if e != nil {
		return nil, domain.Book{}, e
	}
	if g.SourceHash != "" && (g.SourceHash != ptr.Hash || g.SourceManifestHash != "" && g.SourceManifestHash != ptr.ManifestHash) {
		if len(g.SourceManifestHash) != 64 {
			return nil, domain.Book{}, errors.New("generation source manifest checksum thiếu")
		}
		ptr = SourcePointer{Hash: g.SourceHash, ManifestHash: g.SourceManifestHash, ManifestFile: g.SourceManifestFile}
	}
	raw, e := os.ReadFile(s.Path("volumes", v.ID, "source", ptr.Hash, "source.epub"))
	if e != nil {
		return nil, domain.Book{}, e
	}
	if domain.Hash(raw) != ptr.Hash {
		return nil, domain.Book{}, errors.New("source snapshot checksum sai")
	}
	manifestFile := ptr.ManifestFile
	if manifestFile == "" {
		manifestFile = "manifest.json"
	}
	if filepath.Base(manifestFile) != manifestFile {
		return nil, domain.Book{}, errors.New("manifest pointer path sai")
	}
	manifest, e := os.ReadFile(s.Path("volumes", v.ID, "source", ptr.Hash, manifestFile))
	if e != nil {
		return nil, domain.Book{}, e
	}
	if domain.Hash(manifest) != ptr.ManifestHash {
		return nil, domain.Book{}, errors.New("source manifest checksum sai")
	}
	var saved domain.Book
	if e = json.Unmarshal(manifest, &saved); e != nil {
		return nil, domain.Book{}, e
	}
	if saved.SourceHash != ptr.Hash || saved.SchemaVersion != domain.SchemaVersion {
		return nil, domain.Book{}, errors.New("source manifest schema/hash không hợp lệ")
	}
	if saved.ParserVersion != epub.ParserVersion {
		if checkOriginal {
			return nil, saved, errors.New("parser snapshot đổi; cần source plan/apply")
		}
		return raw, saved, nil
	}
	if !checkOriginal {
		return raw, saved, nil
	}
	p, e := epub.Read(raw, v.ID, v.Order)
	if e != nil {
		return nil, domain.Book{}, e
	}
	if domain.Digest(saved) != domain.Digest(p.Book) {
		return nil, saved, errors.New("parser manifest fingerprint đổi; chạy source plan/apply")
	}
	if checkOriginal {
		live, e := os.ReadFile(c.ReadPath(v.Source))
		if e != nil {
			return nil, p.Book, e
		}
		if domain.Hash(live) != ptr.Hash {
			return nil, p.Book, errors.New("file nguồn đăng ký đã đổi; không dùng source mapping stale")
		}
	}
	return raw, p.Book, nil
}

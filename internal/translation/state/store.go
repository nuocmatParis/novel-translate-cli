// Package state implements immutable generations and one workspace writer lock.
package state

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/gofrs/flock"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

// Atomic replaces a single file after fsync. A generation is a single JSON
// artifact; its checksummed pointer is the transaction commit point.
func Atomic(path string, data []byte) error {
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return err
	}
	f, err := os.CreateTemp(filepath.Dir(path), ".write-*")
	if err != nil {
		return err
	}
	defer os.Remove(f.Name())
	if _, err = f.Write(data); err != nil {
		f.Close()
		return err
	}
	if err = f.Sync(); err != nil {
		f.Close()
		return err
	}
	if err = f.Close(); err != nil {
		return err
	}
	return replace(f.Name(), path)
}
func WriteJSON(path string, v any) error {
	b, e := json.MarshalIndent(v, "", "  ")
	if e != nil {
		return e
	}
	return Atomic(path, append(b, '\n'))
}
func ReadJSON(path string, v any) error {
	b, e := os.ReadFile(path)
	if e != nil {
		return e
	}
	return json.Unmarshal(b, v)
}

type Store struct{ Root string }

func (s Store) Path(parts ...string) string {
	return filepath.Join(append([]string{s.Root, ".translation_work"}, parts...)...)
}
func (s Store) Lock(ctx context.Context) (func(), error) {
	p := s.Path("writer.lock")
	if err := os.MkdirAll(filepath.Dir(p), 0755); err != nil {
		return nil, err
	}
	l := flock.New(p)
	ok, err := l.TryLockContext(ctx, 100*time.Millisecond)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, errors.New("workspace đang được tiến trình khác ghi")
	}
	return func() { _ = l.Unlock() }, nil
}

type Pointer struct {
	ID   string `json:"id"`
	Hash string `json:"hash"`
}

func (s Store) History(volume, id string) (domain.Generation, error) {
	g := domain.Generation{}
	if filepath.Base(id) != id || id == "" || id == "." {
		return g, errors.New("generation ID sai")
	}
	var checksum Pointer
	if e := ReadJSON(s.Path("volumes", volume, "generations", id+".checksum.json"), &checksum); e != nil {
		return g, e
	}
	b, e := os.ReadFile(s.Path("volumes", volume, "generations", id+".json"))
	if e != nil {
		return g, e
	}
	if checksum.ID != id || checksum.Hash != domain.Hash(b) {
		return g, errors.New("history checksum sai")
	}
	if e = json.Unmarshal(b, &g); e != nil {
		return g, e
	}
	if g.ID != id || g.SchemaVersion != domain.SchemaVersion || g.Units == nil {
		return g, errors.New("history schema sai")
	}
	return g, nil
}

func (s Store) Load(volume string) (domain.Generation, error) {
	g := domain.Generation{SchemaVersion: domain.SchemaVersion, Units: map[string]domain.Unit{}}
	var p Pointer
	err := ReadJSON(s.Path("volumes", volume, "current.json"), &p)
	if errors.Is(err, os.ErrNotExist) {
		return g, nil
	}
	if err != nil {
		return g, err
	}
	if filepath.Base(p.ID) != p.ID || p.ID == "." || p.ID == "" {
		return g, errors.New("generation pointer không hợp lệ")
	}
	b, err := os.ReadFile(s.Path("volumes", volume, "generations", p.ID+".json"))
	if err != nil {
		return g, err
	}
	if domain.Hash(b) != p.Hash {
		return g, errors.New("generation checksum không khớp")
	}
	err = json.Unmarshal(b, &g)
	if err == nil && (g.SchemaVersion != domain.SchemaVersion || g.ID != p.ID || g.Units == nil) {
		err = errors.New("generation schema/ID không hợp lệ")
	}
	return g, err
}
func (s Store) Commit(volume string, g *domain.Generation, expected string) error {
	current, err := s.Load(volume)
	if err != nil {
		return err
	}
	if current.ID != expected {
		return errors.New("generation đã thay đổi; giữ nguyên dữ liệu hiện tại")
	}
	copy := *g
	copy.SchemaVersion = domain.SchemaVersion
	copy.Parent = expected
	copy.CreatedAt = time.Now().UTC().Format(time.RFC3339Nano)
	copy.ID = domain.Digest(struct {
		Parent, Time string
		Units        map[string]domain.Unit
	}{expected, copy.CreatedAt, copy.Units})[:24]
	b, err := json.MarshalIndent(copy, "", "  ")
	if err != nil {
		return err
	}
	if err = Atomic(s.Path("volumes", volume, "generations", copy.ID+".json"), b); err != nil {
		return err
	}
	if err = WriteJSON(s.Path("volumes", volume, "generations", copy.ID+".checksum.json"), Pointer{copy.ID, domain.Hash(b)}); err != nil {
		return err
	}
	if err = WriteJSON(s.Path("volumes", volume, "current.json"), Pointer{copy.ID, domain.Hash(b)}); err != nil {
		return fmt.Errorf("generation lưu nhưng chưa commit: %w", err)
	}
	*g = copy
	return nil
}

package engine

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type SourceChange struct {
	OldID  string `json:"old_id"`
	NewID  string `json:"new_id"`
	Action string `json:"action"`
	Reason string `json:"reason"`
}
type SourceRevision struct {
	ID           string         `json:"id"`
	Volume       string         `json:"volume"`
	Generation   string         `json:"generation"`
	OldHash      string         `json:"old_hash"`
	NewHash      string         `json:"new_hash"`
	ManifestHash string         `json:"manifest_hash"`
	ManifestFile string         `json:"manifest_file"`
	Changes      []SourceChange `json:"changes"`
}

func saveSourceSnapshot(store state.Store, v workspace.Volume, raw []byte, book domain.Book) (SourcePointer, error) {
	dir := store.Path("volumes", v.ID, "source", book.SourceHash)
	ptr := SourcePointer{Hash: book.SourceHash, ManifestFile: "manifest-" + domain.Digest(book) + ".json"}
	if e := state.Atomic(filepath.Join(dir, "source.epub"), raw); e != nil {
		return ptr, e
	}
	if e := state.WriteJSON(filepath.Join(dir, ptr.ManifestFile), book); e != nil {
		return ptr, e
	}
	b, e := os.ReadFile(filepath.Join(dir, ptr.ManifestFile))
	if e != nil {
		return ptr, e
	}
	ptr.ManifestHash = domain.Hash(b)
	return ptr, nil
}
func PlanSourceRevision(c workspace.Config, v workspace.Volume) (SourceRevision, error) {
	store := state.Store{Root: c.Root}
	_, old, e := Source(c, v, false)
	if e != nil {
		return SourceRevision{}, e
	}
	g, e := store.Load(v.ID)
	if e != nil {
		return SourceRevision{}, e
	}
	if e = revision.Check(c, v, old, g); e != nil {
		return SourceRevision{}, e
	}
	raw, e := os.ReadFile(c.ReadPath(v.Source))
	if e != nil {
		return SourceRevision{}, e
	}
	p, e := epub.Read(raw, v.ID, v.Order)
	if e != nil {
		return SourceRevision{}, e
	}
	if domain.Digest(p.Book) == domain.Digest(old) {
		return SourceRevision{}, errors.New("nguồn chưa thay đổi")
	}
	manifest, e := saveSourceSnapshot(store, v, raw, p.Book)
	if e != nil {
		return SourceRevision{}, e
	}
	plan := SourceRevision{Volume: v.ID, Generation: g.ID, OldHash: old.SourceHash, NewHash: p.Book.SourceHash, ManifestHash: manifest.ManifestHash, ManifestFile: manifest.ManifestFile}
	// Only an exact document/DOM locator/kind correspondence is mapped. Any
	// insertion or ambiguity keeps the original unit in the immutable history.
	newByLocation := map[string]domain.Block{}
	for _, b := range p.Book.Blocks {
		newByLocation[b.Document+"|"+b.Locator+"|"+b.Kind+"|"+b.Tag] = b
	}
	for _, b := range old.Blocks {
		u, exists := g.Units[b.ID]
		if !exists {
			continue
		}
		n, ok := newByLocation[b.Document+"|"+b.Locator+"|"+b.Kind+"|"+b.Tag]
		change := SourceChange{OldID: b.ID, Action: "archive", Reason: "không có mapping chắc chắn; bản cũ được giữ trong generation/source archive"}
		if ok {
			change.NewID = n.ID
			if n.SourceHash == b.SourceHash {
				change.Action = "review"
				change.Reason = "nguồn khối giữ nguyên; đối chiếu lại vị trí/context revision mới"
			} else if u.Origin == "manual" {
				change.Action = "review-manual"
				change.Reason = "nguồn thay đổi; giữ bản sửa tay và yêu cầu đối chiếu"
			} else {
				change.Action = "translate"
				change.Reason = "nguồn khối đã thay đổi"
			}
		}
		plan.Changes = append(plan.Changes, change)
	}
	plan.ID = domain.Digest(plan)[:24]
	return plan, state.WriteJSON(store.Path("source_plans", plan.ID+".json"), plan)
}
func ApplySourceRevision(c workspace.Config, v workspace.Volume, plan SourceRevision) error {
	copy := plan
	copy.ID = ""
	if plan.Volume != v.ID || domain.Digest(copy)[:24] != plan.ID {
		return errors.New("source plan checksum/volume sai")
	}
	store := state.Store{Root: c.Root}
	g, e := store.Load(v.ID)
	if e != nil {
		return e
	}
	if g.ID != plan.Generation {
		return errors.New("source plan generation stale")
	}
	_, old, e := Source(c, v, false)
	if e != nil {
		return e
	}
	if old.SourceHash != plan.OldHash {
		return errors.New("source plan inputs stale")
	}
	if e = revision.Check(c, v, old, g); e != nil {
		return e
	}
	live, e := os.ReadFile(c.ReadPath(v.Source))
	if e != nil {
		return e
	}
	if domain.Hash(live) != plan.NewHash {
		return errors.New("file nguồn đổi sau khi lập plan")
	}
	p, e := epub.Read(live, v.ID, v.Order)
	if e != nil {
		return e
	}
	if filepath.Base(plan.ManifestFile) != plan.ManifestFile {
		return errors.New("source plan manifest path sai")
	}
	manifest, e := os.ReadFile(store.Path("volumes", v.ID, "source", plan.NewHash, plan.ManifestFile))
	if e != nil {
		return e
	}
	if domain.Hash(manifest) != plan.ManifestHash {
		return errors.New("source snapshot checksum sai")
	}
	// Preserve complete editable views before changing markers, even for unmapped
	// or removed chapters. They remain readable without restoring a generation.
	for _, ch := range old.Chapters {
		path, e := revision.ChapterPath(c, v, ch.ID)
		if e != nil {
			return e
		}
		b, e := os.ReadFile(path)
		if errors.Is(e, os.ErrNotExist) {
			continue
		}
		if e != nil {
			return e
		}
		if e = state.Atomic(store.Path("source_view_archive", plan.ID, ch.ID+".md"), b); e != nil {
			return e
		}
	}
	if e = revision.CaptureViews(c, v, p.Book, &g); e != nil {
		return e
	}
	units := map[string]domain.Unit{}
	blocks := map[string]domain.Block{}
	for _, b := range p.Book.Blocks {
		blocks[b.ID] = b
	}
	for _, change := range plan.Changes {
		if change.Action != "review" && change.Action != "review-manual" {
			continue
		}
		u, ok := g.Units[change.OldID]
		if !ok {
			return errors.New("source plan unit thiếu")
		}
		b, ok := blocks[change.NewID]
		if !ok {
			return errors.New("source plan mapping thiếu")
		}
		if _, e = epub.TokensHTML(b, u.Target); e != nil && change.Action == "review-manual" {
			return fmt.Errorf("manual inline mapping cần xử lý: %s: %w", u.ID, e)
		}
		u.ID = b.ID
		u.SourceHash = b.SourceHash
		u.Revision++
		u.Dependencies.Source = b.SourceHash
		u.Review = nil
		u.Events = nil
		u.Rules = nil
		u.Children = nil
		u.StaleReason = ""
		u.RefreshDependencies = true
		units[b.ID] = u
	}
	g.Units = units
	g.SourceHash = plan.NewHash
	g.SourceManifestHash = plan.ManifestHash
	g.SourceManifestFile = plan.ManifestFile
	// The generation pointer commits both source and translation mapping together.
	// source.json is a convenience pointer; Source also honors this generation.
	if e = store.Commit(v.ID, &g, g.ID); e != nil {
		return e
	}
	if e = state.WriteJSON(store.Path("volumes", v.ID, "source.json"), SourcePointer{Hash: plan.NewHash, ManifestHash: plan.ManifestHash, ManifestFile: plan.ManifestFile}); e != nil {
		return e
	}
	return revision.Materialize(c, v, p.Book, g)
}

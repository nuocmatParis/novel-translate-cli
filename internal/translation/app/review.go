package app

import (
	"context"
	"errors"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
)

func (s *AppService) bindGenerationSource(volumeID string, book domain.Book, g *domain.Generation) error {
	if g.SourceHash != "" && g.SourceHash != book.SourceHash {
		return errors.New("generation/source khác nhau")
	}
	g.SourceHash = book.SourceHash
	if g.SourceManifestHash == "" {
		var pointer engine.SourcePointer
		if err := state.ReadJSON(s.Store.Path("volumes", volumeID, "source.json"), &pointer); err != nil {
			return err
		}
		if pointer.Hash != book.SourceHash {
			return errors.New("source pointer không khớp snapshot đang dùng")
		}
		g.SourceManifestHash = pointer.ManifestHash
		g.SourceManifestFile = pointer.ManifestFile
	}
	return nil
}

func (s *AppService) KeepRevision(volumeID, unitID, reason string) error {
	reason = strings.TrimSpace(reason)
	if reason == "" {
		return errors.New("lý do giữ bản dịch là bắt buộc")
	}
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return err
	}
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()

	_, book, err := engine.Source(*s.Config, v, true)
	if err != nil {
		return err
	}
	g, err := s.Store.Load(v.ID)
	if err != nil {
		return err
	}
	if err := s.bindGenerationSource(v.ID, book, &g); err != nil {
		return err
	}
	u, ok := g.Units[unitID]
	if !ok {
		return errors.New("không tìm thấy khối: " + unitID)
	}

	var block *domain.Block
	for i := range book.Blocks {
		if book.Blocks[i].ID == unitID {
			block = &book.Blocks[i]
			break
		}
	}
	if block == nil {
		return errors.New("không tìm thấy khối nguồn: " + unitID)
	}

	// A user keep decision is made against the current source and local
	// dependencies. Capture those fingerprints so a manual revision does not
	// immediately appear stale merely because it has no model-generated deps.
	snapshot, err := skills.Load(*s.Config)
	if err != nil {
		return err
	}
	events, err := engine.ContextEvents(*s.Config, v, s.Store, g, block.Position, nil)
	if err != nil {
		return err
	}
	contextHash, err := engine.ContextDigest(*s.Config, v, events, block.Position)
	if err != nil {
		return err
	}
	rules, _, err := engine.ResolvedRules(*s.Config, v, s.Store, g, snapshot, block.Position, block.Source)
	if err != nil {
		return err
	}
	ruleHashes := make(map[string]string, len(rules))
	for _, rule := range rules {
		ruleHashes[rule.ID] = domain.Digest(rule)
	}

	expectedTargetHash := domain.Hash([]byte(u.Target))
	u.SourceHash = block.SourceHash
	u.StaleReason = ""
	u.RefreshDependencies = false
	u.Dependencies = domain.Dependencies{
		Source:  block.SourceHash,
		Skills:  snapshot.Fingerprint(block.Position, block.Source),
		Rules:   ruleHashes,
		Context: contextHash,
	}
	if u.Review == nil {
		u.Review = &domain.Review{
			SourceHash: block.SourceHash,
			TargetHash: expectedTargetHash,
		}
	}
	u.Review.Decision = "user-kept"
	u.Review.DecisionReason = reason
	u.Review.SourceHash = block.SourceHash
	u.Review.TargetHash = expectedTargetHash

	g.Units[unitID] = u
	if err := s.Store.Commit(v.ID, &g, g.ID); err != nil {
		return err
	}
	return revision.Materialize(*s.Config, v, book, g)
}

func (s *AppService) SaveManualTranslation(volumeID, unitID, newTarget string) error {
	newTarget = strings.TrimSpace(newTarget)
	if newTarget == "" {
		return errors.New("bản dịch không được để trống")
	}
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return err
	}
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()

	_, book, err := engine.Source(*s.Config, v, true)
	if err != nil {
		return err
	}
	g, err := s.Store.Load(v.ID)
	if err != nil {
		return err
	}
	if err := s.bindGenerationSource(v.ID, book, &g); err != nil {
		return err
	}
	if g.Units == nil {
		g.Units = map[string]domain.Unit{}
	}

	var block *domain.Block
	for i := range book.Blocks {
		if book.Blocks[i].ID == unitID {
			block = &book.Blocks[i]
			break
		}
	}
	if block == nil {
		return errors.New("không tìm thấy khối nguồn: " + unitID)
	}

	if _, err := epub.TokensHTML(*block, newTarget); err != nil {
		return err
	}

	u, ok := g.Units[unitID]
	if !ok {
		u = domain.Unit{
			ID:         unitID,
			SourceHash: block.SourceHash,
			Revision:   1,
			Origin:     "manual",
		}
	} else {
		u.Revision++
		u.Origin = "manual"
	}
	u.Target = newTarget
	u.Review = nil
	u.RefreshDependencies = true
	u.StaleReason = ""

	g.Units[unitID] = u
	if err := revision.CaptureViews(*s.Config, v, book, &g); err != nil {
		return err
	}
	if err := s.Store.Commit(v.ID, &g, g.ID); err != nil {
		return err
	}
	return revision.Materialize(*s.Config, v, book, g)
}

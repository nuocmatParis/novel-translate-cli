package app

import (
	"errors"
	"os"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/verify"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type VolumeSummary struct {
	ID             string   `json:"id"`
	Order          int      `json:"order"`
	BookTitle      string   `json:"book_title,omitempty"`
	SourcePath     string   `json:"source_path"`
	PreviousID     string   `json:"previous_id"`
	Accepted       int      `json:"accepted"`
	Eligible       int      `json:"eligible"`
	Translated     int      `json:"translated"`
	StaleCount     int      `json:"stale_count"`
	StaleIDs       []string `json:"stale_ids"`
	FreshnessError string   `json:"freshness_error"`
	Imported       bool     `json:"imported"`
	Images         int      `json:"images"`
	ImageIssues    int      `json:"image_issues"`
	ChaptersCount  int      `json:"chapters_count"`
	BatchesCount   int      `json:"batches_count"`
	GenerationID   string   `json:"generation_id"`
	LastSaveTime   string   `json:"last_save_time"`
}

type VolumeTotals struct {
	TotalEligible    int `json:"total_eligible"`
	TotalAccepted    int `json:"total_accepted"`
	TotalStale       int `json:"total_stale"`
	TotalImages      int `json:"total_images"`
	TotalImageIssues int `json:"total_image_issues"`
}

type DashboardProjection struct {
	SeriesID            string             `json:"series_id"`
	SeriesTitle         string             `json:"series_title"`
	Root                string             `json:"root"`
	Volumes             []VolumeSummary    `json:"volumes"`
	Totals              VolumeTotals       `json:"totals"`
	Usage               model.UsageSummary `json:"usage"`
	RecentRunOrSave     string             `json:"recent_run_or_save"`
	BlockingIssuesCount int                `json:"blocking_issues_count"`
	ImageIssuesCount    int                `json:"image_issues_count"`
	RuleConflictCount   int                `json:"rule_conflict_count"`
}

type ChapterSummary struct {
	ID            string `json:"id"`
	Title         string `json:"title"`
	Order         int    `json:"order"`
	Kind          string `json:"kind"`
	UnitsCount    int    `json:"units_count"`
	EligibleCount int    `json:"eligible_count"`
	AcceptedCount int    `json:"accepted_count"`
	BlockingCount int    `json:"blocking_count"`
	WarningCount  int    `json:"warning_count"`
	StaleCount    int    `json:"stale_count"`
	Status        string `json:"status"`
}

type UnitDetail struct {
	ID             string                `json:"id"`
	ChapterID      string                `json:"chapter_id"`
	Sequence       int                   `json:"sequence"`
	Source         string                `json:"source"`
	SourceHash     string                `json:"source_hash"`
	Target         string                `json:"target"`
	Revision       int                   `json:"revision"`
	Origin         string                `json:"origin"`
	Accepted       bool                  `json:"accepted"`
	Status         string                `json:"status"`
	StaleReason    string                `json:"stale_reason"`
	Model          string                `json:"model,omitempty"`
	Decision       string                `json:"decision,omitempty"`
	DecisionReason string                `json:"decision_reason,omitempty"`
	Issues         []domain.Issue        `json:"issues"`
	RubyDecisions  []domain.RubyDecision `json:"ruby_decisions"`
	Signals        []domain.Issue        `json:"signals"`
	Children       []domain.Child        `json:"children"`
}

type UnitRevision struct {
	GenerationID string         `json:"generation_id"`
	CreatedAt    string         `json:"created_at"`
	Revision     int            `json:"revision"`
	Target       string         `json:"target"`
	Origin       string         `json:"origin"`
	Review       *domain.Review `json:"review,omitempty"`
}

type ReviewQueueItem struct {
	ChapterID      string                `json:"chapter_id"`
	ChapterTitle   string                `json:"chapter_title"`
	UnitID         string                `json:"unit_id"`
	Revision       int                   `json:"revision"`
	Source         string                `json:"source"`
	Target         string                `json:"target"`
	IssuesCount    int                   `json:"issues_count"`
	BlockingCount  int                   `json:"blocking_count"`
	WarningCount   int                   `json:"warning_count"`
	RubyCount      int                   `json:"ruby_count"`
	Status         string                `json:"status"`
	StaleReason    string                `json:"stale_reason,omitempty"`
	Model          string                `json:"model,omitempty"`
	Decision       string                `json:"decision,omitempty"`
	DecisionReason string                `json:"decision_reason,omitempty"`
	Issues         []domain.Issue        `json:"issues,omitempty"`
	RubyDecisions  []domain.RubyDecision `json:"ruby_decisions,omitempty"`
	Signals        []domain.Issue        `json:"signals,omitempty"`
	Children       []domain.Child        `json:"children,omitempty"`
	PrimaryIssue   *domain.Issue         `json:"primary_issue,omitempty"`
}

type ScopePlan struct {
	TotalUnits            int      `json:"total_units"`
	UnitsToTranslate      int      `json:"units_to_translate"`
	UnitsToReview         int      `json:"units_to_review"`
	UnitsSkippedAccepted  int      `json:"units_skipped_accepted"`
	UnitsNeedingAttention int      `json:"units_needing_attention"`
	HasContextGap         bool     `json:"has_context_gap"`
	ContextGapReason      string   `json:"context_gap_reason"`
	Chapters              []string `json:"chapters"`
}

func (s *AppService) QueryDashboard() (DashboardProjection, error) {
	if s.Config == nil {
		return DashboardProjection{}, errors.New("chưa mở workspace")
	}
	c := *s.Config
	store := s.Store

	snapshot, _ := skills.Load(c)
	usage, _ := model.ReadUsage(store)

	p := DashboardProjection{
		SeriesID:    c.Series.ID,
		SeriesTitle: c.Series.Title,
		Root:        c.Root,
		Usage:       usage,
	}

	for _, v := range c.Volumes {
		g, err := store.Load(v.ID)
		if err != nil {
			continue
		}
		_, book, se := engine.Source(c, v, false)
		freshnessError := ""
		affected := map[string]string{}
		if se == nil {
			if _, _, err := engine.Source(c, v, true); err != nil {
				freshnessError = err.Error()
			} else if err = revision.Check(c, v, book, g); err != nil {
				freshnessError = err.Error()
			} else {
				impact, err := engine.ImpactPlan(c, v, store, book, g, snapshot)
				if err != nil {
					freshnessError = err.Error()
				} else {
					for _, a := range impact.Units {
						affected[a.ID] = a.Reason
					}
				}
			}
		}

		eligible, accepted := 0, 0
		var stale []string
		for _, b := range book.Blocks {
			if b.Disposition == "translate" {
				eligible++
			}
			if u, ok := g.Units[b.ID]; ok {
				if u.Accepted() && freshnessError == "" && affected[u.ID] == "" && b.Disposition == "translate" {
					accepted++
				}
				if u.StaleReason != "" || affected[u.ID] != "" || freshnessError != "" {
					stale = append(stale, u.ID)
				}
				if u.Review != nil {
					for _, issue := range u.Review.Issues {
						if issue.Severity == "blocking" {
							p.BlockingIssuesCount++
						}
					}
				}
			}
			_, conflicts, conflictErr := engine.ResolvedRules(c, v, store, g, snapshot, b.Position, b.Source)
			if conflictErr == nil {
				p.RuleConflictCount += len(conflicts)
			}
		}

		var decisions map[string]verify.ImageDecision
		_ = state.ReadJSON(store.Path("volumes", v.ID, "image-decisions.json"), &decisions)
		_, _, imageIssues := verify.ImageAssets(book, decisions, func(path string) ([]byte, error) {
			return os.ReadFile(c.ReadPath(path))
		})

		vs := VolumeSummary{
			ID:             v.ID,
			Order:          v.Order,
			BookTitle:      book.Title,
			SourcePath:     v.Source,
			PreviousID:     v.Previous,
			Accepted:       accepted,
			Eligible:       eligible,
			Translated:     len(g.Units),
			StaleCount:     len(stale),
			StaleIDs:       stale,
			FreshnessError: freshnessError,
			Imported:       se == nil,
			Images:         len(verify.Images(book)),
			ImageIssues:    len(imageIssues),
			ChaptersCount:  len(book.Chapters),
			BatchesCount:   len(engine.PlanBatches(book.Chapters)),
			GenerationID:   g.ID,
			LastSaveTime:   g.CreatedAt,
		}
		p.Volumes = append(p.Volumes, vs)
		p.Totals.TotalEligible += eligible
		p.Totals.TotalAccepted += accepted
		p.Totals.TotalStale += len(stale)
		p.Totals.TotalImages += vs.Images
		p.Totals.TotalImageIssues += vs.ImageIssues
		p.ImageIssuesCount += len(imageIssues)

		if g.CreatedAt != "" && g.CreatedAt > p.RecentRunOrSave {
			p.RecentRunOrSave = g.CreatedAt
		}
	}

	return p, nil
}

func (s *AppService) QueryChapters(volumeID string) ([]ChapterSummary, error) {
	if s.Config == nil {
		return nil, errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return nil, err
	}
	_, book, se := engine.Source(*s.Config, v, false)
	if se != nil {
		return nil, se
	}
	g, err := s.Store.Load(v.ID)
	if err != nil {
		return nil, err
	}

	snapshot, snapshotErr := skills.Load(*s.Config)
	freshnessError := ""
	if snapshotErr != nil {
		freshnessError = snapshotErr.Error()
	}
	impact, err := engine.ImpactPlan(*s.Config, v, s.Store, book, g, snapshot)
	affected := map[string]string{}
	if err == nil {
		for _, a := range impact.Units {
			affected[a.ID] = a.Reason
		}
	} else if freshnessError == "" {
		freshnessError = err.Error()
	}
	if _, _, err := engine.Source(*s.Config, v, true); err != nil {
		freshnessError = err.Error()
	} else if err := revision.Check(*s.Config, v, book, g); err != nil {
		freshnessError = err.Error()
	}

	var chapters []ChapterSummary
	for _, ch := range book.Chapters {
		cs := ChapterSummary{
			ID:    ch.ID,
			Title: ch.Title,
			Order: ch.Order,
			Kind:  ch.Kind,
		}
		hasUnreviewedTarget := false
		hasBlocking := false
		hasWarning := false
		hasManual := false
		hasStale := false

		for _, b := range book.Blocks {
			if b.Chapter != ch.ID {
				continue
			}
			cs.UnitsCount++
			if b.Disposition != "translate" {
				continue
			}
			cs.EligibleCount++
			u, ok := g.Units[b.ID]
			if !ok {
				continue
			}
			if u.Accepted() && affected[u.ID] == "" && u.StaleReason == "" && freshnessError == "" {
				cs.AcceptedCount++
			}
			if u.StaleReason != "" || affected[u.ID] != "" || freshnessError != "" {
				cs.StaleCount++
				hasStale = true
			}
			if u.Origin == "manual" {
				hasManual = true
			}
			if u.Review == nil {
				hasUnreviewedTarget = true
			} else {
				for _, iss := range u.Review.Issues {
					if iss.Severity == "blocking" {
						cs.BlockingCount++
						hasBlocking = true
					} else {
						cs.WarningCount++
						hasWarning = true
					}
				}
			}
		}

		if cs.EligibleCount == 0 {
			cs.Status = "Không có nội dung dịch"
		} else if cs.AcceptedCount == cs.EligibleCount {
			if hasManual {
				cs.Status = "Người dùng giữ"
			} else {
				cs.Status = "Đã đối chiếu"
			}
		} else if hasBlocking {
			cs.Status = "Cần xử lý"
		} else if hasStale {
			cs.Status = "Cần đối chiếu lại"
		} else if hasUnreviewedTarget {
			cs.Status = "Chờ review"
		} else if cs.AcceptedCount > 0 {
			cs.Status = "Đang dịch"
		} else {
			cs.Status = "Chưa dịch"
		}
		if hasWarning && cs.Status == "Đã đối chiếu" {
			cs.Status = "Có lưu ý"
		}

		chapters = append(chapters, cs)
	}
	return chapters, nil
}

func (s *AppService) QueryReviewQueue(volumeID string) ([]ReviewQueueItem, error) {
	if s.Config == nil {
		return nil, errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return nil, err
	}
	_, book, err := engine.Source(*s.Config, v, false)
	if err != nil {
		return nil, err
	}
	g, err := s.Store.Load(v.ID)
	if err != nil {
		return nil, err
	}
	freshness, freshnessError := s.reviewFreshness(v, book, g)

	chapterTitles := map[string]string{}
	for _, ch := range book.Chapters {
		chapterTitles[ch.ID] = ch.Title
	}

	var queue []ReviewQueueItem
	for _, b := range book.Blocks {
		if b.Disposition != "translate" {
			continue
		}
		u, ok := g.Units[b.ID]
		if !ok {
			continue
		}
		item := ReviewQueueItem{
			ChapterID:    b.Chapter,
			ChapterTitle: chapterTitles[b.Chapter],
			UnitID:       b.ID,
			Revision:     u.Revision,
			Source:       b.Source,
			Target:       u.Target,
			RubyCount:    len(b.Ruby),
			StaleReason:  u.StaleReason,
		}
		if item.StaleReason == "" {
			item.StaleReason = freshness[u.ID]
		}
		if item.StaleReason == "" {
			item.StaleReason = freshnessError
		}
		if u.Review != nil {
			item.Model = u.Review.Model
			item.Decision = u.Review.Decision
			item.DecisionReason = u.Review.DecisionReason
			item.Issues = append([]domain.Issue(nil), u.Review.Issues...)
			item.RubyDecisions = append([]domain.RubyDecision(nil), u.Review.RubyDecisions...)
			item.Signals = append([]domain.Issue(nil), u.Review.Signals...)
			item.Children = append([]domain.Child(nil), u.Children...)
			item.IssuesCount = len(u.Review.Issues)
			for _, iss := range u.Review.Issues {
				if iss.Severity == "blocking" {
					item.BlockingCount++
					if item.PrimaryIssue == nil {
						issCopy := iss
						item.PrimaryIssue = &issCopy
					}
				} else {
					item.WarningCount++
					if item.PrimaryIssue == nil {
						issCopy := iss
						item.PrimaryIssue = &issCopy
					}
				}
			}
		}
		switch {
		case item.StaleReason != "":
			item.Status = "Cần đối chiếu lại"
		case u.Review == nil && u.Target == "":
			item.Status = "Chưa dịch"
		case u.Review == nil:
			item.Status = "Chờ review"
		case u.Accepted() && u.Review.Decision == "user-kept":
			item.Status = "Người dùng giữ"
		case u.Accepted():
			item.Status = "Đã đối chiếu"
		case item.BlockingCount > 0:
			item.Status = "Cần xử lý"
		default:
			item.Status = "Chờ review"
		}
		queue = append(queue, item)
	}
	return queue, nil
}

// QueryUnit returns the source block and its current committed revision without
// changing workspace state. A committed review is current only while its source,
// skills, rules and context dependencies are still fresh.
func (s *AppService) QueryUnit(volumeID, unitID string) (UnitDetail, error) {
	if s.Config == nil {
		return UnitDetail{}, errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return UnitDetail{}, err
	}
	_, book, err := engine.Source(*s.Config, v, false)
	if err != nil {
		return UnitDetail{}, err
	}
	g, err := s.Store.Load(v.ID)
	if err != nil {
		return UnitDetail{}, err
	}
	var block *domain.Block
	for i := range book.Blocks {
		if book.Blocks[i].ID == unitID {
			block = &book.Blocks[i]
			break
		}
	}
	if block == nil {
		return UnitDetail{}, errors.New("không tìm thấy khối nguồn: " + unitID)
	}
	detail := UnitDetail{
		ID:         block.ID,
		ChapterID:  block.Chapter,
		Sequence:   block.Sequence,
		Source:     block.Source,
		SourceHash: block.SourceHash,
		Status:     "Chưa dịch",
	}
	u, ok := g.Units[unitID]
	if !ok {
		return detail, nil
	}
	detail.Target = u.Target
	detail.Revision = u.Revision
	detail.Origin = u.Origin
	detail.StaleReason = u.StaleReason
	if u.Review != nil {
		detail.Model = u.Review.Model
		detail.Decision = u.Review.Decision
		detail.DecisionReason = u.Review.DecisionReason
		detail.Issues = append([]domain.Issue(nil), u.Review.Issues...)
		detail.RubyDecisions = append([]domain.RubyDecision(nil), u.Review.RubyDecisions...)
		detail.Signals = append([]domain.Issue(nil), u.Review.Signals...)
	}
	detail.Children = append([]domain.Child(nil), u.Children...)
	if detail.StaleReason == "" {
		freshness, freshnessError := s.reviewFreshness(v, book, g)
		detail.StaleReason = freshness[unitID]
		if detail.StaleReason == "" {
			detail.StaleReason = freshnessError
		}
	}
	detail.Accepted = u.Accepted() && detail.StaleReason == ""
	switch {
	case detail.StaleReason != "":
		detail.Status = "Cần đối chiếu lại"
	case u.Review == nil && u.Target != "":
		detail.Status = "Chờ review"
	case u.Review == nil:
		detail.Status = "Chưa dịch"
	case detail.Accepted && u.Review.Decision == "user-kept":
		detail.Status = "Người dùng giữ"
	case detail.Accepted:
		detail.Status = "Đã đối chiếu"
	default:
		for _, issue := range u.Review.Issues {
			if issue.Severity == "blocking" {
				detail.Status = "Cần xử lý"
				return detail, nil
			}
		}
		detail.Status = "Chờ review"
	}
	return detail, nil
}

func (s *AppService) QueryUnitHistory(volumeID, unitID string) ([]UnitRevision, error) {
	if s.Config == nil {
		return nil, errors.New("chưa mở workspace")
	}
	if _, err := s.Config.Volume(volumeID); err != nil {
		return nil, err
	}
	g, err := s.Store.Load(volumeID)
	if err != nil {
		return nil, err
	}
	var history []UnitRevision
	for generationCount := 0; g.ID != "" && generationCount < 1000; generationCount++ {
		if unit, ok := g.Units[unitID]; ok {
			revision := UnitRevision{
				GenerationID: g.ID,
				CreatedAt:    g.CreatedAt,
				Revision:     unit.Revision,
				Target:       unit.Target,
				Origin:       unit.Origin,
			}
			if unit.Review != nil {
				review := *unit.Review
				revision.Review = &review
			}
			history = append(history, revision)
		}
		if g.Parent == "" {
			break
		}
		g, err = s.Store.History(volumeID, g.Parent)
		if err != nil {
			return history, err
		}
	}
	return history, nil
}

func (s *AppService) reviewFreshness(v workspace.Volume, book domain.Book, g domain.Generation) (map[string]string, string) {
	stale := map[string]string{}
	c := *s.Config
	if _, _, err := engine.Source(c, v, true); err != nil {
		return stale, err.Error()
	}
	if err := revision.Check(c, v, book, g); err != nil {
		return stale, err.Error()
	}
	snapshot, err := skills.Load(c)
	if err != nil {
		return stale, err.Error()
	}
	impact, err := engine.ImpactPlan(c, v, s.Store, book, g, snapshot)
	if err != nil {
		return stale, err.Error()
	}
	for _, unit := range impact.Units {
		stale[unit.ID] = unit.Reason
	}
	return stale, ""
}

func (s *AppService) QueryScopePlan(volumeID string, chapters []string) (ScopePlan, error) {
	if s.Config == nil {
		return ScopePlan{}, errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return ScopePlan{}, err
	}
	_, book, err := engine.Source(*s.Config, v, false)
	if err != nil {
		return ScopePlan{}, err
	}
	g, err := s.Store.Load(v.ID)
	if err != nil {
		return ScopePlan{}, err
	}

	selected := map[string]bool{}
	for _, ch := range chapters {
		selected[ch] = true
	}

	plan := ScopePlan{
		Chapters: chapters,
	}
	freshness, freshnessError := s.reviewFreshness(v, book, g)
	if len(selected) > 0 {
		firstSelected := -1
		for i, chapter := range book.Chapters {
			if selected[chapter.ID] {
				firstSelected = i
				break
			}
		}
		if firstSelected > 0 {
			for _, block := range book.Blocks {
				if block.Chapter != book.Chapters[firstSelected].ID {
					continue
				}
				events, contextErr := engine.ContextEvents(*s.Config, v, s.Store, g, block.Position, nil)
				if contextErr != nil {
					plan.HasContextGap = true
					plan.ContextGapReason = "Không đọc được ngữ cảnh đã lưu trước phạm vi: " + contextErr.Error()
				} else if len(events) == 0 {
					plan.HasContextGap = true
					plan.ContextGapReason = "Chưa có evidence ngữ cảnh đã lưu trước chương bắt đầu phạm vi."
				}
				break
			}
		}
	}

	for _, b := range book.Blocks {
		if b.Disposition != "translate" {
			continue
		}
		if len(selected) > 0 && !selected[b.Chapter] {
			continue
		}
		plan.TotalUnits++
		u, ok := g.Units[b.ID]
		if !ok {
			plan.UnitsToTranslate++
		} else if u.StaleReason != "" || freshness[u.ID] != "" || freshnessError != "" {
			plan.UnitsNeedingAttention++
		} else if u.Accepted() {
			plan.UnitsSkippedAccepted++
		} else if u.Review != nil {
			hasBlocking := false
			for _, iss := range u.Review.Issues {
				if iss.Severity == "blocking" {
					hasBlocking = true
					break
				}
			}
			if hasBlocking {
				plan.UnitsNeedingAttention++
			} else {
				plan.UnitsToReview++
			}
		} else {
			plan.UnitsToReview++
		}
	}
	return plan, nil
}

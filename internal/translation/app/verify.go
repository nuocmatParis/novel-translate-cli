package app

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/verify"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type VerifyGate struct {
	Name   string `json:"name"`
	Status string `json:"status"`
	Detail string `json:"detail,omitempty"`
}

type VerifyProjection struct {
	VolumeID          string              `json:"volume_id"`
	Imported          bool                `json:"imported"`
	SourceError       string              `json:"source_error,omitempty"`
	Report            domain.Report       `json:"report"`
	Gates             []VerifyGate        `json:"gates"`
	Images            []verify.Image      `json:"images"`
	Diagnostics       []domain.Diagnostic `json:"diagnostics"`
	LatestExport      *domain.Report      `json:"latest_export,omitempty"`
	LatestExportPath  string              `json:"latest_export_path,omitempty"`
	LatestExportTime  string              `json:"latest_export_time,omitempty"`
	LatestExportFresh bool                `json:"latest_export_fresh"`
}

type ExportResult struct {
	Path   string        `json:"path"`
	Report domain.Report `json:"report"`
}

func (s *AppService) QueryVerify(volumeID string) (VerifyProjection, error) {
	if s.Config == nil {
		return VerifyProjection{}, errors.New("chưa mở workspace")
	}
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return VerifyProjection{}, err
	}
	p := VerifyProjection{VolumeID: volumeID}
	_, book, err := engine.Source(*s.Config, v, false)
	if err != nil {
		p.SourceError = err.Error()
		p.Gates = []VerifyGate{
			{Name: "Bóc tách văn bản", Status: "Chưa chạy", Detail: "Import EPUB để đọc manifest và nội dung."},
			{Name: "Đầy đủ bản dịch", Status: "Chưa chạy"},
			{Name: "Freshness", Status: "Chưa chạy"},
			{Name: "Hình ảnh", Status: "Chưa chạy"},
			{Name: "Resources", Status: "Chưa chạy"},
			{Name: "Cấu trúc EPUB xuất", Status: "Chưa chạy"},
			{Name: "EPUBCheck", Status: "Chưa chạy"},
		}
		return p, nil
	}
	p.Imported = true
	g, err := s.Store.Load(v.ID)
	if err != nil {
		return p, err
	}
	p.Report, p.Diagnostics, p.Images = evaluateVolume(*s.Config, v, s.Store, book, g)
	appendGate := func(name, status, detail string) {
		p.Gates = append(p.Gates, VerifyGate{Name: name, Status: status, Detail: detail})
	}

	if p.Report.Extraction {
		appendGate("Bóc tách văn bản", "Đạt", fmt.Sprintf("%d/%d text node và %d/%d text attribute được mapping.", book.CoveredTextNodes, book.TextNodes, book.CoveredTextAttributes, book.TextAttributes))
	} else {
		appendGate("Bóc tách văn bản", "Có vấn đề", fmt.Sprintf("%d/%d text node và %d/%d text attribute được mapping.", book.CoveredTextNodes, book.TextNodes, book.CoveredTextAttributes, book.TextAttributes))
	}
	if p.Report.Eligible == 0 {
		appendGate("Đầy đủ bản dịch", "Không áp dụng", "Không có nội dung dịch trong snapshot này.")
	} else if p.Report.Accepted == p.Report.Eligible {
		appendGate("Đầy đủ bản dịch", "Đạt", fmt.Sprintf("%d/%d khối eligible đã được review hoặc người dùng giữ.", p.Report.Accepted, p.Report.Eligible))
	} else {
		appendGate("Đầy đủ bản dịch", "Có vấn đề", fmt.Sprintf("%d/%d khối eligible đã được review hoặc người dùng giữ.", p.Report.Accepted, p.Report.Eligible))
	}
	freshCount := 0
	for _, d := range p.Diagnostics {
		if d.Code == "stale-or-dependency" || d.Code == "source-changed" || d.Code == "revision-check" || d.Code == "skills-load" {
			freshCount++
		}
	}
	if freshCount == 0 {
		appendGate("Freshness", "Đạt", "Source, skills, rules và context hiện hành khớp generation.")
	} else {
		appendGate("Freshness", "Có vấn đề", fmt.Sprintf("Có %d khối hoặc dependency cần đối chiếu lại.", freshCount))
	}
	imageIssues := 0
	for _, d := range p.Diagnostics {
		if strings.HasPrefix(d.Code, "image-") {
			imageIssues++
		}
	}
	switch {
	case len(p.Images) == 0:
		appendGate("Hình ảnh", "Không áp dụng", "EPUB không có hình ảnh cần đánh giá.")
	case imageIssues == 0:
		appendGate("Hình ảnh", "Đạt", fmt.Sprintf("Đã có quyết định hợp lệ cho %d occurrence ảnh.", len(p.Images)))
	default:
		appendGate("Hình ảnh", "Có vấn đề", fmt.Sprintf("%d/%d occurrence ảnh cần đánh giá hoặc sửa file.", imageIssues, len(p.Images)))
	}
	resourceIssues := 0
	for _, d := range book.Diagnostics {
		if d.Blocking {
			resourceIssues++
		}
	}
	if resourceIssues == 0 {
		appendGate("Resources", "Đạt", "Không có diagnostic cấu trúc nguồn đang chặn.")
	} else {
		appendGate("Resources", "Có vấn đề", fmt.Sprintf("Có %d diagnostic nguồn đang chặn.", resourceIssues))
	}
	appendGate("Cấu trúc EPUB xuất", "Chưa chạy", "Chỉ được xác nhận sau khi build và đọc lại file xuất.")
	appendGate("EPUBCheck", "Chưa chạy", "Chỉ được xác nhận khi chạy EPUBCheck trên file xuất.")
	s.loadLatestExport(&p, v, g, book)
	return p, nil
}

func evaluateVolume(c workspace.Config, v workspace.Volume, store state.Store, book domain.Book, g domain.Generation) (domain.Report, []domain.Diagnostic, []verify.Image) {
	report := domain.Report{
		SchemaVersion: 1,
		SourceHash:    book.SourceHash,
		Revision:      g.ID,
		Extraction:    book.TextNodes == book.CoveredTextNodes && book.TextAttributes == book.CoveredTextAttributes,
		Semantic:      "chưa đối chiếu",
		Format:        "chưa chạy EPUBCheck",
	}
	diagnostics := append([]domain.Diagnostic(nil), book.Diagnostics...)
	snapshot, snapshotErr := skills.Load(c)
	if snapshotErr != nil {
		diagnostics = append(diagnostics, domain.Diagnostic{Code: "skills-load", Detail: snapshotErr.Error(), Blocking: true})
	}
	var impactByUnit map[string]string
	freshnessError := ""
	if _, _, err := engine.Source(c, v, true); err != nil {
		freshnessError = err.Error()
		diagnostics = append(diagnostics, domain.Diagnostic{Code: "source-changed", Detail: err.Error(), Blocking: true})
	} else if err := revision.Check(c, v, book, g); err != nil {
		freshnessError = err.Error()
		diagnostics = append(diagnostics, domain.Diagnostic{Code: "revision-check", Detail: err.Error(), Blocking: true})
	} else if snapshotErr == nil {
		impact, err := engine.ImpactPlan(c, v, store, book, g, snapshot)
		if err != nil {
			freshnessError = err.Error()
			diagnostics = append(diagnostics, domain.Diagnostic{Code: "revision-check", Detail: err.Error(), Blocking: true})
		} else {
			impactByUnit = make(map[string]string, len(impact.Units))
			for _, unit := range impact.Units {
				impactByUnit[unit.ID] = unit.Reason
			}
		}
	}
	if g.Units == nil {
		g.Units = map[string]domain.Unit{}
	}
	for _, block := range book.Blocks {
		if block.Disposition != "translate" {
			continue
		}
		report.Eligible++
		u, ok := g.Units[block.ID]
		if !ok || u.Target == "" || u.SourceHash != block.SourceHash {
			diagnostics = append(diagnostics, domain.Diagnostic{Code: "unreviewed-or-stale", Path: block.Document, Detail: block.ID + ": chưa có bản dịch hợp lệ cho source hiện tại", Blocking: true})
			continue
		}
		report.Translated++
		if snapshotErr != nil || freshnessError != "" || impactByUnit[block.ID] != "" || u.StaleReason != "" || !u.Accepted() {
			reason := u.StaleReason
			if reason == "" {
				reason = impactByUnit[block.ID]
			}
			if reason == "" {
				reason = freshnessError
			}
			if reason == "" {
				reason = "chưa có review accepted còn hiệu lực"
			}
			code := "unreviewed-or-stale"
			if reason != "chưa có review accepted còn hiệu lực" {
				code = "stale-or-dependency"
			}
			diagnostics = append(diagnostics, domain.Diagnostic{Code: code, Path: block.Document, Detail: block.ID + ": " + reason, Blocking: true})
			continue
		}
		contextEvents, contextErr := engine.ContextEvents(c, v, store, g, block.Position, nil)
		contextHash, contextDigestErr := engine.ContextDigest(c, v, contextEvents, block.Position)
		rules, conflicts, ruleErr := engine.ResolvedRules(c, v, store, g, snapshot, block.Position, block.Source)
		ruleHashes := map[string]string{}
		for _, rule := range rules {
			ruleHashes[rule.ID] = domain.Digest(rule)
		}
		if contextErr != nil || contextDigestErr != nil || ruleErr != nil || len(conflicts) > 0 || u.Dependencies.Skills != snapshot.Fingerprint(block.Position, block.Source) || u.Dependencies.Context != contextHash || domain.Digest(u.Dependencies.Rules) != domain.Digest(ruleHashes) {
			diagnostics = append(diagnostics, domain.Diagnostic{Code: "stale-or-dependency", Path: block.Document, Detail: block.ID + ": context hoặc rules cần đối chiếu lại", Blocking: true})
			continue
		}
		report.Accepted++
	}
	var imageDecisions map[string]verify.ImageDecision
	if err := state.ReadJSON(store.Path("volumes", v.ID, "image-decisions.json"), &imageDecisions); err != nil && !errors.Is(err, os.ErrNotExist) {
		diagnostics = append(diagnostics, domain.Diagnostic{Code: "image-decisions-read", Detail: err.Error(), Blocking: true})
	}
	_, _, imageIssues := verify.ImageAssets(book, imageDecisions, func(path string) ([]byte, error) {
		return os.ReadFile(c.ReadPath(path))
	})
	diagnostics = append(diagnostics, imageIssues...)
	report.Diagnostics = append([]domain.Diagnostic(nil), diagnostics...)
	if report.Eligible == report.Accepted && report.Eligible > 0 {
		report.Semantic = "đã đối chiếu; không bảo đảm đúng nghĩa 100%"
	}
	return report, diagnostics, verify.Images(book)
}

func (s *AppService) loadLatestExport(p *VerifyProjection, v workspace.Volume, g domain.Generation, book domain.Book) {
	if s.Config == nil {
		return
	}
	pattern := filepath.Join(s.Config.ReadPath(s.Config.Output.Directory), v.ID+"_VI*.epub.report.json")
	files, err := filepath.Glob(pattern)
	if err != nil {
		return
	}
	var chosen string
	var chosenTime time.Time
	for _, name := range files {
		info, err := os.Stat(name)
		if err == nil && info.ModTime().After(chosenTime) {
			chosen, chosenTime = name, info.ModTime()
		}
	}
	if chosen == "" {
		return
	}
	var report domain.Report
	if err := state.ReadJSON(chosen, &report); err != nil {
		return
	}
	p.LatestExport = &report
	p.LatestExportPath = strings.TrimSuffix(chosen, ".report.json")
	p.LatestExportTime = chosenTime.Local().Format(time.RFC3339)
	p.LatestExportFresh = report.SourceHash == book.SourceHash && report.Revision == g.ID
	for i := range p.Gates {
		switch p.Gates[i].Name {
		case "Cấu trúc EPUB xuất":
			if p.LatestExportFresh && report.OutputHash != "" {
				if report.Structure {
					p.Gates[i] = VerifyGate{Name: p.Gates[i].Name, Status: "Đạt", Detail: "File xuất gần nhất đã được đọc lại; hash khớp report."}
				} else {
					p.Gates[i] = VerifyGate{Name: p.Gates[i].Name, Status: "Có vấn đề", Detail: "Lần xuất gần nhất chưa qua kiểm tra cấu trúc."}
				}
			} else if !p.LatestExportFresh {
				p.Gates[i] = VerifyGate{Name: p.Gates[i].Name, Status: "Cần chạy lại", Detail: "Source hoặc generation đã đổi sau lần xuất gần nhất."}
			}
		case "EPUBCheck":
			if p.LatestExportFresh && report.Format != "chưa chạy EPUBCheck" {
				status := "Đạt"
				format := strings.ToLower(strings.TrimSpace(report.Format))
				if !strings.HasPrefix(format, "epubcheck passed") && !strings.HasPrefix(format, "epubcheck đạt") {
					status = "Có vấn đề"
				}
				p.Gates[i] = VerifyGate{Name: p.Gates[i].Name, Status: status, Detail: report.Format}
			} else if !p.LatestExportFresh {
				p.Gates[i] = VerifyGate{Name: p.Gates[i].Name, Status: "Cần chạy lại", Detail: "Kết quả EPUBCheck thuộc source/generation cũ."}
			}
		}
	}
}

func (s *AppService) ExportVolume(ctx context.Context, volumeID string, preview bool, onStage func(string)) (ExportResult, error) {
	result := ExportResult{}
	if s.Config == nil {
		return result, errors.New("chưa mở workspace")
	}
	unlock, err := s.Store.Lock(ctx)
	if err != nil {
		return result, err
	}
	defer unlock()
	c, err := workspace.Load(s.Config.Root)
	if err != nil {
		return result, err
	}
	v, err := c.Volume(volumeID)
	if err != nil {
		return result, err
	}
	stage := func(message string) {
		if onStage != nil {
			onStage(message)
		}
	}
	stage("Đọc và kiểm tra snapshot nguồn…")
	raw, book, err := engine.Source(c, v, true)
	if err != nil {
		return result, err
	}
	g, err := s.Store.Load(v.ID)
	if err != nil {
		return result, err
	}
	report, diagnostics, _ := evaluateVolume(c, v, s.Store, book, g)
	report.Diagnostics = diagnostics
	if !preview {
		if report.Eligible == 0 {
			return result, fmt.Errorf("%w: không có khối dịch eligible để xác nhận tập đã hoàn tất", engine.ErrUnresolved)
		}
		if report.Accepted != report.Eligible {
			return result, fmt.Errorf("%w: mới đối chiếu %d/%d khối", engine.ErrUnresolved, report.Accepted, report.Eligible)
		}
		for _, d := range diagnostics {
			if d.Blocking {
				return result, fmt.Errorf("%w: %s", engine.ErrUnresolved, d.Detail)
			}
		}
	}
	if err := ctx.Err(); err != nil {
		return result, err
	}
	stage("Chuẩn bị font và quyết định ảnh…")
	fonts := map[string][]byte{}
	if c.Output.FontDirectory != "" {
		for _, name := range []string{"LiberationSerif-Regular.ttf", "LiberationSerif-Bold.ttf", "LiberationSerif-Italic.ttf", "LiberationSerif-BoldItalic.ttf"} {
			data, err := os.ReadFile(filepath.Join(c.ReadPath(c.Output.FontDirectory), name))
			if err != nil {
				return result, err
			}
			fonts[name] = data
		}
	}
	var imageDecisions map[string]verify.ImageDecision
	if err := state.ReadJSON(s.Store.Path("volumes", v.ID, "image-decisions.json"), &imageDecisions); err != nil && !errors.Is(err, os.ErrNotExist) {
		return result, err
	}
	imageReplacements, imageTranscripts, imageIssues := verify.ImageAssets(book, imageDecisions, func(path string) ([]byte, error) {
		return os.ReadFile(c.ReadPath(path))
	})
	if len(imageIssues) > 0 && !preview {
		return result, fmt.Errorf("%w: kiểm tra lại quyết định/file ảnh", engine.ErrUnresolved)
	}
	opts := epub.ExportOptions{
		Preview:           preview,
		Fonts:             fonts,
		Title:             fmt.Sprintf("%s — Tập %d", c.Series.Title, v.Order),
		ImageReplacements: imageReplacements,
		ImageTranscripts:  imageTranscripts,
	}
	stage("Đang tạo EPUB…")
	data, err := epub.Write(raw, v.ID, v.Order, g, opts)
	if err != nil {
		return result, err
	}
	if err := ctx.Err(); err != nil {
		return result, err
	}
	stage("Đọc lại cấu trúc EPUB vừa tạo…")
	verified, verifyErr := epub.VerifyOutput(raw, data, v.ID, v.Order, g, opts)
	report.Structure = verified.Structure
	report.Diagnostics = append(report.Diagnostics, verified.Diagnostics...)
	report.Preview = preview
	report.OutputHash = domain.Hash(data)
	report.Repairs, err = epub.MetadataRepairs(raw, v.ID, v.Order, nil)
	if err != nil {
		return result, err
	}
	if verifyErr != nil && !preview {
		return result, verifyErr
	}
	tmp := s.Store.Path("exports", domain.Hash(data)+".epub")
	if err := state.Atomic(tmp, data); err != nil {
		return result, err
	}
	stage("Đang chạy EPUBCheck trên file xuất…")
	checker := c.Output.EPUBCheck
	if strings.ContainsAny(checker, "/\\") {
		checker = c.ReadPath(checker)
	}
	checkCtx, cancel := context.WithTimeout(ctx, 2*time.Minute)
	log, checkErr := verify.EPUBCheck(checkCtx, checker, tmp)
	cancel()
	if checkErr == nil {
		report.Format = "EPUBCheck passed"
	} else {
		report.Format = "EPUBCheck chưa pass: " + checkErr.Error()
	}
	if err := state.Atomic(tmp+".epubcheck.txt", []byte(log)); err != nil {
		return result, err
	}
	if err := state.WriteJSON(tmp+".report.json", report); err != nil {
		return result, err
	}
	if checkErr != nil && !preview {
		return ExportResult{Report: report}, fmt.Errorf("%w: %v", engine.ErrUnresolved, checkErr)
	}
	if err := ctx.Err(); err != nil {
		return result, err
	}
	name := v.ID + "_VI.epub"
	if preview {
		name = strings.TrimSuffix(name, ".epub") + ".preview.epub"
	}
	dest, err := c.WritePath(filepath.Join(c.Output.Directory, name))
	if err != nil {
		return result, err
	}
	if err := state.Atomic(dest, data); err != nil {
		return result, err
	}
	if err := state.WriteJSON(dest+".report.json", report); err != nil {
		return result, err
	}
	stage("Đã xuất " + dest)
	return ExportResult{Path: dest, Report: report}, nil
}

func (s *AppService) ImportVolume(ctx context.Context, volumeID string) (domain.Book, error) {
	if s.Config == nil {
		return domain.Book{}, errors.New("chưa mở workspace")
	}
	unlock, err := s.Store.Lock(ctx)
	if err != nil {
		return domain.Book{}, err
	}
	defer unlock()
	v, err := s.Config.Volume(volumeID)
	if err != nil {
		return domain.Book{}, err
	}
	if err := ctx.Err(); err != nil {
		return domain.Book{}, err
	}
	return engine.Import(*s.Config, v)
}

func (s *AppService) SaveImageAssessment(volumeID, imageID, disposition, reason, replacement, transcript string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()
	c, err := workspace.Load(s.Config.Root)
	if err != nil {
		return err
	}
	v, err := c.Volume(volumeID)
	if err != nil {
		return err
	}
	_, book, err := engine.Source(c, v, true)
	if err != nil {
		return err
	}
	var replacementBytes []byte
	if strings.TrimSpace(replacement) != "" {
		replacementBytes, err = os.ReadFile(c.ReadPath(replacement))
		if err != nil {
			return fmt.Errorf("không đọc được ảnh thay thế: %w", err)
		}
	}
	decision, err := verify.AssessImage(book, imageID, disposition, reason, replacement, transcript, replacementBytes)
	if err != nil {
		return err
	}
	path := s.Store.Path("volumes", v.ID, "image-decisions.json")
	var decisions map[string]verify.ImageDecision
	if err := state.ReadJSON(path, &decisions); err != nil && !errors.Is(err, os.ErrNotExist) {
		return err
	}
	if decisions == nil {
		decisions = map[string]verify.ImageDecision{}
	}
	decisions[imageID] = decision
	return state.WriteJSON(path, decisions)
}

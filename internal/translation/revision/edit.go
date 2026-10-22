package revision

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"unicode/utf8"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func Render(ch domain.Chapter, g domain.Generation) string {
	var out strings.Builder
	out.WriteString("# " + ch.Title + "\n\n")
	for _, id := range ch.BlockIDs {
		if u, ok := g.Units[id]; ok {
			out.WriteString("<!-- unit:" + id + " -->\n" + u.Target + "\n<!-- /unit -->\n\n")
		}
	}
	return out.String()
}
func ChapterPath(c workspace.Config, v workspace.Volume, id string) (string, error) {
	return c.WritePath(filepath.Join(v.Directory, "chapters", id+".md"))
}
func Check(c workspace.Config, v workspace.Volume, book domain.Book, g domain.Generation) error {
	for _, ch := range book.Chapters {
		p, e := ChapterPath(c, v, ch.ID)
		if e != nil {
			return e
		}
		b, e := os.ReadFile(p)
		if errors.Is(e, os.ErrNotExist) {
			for _, id := range ch.BlockIDs {
				if _, ok := g.Units[id]; ok {
					return fmt.Errorf("chapter view bị mất: %s; cần xử lý mapping", p)
				}
			}
			continue
		}
		if e != nil {
			return e
		}
		if string(b) != Render(ch, g) {
			return fmt.Errorf("bản sửa tay/sidecar khác generation tại %s; chạy sync trước", p)
		}
	}
	return nil
}

func CaptureViews(c workspace.Config, v workspace.Volume, book domain.Book, g *domain.Generation) error {
	g.ViewBases = map[string]string{}
	for _, ch := range book.Chapters {
		p, e := ChapterPath(c, v, ch.ID)
		if e != nil {
			return e
		}
		b, e := os.ReadFile(p)
		if errors.Is(e, os.ErrNotExist) {
			g.ViewBases[ch.ID] = ""
			continue
		}
		if e != nil {
			return e
		}
		g.ViewBases[ch.ID] = domain.Hash(b)
	}
	return nil
}

// Recover materializes only a known pre-commit view. Any independent edit or
// deletion remains a conflict, while a crash after generation commit is replayable.
func Recover(c workspace.Config, v workspace.Volume, book domain.Book, g domain.Generation) error {
	for _, ch := range book.Chapters {
		has := false
		for _, id := range ch.BlockIDs {
			if _, ok := g.Units[id]; ok {
				has = true
				break
			}
		}
		p, e := ChapterPath(c, v, ch.ID)
		if e != nil {
			return e
		}
		b, e := os.ReadFile(p)
		missing := errors.Is(e, os.ErrNotExist)
		if e != nil && !missing {
			return e
		}
		if missing && !has {
			continue
		}
		target := Render(ch, g)
		if !missing && string(b) == target {
			continue
		}
		base, known := g.ViewBases[ch.ID]
		if !known {
			return fmt.Errorf("view không khớp generation, không tự overwrite: %s", p)
		}
		if missing && base != "" || !missing && domain.Hash(b) != base {
			return fmt.Errorf("sửa tay/marker thay đổi: %s; chạy sync", p)
		}
		if e = state.Atomic(p, []byte(target)); e != nil {
			return e
		}
	}
	return nil
}
func Materialize(c workspace.Config, v workspace.Volume, book domain.Book, g domain.Generation) error {
	return Recover(c, v, book, g)
}

func parse(text string, ids []string) (map[string]string, error) {
	result := map[string]string{}
	cursor := 0
	order := []string{}
	for {
		start := strings.Index(text[cursor:], "<!-- unit:")
		if start < 0 {
			break
		}
		start += cursor
		headerEnd := strings.Index(text[start:], " -->\n")
		if headerEnd < 0 {
			return nil, errors.New("unit marker chưa đóng")
		}
		headerEnd += start
		id := text[start+len("<!-- unit:") : headerEnd]
		targetStart := headerEnd + len(" -->\n")
		end := strings.Index(text[targetStart:], "\n<!-- /unit -->")
		if end < 0 {
			return nil, errors.New("unit end marker bị mất")
		}
		end += targetStart
		if _, exists := result[id]; exists {
			return nil, errors.New("unit marker trùng")
		}
		target := text[targetStart:end]
		if strings.Contains(target, "<!-- unit:") || strings.TrimSpace(target) == "" {
			return nil, errors.New("unit trống/lồng marker")
		}
		result[id] = target
		order = append(order, id)
		cursor = end + len("\n<!-- /unit -->")
	}
	if len(order) != len(ids) {
		return nil, errors.New("thiếu/thừa unit markers; giữ file để xử lý mapping")
	}
	for i, id := range ids {
		if order[i] != id {
			return nil, errors.New("unit markers bị đổi thứ tự/ID")
		}
	}
	return result, nil
}
func Sync(c workspace.Config, v workspace.Volume, book domain.Book, g *domain.Generation) (int, error) {
	changes := map[string]string{}
	for _, ch := range book.Chapters {
		p, e := ChapterPath(c, v, ch.ID)
		if e != nil {
			return 0, e
		}
		b, e := os.ReadFile(p)
		if errors.Is(e, os.ErrNotExist) {
			continue
		}
		if e != nil {
			return 0, e
		}
		var ids []string
		for _, id := range ch.BlockIDs {
			if _, ok := g.Units[id]; ok {
				ids = append(ids, id)
			}
		}
		parsed, e := parse(string(b), ids)
		if e != nil {
			return 0, fmt.Errorf("%s: %w", p, e)
		}
		for id, target := range parsed {
			if g.Units[id].Target != target {
				changes[id] = target
			}
		}
	}
	return applyManualChanges(book, g, changes), nil
}

// EditUnit edits only target text; markers and source mapping stay out of the
// editor. Reuse the same invalidation behavior as syncing chapter Markdown.
func EditUnit(book domain.Book, g *domain.Generation, id, target string) (int, error) {
	u, ok := g.Units[id]
	if !ok {
		return 0, errors.New("khối chưa có bản dịch để sửa")
	}
	if strings.TrimSpace(target) == "" {
		return 0, errors.New("bản dịch không được trống")
	}
	if !utf8.ValidString(target) {
		return 0, errors.New("bản dịch phải là UTF-8")
	}
	if strings.Contains(target, "<!-- unit:") || strings.Contains(target, "<!-- /unit -->") {
		return 0, errors.New("không nhập unit markers vào phần bản dịch")
	}
	for _, r := range target {
		if r < 0x20 && r != '\n' && r != '\r' && r != '\t' || r == 0xfffe || r == 0xffff {
			return 0, errors.New("bản dịch chứa ký tự XML không hợp lệ")
		}
	}
	for _, b := range book.Blocks {
		if b.ID != id {
			continue
		}
		if u.SourceHash != b.SourceHash {
			return 0, errors.New("nguồn của khối đã thay đổi")
		}
		if b.Kind != "attribute" {
			if _, err := epub.TokensHTML(b, target); err != nil {
				return 0, err
			}
		}
		if u.Target == target {
			return 0, nil
		}
		return applyManualChanges(book, g, map[string]string{id: target}), nil
	}
	return 0, errors.New("không tìm thấy khối nguồn")
}

func applyManualChanges(book domain.Book, g *domain.Generation, changes map[string]string) int {
	if len(changes) == 0 {
		return 0
	}
	earliest := len(book.Blocks)
	for _, b := range book.Blocks {
		if target, ok := changes[b.ID]; ok {
			u := g.Units[b.ID]
			u.Target = target
			u.Revision++
			u.Origin = "manual"
			u.Review = nil
			u.Events = nil
			u.Rules = nil
			u.Children = nil
			u.RefreshDependencies = true
			u.StaleReason = ""
			g.Units[b.ID] = u
			if b.Sequence < earliest {
				earliest = b.Sequence
			}
		}
	}
	// A changed early scene can affect subsequent pair states. Preserve text and
	// invalidate downstream review/context rather than silently replacing it.
	for _, b := range book.Blocks {
		if b.Sequence > earliest {
			u, ok := g.Units[b.ID]
			if ok {
				if _, changed := changes[b.ID]; !changed {
					u.StaleReason = "context từ bản sửa tay trước đó thay đổi"
					g.Units[b.ID] = u
				}
			}
		}
	}
	return len(changes)
}

type Impact struct {
	SchemaVersion int        `json:"schema_version"`
	ID            string     `json:"id"`
	Volume        string     `json:"volume"`
	Generation    string     `json:"generation"`
	Source        string     `json:"source"`
	Skills        string     `json:"skills"`
	Units         []Affected `json:"units"`
	Inputs        string     `json:"inputs,omitempty"`
}
type Affected struct {
	ID     string `json:"id"`
	Action string `json:"action"`
	Reason string `json:"reason"`
}

func Plan(v workspace.Volume, book domain.Book, g domain.Generation, s skills.Snapshot) Impact {
	p := Impact{SchemaVersion: 1, Volume: v.ID, Generation: g.ID, Source: book.SourceHash, Skills: s.Digest}
	first := -1
	for _, b := range book.Blocks {
		u, ok := g.Units[b.ID]
		if !ok {
			continue
		}
		reason := u.StaleReason
		action := "review"
		if u.SourceHash != b.SourceHash || g.SourceHash != book.SourceHash {
			reason = "source mapping thay đổi"
			action = "translate"
		}
		if u.Dependencies.Skills != s.Fingerprint(b.Position, b.Source) {
			reason = "skill core/reference snapshot thay đổi; phạm vi bảo thủ"
			action = "translate"
		} else {
			rules, _ := s.ResolveFor(b.Position, b.Source)
			for _, r := range rules {
				if u.Dependencies.Rules[r.ID] != domain.Digest(r) {
					reason = "rule " + r.ID + " đổi"
					action = "translate"
					break
				}
			}
			for id := range u.Dependencies.Rules {
				exists := false
				for _, r := range rules {
					if r.ID == id {
						exists = true
						break
					}
				}
				if !exists {
					reason = "rule " + id + " bị bỏ/đổi scope"
					action = "translate"
				}
			}
		}
		if reason == "" && first >= 0 && b.Sequence > first {
			reason = "downstream continuity cần đối chiếu sau scene thay đổi"
		}
		if reason != "" {
			if first < 0 && (len(u.Events) > 0 || len(u.Rules) > 0) {
				first = b.Sequence
			}
			if u.Origin == "manual" {
				action = "review"
			}
			p.Units = append(p.Units, Affected{b.ID, action, reason})
		}
	}
	p.ID = domain.Digest(p)[:24]
	return p
}
func Apply(g *domain.Generation, p Impact) error {
	if g.ID != p.Generation {
		return errors.New("plan generation stale; lập plan mới")
	}
	copy := p
	id := copy.ID
	copy.ID = ""
	if domain.Digest(copy)[:24] != id {
		return errors.New("plan checksum không khớp")
	}
	for _, a := range p.Units {
		u, ok := g.Units[a.ID]
		if !ok {
			return errors.New("plan unit không tồn tại")
		}
		if a.Action == "translate" && u.Origin != "manual" {
			delete(g.Units, a.ID)
		} else {
			u.Review = nil
			u.Events = nil
			u.Rules = nil
			u.StaleReason = ""
			u.RefreshDependencies = true
			g.Units[a.ID] = u
		}
	}
	return nil
}

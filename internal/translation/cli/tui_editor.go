package cli

import (
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"strings"
	"unicode"
	"unicode/utf8"

	"github.com/charmbracelet/bubbles/textarea"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type tuiEditor struct {
	kind, stage          string
	config               workspace.Config
	volume               workspace.Volume
	book                 domain.Book
	generation           domain.Generation
	chapters             []domain.Chapter
	chapter, unit        int
	units                []domain.Block
	input                textarea.Model
	base, fileHash       string
	discard, sourceFocus bool
	sourceScroll         int
	sharedDirectory      string
	sharedSkills         []skills.Skill
}

type tuiEditorLoadedMsg struct{ editor *tuiEditor }
type tuiEditorSavedMsg struct {
	generation *domain.Generation
	text, hash string
}

func (m *translationTUI) openEditor(kind string) tea.Cmd {
	if kind == "shared" {
		return m.openSharedEditor()
	}
	c := *m.config
	var v workspace.Volume
	if kind == "translation" {
		v = c.Volumes[m.volume]
	}
	return m.startOperation("Mở trình sửa", c.Root, func(ctx context.Context, out, progress io.Writer) error {
		unlock, err := (state.Store{Root: c.Root}).Lock(ctx)
		if err != nil {
			return err
		}
		defer unlock()
		c, err = workspace.Load(c.Root)
		if err != nil {
			return err
		}
		if kind == "translation" {
			v, err = c.Volume(v.ID)
			if err != nil {
				return err
			}
		}
		e := &tuiEditor{kind: kind, config: c, volume: v, stage: "chapters"}
		if kind == "skill" {
			path, err := c.WritePath(c.SeriesSkill)
			if err != nil {
				return err
			}
			data, err := os.ReadFile(path)
			if err != nil {
				return err
			}
			e.stage, e.base, e.fileHash = "edit", string(data), domain.Hash(data)
			if e.base, err = prepareEditorText(e.base); err != nil {
				return err
			}
		} else {
			_, e.book, err = engine.Source(c, v, true)
			if err != nil {
				return err
			}
			e.generation, err = (state.Store{Root: c.Root}).Load(v.ID)
			if err != nil {
				return err
			}
			if err := revision.Check(c, v, e.book, e.generation); err != nil {
				return err
			}
			for _, ch := range e.book.Chapters {
				for _, id := range ch.BlockIDs {
					if _, ok := e.generation.Units[id]; ok {
						e.chapters = append(e.chapters, ch)
						break
					}
				}
			}
			if len(e.chapters) == 0 {
				return errors.New("chưa có bản dịch; nhập EPUB và dịch một phần trước khi sửa")
			}
		}
		select {
		case progress.(tuiProgress).events <- tuiEditorLoadedMsg{e}:
		case <-ctx.Done():
			return ctx.Err()
		}
		return nil
	})
}

// Normalize before handing text to textarea: its sanitizer maps CR and LF
// independently and would otherwise double Windows CRLF line breaks.
func prepareEditorText(text string) (string, error) {
	if !utf8.ValidString(text) {
		return "", errors.New("trình sửa cần nội dung UTF-8 hợp lệ; file được giữ nguyên")
	}
	text = strings.ReplaceAll(strings.ReplaceAll(text, "\r\n", "\n"), "\r", "\n")
	text = strings.ReplaceAll(text, "\t", "    ")
	if strings.Count(text, "\n") >= 9999 {
		return "", errors.New("nội dung quá dài để mở trong trình sửa (tối đa 9999 dòng); file được giữ nguyên")
	}
	for _, r := range text {
		if r == utf8.RuneError || unicode.IsControl(r) && r != '\n' {
			return "", errors.New("nội dung có ký tự trình sửa không hỗ trợ; file được giữ nguyên")
		}
	}
	return text, nil
}

func (e *tuiEditor) initInput(width, height int) tea.Cmd {
	e.input = textarea.New()
	e.input.CharLimit = 0
	e.input.MaxHeight = 0
	e.input.MaxWidth = 0
	e.input.ShowLineNumbers = true
	e.resize(width, height)
	e.input.SetValue(e.base)
	focus := e.input.Focus()
	// CursorStart only resets the column of the last line after SetValue.
	// Start at the beginning of the document, including the viewport.
	var move tea.Cmd
	e.input, move = e.input.Update(tea.KeyMsg{Type: tea.KeyCtrlHome})
	return tea.Batch(focus, move)
}

func (e *tuiEditor) resize(width, height int) {
	e.input.SetWidth(max(10, width-2))
	reserved := 6
	if e.kind == "translation" {
		reserved += max(3, (height-6)/3) + 2
	}
	e.input.SetHeight(max(3, height-reserved))
}

func (e *tuiEditor) selectChapter() {
	e.units = nil
	for _, block := range e.book.Blocks {
		if block.Chapter == e.chapters[e.chapter].ID {
			if _, ok := e.generation.Units[block.ID]; ok {
				e.units = append(e.units, block)
			}
		}
	}
	e.unit, e.stage = 0, "units"
}

func (m *translationTUI) updateEditor(msg tea.Msg) tea.Cmd {
	e := m.editor
	if size, ok := msg.(tea.WindowSizeMsg); ok {
		e.resize(size.Width, size.Height)
		return nil
	}
	key, ok := msg.(tea.KeyMsg)
	if !ok {
		if e.stage == "edit" {
			var cmd tea.Cmd
			e.input, cmd = e.input.Update(msg)
			return cmd
		}
		return nil
	}
	k := key.String()
	if e.discard {
		switch k {
		case "y":
			e.discard = false
			m.leaveEditor()
		case "n", "esc":
			e.discard = false
		}
		return nil
	}
	if k == "esc" || k == "ctrl+c" {
		if e.stage == "edit" && e.input.Value() != e.base {
			e.discard = true
		} else {
			m.leaveEditor()
		}
		return nil
	}
	if e.stage != "edit" {
		index, count := &e.chapter, len(e.chapters)
		if e.stage == "skills" {
			count = len(e.sharedSkills)
		}
		if e.stage == "units" {
			index, count = &e.unit, len(e.units)
		}
		switch k {
		case "up", "k":
			*index = (*index + count - 1) % count
		case "down", "j":
			*index = (*index + 1) % count
		case "pgup":
			*index = max(0, *index-10)
		case "pgdown":
			*index = min(count-1, *index+10)
		case "enter":
			if e.stage == "skills" {
				return m.selectSharedSkill()
			}
			if e.stage == "chapters" {
				e.selectChapter()
			} else {
				text, err := prepareEditorText(e.generation.Units[e.units[e.unit].ID].Target)
				if err != nil {
					m.notice = err.Error()
					return nil
				}
				e.stage, e.base = "edit", text
				e.sourceScroll, e.sourceFocus = 0, false
				return e.initInput(m.width, m.height)
			}
		}
		return nil
	}
	if k == "ctrl+s" {
		return m.saveEditor()
	}
	if k == "tab" && e.kind == "translation" {
		e.sourceFocus = !e.sourceFocus
		if e.sourceFocus {
			e.input.Blur()
		} else {
			return e.input.Focus()
		}
		return nil
	}
	if e.sourceFocus {
		switch k {
		case "down", "j":
			e.sourceScroll++
		case "up", "k":
			e.sourceScroll = max(0, e.sourceScroll-1)
		case "pgdown":
			e.sourceScroll += 5
		case "pgup":
			e.sourceScroll = max(0, e.sourceScroll-5)
		}
		return nil
	}
	var cmd tea.Cmd
	e.input, cmd = e.input.Update(msg)
	return cmd
}

func (m *translationTUI) leaveEditor() {
	e := m.editor
	if e.kind == "shared" {
		if e.stage == "edit" {
			e.stage = "skills"
			e.input.Blur()
		} else {
			m.editor = nil
		}
		return
	}
	switch {
	case e.kind == "skill" || e.stage == "chapters":
		m.editor = nil
	case e.stage == "units":
		e.stage = "chapters"
	default:
		e.stage = "units"
		e.sourceFocus = false
		e.input.Blur()
	}
}

func (m *translationTUI) saveEditor() tea.Cmd {
	e := m.editor
	text, base, kind := e.input.Value(), e.base, e.kind
	if text == base {
		m.notice = "Không có thay đổi để lưu."
		return nil
	}
	if kind == "shared" {
		return m.saveSharedEditor()
	}
	c, v, book, generation, fileHash := e.config, e.volume, e.book, e.generation.ID, e.fileHash
	id := ""
	if kind == "translation" {
		id = e.units[e.unit].ID
	}
	return m.startOperation("Lưu bản sửa", c.Root, func(ctx context.Context, out, progress io.Writer) error {
		store := state.Store{Root: c.Root}
		unlock, err := store.Lock(ctx)
		if err != nil {
			return err
		}
		defer unlock()
		currentConfig, err := workspace.Load(c.Root)
		if err != nil {
			return err
		}
		if domain.Digest(currentConfig) != domain.Digest(c) {
			return errors.New("cấu hình workspace đã thay đổi; nội dung đang nhập vẫn được giữ, hãy mở lại để đối chiếu")
		}
		result := tuiEditorSavedMsg{text: text}
		if kind == "skill" {
			path, err := c.WritePath(c.SeriesSkill)
			if err != nil {
				return err
			}
			current, err := os.ReadFile(path)
			if err != nil {
				return err
			}
			if domain.Hash(current) != fileHash {
				return errors.New("quy ước đã được sửa ở nơi khác; giữ nội dung đang nhập và mở lại để đối chiếu")
			}
			if _, err := skills.Parse(path, []byte(text)); err != nil {
				return fmt.Errorf("quy ước chưa hợp lệ: %w", err)
			}
			if err := state.Atomic(path, []byte(text)); err != nil {
				return err
			}
			result.hash = domain.Hash([]byte(text))
		} else {
			_, currentBook, err := engine.Source(c, v, true)
			if err != nil {
				return err
			}
			if currentBook.SourceHash != book.SourceHash {
				return errors.New("nguồn đã thay đổi; cần mở lại bản dịch")
			}
			g, err := store.Load(v.ID)
			if err != nil {
				return err
			}
			if g.ID != generation {
				return errors.New("bản dịch đã được cập nhật ở nơi khác; nội dung đang nhập vẫn được giữ để đối chiếu")
			}
			if err := revision.Check(c, v, currentBook, g); err != nil {
				return err
			}
			if _, err := revision.EditUnit(currentBook, &g, id, text); err != nil {
				return err
			}
			if err := revision.CaptureViews(c, v, currentBook, &g); err != nil {
				return err
			}
			if err := store.Commit(v.ID, &g, g.ID); err != nil {
				return err
			}
			if err := revision.Materialize(c, v, currentBook, g); err != nil {
				return err
			}
			result.generation = &g
		}
		select {
		case progress.(tuiProgress).events <- result:
		case <-ctx.Done():
			return ctx.Err()
		}
		fmt.Fprintln(out, "Đã lưu bản sửa; bản dịch hoặc ngữ cảnh bị ảnh hưởng cần review lại.")
		return nil
	})
}

func (e *tuiEditor) view(width, height int, notice string, busy bool) string {
	w := max(1, width-2)
	clip := func(s string) string { return ansi.Truncate(s, w, "…") }
	lines := []string{"SỬA BẢN DỊCH / QUY ƯỚC"}
	if e.stage != "edit" {
		index, count := e.chapter, len(e.chapters)
		if e.stage == "skills" {
			count = len(e.sharedSkills)
			lines = append(lines, "Skill chung cho các bộ truyện · "+e.sharedDirectory)
		} else if e.stage == "units" {
			index, count = e.unit, len(e.units)
			lines = append(lines, "Chọn đoạn · "+e.chapters[e.chapter].Title)
		} else {
			lines = append(lines, "Chọn chương để đọc/sửa")
		}
		visible := max(1, height-6)
		start := max(0, min(index-visible/2, count-visible))
		for i := start; i < min(count, start+visible); i++ {
			label := ""
			if e.stage == "skills" {
				label = e.sharedSkills[i].ID
			} else if e.stage == "chapters" {
				label = e.chapters[i].Title
			} else {
				label = e.generation.Units[e.units[i].ID].Target
			}
			prefix := "  "
			if i == index {
				prefix = "› "
			}
			lines = append(lines, clip(prefix+cleanTUIText(label)))
		}
		lines = append(lines, fmt.Sprintf("%d/%d · ↑/↓ chọn · Enter mở · Esc quay lại", index+1, count))
	} else {
		if e.kind == "translation" {
			lines = append(lines, "Nguồn Nhật · Tab chuyển vùng · ↑/↓ hoặc PgUp/PgDn cuộn")
			source := strings.Split(ansi.Hardwrap(e.units[e.unit].Source, w, true), "\n")
			n := max(3, (height-6)/3)
			start := min(e.sourceScroll, max(0, len(source)-n))
			for _, line := range source[start:min(len(source), start+n)] {
				lines = append(lines, clip(cleanTUIText(line)))
			}
			lines = append(lines, "Bản dịch Việt · giữ nguyên inline tokens ⟦…⟧")
		} else if e.kind == "shared" {
			lines = append(lines, "Skill chung · "+e.sharedSkills[e.chapter].ID+" · thay đổi ảnh hưởng nhiều bộ truyện")
		} else {
			lines = append(lines, "Quy ước riêng của bộ truyện · Markdown/YAML frontmatter")
		}
		lines = append(lines, e.input.View())
		lines = append(lines, "Ctrl+S lưu · Esc quay lại · chưa tự chấp nhận bản sửa")
		if e.discard {
			lines = append(lines, "Bỏ thay đổi chưa lưu? y: bỏ · n/Esc: tiếp tục sửa")
		}
	}
	if busy {
		lines = append(lines, "Đang xử lý… Ctrl+C dừng")
	} else if notice != "" {
		lines = append(lines, clip(cleanTUIText(notice)))
	}
	// Textarea renders multiple lines inside a single item. Clip each visual
	// line, including headings and controls, to fit narrow terminals.
	visual := strings.Split(strings.Join(lines, "\n"), "\n")
	for i := range visual {
		visual[i] = clip(visual[i])
	}
	return strings.Join(visual, "\n")
}

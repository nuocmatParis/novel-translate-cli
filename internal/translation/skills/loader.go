package skills

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
	"gopkg.in/yaml.v3"
)

type Skill struct {
	ID   string `json:"id"`
	Path string `json:"path"`
	// IdentityPath preserves historical dependency hashes when a shared file
	// replaces its old workspace copy. Path remains the actual source location.
	IdentityPath string           `json:"identity_path,omitempty"`
	Hash         string           `json:"hash"`
	Body         string           `json:"body"`
	References   []string         `json:"references,omitempty"`
	From         domain.Position  `json:"from"`
	Until        *domain.Position `json:"until,omitempty"`
	Priority     int              `json:"priority"`
}
type Snapshot struct {
	Digest     string        `json:"digest"`
	CoreDigest string        `json:"core_digest"`
	Skills     []Skill       `json:"skills"`
	Rules      []domain.Rule `json:"rules"`
	Conflicts  []string      `json:"conflicts"`
}

func (s Snapshot) CoreHash() string {
	if s.CoreDigest != "" {
		return s.CoreDigest
	}
	return s.Digest
}

// Fingerprint includes only instructions and matching reference paragraphs that
// are allowed at this publication position. Future library edits stay unrelated.
func (s Snapshot) Fingerprint(p domain.Position, source string) string {
	if len(s.Skills) == 0 {
		return s.CoreHash()
	}
	var relevant []Skill
	for _, sk := range s.Skills {
		if !Applies(sk.From, sk.Until, p) {
			continue
		}
		if strings.Contains(sk.ID, ":") {
			var selected []string
			for _, para := range referenceSections(sk.Body) {
				if referenceMatches(para, source) {
					selected = append(selected, para)
				}
			}
			if len(selected) == 0 {
				continue
			}
			sk.Body = strings.Join(selected, "\n\n")
			sk.Hash = domain.Hash([]byte(sk.Body))
		}
		if sk.IdentityPath != "" {
			sk.Path = sk.IdentityPath
			sk.IdentityPath = ""
		}
		relevant = append(relevant, sk)
	}
	return domain.Digest(relevant)
}
func referenceMatches(para, source string) bool {
	for _, term := range strings.Fields(para) {
		term = strings.Trim(term, "|`*()[]，。、")
		if (len([]rune(term)) >= 2 || isSingleKanji(term)) && strings.Contains(source, term) {
			return true
		}
	}
	return false
}

type metadata struct {
	ID         string           `yaml:"id"`
	References []string         `yaml:"references"`
	From       domain.Position  `yaml:"from"`
	Until      *domain.Position `yaml:"until"`
	Priority   int              `yaml:"priority"`
}

func Parse(path string, b []byte) (Skill, error) {
	s := Skill{Path: path, Hash: domain.Hash(b)}
	body := strings.TrimPrefix(strings.ReplaceAll(string(b), "\r\n", "\n"), "\uFEFF")
	if strings.HasPrefix(body, "---\n") {
		parts := strings.SplitN(body[4:], "\n---", 2)
		if len(parts) != 2 {
			return s, errors.New("skill frontmatter chưa đóng")
		}
		var m metadata
		d := yaml.NewDecoder(bytes.NewBufferString(parts[0]))
		d.KnownFields(true)
		if e := d.Decode(&m); e != nil {
			return s, e
		}
		s.ID = m.ID
		s.References = m.References
		s.From = m.From
		s.Until = m.Until
		s.Priority = m.Priority
		if s.From.Volume < 0 || s.From.Chapter < 0 || s.From.Section < 0 || s.From.Block < 0 || (s.Until != nil && !s.From.Before(*s.Until)) {
			return s, errors.New("skill scope không hợp lệ")
		}
		body = strings.TrimPrefix(parts[1], "\n")
	}
	if s.ID == "" {
		s.ID = strings.TrimSuffix(filepath.Base(path), filepath.Ext(path))
	}
	s.Body = body
	return s, nil
}
func InstallDefaults(c workspace.Config) error {
	if _, err := EnsureShared(); err != nil {
		return err
	}
	p, e := c.WritePath(c.SeriesSkill)
	if e != nil {
		return e
	}
	if _, e = os.Stat(p); errors.Is(e, os.ErrNotExist) {
		return state.Atomic(p, []byte("---\nid: "+c.Series.ID+"\npriority: 100\n---\nQuy ước bộ truyện: ưu tiên nguồn Nhật và quy tắc đã xác nhận đúng phạm vi.\n"))
	}
	return nil
}
func Load(c workspace.Config) (Snapshot, error) {
	s := Snapshot{}
	libraries := c.SkillLibraries
	shared := len(libraries) == 0
	if shared {
		directory, err := EnsureShared()
		if err != nil {
			return s, err
		}
		libraries = []string{directory}
	}
	for _, id := range c.EnabledSkills {
		var found []string
		for _, lib := range libraries {
			paths, err := skillFiles(c.ReadPath(lib), id)
			if err != nil {
				return s, err
			}
			found = append(found, paths...)
		}
		if len(found) == 0 {
			return s, fmt.Errorf("thiếu skill %s", id)
		}
		if len(found) > 1 {
			return s, fmt.Errorf("skill %s có nhiều định nghĩa; chọn một library", id)
		}
		b, e := os.ReadFile(found[0])
		if e != nil {
			return s, e
		}
		sk, e := Parse(found[0], b)
		if e != nil {
			return s, e
		}
		if sk.ID != id {
			return s, fmt.Errorf("skill ID không khớp: %s", id)
		}
		if shared {
			sk.IdentityPath = c.ReadPath(filepath.Join("Skills", "core", id, "SKILL.md"))
		}
		// Series instructions always override shared defaults.
		if sk.Priority >= 100 {
			sk.Priority = 99
		}
		s.Skills = append(s.Skills, sk)
	}
	b, e := os.ReadFile(c.ReadPath(c.SeriesSkill))
	if e != nil {
		return s, e
	}
	series, e := Parse(c.ReadPath(c.SeriesSkill), b)
	if e != nil {
		return s, e
	}
	series.Priority = 100
	s.Skills = append(s.Skills, series)
	if e = state.ReadJSON(c.ReadPath("Skills/rules.json"), &s.Rules); e != nil && !errors.Is(e, os.ErrNotExist) {
		return s, e
	}
	var imported []Reference
	if e = state.ReadJSON(c.ReadPath("Skills/imported/index.json"), &imported); e != nil && !errors.Is(e, os.ErrNotExist) {
		return s, e
	}
	for _, r := range imported {
		if r.Status != "scoped-reference" {
			continue
		}
		b, e := os.ReadFile(r.Path)
		if e != nil {
			return s, e
		}
		if domain.Hash(b) != r.Hash {
			return s, fmt.Errorf("imported reference checksum sai: %s", r.Path)
		}
		s.Skills = append(s.Skills, Skill{ID: "import:" + r.ID, Path: fmt.Sprintf("%s#paragraph-%d", r.Path, r.Paragraph), Hash: domain.Hash([]byte(r.Body)), Body: r.Body, From: r.From, Until: r.Until, Priority: 50})
	}
	for _, sk := range s.Skills {
		for _, r := range sk.References {
			p := filepath.Join(filepath.Dir(sk.Path), filepath.FromSlash(r))
			b, e := os.ReadFile(p)
			if e != nil {
				return s, fmt.Errorf("skill reference %s: %w", r, e)
			}
			identity := ""
			if sk.IdentityPath != "" {
				identity = filepath.Join(filepath.Dir(sk.IdentityPath), filepath.FromSlash(r))
			}
			s.Skills = append(s.Skills, Skill{ID: sk.ID + ":" + r, Path: p, IdentityPath: identity, Hash: domain.Hash(b), Body: string(b), From: sk.From, Until: sk.Until, Priority: sk.Priority})
		}
	}
	core := s
	core.Rules = nil
	core.Digest = ""
	core.CoreDigest = ""
	s.CoreDigest = domain.Digest(core)
	copy := s
	copy.Digest = ""
	s.Digest = domain.Digest(copy)
	return s, nil
}
func Applies(from domain.Position, until *domain.Position, p domain.Position) bool {
	return !p.Before(from) && (until == nil || p.Before(*until))
}
func (s Snapshot) Resolve(p domain.Position) ([]domain.Rule, []string) {
	byKey := map[string]domain.Rule{}
	conflicts := []string{}
	for _, r := range s.Rules {
		if !Applies(r.From, r.Until, p) {
			continue
		}
		if r.Status == "conflict" {
			conflicts = append(conflicts, r.Key)
			continue
		}
		if r.Status != "confirmed" {
			continue
		}
		old, ok := byKey[r.Key]
		if !ok || old.Priority < r.Priority {
			byKey[r.Key] = r
		} else if old.Priority == r.Priority && old.Value != r.Value {
			conflicts = append(conflicts, r.Key)
		}
	}
	var rules []domain.Rule
	for _, r := range byKey {
		rules = append(rules, r)
	}
	sort.Slice(rules, func(i, j int) bool { return rules[i].ID < rules[j].ID })
	sort.Strings(conflicts)
	return rules, conflicts
}

// Keyed lexical rules only affect a block that contains the key or its source
// evidence. Scene-wide conventions remain applicable without an explicit name.
func (s Snapshot) ResolveFor(p domain.Position, source string) ([]domain.Rule, []string) {
	copy := s
	copy.Rules = nil
	for _, r := range s.Rules {
		lexical := r.Kind == "glossary" || r.Kind == "name" || r.Kind == "term"
		relevant := !lexical || strings.Contains(source, r.Key)
		for _, ev := range r.Evidence {
			if ev.BlockID != "" && ev.Quote != "" && strings.Contains(source, ev.Quote) {
				relevant = true
			}
		}
		if relevant {
			copy.Rules = append(copy.Rules, r)
		}
	}
	return copy.Resolve(p)
}

// Context bounds reference retrieval. UTF-8 byte length is an upper token bound,
// not a fixed character/token estimate; model-specific tokenization may do better.
func (s Snapshot) Context(p domain.Position, source string, limit int) (string, []string, error) {
	var out strings.Builder
	var used []string
	for _, sk := range s.Skills {
		if !Applies(sk.From, sk.Until, p) {
			continue
		}
		if strings.Contains(sk.ID, ":") {
			for i, para := range referenceSections(sk.Body) {
				if referenceMatches(para, source) && out.Len()+len(para)+len("\n<reference-data></reference-data>\n") <= limit {
					out.WriteString("\n<reference-data>" + para + "</reference-data>\n")
					used = append(used, fmt.Sprintf("%s#paragraph-%d", sk.Path, i+1))
				}
			}
			continue
		}
		if out.Len()+len(sk.Body) > limit {
			return "", nil, fmt.Errorf("skill core vượt budget: %s", sk.ID)
		}
		out.WriteString("\n" + sk.Body + "\n")
		used = append(used, sk.Path)
	}
	return out.String(), used, nil
}
func SaveSnapshot(c workspace.Config, s Snapshot) error {
	return state.WriteJSON(state.Store{Root: c.Root}.Path("skill_snapshots", s.Digest, "snapshot.json"), s)
}

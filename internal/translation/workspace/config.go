package workspace

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/state"
	"gopkg.in/yaml.v3"
)

type Series struct {
	ID    string `yaml:"id" json:"id"`
	Title string `yaml:"title" json:"title"`
}
type Volume struct {
	ID        string `yaml:"id" json:"id"`
	Order     int    `yaml:"order" json:"order"`
	Previous  string `yaml:"previous,omitempty" json:"previous,omitempty"`
	Directory string `yaml:"directory" json:"directory"`
	Source    string `yaml:"source" json:"source"`
}
type Run struct {
	ChaptersPerBatch       int  `yaml:"chapters_per_batch"`
	AutoContinue           bool `yaml:"auto_continue"`
	MaxContractCorrections int  `yaml:"max_contract_corrections"`
	MaxRepairRounds        int  `yaml:"max_repair_rounds"`
	MaxTransportRetries    int  `yaml:"max_transport_retries"`
	RequestTimeoutSeconds  int  `yaml:"request_timeout_seconds"`
	MaxInputBytes          int  `yaml:"max_input_bytes"`
	MaxOutputTokens        int  `yaml:"max_output_tokens"`
	MaxRequests            int  `yaml:"max_requests,omitempty"`
}
type Output struct {
	Directory     string `yaml:"directory"`
	Template      string `yaml:"template"`
	EPUBCheck     string `yaml:"epubcheck,omitempty"`
	FontDirectory string `yaml:"font_directory,omitempty"`
}
type ResearchSource struct {
	ID      string `yaml:"id" json:"id"`
	API     string `yaml:"api" json:"api"`
	BaseURL string `yaml:"base_url" json:"base_url"`
	Edition string `yaml:"edition" json:"edition"`
	Kind    string `yaml:"kind" json:"kind"`
}
type Config struct {
	SchemaVersion   int              `yaml:"schema_version"`
	Series          Series           `yaml:"series"`
	SourceLanguage  string           `yaml:"source_language"`
	TargetLanguage  string           `yaml:"target_language"`
	ProviderConfig  string           `yaml:"provider_config,omitempty"`
	SkillLibraries  []string         `yaml:"skill_libraries,omitempty"`
	EnabledSkills   []string         `yaml:"enabled_skills"`
	SeriesSkill     string           `yaml:"series_skill"`
	Volumes         []Volume         `yaml:"volumes"`
	Run             Run              `yaml:"run"`
	Output          Output           `yaml:"output"`
	ResearchSources []ResearchSource `yaml:"research_sources,omitempty"`
	Root            string           `yaml:"-"`
}

var idPattern = regexp.MustCompile(`^[a-zA-Z0-9][a-zA-Z0-9_-]{0,79}$`)

func ValidID(id string) bool {
	if !idPattern.MatchString(id) {
		return false
	}
	name := strings.ToUpper(id)
	if name == "CON" || name == "PRN" || name == "AUX" || name == "NUL" {
		return false
	}
	if len(name) == 4 && (strings.HasPrefix(name, "COM") || strings.HasPrefix(name, "LPT")) && name[3] >= '1' && name[3] <= '9' {
		return false
	}
	return true
}
func Defaults() Config {
	return Config{SchemaVersion: 1, SourceLanguage: "ja", TargetLanguage: "vi", SeriesSkill: "Skills/SKILL.md", EnabledSkills: []string{"jp-vn-translation", "dialogue-and-honorifics", "translation-review"}, Run: Run{ChaptersPerBatch: 2, AutoContinue: true, MaxContractCorrections: 3, MaxRepairRounds: 3, MaxTransportRetries: 3, RequestTimeoutSeconds: 180, MaxInputBytes: 16000, MaxOutputTokens: 12000}, Output: Output{Directory: "Output", Template: "sample-vi"}, Volumes: []Volume{}}
}
func ConfigPath(p string) (string, error) {
	a, e := filepath.Abs(p)
	if e != nil {
		return "", e
	}
	if strings.EqualFold(filepath.Ext(a), ".yaml") || strings.EqualFold(filepath.Ext(a), ".yml") {
		return a, nil
	}
	return filepath.Join(a, "workspace.yaml"), nil
}
func Load(p string) (Config, error) {
	c := Defaults()
	name, e := ConfigPath(p)
	if e != nil {
		return c, e
	}
	b, e := os.ReadFile(name)
	if e != nil {
		return c, e
	}
	d := yaml.NewDecoder(bytes.NewReader(b))
	d.KnownFields(true)
	if e = d.Decode(&c); e != nil {
		return c, fmt.Errorf("workspace YAML: %w", e)
	}
	var extra any
	if e = d.Decode(&extra); e != io.EOF {
		return c, errors.New("workspace chỉ được có một YAML document")
	}
	c.Root = filepath.Dir(name)
	return c, c.Validate()
}
func (c Config) ReadPath(p string) string {
	if filepath.IsAbs(p) {
		return filepath.Clean(p)
	}
	return filepath.Join(c.Root, filepath.FromSlash(p))
}

// WritePath confines durable artifacts to this workspace, including symlinks.
func (c Config) WritePath(p string) (string, error) {
	if p == "" || filepath.IsAbs(p) {
		return "", errors.New("đường dẫn ghi phải tương đối với workspace")
	}
	a := filepath.Clean(c.ReadPath(p))
	rel, e := filepath.Rel(c.Root, a)
	if e != nil || rel == ".." || strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
		return "", errors.New("đường dẫn ghi thoát workspace")
	}
	for _, part := range strings.Split(rel, string(filepath.Separator)) {
		if part == ".git" || part == ".codex" || part == ".agents" {
			return "", errors.New("không ghi vào thư mục hệ thống")
		}
	}
	check := a
	for {
		info, err := os.Lstat(check)
		if err == nil {
			if info.Mode()&os.ModeSymlink != 0 {
				resolved, err := filepath.EvalSymlinks(check)
				if err != nil {
					return "", err
				}
				r, err := filepath.Rel(c.Root, resolved)
				if err != nil || r == ".." || strings.HasPrefix(r, ".."+string(filepath.Separator)) {
					return "", errors.New("symlink ghi thoát workspace")
				}
			}
		}
		if check == c.Root {
			break
		}
		next := filepath.Dir(check)
		if next == check {
			break
		}
		check = next
	}
	return a, nil
}
func (c Config) Validate() error {
	if _, e := c.WritePath(".translation_work"); e != nil {
		return e
	}
	if c.SchemaVersion != 1 {
		return errors.New("schema_version không được hỗ trợ")
	}
	if !ValidID(c.Series.ID) {
		return errors.New("series.id không hợp lệ")
	}
	seenSkills := map[string]bool{}
	if len(c.EnabledSkills) == 0 {
		return errors.New("cần ít nhất một skill chung trong enabled_skills")
	}
	for _, id := range c.EnabledSkills {
		if !ValidID(id) || seenSkills[id] {
			return errors.New("enabled_skills chứa ID không hợp lệ hoặc trùng")
		}
		seenSkills[id] = true
	}
	if c.SourceLanguage != "ja" || c.TargetLanguage != "vi" {
		return errors.New("CLI chỉ hỗ trợ ja → vi")
	}
	if c.Run.ChaptersPerBatch != 2 || !c.Run.AutoContinue {
		return errors.New("run phải dùng hai chương mỗi đợt và auto_continue=true")
	}
	if c.Run.MaxContractCorrections < 0 || c.Run.MaxContractCorrections > 10 || c.Run.MaxRepairRounds < 0 || c.Run.MaxRepairRounds > 10 || c.Run.MaxTransportRetries < 0 || c.Run.MaxTransportRetries > 10 || c.Run.RequestTimeoutSeconds < 1 || c.Run.MaxInputBytes < 1024 || c.Run.MaxOutputTokens < 256 || c.Run.MaxRequests < 0 {
		return errors.New("giới hạn run không hợp lệ")
	}
	if c.Output.Template != "sample-vi" {
		return errors.New("output.template chưa được hỗ trợ")
	}
	if _, e := c.WritePath(c.Output.Directory); e != nil {
		return e
	}
	if _, e := c.WritePath(c.SeriesSkill); e != nil {
		return e
	}
	ids := map[string]Volume{}
	orders := map[int]bool{}
	dirs := map[string]bool{}
	for _, v := range c.Volumes {
		if !ValidID(v.ID) || v.Order < 1 || ids[v.ID].ID != "" || orders[v.Order] {
			return errors.New("volume ID/order trùng hoặc không hợp lệ")
		}
		d, e := c.WritePath(v.Directory)
		if e != nil {
			return e
		}
		if dirs[strings.ToLower(d)] {
			return errors.New("hai volume dùng chung directory")
		}
		dirs[strings.ToLower(d)] = true
		if v.Source == "" {
			return errors.New("volume thiếu source")
		}
		ids[v.ID] = v
		orders[v.Order] = true
	}
	for _, v := range c.Volumes {
		if v.Previous != "" {
			p, ok := ids[v.Previous]
			if !ok || p.Order >= v.Order {
				return fmt.Errorf("previous của %s không hợp lệ", v.ID)
			}
		}
	}
	return nil
}
func (c Config) Volume(id string) (Volume, error) {
	for _, v := range c.Volumes {
		if v.ID == id {
			return v, nil
		}
	}
	return Volume{}, fmt.Errorf("tập %q chưa đăng ký", id)
}
func (c Config) Save() error {
	if e := c.Validate(); e != nil {
		return e
	}
	b, e := yaml.Marshal(c)
	if e != nil {
		return e
	}
	return state.Atomic(filepath.Join(c.Root, "workspace.yaml"), b)
}
func Create(path, id, title string) (Config, error) {
	c := Defaults()
	p, e := ConfigPath(path)
	if e != nil {
		return c, e
	}
	c.Root = filepath.Dir(p)
	c.Series = Series{id, title}
	if title == "" {
		c.Series.Title = id
	}
	if _, e = os.Stat(p); !errors.Is(e, os.ErrNotExist) {
		return c, errors.New("workspace đã tồn tại hoặc không truy cập được")
	}
	if e = c.Save(); e != nil {
		return c, e
	}
	return c, nil
}
func (c *Config) Add(v Volume) error {
	c.Volumes = append(c.Volumes, v)
	if e := c.Validate(); e != nil {
		c.Volumes = c.Volumes[:len(c.Volumes)-1]
		return e
	}
	sort.Slice(c.Volumes, func(i, j int) bool { return c.Volumes[i].Order < c.Volumes[j].Order })
	return c.Save()
}

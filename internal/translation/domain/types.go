// Package domain defines persisted translation data without runtime dependencies.
package domain

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
)

const SchemaVersion = 1

func Hash(b []byte) string { h := sha256.Sum256(b); return hex.EncodeToString(h[:]) }
func Digest(v any) string  { b, _ := json.Marshal(v); return Hash(b) }

type Position struct {
	Volume  int `json:"volume" yaml:"volume"`
	Chapter int `json:"chapter" yaml:"chapter"`
	Section int `json:"section" yaml:"section"`
	Block   int `json:"block" yaml:"block"`
}

func (p Position) Before(q Position) bool {
	a, b := [4]int{p.Volume, p.Chapter, p.Section, p.Block}, [4]int{q.Volume, q.Chapter, q.Section, q.Block}
	for i := range a {
		if a[i] != b[i] {
			return a[i] < b[i]
		}
	}
	return false
}

type Diagnostic struct {
	Code     string `json:"code"`
	Path     string `json:"path,omitempty"`
	Detail   string `json:"detail"`
	Blocking bool   `json:"blocking"`
}
type Resource struct {
	ID         string `json:"id"`
	Path       string `json:"path"`
	MediaType  string `json:"media_type"`
	Hash       string `json:"hash"`
	Properties string `json:"properties,omitempty"`
}
type Document struct {
	Path       string   `json:"path"`
	Spine      int      `json:"spine"`
	Linear     bool     `json:"linear"`
	Properties string   `json:"properties,omitempty"`
	BlockIDs   []string `json:"block_ids"`
}
type Chapter struct {
	ID       string   `json:"id"`
	Order    int      `json:"order"`
	Title    string   `json:"title"`
	Kind     string   `json:"kind"`
	BlockIDs []string `json:"block_ids"`
}
type Token struct {
	ID         string            `json:"id"`
	Kind       string            `json:"kind"`
	Name       string            `json:"name"`
	Attributes map[string]string `json:"attributes,omitempty"`
	Pair       string            `json:"pair,omitempty"`
	Locator    string            `json:"locator,omitempty"`
}
type Ruby struct {
	Base    string `json:"base"`
	Reading string `json:"reading"`
	Policy  string `json:"policy"`
	Start   int    `json:"start"`
	End     int    `json:"end"`
}
type RubyDecision struct {
	Index       int    `json:"index"`
	Base        string `json:"base"`
	Reading     string `json:"reading"`
	Decision    string `json:"decision"`
	TargetQuote string `json:"target_quote"`
	Reason      string `json:"reason"`
}
type Block struct {
	ID          string   `json:"id"`
	Document    string   `json:"document"`
	Locator     string   `json:"locator"`
	Sequence    int      `json:"sequence"`
	Chapter     string   `json:"chapter"`
	Kind        string   `json:"kind"`
	Tag         string   `json:"tag"`
	Anchor      string   `json:"anchor,omitempty"`
	Source      string   `json:"source"`
	SourceHash  string   `json:"source_hash"`
	Tokens      []Token  `json:"tokens,omitempty"`
	Ruby        []Ruby   `json:"ruby,omitempty"`
	Resource    string   `json:"resource,omitempty"`
	Disposition string   `json:"disposition"`
	Reason      string   `json:"reason,omitempty"`
	Position    Position `json:"position"`
}
type Book struct {
	SchemaVersion         int          `json:"schema_version"`
	ParserVersion         string       `json:"parser_version"`
	SourceHash            string       `json:"source_hash"`
	PackagePath           string       `json:"package_path"`
	Title                 string       `json:"title"`
	Language              string       `json:"language"`
	Creator               string       `json:"creator"`
	Resources             []Resource   `json:"resources"`
	Documents             []Document   `json:"documents"`
	Chapters              []Chapter    `json:"chapters"`
	Blocks                []Block      `json:"blocks"`
	Diagnostics           []Diagnostic `json:"diagnostics"`
	ZIPEntries            int          `json:"zip_entries"`
	TextNodes             int          `json:"text_nodes"`
	CoveredTextNodes      int          `json:"covered_text_nodes"`
	TextAttributes        int          `json:"text_attributes"`
	CoveredTextAttributes int          `json:"covered_text_attributes"`
}
type Evidence struct {
	BlockID string `json:"block_id"`
	Quote   string `json:"quote"`
	URL     string `json:"url,omitempty"`
	Hash    string `json:"hash,omitempty"`
}
type Rule struct {
	ID             string     `json:"id"`
	Key            string     `json:"key"`
	Value          string     `json:"value"`
	Kind           string     `json:"kind"`
	From           Position   `json:"from"`
	Until          *Position  `json:"until,omitempty"`
	Priority       int        `json:"priority"`
	Status         string     `json:"status"`
	Evidence       []Evidence `json:"evidence"`
	Version        int        `json:"version"`
	DecisionReason string     `json:"decision_reason,omitempty"`
}
type Event struct {
	ID           string    `json:"id"`
	Speaker      string    `json:"speaker"`
	Listener     string    `json:"listener"`
	Value        string    `json:"value"`
	Position     Position  `json:"position"`
	StoryTime    *Position `json:"story_time,omitempty"`
	Uncertainty  string    `json:"uncertainty,omitempty"`
	Evidence     Evidence  `json:"evidence"`
	UnitID       string    `json:"unit_id"`
	RevisionHash string    `json:"revision_hash"`
	SourceOffset int       `json:"source_offset,omitempty"`
}

type Entity struct {
	ID         string     `json:"id"`
	Aliases    []string   `json:"aliases"`
	Evidence   []Evidence `json:"evidence"`
	RevealedAt Position   `json:"revealed_at"`
	RuleID     string     `json:"rule_id"`
}
type Issue struct {
	ID          string `json:"id"`
	Type        string `json:"type"`
	Severity    string `json:"severity"`
	BlockID     string `json:"block_id"`
	SourceQuote string `json:"source_quote"`
	TargetQuote string `json:"target_quote"`
	Explanation string `json:"explanation"`
	Resolution  string `json:"resolution,omitempty"`
}
type Review struct {
	TargetHash     string         `json:"target_hash"`
	SourceHash     string         `json:"source_hash"`
	Issues         []Issue        `json:"issues"`
	Model          string         `json:"model"`
	Decision       string         `json:"decision"`
	DecisionReason string         `json:"decision_reason,omitempty"`
	RubyDecisions  []RubyDecision `json:"ruby_decisions,omitempty"`
	Signals        []Issue        `json:"signals,omitempty"`
}
type Dependencies struct {
	Source  string            `json:"source"`
	Skills  string            `json:"skills"`
	Rules   map[string]string `json:"rules"`
	Context string            `json:"context"`
	Model   string            `json:"model"`
	Prompt  string            `json:"prompt"`
}
type Unit struct {
	ID                  string       `json:"id"`
	Target              string       `json:"target"`
	SourceHash          string       `json:"source_hash"`
	Revision            int          `json:"revision"`
	Origin              string       `json:"origin"`
	Dependencies        Dependencies `json:"dependencies"`
	Review              *Review      `json:"review,omitempty"`
	Events              []Event      `json:"events,omitempty"`
	Rules               []Rule       `json:"rules,omitempty"`
	StaleReason         string       `json:"stale_reason,omitempty"`
	Children            []Child      `json:"children,omitempty"`
	RefreshDependencies bool         `json:"refresh_dependencies,omitempty"`
}

type Child struct {
	ID     string          `json:"id"`
	Start  int             `json:"start"`
	End    int             `json:"end"`
	Source string          `json:"source"`
	Target string          `json:"target"`
	Review *Review         `json:"review,omitempty"`
	Result json.RawMessage `json:"result,omitempty"`
}

func (u Unit) Accepted() bool {
	return u.StaleReason == "" && u.Review != nil && u.Review.SourceHash == u.SourceHash && u.Review.TargetHash == Hash([]byte(u.Target)) && (u.Review.Decision == "accepted" || u.Review.Decision == "user-kept")
}

type Generation struct {
	SchemaVersion      int               `json:"schema_version"`
	ID                 string            `json:"id"`
	Parent             string            `json:"parent"`
	SourceHash         string            `json:"source_hash"`
	Units              map[string]Unit   `json:"units"`
	CreatedAt          string            `json:"created_at"`
	ViewBases          map[string]string `json:"view_bases,omitempty"`
	SourceManifestHash string            `json:"source_manifest_hash,omitempty"`
	SourceManifestFile string            `json:"source_manifest_file,omitempty"`
}
type Batch struct {
	Number   int      `json:"number"`
	Chapters []string `json:"chapters"`
}
type Report struct {
	SchemaVersion  int          `json:"schema_version"`
	SourceHash     string       `json:"source_hash"`
	Revision       string       `json:"revision"`
	Scope          []string     `json:"scope"`
	RequestedScope []string     `json:"requested_scope,omitempty"`
	Eligible       int          `json:"eligible"`
	Translated     int          `json:"translated"`
	Accepted       int          `json:"accepted"`
	Extraction     bool         `json:"extraction_checked"`
	Structure      bool         `json:"structure_checked"`
	Semantic       string       `json:"semantic"`
	Format         string       `json:"format"`
	Diagnostics    []Diagnostic `json:"diagnostics"`
	OutputHash     string       `json:"output_hash,omitempty"`
	Preview        bool         `json:"preview"`
	Repairs        []Diagnostic `json:"metadata_repairs,omitempty"`
}

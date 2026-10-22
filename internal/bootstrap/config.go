// Package bootstrap builds provider models for translation runtime.
package bootstrap

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"
	"unicode"

	"github.com/voocel/agentcore/llm"
	"github.com/voocel/ainovel-cli/internal/errs"
	"github.com/voocel/ainovel-cli/internal/models"
)

const DefaultContextWindow = 32768

type ProviderConfig struct {
	Type              string         `json:"type,omitempty"`
	API               string         `json:"api,omitempty"`
	APIKey            string         `json:"api_key,omitempty"`
	APIKeyEnv         string         `json:"api_key_env,omitempty"`
	AllowNoAPIKey     bool           `json:"allow_no_api_key,omitempty"`
	BaseURL           string         `json:"base_url,omitempty"`
	Models            []ModelConfig  `json:"models,omitempty"`
	ExtraBody         map[string]any `json:"extra_body,omitempty"`
	Extra             map[string]any `json:"extra,omitempty"`
	StreamIdleTimeout string         `json:"stream_idle_timeout,omitempty"`
}
type ModelConfig struct {
	Name          string `json:"name"`
	ContextWindow int    `json:"context_window,omitempty"`
	JSONSchema    *bool  `json:"json_schema,omitempty"`
}

func (m *ModelConfig) UnmarshalJSON(b []byte) error {
	var name string
	if json.Unmarshal(b, &name) == nil {
		*m = ModelConfig{Name: name}
		return nil
	}
	type plain ModelConfig
	var p plain
	if e := json.Unmarshal(b, &p); e != nil {
		return e
	}
	*m = ModelConfig(p)
	return nil
}
func (p ProviderConfig) ModelConfig(name string) (ModelConfig, bool) {
	for _, m := range p.Models {
		if m.Name == name {
			return m, true
		}
	}
	return ModelConfig{}, false
}
func (p ProviderConfig) ProviderType(name string) (string, error) {
	if p.Type != "" {
		return p.Type, nil
	}
	if llm.IsProviderRegistered(name) {
		return name, nil
	}
	return "", fmt.Errorf("provider %s cần type: %w", name, errs.ErrConfig)
}
func (p ProviderConfig) RequiresAPIKey(name string) bool {
	protocol, e := p.ProviderType(name)
	return e == nil && !p.AllowNoAPIKey && protocol != "ollama" && protocol != "bedrock"
}
func (p ProviderConfig) StreamIdleTimeoutValue() (time.Duration, error) {
	if p.StreamIdleTimeout == "" {
		return 5 * time.Minute, nil
	}
	d, e := time.ParseDuration(p.StreamIdleTimeout)
	if e != nil {
		return 0, e
	}
	if d <= 0 {
		return 0, fmt.Errorf("stream_idle_timeout phải dương")
	}
	return d, nil
}

type ModelRef struct {
	Provider string `json:"provider"`
	Model    string `json:"model"`
}
type RoleConfig struct {
	Provider        string     `json:"provider"`
	Model           string     `json:"model"`
	Fallbacks       []ModelRef `json:"fallbacks,omitempty"`
	ReasoningEffort string     `json:"reasoning_effort,omitempty"`
}
type BudgetConfig struct {
	BookUSD float64 `json:"book_usd,omitempty"`
}

func (b BudgetConfig) Enabled() bool { return b.BookUSD > 0 }

type Config struct {
	Provider        string                    `json:"provider"`
	ModelName       string                    `json:"model"`
	Providers       map[string]ProviderConfig `json:"providers"`
	Roles           map[string]RoleConfig     `json:"roles,omitempty"`
	ContextWindow   int                       `json:"context_window,omitempty"`
	ReasoningEffort string                    `json:"reasoning_effort,omitempty"`
	Budget          BudgetConfig              `json:"budget,omitempty"`
}

var knownRoles = map[string]bool{"translation": true, "translation_review": true, "translation_research": true}

func controls(s string) bool {
	for _, r := range s {
		if unicode.IsControl(r) {
			return true
		}
	}
	return false
}
func (c *Config) ValidateBase() error {
	if c.Provider == "" || c.ModelName == "" || controls(c.Provider) || controls(c.ModelName) {
		return fmt.Errorf("cần provider/model hợp lệ: %w", errs.ErrConfig)
	}
	if c.ContextWindow < 0 || c.Budget.BookUSD < 0 {
		return errs.ErrConfig
	}
	for name, p := range c.Providers {
		if name == "" || controls(name) || controls(p.Type) || controls(p.API) || controls(p.APIKey) || controls(p.BaseURL) {
			return errs.ErrConfig
		}
		switch p.API {
		case "", "chat", "responses":
		default:
			return fmt.Errorf("API phải chat hoặc responses: %w", errs.ErrConfig)
		}
		protocol, e := p.ProviderType(name)
		if e != nil {
			return e
		}
		if p.API != "" && protocol != "openai" {
			return fmt.Errorf("api chỉ dùng cho OpenAI protocol: %w", errs.ErrConfig)
		}
		if _, e = p.StreamIdleTimeoutValue(); e != nil {
			return e
		}
		seen := map[string]bool{}
		for _, m := range p.Models {
			if m.Name == "" || controls(m.Name) || seen[m.Name] || m.ContextWindow < 0 {
				return errs.ErrConfig
			}
			seen[m.Name] = true
		}
	}
	check := func(ref ModelRef) error {
		p, ok := c.Providers[ref.Provider]
		if !ok || ref.Model == "" || controls(ref.Model) {
			return fmt.Errorf("model ref không hợp lệ: %w", errs.ErrConfig)
		}
		if p.RequiresAPIKey(ref.Provider) && p.APIKey == "" {
			return fmt.Errorf("provider %s thiếu api_key: %w", ref.Provider, errs.ErrConfig)
		}
		return nil
	}
	if e := check(ModelRef{c.Provider, c.ModelName}); e != nil {
		return e
	}
	for role, r := range c.Roles {
		if !knownRoles[role] {
			return fmt.Errorf("role %s không hỗ trợ; dùng translation/translation_review/translation_research: %w", role, errs.ErrConfig)
		}
		if e := check(ModelRef{r.Provider, r.Model}); e != nil {
			return e
		}
		for _, fallback := range r.Fallbacks {
			if e := check(fallback); e != nil {
				return e
			}
		}
	}
	return nil
}
func (c *Config) DefaultProviderConfig() ProviderConfig { return c.Providers[c.Provider] }
func (c Config) ModelJSONSchema(provider, name string) *bool {
	if m, ok := c.Providers[provider].ModelConfig(name); ok {
		return m.JSONSchema
	}
	return nil
}

type ContextWindowSource string

const (
	CtxWindowModelConfig ContextWindowSource = "model_config"
	CtxWindowConfig      ContextWindowSource = "config"
	CtxWindowRegistry    ContextWindowSource = "registry"
	CtxWindowDefault     ContextWindowSource = "default"
)

func (c Config) ResolveContextWindow(provider, name string) (int, ContextWindowSource) {
	if m, ok := c.Providers[provider].ModelConfig(name); ok && m.ContextWindow > 0 {
		return m.ContextWindow, CtxWindowModelConfig
	}
	if c.ContextWindow > 0 {
		return c.ContextWindow, CtxWindowConfig
	}
	if n := models.DefaultRegistry().ResolveContextWindow(name); n > 0 {
		return n, CtxWindowRegistry
	}
	return DefaultContextWindow, CtxWindowDefault
}
func (c Config) ResolveReasoningEffort(role string) string {
	if r, ok := c.Roles[role]; ok && r.ReasoningEffort != "" {
		return r.ReasoningEffort
	}
	return c.ReasoningEffort
}
func (c Config) CandidateModels(provider string) []string {
	var names []string
	seen := map[string]bool{}
	add := func(s string) {
		s = strings.TrimSpace(s)
		if s != "" && !seen[s] {
			names = append(names, s)
			seen[s] = true
		}
	}
	for _, m := range c.Providers[provider].Models {
		add(m.Name)
	}
	if c.Provider == provider {
		add(c.ModelName)
	}
	for _, r := range c.Roles {
		if r.Provider == provider {
			add(r.Model)
		}
		for _, f := range r.Fallbacks {
			if f.Provider == provider {
				add(f.Model)
			}
		}
	}
	return names
}

package model

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/voocel/agentcore"
	"github.com/voocel/agentcore/llm"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type Caller interface {
	Call(context.Context, string, string, any, llmcontract.Contract, any, func() error) error
	Identity(string) string
	InputLimit(string) int
}
type Adapter struct {
	Models      *bootstrap.ModelSet
	Config      bootstrap.Config
	Run         workspace.Run
	Store       state.Store
	mu          sync.Mutex
	Requests    int
	Cost        float64
	UnknownCost bool
	LastModels  map[string]string
}
type Usage struct {
	ID          string           `json:"id"`
	Role        string           `json:"role"`
	RequestHash string           `json:"request_hash"`
	Time        string           `json:"time"`
	Model       string           `json:"model"`
	Usage       *agentcore.Usage `json:"usage,omitempty"`
	Failed      bool             `json:"failed"`
	CostStatus  string           `json:"cost_status"`
	Status      string           `json:"status"`
}

func New(c workspace.Config) (*Adapter, error) {
	p := c.ProviderConfig
	if p == "" {
		p = bootstrap.DefaultConfigPath()
	} else {
		p = c.ReadPath(p)
	}
	cfg, e := bootstrap.LoadConfigFile(p, c.Root)
	if e != nil {
		return nil, fmt.Errorf("cấu hình provider %s: %w", p, e)
	}
	if e = cfg.ValidateBase(); e != nil {
		return nil, e
	}
	ms, e := bootstrap.NewModelSet(cfg)
	if e != nil {
		return nil, e
	}
	return &Adapter{Models: ms, Config: cfg, Run: c.Run, Store: state.Store{Root: c.Root}}, nil
}
func (a *Adapter) Identity(role string) string {
	a.mu.Lock()
	defer a.mu.Unlock()
	if actual := a.LastModels[role]; actual != "" {
		return actual
	}
	p, m, _ := a.Models.CurrentSelection(role)
	return p + "/" + m
}
func (a *Adapter) InputLimit(role string) int {
	p, m, _ := a.Models.CurrentSelection(role)
	window, source := a.Models.ResolveContextWindow(p, m)
	if source == bootstrap.CtxWindowDefault {
		window = 32768
	}
	n := window - a.Run.MaxOutputTokens - 2048
	if n > a.Run.MaxInputBytes {
		n = a.Run.MaxInputBytes
	}
	return n
}

type metered struct {
	a           *Adapter
	role        string
	requestHash string
	model       agentcore.ChatModel
	identity    string
	inputLimit  int
}

func (m *metered) SupportsTools() bool { return m.model.SupportsTools() }
func (m *metered) GenerateStream(context.Context, []agentcore.Message, []agentcore.ToolSpec, ...agentcore.CallOption) (<-chan agentcore.StreamEvent, error) {
	return nil, errors.New("translation uses metered non-streaming calls")
}

func (m *metered) StructuredOutputFacts() llmcontract.ModelFacts {
	if f, ok := m.model.(interface{ StructuredOutputFacts() llmcontract.ModelFacts }); ok {
		return f.StructuredOutputFacts()
	}
	facts := llmcontract.ModelFacts{JSONSchemaOverride: m.JSONSchemaOverride()}
	if c, ok := m.model.(llm.CapabilityProvider); ok {
		facts.Capabilities = c.Capabilities()
	}
	if c, ok := m.model.(interface{ Info() llm.ModelInfo }); ok {
		facts.Info = c.Info()
	}
	return facts
}

func (m *metered) JSONSchemaOverride() *bool {
	if v, ok := m.model.(interface{ JSONSchemaOverride() *bool }); ok {
		return v.JSONSchemaOverride()
	}
	return nil
}
func (m *metered) Generate(ctx context.Context, messages []agentcore.Message, tools []agentcore.ToolSpec, opts ...agentcore.CallOption) (*agentcore.LLMResponse, error) {
	bytes := 0
	for _, message := range messages {
		bytes += len(message.TextContent())
	}
	limit := m.inputLimit
	if limit == 0 {
		limit = m.a.InputLimit(m.role)
	}
	if bytes > limit {
		return nil, errors.New("correction messages vượt context budget")
	}
	m.a.mu.Lock()
	if m.a.Run.MaxRequests > 0 && m.a.Requests >= m.a.Run.MaxRequests {
		m.a.mu.Unlock()
		return nil, errors.New("đã chạm max_requests")
	}
	if m.a.Config.Budget.Enabled() && (m.a.UnknownCost || m.a.Cost >= m.a.Config.Budget.BookUSD) {
		m.a.mu.Unlock()
		return nil, errors.New("ngân sách đã chạm hoặc chi phí request trước không xác định")
	}
	m.a.Requests++
	event := Usage{Role: m.role, RequestHash: m.requestHash, Time: time.Now().UTC().Format(time.RFC3339Nano), Model: m.identity, Failed: true, CostStatus: "unknown", Status: "in-flight"}
	event.ID = domain.Digest(event)[:24]
	if e := state.WriteJSON(m.a.Store.Path("usage", event.ID+".json"), event); e != nil {
		m.a.mu.Unlock()
		return nil, fmt.Errorf("không lưu request journal: %w", e)
	}
	m.a.mu.Unlock()
	r, e := m.model.Generate(ctx, messages, tools, opts...)
	event.Failed = e != nil
	event.Status = "completed"
	if r != nil {
		event.Usage = r.Message.Usage
	}
	if event.Usage != nil && event.Usage.Provider != "" && event.Usage.Model != "" {
		event.Model = event.Usage.Provider + "/" + event.Usage.Model
	}
	m.a.mu.Lock()
	defer m.a.mu.Unlock()
	if m.a.LastModels == nil {
		m.a.LastModels = map[string]string{}
	}
	m.a.LastModels[m.role] = event.Model
	if event.Usage != nil && event.Usage.Cost != nil {
		event.CostStatus = "known"
		m.a.Cost += event.Usage.Cost.Total
	} else {
		m.a.UnknownCost = true
	}
	if we := state.WriteJSON(m.a.Store.Path("usage", event.ID+".json"), event); we != nil {
		return nil, fmt.Errorf("không lưu usage; dừng: %w", we)
	}
	return r, e
}
func (a *Adapter) Call(ctx context.Context, role, system string, payload any, contract llmcontract.Contract, out any, validate func() error) error {
	b, e := json.Marshal(payload)
	if e != nil {
		return e
	}
	if len(b)+len(system) > a.InputLimit(role) {
		return errors.New("request vượt context budget (UTF-8 byte upper bound)")
	}
	ctx, cancel := context.WithTimeout(ctx, time.Duration(a.Run.RequestTimeoutSeconds)*time.Second)
	defer cancel()
	m := a.Models.ForRoleWithFailover(role, nil, func(provider, name string, inner agentcore.ChatModel) agentcore.ChatModel {
		window, _ := a.Config.ResolveContextWindow(provider, name)
		limit := min(a.Run.MaxInputBytes, window-a.Run.MaxOutputTokens-2048)
		return &metered{a: a, role: role, requestHash: domain.Hash(append([]byte(system), b...)), model: inner, identity: provider + "/" + name, inputLimit: limit}
	})
	callOptions := []agentcore.CallOption{agentcore.WithMaxTokens(a.Run.MaxOutputTokens)}
	if effort := a.Config.ResolveReasoningEffort(role); effort != "" {
		callOptions = append(callOptions, agentcore.WithThinking(agentcore.ThinkingLevel(effort)))
	}
	// A raw object keeps the shared schema validator authoritative, followed by
	// exact DTO decoding and the translation-specific business validator.
	_, e = llmcontract.Execute[map[string]any](ctx, m, llmcontract.Request[map[string]any]{Contract: contract, SystemPrompt: system, Payload: string(b), Agent: role, Options: callOptions, MaxCorrections: &a.Run.MaxContractCorrections, MaxTransportRetries: &a.Run.MaxTransportRetries, Validate: func(v *map[string]any) error {
		data, e := json.Marshal(v)
		if e != nil {
			return e
		}
		if e = json.Unmarshal(data, out); e != nil {
			return e
		}
		return validate()
	}})
	return e
}
func (a *Adapter) RestoreUsage() error {
	files, e := filepath.Glob(a.Store.Path("usage", "*.json"))
	if e != nil {
		return e
	}
	for _, f := range files {
		var u Usage
		if e = state.ReadJSON(f, &u); e != nil {
			return e
		}
		a.Requests++
		if u.Usage != nil && u.Usage.Cost != nil {
			a.Cost += u.Usage.Cost.Total
		} else {
			a.UnknownCost = true
		}
	}
	return nil
}

var _ = os.ErrNotExist

package cli

import (
	"fmt"
	"slices"
	"strings"

	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
)

const (
	modelProviderField = iota
	modelVersionField
	modelEffortField
	modelKeyField
	modelURLField
	modelAPIField
	modelContextField
)

const customModelChoice = "__custom_model__"

type modelPreset struct {
	id, name string
	efforts  []string
}

// Curated text-generation models, checked against the official docs on 2026-10-21.
// See README.md's model selection notes for sources. Account access
// still depends on the API key; custom IDs remain available for other models.
var modelPresets = map[string][]modelPreset{
	"openai": {
		{"gpt-6-luna", "GPT-6 Luna", []string{"off", "low", "medium", "high", "xhigh", "max"}},
		{"gpt-6-sol", "GPT-6 Sol", []string{"off", "low", "medium", "high", "xhigh", "max"}},
		{"gpt-6.1-sol", "GPT-6.1 Sol", []string{"low", "medium", "high", "xhigh", "max"}},
		{"gpt-6-astra", "GPT-6 Astra", []string{"low", "medium", "high", "xhigh", "max"}},
	},
	"gemini": {
		{"gemini-3.8-flash", "Gemini 3.8 Flash", []string{"low", "medium", "high"}},
		{"gemini-4-argon", "Gemini 4 Argon", []string{"low", "medium", "high"}},
		{"gemini-3.6-flash", "Gemini 3.6 Flash", []string{"minimal", "low", "medium", "high"}},
		{"gemini-3.5-flash-lite", "Gemini 3.5 Flash-Lite", []string{"minimal", "low", "medium", "high"}},
		{"gemini-3.1-pro-preview", "Gemini 3.1 Pro (preview)", []string{"low", "medium", "high"}},
		{"gemini-3-flash-preview", "Gemini 3 Flash (preview)", []string{"minimal", "low", "medium", "high"}},
		// The installed generateContent adapter cannot configure thinking on 2.5.
		{"gemini-2.5-pro", "Gemini 2.5 Pro", nil},
		{"gemini-2.5-flash", "Gemini 2.5 Flash", nil},
	},
	"deepseek": {
		// The installed adapter folds low into high. Offer only exact mappings.
		{"deepseek-flash", "DeepSeek Flash (V4.1 Flash)", []string{"off", "high", "max"}},
		{"deepseek-v4-pro", "DeepSeek V4 Pro", []string{"off", "high", "max"}},
	},
	"anthropic": {
		{"claude-sonnet-5-5", "Claude Sonnet 5.5", []string{"low", "medium", "high", "xhigh", "max"}},
		{"claude-opus-5-5", "Claude Opus 5.5", []string{"low", "medium", "high", "xhigh", "max"}},
		{"claude-haiku-5-5", "Claude Haiku 5.5", []string{"low", "medium", "high", "xhigh", "max"}},
	},
}

var modelProviders = []string{"openai", "gemini", "deepseek", "anthropic", "openrouter", "ollama"}

func providerLabel(id string) string {
	switch id {
	case "openai":
		return "ChatGPT / OpenAI"
	case "gemini":
		return "Gemini / Google"
	case "deepseek":
		return "DeepSeek"
	case "anthropic":
		return "Claude / Anthropic"
	case "openrouter":
		return "OpenRouter"
	case "ollama":
		return "Ollama (local)"
	default:
		return id
	}
}

func findModelPreset(provider, id string) (modelPreset, bool) {
	for _, p := range modelPresets[provider] {
		if p.id == id {
			return p, true
		}
	}
	return modelPreset{}, false
}

func modelEfforts(provider, id string) []string {
	p, _ := findModelPreset(provider, id)
	return append([]string{""}, p.efforts...)
}

func setFormValue(f *tuiForm, index int, value string) {
	f.fields[index].value = value
	f.fields[index].input.SetValue(value)
	f.fields[index].input.CursorEnd()
}

func (f *tuiForm) refreshModelChoices(resetProvider bool) {
	provider := f.fields[modelProviderField].value
	versions := []string{}
	for _, p := range modelPresets[provider] {
		versions = append(versions, p.id)
	}
	if resetProvider {
		f.fields[modelVersionField].custom = false
		id := ""
		if len(versions) > 0 {
			id = versions[0]
		}
		setFormValue(f, modelVersionField, id)
		setFormValue(f, modelEffortField, "")
		setFormValue(f, modelKeyField, "")
		setFormValue(f, modelURLField, "")
		setFormValue(f, modelContextField, "0")
		api := "chat"
		if provider == "openai" {
			api = "responses"
		}
		setFormValue(f, modelAPIField, api)
	}
	id := f.fields[modelVersionField].value
	_, known := findModelPreset(provider, id)
	f.fields[modelVersionField].custom = f.fields[modelVersionField].custom || !known && id != ""
	if id != "" && !slices.Contains(versions, id) {
		versions = append(versions, id)
	}
	f.fields[modelVersionField].choices = append(versions, customModelChoice)
	efforts := modelEfforts(provider, id)
	f.fields[modelEffortField].choices = efforts
	if !slices.Contains(efforts, f.fields[modelEffortField].value) {
		setFormValue(f, modelEffortField, "")
	}
}

func newModelForm(c bootstrap.Config) []tuiField {
	pc := c.DefaultProviderConfig()
	provider, id, api, window := pc.Type, c.ModelName, pc.API, 0
	if provider == "" {
		provider = c.Provider
	}
	if provider == "" {
		provider = "openai"
	}
	if id == "" && len(modelPresets[provider]) > 0 {
		id = modelPresets[provider][0].id
	}
	if api == "" {
		api = "chat"
		if c.ModelName == "" && provider == "openai" {
			api = "responses"
		}
	}
	if mc, ok := pc.ModelConfig(id); ok {
		window = mc.ContextWindow
	}
	effort := c.ReasoningEffort
	if effort == "auto" {
		effort = ""
	}
	if effort == "none" {
		effort = "off"
	}
	providers := slices.Clone(modelProviders)
	if !slices.Contains(providers, provider) {
		providers = append(providers, provider)
	}
	return []tuiField{
		{label: "1. Nhà cung cấp AI", value: provider, choices: providers, selectOnly: true},
		{label: "2. Phiên bản model", value: id, selectOnly: true},
		{label: "3. Mức suy luận", value: effort, selectOnly: true},
		{label: "API key (trống: giữ key cùng nhà cung cấp)", secret: true},
		{label: "Base URL (trống: mặc định)", value: pc.BaseURL},
		{label: "API OpenAI (Ctrl+N đổi)", value: api, choices: []string{"responses", "chat"}},
		{label: "Context tokens (0: tự nhận)", value: fmt.Sprint(window)},
	}
}

func (f *tuiForm) choiceLabel(index int, value string) string {
	if f.kind != "model" {
		return value
	}
	switch index {
	case modelProviderField:
		return providerLabel(value)
	case modelVersionField:
		if value == customModelChoice {
			return "Nhập model ID khác…"
		}
		if p, ok := findModelPreset(f.fields[modelProviderField].value, value); ok {
			return p.name + "  [" + value + "]"
		}
	case modelEffortField:
		if value == "" {
			if len(f.fields[index].choices) == 1 {
				return "Mặc định API (chưa có mức tùy chỉnh được hỗ trợ)"
			}
			return "Mặc định API (auto)"
		}
		if value == "off" {
			return "Tắt suy luận (off)"
		}
	}
	return value
}

type tuiChoices struct {
	filter textinput.Model
	index  int
}

func (f *tuiForm) fieldVisible(index int) bool {
	return f.kind != "model" || index != modelAPIField || f.fields[modelProviderField].value == "openai"
}

func (f *tuiForm) focusNext(delta int) tea.Cmd {
	f.fields[f.index].input.Blur()
	for {
		f.index = (f.index + delta + len(f.fields)) % len(f.fields)
		if f.fieldVisible(f.index) {
			return f.fields[f.index].input.Focus()
		}
	}
}

func (f *tuiForm) filteredChoices() []string {
	var values []string
	query := strings.ToLower(f.selection.filter.Value())
	for _, value := range f.fields[f.index].choices {
		if strings.Contains(strings.ToLower(f.choiceLabel(f.index, value)), query) || value == customModelChoice {
			values = append(values, value)
		}
	}
	return values
}

func (m *translationTUI) openChoices() tea.Cmd {
	f := m.form
	filter := textinput.New()
	filter.Prompt = "Tìm: "
	filter.Width = max(1, m.width-10)
	field := &f.fields[f.index]
	field.input.Blur()
	f.selection = &tuiChoices{filter: filter, index: max(0, slices.Index(field.choices, field.value))}
	return f.selection.filter.Focus()
}

func (m *translationTUI) updateChoices(key tea.KeyMsg) tea.Cmd {
	f := m.form
	s := f.selection
	values := f.filteredChoices()
	switch key.String() {
	case "esc", "ctrl+c":
		f.selection = nil
		return f.fields[f.index].input.Focus()
	case "up":
		s.index = max(0, s.index-1)
	case "down":
		s.index = min(max(0, len(values)-1), s.index+1)
	case "pgup":
		s.index = max(0, s.index-8)
	case "pgdown":
		s.index = min(max(0, len(values)-1), s.index+8)
	case "enter":
		if len(values) == 0 {
			return nil
		}
		value := values[s.index]
		f.selection = nil
		if value == customModelChoice {
			f.fields[f.index].custom = true
			setFormValue(f, f.index, "")
			setFormValue(f, modelEffortField, "")
			f.fields[modelEffortField].choices = []string{""}
			m.notice = "Nhập chính xác model ID của API; Ctrl+N mở lại danh sách."
			return f.fields[f.index].input.Focus()
		}
		changed := value != f.fields[f.index].value
		f.fields[f.index].custom = false
		setFormValue(f, f.index, value)
		if changed && (f.index == modelProviderField || f.index == modelVersionField) {
			if f.index == modelVersionField {
				setFormValue(f, modelContextField, "0")
			}
			f.refreshModelChoices(f.index == modelProviderField)
		}
		return f.focusNext(1)
	default:
		var cmd tea.Cmd
		s.filter, cmd = s.filter.Update(key)
		s.index = 0
		return cmd
	}
	return nil
}

func (f *tuiForm) choicesView(width, height int) string {
	clip := func(s string) string { return ansi.Truncate(s, max(1, width-2), "…") }
	lines := []string{clip("CHỌN " + f.fields[f.index].label), clip(f.selection.filter.View()), ""}
	values := f.filteredChoices()
	n := max(1, height-8)
	start := max(0, min(f.selection.index-n/2, len(values)-n))
	for i := start; i < min(len(values), start+n); i++ {
		prefix := "  "
		if i == f.selection.index {
			prefix = "› "
		}
		lines = append(lines, clip(prefix+cleanTUIText(f.choiceLabel(f.index, values[i]))))
	}
	if len(values) == 0 {
		lines = append(lines, clip("Không có kết quả."))
	}
	lines = append(lines, "", clip("↑/↓ chọn · Gõ để tìm · Enter xác nhận · Esc quay lại"))
	return strings.Join(lines, "\n")
}

package cli

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"slices"
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/x/ansi"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/translation/model"
)

// Exercise the same key handling used by the TUI, including filtered selection.
func chooseModelField(t *testing.T, m *translationTUI, index int, query, want string) {
	t.Helper()
	m.form.index = index
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlN})
	if m.form.selection == nil {
		t.Fatal("selector did not open")
	}
	if query != "" {
		m.Update(tuiKey(query))
	}
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if m.form.selection != nil || m.form.fields[index].value != want {
		t.Fatalf("selected %q, want %q", m.form.fields[index].value, want)
	}
}

func TestTUISavedReasoningReachesProviderRequest(t *testing.T) {
	const translated = `{"units":[{"id":"b","target":"Bản dịch"}],"uncertainties":[],"continuity_candidates":[],"rule_candidates":[]}`
	for _, tc := range []struct {
		provider, id, effort, api, path, effortPath, wireEffort string
	}{
		{"openai", "gpt-6-luna", "max", "responses", "/responses", "reasoning.effort", "max"},
		{"openai", "gpt-6-sol", "off", "chat", "/chat/completions", "reasoning_effort", "none"},
		{"gemini", "gemini-3.8-flash", "medium", "", "/v1beta/models/gemini-3.8-flash:generateContent", "generationConfig.thinkingConfig.thinkingLevel", "medium"},
		{"deepseek", "deepseek-v4-pro", "max", "", "/chat/completions", "reasoning_effort", "max"},
		{"deepseek", "deepseek-flash", "off", "", "/chat/completions", "thinking.type", "disabled"},
		{"anthropic", "claude-sonnet-5-5", "high", "", "/v1/messages", "output_config.effort", "high"},
	} {
		t.Run(tc.provider+"/"+tc.id+"/"+tc.effort, func(t *testing.T) {
			t.Setenv("NOVEL_TRANSLATE_API_KEY", "")
			calls := 0
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				calls++
				if !strings.HasSuffix(r.URL.Path, tc.path) {
					t.Errorf("wrong API endpoint %s", r.URL.Path)
				}
				var body map[string]any
				if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
					t.Error(err)
				}
				var effort any = body
				for _, key := range strings.Split(tc.effortPath, ".") {
					obj, _ := effort.(map[string]any)
					effort = obj[key]
				}
				if effort != tc.wireEffort {
					t.Errorf("wire effort %v, want %s", effort, tc.wireEffort)
				}
				if tc.provider != "gemini" && body["model"] != tc.id {
					t.Errorf("wire model %v, want %s", body["model"], tc.id)
				}
				var response any
				switch {
				case tc.api == "responses":
					response = map[string]any{"id": "resp_test", "model": tc.id, "status": "completed", "output": []any{map[string]any{"type": "message", "role": "assistant", "content": []any{map[string]any{"type": "output_text", "text": translated}}}}}
				case tc.provider == "gemini":
					response = map[string]any{"candidates": []any{map[string]any{"content": map[string]any{"role": "model", "parts": []any{map[string]any{"text": translated}}}, "finishReason": "STOP"}}}
				case tc.provider == "anthropic":
					response = map[string]any{"id": "msg_test", "type": "message", "role": "assistant", "model": tc.id, "stop_reason": "end_turn", "content": []any{map[string]any{"type": "text", "text": translated}}, "usage": map[string]any{"input_tokens": 10, "output_tokens": 10}}
				default:
					response = map[string]any{"id": "chat_test", "model": tc.id, "choices": []any{map[string]any{"index": 0, "finish_reason": "stop", "message": map[string]any{"role": "assistant", "content": translated}}}}
				}
				w.Header().Set("Content-Type", "application/json")
				json.NewEncoder(w).Encode(response)
			}))
			defer server.Close()
			m := setupTUI(t)
			values := []string{tc.provider, tc.id, tc.effort, "fake-key", server.URL, tc.api, "0"}
			var out bytes.Buffer
			if err := m.saveModelSetup(values)(context.Background(), &out, &out); err != nil {
				t.Fatal(err)
			}
			m.config.ProviderConfig = ".translation_work/provider.json"
			adapter, err := model.New(*m.config)
			if err != nil {
				t.Fatal(err)
			}
			adapter.Run.MaxRequests = 1
			adapter.Run.MaxTransportRetries = 0
			var result map[string]any
			if err := adapter.Call(context.Background(), "translation", "Translate.", map[string]string{"source": "日本語"}, model.TranslationContract(), &result, func() error { return nil }); err != nil {
				t.Fatal(err)
			}
			if calls != 1 {
				t.Fatalf("requests = %d, want 1", calls)
			}
		})
	}
}

func TestTUIModelProviderVersionAndEffortSelection(t *testing.T) {
	m := setupTUI(t)
	m.openForm("model")
	chooseModelField(t, m, modelProviderField, "OpenAI", "openai")
	chooseModelField(t, m, modelVersionField, "6 Sol", "gpt-6-sol")
	chooseModelField(t, m, modelEffortField, "max", "max")
	if m.form.fields[modelAPIField].value != "responses" {
		t.Fatal("fresh OpenAI setup should use Responses")
	}
	// Choosing a model that cannot disable reasoning clears an invalid effort.
	setFormValue(m.form, modelEffortField, "off")
	chooseModelField(t, m, modelVersionField, "6.1", "gpt-6.1-sol")
	if m.form.fields[modelEffortField].value != "" || slices.Contains(m.form.fields[modelEffortField].choices, "off") {
		t.Fatal("invalid effort survived model change")
	}
	setFormValue(m.form, modelKeyField, "fake-openai-key")
	setFormValue(m.form, modelURLField, "https://example.invalid/v1")
	setFormValue(m.form, modelContextField, "10000")
	chooseModelField(t, m, modelProviderField, "Gemini", "gemini")
	if m.form.fields[modelVersionField].value != "gemini-3.8-flash" || m.form.fields[modelKeyField].value != "" || m.form.fields[modelURLField].value != "" || m.form.fields[modelContextField].value != "0" {
		t.Fatal("provider change retained another provider's setup")
	}
	if slices.Contains(m.form.fields[modelEffortField].choices, "max") {
		t.Fatal("OpenAI-only effort leaked to Gemini")
	}
	chooseModelField(t, m, modelEffortField, "medium", "medium")
	chooseModelField(t, m, modelProviderField, "DeepSeek", "deepseek")
	chooseModelField(t, m, modelVersionField, "Pro", "deepseek-v4-pro")
	chooseModelField(t, m, modelEffortField, "max", "max")
}

func TestTUIModelSelectorCustomCancelAndNarrowTerminal(t *testing.T) {
	m := setupTUI(t)
	m.Update(tea.WindowSizeMsg{Width: 38, Height: 14})
	m.openForm("model")
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	for _, label := range []string{"ChatGPT", "Gemini", "DeepSeek"} {
		if !strings.Contains(m.View(), label) {
			t.Fatalf("provider selector missing %s", label)
		}
	}
	for _, line := range strings.Split(m.View(), "\n") {
		if ansi.StringWidth(line) > 38 {
			t.Fatal("selector overflows terminal width")
		}
	}
	if len(strings.Split(m.View(), "\n")) > 14 {
		t.Fatal("selector overflows terminal height")
	}
	m.Update(tuiKey("nothing matches"))
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if m.form.selection == nil {
		t.Fatal("empty result closed selector")
	}
	m.Update(tea.KeyMsg{Type: tea.KeyEsc})
	if m.form == nil || m.form.fields[modelProviderField].value != "openai" {
		t.Fatal("cancel changed form")
	}
	m.form.index = modelVersionField
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlN})
	m.Update(tuiKey("Nhập model ID"))
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if !m.form.fields[modelVersionField].custom {
		t.Fatal("custom model input not enabled")
	}
	m.Update(tuiKey("my-model"))
	m.Update(tea.KeyMsg{Type: tea.KeyCtrlU})
	m.Update(tuiKey("my-new-model"))
	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	if m.form.fields[modelVersionField].value != "my-new-model" || m.form.index != modelEffortField || len(m.form.fields[modelEffortField].choices) != 1 {
		t.Fatal("custom model did not retain ID or clear unverified efforts")
	}
	chooseModelField(t, m, modelProviderField, "Gemini", "gemini")
	if m.form.fields[modelVersionField].custom {
		t.Fatal("custom input mode survived provider change")
	}
	m.form.index = modelURLField
	m.Update(tea.KeyMsg{Type: tea.KeyTab})
	if m.form.index != modelContextField || strings.Contains(m.View(), "API OpenAI") {
		t.Fatal("OpenAI API field should be skipped for Gemini")
	}
}

func TestTUIModelEffortPersistsAndProviderSwitchRequiresOwnKey(t *testing.T) {
	t.Setenv("NOVEL_TRANSLATE_API_KEY", "")
	t.Setenv("NOVEL_TRANSLATE_GEMINI_API_KEY", "")
	m := setupTUI(t)
	values := []string{"openai", "gpt-6-luna", "max", "fake-key", "", "responses", "0"}
	var out bytes.Buffer
	if err := m.saveModelSetup(values)(context.Background(), &out, &out); err != nil {
		t.Fatal(err)
	}
	// Reload from the workspace, as the asynchronous save completion does.
	m.config.ProviderConfig = ".translation_work/provider.json"
	cfg, err := bootstrap.LoadConfigFile(providerPath(*m.config), m.config.Root)
	if err != nil || cfg.ResolveReasoningEffort("translation") != "max" || cfg.ResolveReasoningEffort("translation_review") != "max" {
		t.Fatal("saved effort not applied to translation roles", err)
	}
	m.openForm("model")
	if m.form.fields[modelEffortField].value != "max" || m.form.fields[modelVersionField].value != "gpt-6-luna" || m.form.fields[modelKeyField].value != "" {
		t.Fatal("reopening setup did not restore model/effort safely")
	}
	values = []string{"gemini", "gemini-3.8-flash", "high", "", "", "chat", "0"}
	if err := m.saveModelSetup(values)(context.Background(), &out, &out); err == nil {
		t.Fatal("provider switch reused another provider's key")
	}
	values[modelKeyField] = "fake-gemini-key"
	values[modelEffortField] = "max"
	if err := m.saveModelSetup(values)(context.Background(), &out, &out); err == nil {
		t.Fatal("unsupported effort accepted")
	}
	values[modelEffortField] = "high"
	if err := m.saveModelSetup(values)(context.Background(), &out, &out); err != nil {
		t.Fatal(err)
	}
	cfg, err = bootstrap.LoadConfigFile(providerPath(*m.config), m.config.Root)
	if err != nil || cfg.DefaultProviderConfig().APIKey != "fake-gemini-key" || cfg.ReasoningEffort != "high" {
		t.Fatal("provider-specific key/effort did not round trip", err)
	}
}

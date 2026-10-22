package bootstrap

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/voocel/agentcore"
	"github.com/voocel/ainovel-cli/internal/llmcontract"
)

type fakeProvider struct {
	Calls int
	Err   error
}

func (m *fakeProvider) Generate(context.Context, []agentcore.Message, []agentcore.ToolSpec, ...agentcore.CallOption) (*agentcore.LLMResponse, error) {
	m.Calls++
	return &agentcore.LLMResponse{Message: agentcore.Message{Content: []agentcore.ContentBlock{agentcore.TextBlock("{}")}}}, m.Err
}
func (*fakeProvider) GenerateStream(context.Context, []agentcore.Message, []agentcore.ToolSpec, ...agentcore.CallOption) (<-chan agentcore.StreamEvent, error) {
	return nil, errors.New("unused")
}
func (*fakeProvider) SupportsTools() bool { return false }
func TestTranslationRolesAndConfig(t *testing.T) {
	p := filepath.Join(t.TempDir(), "provider.json")
	if e := os.WriteFile(p, []byte(`{"provider":"local","model":"fake","providers":{"local":{"type":"openai","allow_no_api_key":true,"base_url":"http://127.0.0.1:9999/v1","models":["fake"]}},"roles":{"translation_review":{"provider":"local","model":"fake"}}}`), 0600); e != nil {
		t.Fatal(e)
	}
	c, e := LoadConfigFile(p)
	if e != nil {
		t.Fatal(e)
	}
	if e = c.ValidateBase(); e != nil {
		t.Fatal(e)
	}
	c.Roles["writer"] = RoleConfig{Provider: "local", Model: "fake"}
	if e = c.ValidateBase(); e == nil {
		t.Fatal("creative role still accepted")
	}
}

func TestExampleProviderConfigsAndWindowsBOM(t *testing.T) {
	t.Setenv("NOVEL_TRANSLATE_API_KEY", "test-key-not-a-secret")
	for _, name := range []string{"config.example.jsonc", "config.gemini.example.jsonc", "config.anthropic.example.jsonc"} {
		t.Run(name, func(t *testing.T) {
			data, err := os.ReadFile(filepath.Join("..", "..", name))
			if err != nil {
				t.Fatal(err)
			}
			text := strings.ReplaceAll(string(data), "YOUR_MODEL_ID", "sample-model")
			text = strings.ReplaceAll(text, "YOUR_GEMINI_MODEL_ID", "sample-gemini-model")
			text = strings.ReplaceAll(text, "YOUR_CLAUDE_MODEL_ID", "sample-claude-model")
			text = strings.ReplaceAll(text, "https://YOUR_SERVICE_HOST/v1", "http://127.0.0.1:9999/v1")
			path := filepath.Join(t.TempDir(), "provider.jsonc")
			if err := os.WriteFile(path, append([]byte{0xef, 0xbb, 0xbf}, []byte(text)...), 0600); err != nil {
				t.Fatal(err)
			}
			c, err := LoadConfigFile(path)
			if err != nil {
				t.Fatal(err)
			}
			if err := c.ValidateBase(); err != nil {
				t.Fatal(err)
			}
			if _, err := NewModelSet(c); err != nil {
				t.Fatal("provider adapter construction failed", err)
			}
		})
	}
}
func TestBoundedContractsWithFakeGenerator(t *testing.T) {
	m := &fakeProvider{}
	limit := 2
	transport := 0
	c := llmcontract.Contract{Name: "fake", Schema: map[string]any{"type": "object", "properties": map[string]any{"value": map[string]any{"type": "string"}}, "required": []string{"value"}, "additionalProperties": false}}
	_, e := llmcontract.Execute[map[string]any](context.Background(), m, llmcontract.Request[map[string]any]{Contract: c, MaxCorrections: &limit, MaxTransportRetries: &transport})
	if e == nil || m.Calls != 3 {
		t.Fatal("contract retry limit", m.Calls, e)
	}
}

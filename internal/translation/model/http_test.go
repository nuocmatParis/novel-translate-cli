package model

import (
	"context"
	"encoding/json"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestConfiguredHTTPProviderFailoverIsMetered(t *testing.T) {
	primaryCalls, fallbackCalls := 0, 0
	primary := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		primaryCalls++
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(429)
		w.Write([]byte(`{"error":{"message":"scripted rate limit","type":"rate_limit_error"}}`))
	}))
	defer primary.Close()
	fallback := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		fallbackCalls++
		var request map[string]any
		if e := json.NewDecoder(r.Body).Decode(&request); e != nil {
			t.Error(e)
		}
		if request["model"] != "fallback" {
			t.Errorf("wrong model %v", request["model"])
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]any{"id": "local-script", "object": "chat.completion", "model": "fallback", "choices": []any{map[string]any{"index": 0, "finish_reason": "stop", "message": map[string]any{"role": "assistant", "content": `{"units":[{"id":"b","target":"Bản dịch"}],"uncertainties":[],"continuity_candidates":[],"rule_candidates":[]}`}}}, "usage": map[string]any{"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}})
	}))
	defer fallback.Close()
	c := workspace.Defaults()
	c.Root = t.TempDir()
	c.ProviderConfig = "provider.json"
	config := map[string]any{"provider": "primary", "model": "primary", "providers": map[string]any{"primary": map[string]any{"type": "openai", "allow_no_api_key": true, "base_url": primary.URL + "/v1", "models": []any{map[string]any{"name": "primary", "json_schema": false}}}, "backup": map[string]any{"type": "openai", "allow_no_api_key": true, "base_url": fallback.URL + "/v1", "models": []any{map[string]any{"name": "fallback", "json_schema": false}}}}, "roles": map[string]any{"translation": map[string]any{"provider": "primary", "model": "primary", "fallbacks": []any{map[string]any{"provider": "backup", "model": "fallback"}}}}}
	raw, _ := json.Marshal(config)
	if e := os.WriteFile(filepath.Join(c.Root, c.ProviderConfig), raw, 0600); e != nil {
		t.Fatal(e)
	}
	a, e := New(c)
	if e != nil {
		t.Fatal(e)
	}
	a.Run.MaxTransportRetries = 0
	a.Run.MaxRequests = 2
	var out map[string]any
	if e = a.Call(context.Background(), "translation", "Translate.", map[string]string{"source": "日本語"}, TranslationContract(), &out, func() error { return nil }); e != nil {
		t.Fatal(e)
	}
	if primaryCalls != 1 || fallbackCalls != 1 || a.Requests != 2 || !strings.Contains(a.Identity("translation"), "fallback") {
		t.Fatal("failover not counted/identified", primaryCalls, fallbackCalls, a.Requests, a.Identity("translation"))
	}
	s, e := ReadUsage(a.Store)
	if e != nil || s.Requests != 2 || s.Failed != 1 || s.InputTokens != 10 || s.UnknownCostRequests != 2 {
		t.Fatal(s, e)
	}
	resumed, e := New(c)
	if e != nil {
		t.Fatal(e)
	}
	if e = resumed.RestoreUsage(); e != nil || resumed.Requests != 2 {
		t.Fatal("usage not restored", e)
	}
}

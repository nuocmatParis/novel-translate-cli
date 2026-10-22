package model

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/voocel/agentcore"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/llmcontract"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type responseModel struct {
	Responses []*agentcore.LLMResponse
	Calls     int
}

func (m *responseModel) Generate(context.Context, []agentcore.Message, []agentcore.ToolSpec, ...agentcore.CallOption) (*agentcore.LLMResponse, error) {
	m.Calls++
	if len(m.Responses) == 0 {
		return nil, errors.New("no scripted response")
	}
	r := m.Responses[0]
	m.Responses = m.Responses[1:]
	return r, nil
}
func (*responseModel) GenerateStream(context.Context, []agentcore.Message, []agentcore.ToolSpec, ...agentcore.CallOption) (<-chan agentcore.StreamEvent, error) {
	return nil, errors.New("unused")
}
func (*responseModel) SupportsTools() bool { return false }
func response(text string) *agentcore.LLMResponse {
	return &agentcore.LLMResponse{Message: agentcore.Message{Content: []agentcore.ContentBlock{agentcore.TextBlock(text)}, Usage: &agentcore.Usage{Provider: "fake", Model: "script", Input: 10, Output: 5}}}
}
func TestAdapterMalformedContractUsageAndBudget(t *testing.T) {
	m := &responseModel{Responses: []*agentcore.LLMResponse{response("malformed"), response(`{"units":[{"id":"b","target":"Bản dịch"}],"uncertainties":[],"continuity_candidates":[],"rule_candidates":[]}`)}}
	a := &Adapter{Models: &bootstrap.ModelSet{Default: bootstrap.NewSwappableModel("fake", "script", m, nil)}, Run: workspace.Defaults().Run, Store: state.Store{Root: t.TempDir()}}
	a.Run.MaxRequests = 2
	var out map[string]any
	if e := a.Call(context.Background(), "translation", "Translate.", map[string]string{"source": "日本語"}, TranslationContract(), &out, func() error { return nil }); e != nil {
		t.Fatal(e)
	}
	if m.Calls != 2 || !a.UnknownCost {
		t.Fatal("failed correction/unknown cost not recorded")
	}
	files, e := a.Store.Load("unused")
	if e != nil || len(files.Units) != 0 {
		t.Fatal(e)
	}
	if e = a.Call(context.Background(), "translation", "Translate.", nil, TranslationContract(), &out, func() error { return nil }); e == nil {
		t.Fatal("request budget not enforced")
	}
}
func TestStrictSchemaReady(t *testing.T) {
	for _, c := range []llmcontract.Contract{TranslationContract(), ReviewContract(), ReviewContractFor("unit", "target-hash")} {
		if e := llmcontract.ValidateStrictReady(c.Schema); e != nil {
			t.Fatal(e)
		}
	}
}

func TestAdapterCorrectsInvalidReviewSeverity(t *testing.T) {
	const hash = "current-revision-hash"
	bad := `{"reviewed":[{"id":"unit","target_hash":"` + hash + `"}],"issues":[{"id":"i","type":"omission","severity":"major","block_id":"unit","source_quote":"雨","target_quote":"","explanation":"Mất mệnh đề."}],"ruby_decisions":[]}`
	good := strings.Replace(bad, `"severity":"major"`, `"severity":"blocking"`, 1)
	m := &responseModel{Responses: []*agentcore.LLMResponse{response(bad), response(good)}}
	a := &Adapter{Models: &bootstrap.ModelSet{Default: bootstrap.NewSwappableModel("fake", "script", m, nil)}, Run: workspace.Defaults().Run, Store: state.Store{Root: t.TempDir()}}
	a.Run.MaxRequests = 2
	var out struct {
		Issues []domain.Issue `json:"issues"`
	}
	checks := 0
	err := a.Call(context.Background(), "translation_review", "Return a clause-level review.", map[string]string{"id": "unit", "target_hash": hash}, ReviewContractFor("unit", hash), &out, func() error { checks++; return nil })
	if err != nil {
		t.Fatal(err)
	}
	if m.Calls != 2 || checks != 1 || len(out.Issues) != 1 || out.Issues[0].Severity != "blocking" {
		t.Fatal("invalid severity accepted or blocking issue weakened", m.Calls, checks, out)
	}
}

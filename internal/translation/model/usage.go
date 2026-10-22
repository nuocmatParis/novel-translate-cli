package model

import (
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"path/filepath"
)

type UsageSummary struct {
	Requests            int     `json:"requests"`
	Failed              int     `json:"failed"`
	InFlight            int     `json:"in_flight"`
	InputTokens         int     `json:"input_tokens"`
	OutputTokens        int     `json:"output_tokens"`
	KnownCostUSD        float64 `json:"known_cost_usd"`
	UnknownCostRequests int     `json:"unknown_cost_requests"`
	Scope               string  `json:"scope"`
}

func ReadUsage(store state.Store) (UsageSummary, error) {
	s := UsageSummary{Scope: "workspace lifetime; includes retries and failover"}
	files, e := filepath.Glob(store.Path("usage", "*.json"))
	if e != nil {
		return s, e
	}
	for _, f := range files {
		var u Usage
		if e = state.ReadJSON(f, &u); e != nil {
			return s, e
		}
		s.Requests++
		if u.Failed {
			s.Failed++
		}
		if u.Status == "in-flight" {
			s.InFlight++
		}
		if u.Usage != nil {
			s.InputTokens += u.Usage.Input
			s.OutputTokens += u.Usage.Output
		}
		if u.Usage != nil && u.Usage.Cost != nil {
			s.KnownCostUSD += u.Usage.Cost.Total
		} else {
			s.UnknownCostRequests++
		}
	}
	return s, nil
}

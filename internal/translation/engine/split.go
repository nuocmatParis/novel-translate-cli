package engine

import (
	"fmt"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"strings"
)

// SplitBlock keeps ruby bases and complete inline elements within a request.
// An indivisible element exceeding the budget is reported instead of corrupted.
func SplitBlock(b domain.Block, limit int) ([]domain.Child, error) {
	type span struct{ start, end int }
	var protected []span
	for _, r := range b.Ruby {
		protected = append(protected, span{r.Start, r.End})
	}
	for _, t := range b.Tokens {
		start := strings.Index(b.Source, "⟦"+t.ID+"⟧")
		if start < 0 {
			return nil, fmt.Errorf("token mapping thiếu %s", t.ID)
		}
		protected = append(protected, span{start, start + len("⟦"+t.ID+"⟧")})
		if t.Kind == "close" {
			open := strings.Index(b.Source, "⟦"+t.Pair+"⟧")
			if open < 0 || open >= start {
				return nil, fmt.Errorf("token pair mapping sai %s", t.ID)
			}
			protected = append(protected, span{open, start + len("⟦"+t.ID+"⟧")})
		}
	}
	var out []domain.Child
	for start := 0; start < len(b.Source); {
		parts := Split(b.ID, b.Source[start:], limit)
		end := start + parts[0].End
		for {
			before := end
			for _, p := range protected {
				if p.start < end && end < p.end {
					end = p.start
				}
			}
			if before == end {
				break
			}
		}
		if end <= start {
			return nil, fmt.Errorf("%w: ruby/inline span vượt chunk budget %d bytes tại %s; tăng context/output budget", ErrUnresolved, limit, b.ID)
		}
		out = append(out, domain.Child{ID: fmt.Sprintf("%s-child-%04d", b.ID, len(out)+1), Start: start, End: end, Source: b.Source[start:end]})
		start = end
	}
	return out, nil
}

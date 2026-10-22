package verify

import (
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"regexp"
	"strings"
	"unicode"
)

var number = regexp.MustCompile(`[0-9０-９]+`)

func digits(s string) string {
	return strings.Map(func(r rune) rune {
		if r >= '０' && r <= '９' {
			return r - '０' + '0'
		}
		return r
	}, s)
}

// Signals are evidence for a reviewer, not an automatic semantic score.
func Signals(b domain.Block, u domain.Unit, rules ...[]domain.Rule) []domain.Issue {
	var issues []domain.Issue
	japanese := false
	for _, r := range b.Source {
		if unicode.In(r, unicode.Hiragana, unicode.Katakana, unicode.Han) {
			japanese = true
			break
		}
	}
	if japanese && b.Source == u.Target {
		issues = append(issues, domain.Issue{ID: "source-echo", BlockID: b.ID, Type: "unresolved-source", Severity: "blocking", SourceQuote: b.Source, TargetQuote: u.Target, Explanation: "Target giữ nguyên toàn bộ nguồn Nhật; cần quyết định rõ cho text được giữ nguyên hoặc dịch/review lại."})
	}
	seen := map[string]bool{}
	for _, value := range number.FindAllString(b.Source, -1) {
		canonical := digits(value)
		if seen[canonical] || strings.Contains(digits(u.Target), canonical) {
			continue
		}
		seen[canonical] = true
		issues = append(issues, domain.Issue{ID: "number-signal-" + canonical, BlockID: b.ID, Type: "number/name", Severity: "warning", SourceQuote: value, Explanation: "Dãy số nguồn không thấy nguyên dạng trong target; kiểm tra cách viết/đổi định dạng trước khi kết luận."})
	}
	for _, suffix := range []struct{ JP, VI string }{{"さん", "san"}, {"様", "sama"}, {"さま", "sama"}, {"くん", "kun"}, {"君", "kun"}, {"ちゃん", "chan"}, {"たん", "tan"}} {
		if strings.Contains(b.Source, suffix.JP) && !strings.Contains(strings.ToLower(u.Target), suffix.VI) {
			issues = append(issues, domain.Issue{ID: "honorific-signal-" + suffix.JP, BlockID: b.ID, Type: "honorific", Severity: "warning", SourceQuote: suffix.JP, Explanation: "Hậu tố nguồn không thấy dạng romanization trong target; đối chiếu tên, vai trò từ và quy ước có scope trước khi kết luận."})
		}
	}
	if len(rules) > 0 {
		for _, rule := range rules[0] {
			if rule.Kind != "glossary" && rule.Kind != "name" && rule.Kind != "term" {
				continue
			}
			quote := ""
			if rule.Key != "" && strings.Contains(b.Source, rule.Key) {
				quote = rule.Key
			}
			if quote == "" {
				for _, ev := range rule.Evidence {
					if ev.BlockID != "" && ev.Quote != "" && strings.Contains(b.Source, ev.Quote) {
						quote = ev.Quote
						break
					}
				}
			}
			if quote != "" && rule.Value != "" && !strings.Contains(u.Target, rule.Value) {
				issues = append(issues, domain.Issue{ID: "glossary-signal-" + rule.ID, BlockID: b.ID, Type: "glossary", Severity: "warning", SourceQuote: quote, Explanation: "Không thấy giá trị quy ước " + rule.Value + " trong target; kiểm tra biến thể/ngữ cảnh, không tự thay bằng regex."})
			}
		}
	}
	return issues
}

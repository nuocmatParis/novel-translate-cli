package testutil

import (
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"strings"
)

// TranslateFixture is a fixed script for original synthetic excerpts. It is
// deliberately not a fake production translation mode or a language model.
func TranslateFixture(source string) string {
	if source == "前名後⟦t0⟧強調⟦t1⟧と⟦t2⟧注⟦t3⟧。" {
		return "Trước tên, sau ⟦t0⟧nhấn mạnh⟦t1⟧ và ⟦t2⟧chú thích⟦t3⟧."
	}
	return strings.NewReplacer("雨が降るが、彼は走った。", "Dù trời mưa, cậu ấy đã chạy.", "彼は来た。", "Cậu ấy đã đến.", "彼は帰った。", "Cậu ấy đã trở về.", "第一章", "Chương một", "第二章", "Chương hai", "前置き。", "Lời mở đầu.", "試験", "Thử nghiệm", "続き。", "Phần tiếp theo.", "注釈。", "Chú thích. ", "戻る", "Quay lại", "強調", "nhấn mạnh", "前", "Trước", "名", "Tên", "後", "Sau", "注", "Chú thích", "と", "và", "序", "Mở đầu", "日本語。", "Tiếng Nhật.").Replace(source)
}
func RubyDecisions(block domain.Block) []domain.RubyDecision {
	result := []domain.RubyDecision{}
	for i, r := range block.Ruby {
		result = append(result, domain.RubyDecision{Index: i, Base: r.Base, Reading: r.Reading, Decision: "phonetic", Reason: "scripted fixture; no live semantic assessment"})
	}
	return result
}

package epub

import (
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"testing"
)

func TestForeignContentAndGeneratedTextBlockCompleteness(t *testing.T) {
	for _, tc := range []struct{ name, body, code string }{
		{"foreign", `<p>本文</p><extra xmlns="urn:unhandled">別の本文</extra>`, "unclassified-namespace"},
		{"generated", `<style>p::after { CONTENT : "追加"; }</style><p>本文</p>`, "generated-text"},
		{"handler", `<p onclick="alert(1)">本文</p>`, "script"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			files := testutil.Files()
			files["OPS/a.xhtml"] = testutil.XHTML(tc.body)
			p, err := Read(testutil.ZIP(files), "v1", 1)
			if err != nil {
				t.Fatal(err)
			}
			for _, d := range p.Book.Diagnostics {
				if d.Code == tc.code && d.Blocking {
					return
				}
			}
			t.Fatal("unsupported content was not blocking", p.Book.Diagnostics)
		})
	}
}

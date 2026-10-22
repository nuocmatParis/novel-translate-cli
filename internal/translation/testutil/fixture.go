// Package testutil creates small original fixtures; it contains no book excerpts.
package testutil

import (
	"archive/zip"
	"bytes"
	"fmt"
	"sort"
)

func Files() map[string]string {
	return map[string]string{
		"mimetype":               "application/epub+zip",
		"META-INF/container.xml": `<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>`,
		"OPS/book.opf":           `<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="uid">urn:uuid:c3588b17-77a1-40f9-8bc7-4b892b0b60fb</dc:identifier><dc:title>試験</dc:title><dc:language>ja</dc:language><dc:creator>Fixture author</dc:creator><meta property="dcterms:modified">2026-10-10T00:00:00Z</meta></metadata><manifest><item id="z" href="z.xhtml" media-type="application/xhtml+xml"/><item id="a" href="a.xhtml" media-type="application/xhtml+xml"/><item id="picture" href="picture.xhtml" media-type="application/xhtml+xml" properties="svg"/><item id="end" href="end.xhtml" media-type="application/xhtml+xml"/><item id="notes" href="notes.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="ncx" href="nav.ncx" media-type="application/x-dtbncx+xml"/><item id="image" href="image.svg" media-type="image/svg+xml"/></manifest><spine toc="ncx" page-progression-direction="rtl"><itemref idref="z"/><itemref idref="a"/><itemref idref="picture"/><itemref idref="end"/></spine></package>`,
		"OPS/z.xhtml":            XHTML(`<h1 id="front">序</h1><p>前置き。</p>`),
		"OPS/a.xhtml":            XHTML(`<h1 id="c1">第一章</h1><p id="b1">彼は来た。⟦</p><p>前<ruby>名<rt>な</rt></ruby>後<strong>強調</strong>と<a epub:type="noteref" href="notes.xhtml#n1">注</a>。</p>`),
		"OPS/picture.xhtml":      XHTML(`<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 10 10"><image width="10" height="10" xlink:href="image.svg"/></svg>`),
		"OPS/end.xhtml":          XHTML(`<p>続き。</p><h1 id="c2">第二章</h1><p>雨が降るが、彼は走った。</p>`),
		"OPS/notes.xhtml":        XHTML(`<aside epub:type="footnote" id="n1"><p>注釈。<a href="a.xhtml#b1">戻る</a></p></aside>`),
		"OPS/nav.xhtml":          XHTML(`<nav epub:type="toc"><ol><li><a href="z.xhtml#front">序</a></li><li><a href="a.xhtml#c1">第一章</a></li><li><a href="end.xhtml#c2">第二章</a></li></ol></nav>`),
		"OPS/nav.ncx":            `<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1"><head><meta name="dtb:uid" content="urn:uuid:c3588b17-77a1-40f9-8bc7-4b892b0b60fb"/><meta name="dtb:depth" content="1"/><meta name="dtb:totalPageCount" content="0"/><meta name="dtb:maxPageNumber" content="0"/></head><docTitle><text>試験</text></docTitle><navMap><navPoint id="n1" playOrder="1"><navLabel><text>序</text></navLabel><content src="z.xhtml#front"/></navPoint><navPoint id="n2" playOrder="2"><navLabel><text>第一章</text></navLabel><content src="a.xhtml#c1"/></navPoint><navPoint id="n3" playOrder="3"><navLabel><text>第二章</text></navLabel><content src="end.xhtml#c2"/></navPoint></navMap></ncx>`,
		"OPS/image.svg":          `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="blue"/></svg>`,
	}
}
func XHTML(body string) string {
	return `<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>試験</title></head><body>` + body + `</body></html>`
}
func ZIP(files map[string]string) []byte {
	var b bytes.Buffer
	z := zip.NewWriter(&b)
	keys := []string{}
	for k := range files {
		if k != "mimetype" {
			keys = append(keys, k)
		}
	}
	sort.Strings(keys)
	keys = append([]string{"mimetype"}, keys...)
	for _, k := range keys {
		method := uint16(zip.Deflate)
		if k == "mimetype" {
			method = zip.Store
		}
		w, e := z.CreateHeader(&zip.FileHeader{Name: k, Method: method})
		if e != nil {
			panic(e)
		}
		if _, e = fmt.Fprint(w, files[k]); e != nil {
			panic(e)
		}
	}
	if e := z.Close(); e != nil {
		panic(e)
	}
	return b.Bytes()
}
func EPUB() []byte {
	f := Files()
	f["OPS/a.xhtml"] = XHTML(`<h1 id="c1">第一章</h1><p id="b1">彼は来た。</p><p>前<ruby>名<rt>な</rt></ruby>後<strong>強調</strong>と<a epub:type="noteref" href="notes.xhtml#n1">注</a>。</p>`)
	return ZIP(f)
}

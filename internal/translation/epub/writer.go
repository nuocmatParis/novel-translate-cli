package epub

import (
	"archive/zip"
	"bytes"
	"encoding/xml"
	"errors"
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

const VietnameseCSS = `html,body{writing-mode:horizontal-tb!important;-epub-writing-mode:horizontal-tb!important;font-family:"Liberation Serif",serif;line-height:1.55}body{margin:5%;}p{text-indent:1.2em;margin:.35em 0;text-align:justify;}h1,h2,h3{text-indent:0;text-align:center;}img,svg{max-width:100%;height:auto;}figure{text-align:center;margin:1.5em 0;}a{color:inherit;}[epub\:type=footnote]{font-size:.85em;}
.translation-image-transcript{margin:1.2em auto;padding:.8em 1.2em;border-left:3px solid #6c757d;background:rgba(128,128,128,0.08);font-size:.9em;line-height:1.5;max-width:92%;box-sizing:border-box;}
.translation-image-transcript-title{font-weight:bold;margin-bottom:.4em;text-align:left;}
.translation-image-transcript p{margin:.2em 0;text-indent:0;text-align:left;}`

type ExportOptions struct {
	Preview           bool
	Chapters          []string
	Fonts             map[string][]byte
	Title             string
	ImageReplacements map[string][]byte
	ImageTranscripts  map[string]string
}

func Write(source []byte, volume string, order int, g domain.Generation, opts ExportOptions) ([]byte, error) {
	p, e := Read(source, volume, order)
	if e != nil {
		return nil, e
	}
	if e = p.SelectScope(opts.Chapters); e != nil {
		return nil, e
	}
	if !opts.Preview {
		for _, d := range p.Book.Diagnostics {
			if d.Blocking {
				return nil, fmt.Errorf("nguồn chưa xử lý: %s %s", d.Code, d.Path)
			}
		}
	}
	for resPath, repBytes := range opts.ImageReplacements {
		if _, ok := p.Files[resPath]; ok {
			p.Files[resPath] = repBytes
		}
	}
	for _, b := range p.Book.Blocks {
		n := p.Nodes[b.ID]
		if n == nil {
			return nil, errors.New("mapping node bị thiếu")
		}
		if b.Kind == "image" {
			n.Set("data-source-block", b.ID)
			n.Set("data-source-page", b.Document)
			continue
		}
		u, ok := g.Units[b.ID]
		if !ok {
			if !opts.Preview && b.Disposition == "translate" {
				return nil, fmt.Errorf("khối chưa dịch: %s", b.ID)
			}
			continue
		}
		if u.SourceHash != b.SourceHash {
			return nil, fmt.Errorf("source hash stale: %s", b.ID)
		}
		if !opts.Preview && !u.Accepted() {
			return nil, fmt.Errorf("revision chưa đối chiếu: %s", b.ID)
		}
		if b.Kind == "attribute" {
			n.Set(b.Tag, u.Target)
			n.Set("data-source-attr-"+b.Tag, b.ID)
			continue
		}
		html, e := TokensHTML(translatedTokenAttributes(b, p.Book, g), u.Target)
		if e != nil {
			return nil, e
		}
		fragment, e := Parse([]byte("<wrapper xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" xmlns:xlink=\"http://www.w3.org/1999/xlink\">" + html + "</wrapper>"))
		if e != nil {
			return nil, e
		}
		wrapper := fragment.Find("wrapper")[0]
		if n.Name == "" {
			n.Name = "span"
			n.NS = "http://www.w3.org/1999/xhtml"
			n.Text = ""
		}
		n.Children = wrapper.Children
		n.Set("data-source-block", b.ID)
	}
	if e := writeImageTranscripts(p, opts.ImageTranscripts); e != nil {
		return nil, e
	}
	opf := p.Trees[p.Book.PackagePath]
	repairMetadata(p)
	identifier := "urn:sha256:" + domain.Digest(struct {
		Source, Revision string
		Scope            []string
		Preview          bool
	}{p.Book.SourceHash, g.ID, opts.Chapters, opts.Preview})
	for _, pkg := range opf.Find("package") {
		for _, id := range pkg.Find("identifier") {
			if id.Get("id") == pkg.Get("unique-identifier") {
				id.Children = []*Node{{Text: identifier}}
			}
		}
	}
	for _, meta := range opf.Find("metadata") {
		source := element("meta", map[string]string{"property": "dcterms:source"})
		source.Children = []*Node{{Text: "urn:sha256:" + p.Book.SourceHash}}
		meta.Children = append(meta.Children, source)
	}
	if opts.Title != "" {
		for _, n := range opf.Find("title") {
			n.Children = []*Node{{Text: opts.Title}}
		}
	}
	for _, n := range opf.Find("language") {
		n.Children = []*Node{{Text: "vi"}}
	}
	for _, n := range opf.Find("spine") {
		n.Set("page-progression-direction", "ltr")
	}
	for _, n := range opf.Find("meta") {
		if n.Get("property") == "rendition:orientation" {
			n.Children = []*Node{{Text: "auto"}}
		}
		if n.Get("property") == "dcterms:modified" {
			n.Children = []*Node{{Text: time.Now().UTC().Format("2006-01-02T15:04:05Z")}}
		}
	}
	if opts.Preview {
		for _, n := range opf.Find("title") {
			n.Children = []*Node{{Text: "[BẢN XEM TRƯỚC — CHƯA HOÀN CHỈNH] " + n.AllText()}}
		}
	}
	css := VietnameseCSS
	manifest := opf.Find("manifest")
	if len(manifest) != 1 {
		return nil, errors.New("manifest thiếu")
	}
	for _, item := range manifest[0].Find("item") {
		file, _, e := Resolve(p.Book.PackagePath, item.Get("href"))
		if e != nil {
			continue
		}
		t := p.Trees[file]
		if item.Get("media-type") == "application/xhtml+xml" && t != nil && len(t.Find("svg")) > 0 && !strings.Contains(" "+item.Get("properties")+" ", " svg ") {
			item.Set("properties", strings.TrimSpace(item.Get("properties")+" svg"))
		}
	}
	// EPUB 3 requires every non-linear spine document to be reachable. Source
	// cover pages are retained and linked through a generated landmarks nav.
	for _, d := range p.Book.Documents {
		if !strings.Contains(d.Properties, "nav") {
			continue
		}
		t := p.Trees[d.Path]
		if t == nil {
			continue
		}
		bodies := t.Find("body")
		if len(bodies) == 0 {
			continue
		}
		nav := element("nav", map[string]string{"epub:type": "landmarks", "data-translation-navigation": "landmarks"})
		var existing *Node
		for _, n := range t.Find("nav") {
			if strings.Contains(" "+n.Get("epub:type")+" ", " landmarks ") {
				existing = n
				break
			}
		}
		heading := element("h2", nil)
		heading.Children = []*Node{{Text: "Phần bổ sung"}}
		ol := element("ol", nil)
		for _, target := range p.Book.Documents {
			if target.Spine < 0 || target.Linear {
				continue
			}
			href, _ := relative(d.Path, target.Path)
			li := element("li", map[string]string{"data-translation-navigation": "nonlinear"})
			kind := "bodymatter"
			if strings.Contains(strings.ToLower(target.Path), "cover") {
				kind = "cover"
			}
			alreadyLinked := false
			if existing != nil {
				for _, link := range existing.Find("a") {
					file, _, e := Resolve(d.Path, link.Get("href"))
					if e == nil && file == target.Path && link.Get("epub:type") == kind {
						alreadyLinked = true
						break
					}
				}
			}
			if alreadyLinked {
				continue
			}
			a := element("a", map[string]string{"href": href, "epub:type": kind})
			a.Children = []*Node{{Text: "Trang nguồn"}}
			li.Children = []*Node{a}
			ol.Children = append(ol.Children, li)
		}
		if len(ol.Children) > 0 {
			if existing != nil {
				lists := existing.Find("ol")
				if len(lists) > 0 {
					lists[0].Children = append(lists[0].Children, ol.Children...)
				} else {
					existing.Children = append(existing.Children, ol)
				}
			} else {
				nav.Children = []*Node{heading, ol}
				bodies[0].Children = append(bodies[0].Children, nav)
			}
		}
	}
	fontNames := []string{"LiberationSerif-Regular.ttf", "LiberationSerif-Bold.ttf", "LiberationSerif-Italic.ttf", "LiberationSerif-BoldItalic.ttf"}
	for i, name := range fontNames {
		data, ok := opts.Fonts[name]
		if !ok {
			continue
		}
		file := "translation-fonts/" + name
		p.Files[file] = data
		style := []string{"normal", "normal", "italic", "italic"}[i]
		weight := []string{"normal", "bold", "normal", "bold"}[i]
		css += fmt.Sprintf("@font-face{font-family:'Liberation Serif';font-style:%s;font-weight:%s;src:url('%s')}", style, weight, file)
		href, e := relative(p.Book.PackagePath, file)
		if e != nil {
			return nil, e
		}
		manifest[0].Children = append(manifest[0].Children, element("item", map[string]string{"id": fmt.Sprintf("translation-font-%d", i), "href": href, "media-type": "font/ttf"}))
	}
	cssPath := "translation-vi.css"
	p.Files[cssPath] = []byte(css)
	href, _ := relative(p.Book.PackagePath, cssPath)
	manifest[0].Children = append(manifest[0].Children, element("item", map[string]string{"id": "translation-vi-css", "href": href, "media-type": "text/css"}))
	for _, d := range p.Book.Documents {
		t := p.Trees[d.Path]
		if t == nil {
			continue
		}
		for _, n := range t.Find("html") {
			n.Set("lang", "vi")
			n.Set("xml:lang", "vi")
			n.Set("dir", "ltr")
		}
		for _, head := range t.Find("head") {
			href, e := relative(d.Path, cssPath)
			if e != nil {
				return nil, e
			}
			head.Children = append(head.Children, element("link", map[string]string{"rel": "stylesheet", "type": "text/css", "href": href}))
		}
	}
	// NCX labels use the translated navigation labels when the href matches.
	labels := map[string]string{}
	for _, d := range p.Book.Documents {
		if t := p.Trees[d.Path]; t != nil {
			for _, a := range t.Find("a") {
				f, frag, e := Resolve(d.Path, a.Get("href"))
				if e == nil {
					labels[f+"#"+frag] = a.AllText()
				}
			}
		}
	}
	for _, r := range p.Book.Resources {
		if r.MediaType == "application/x-dtbncx+xml" {
			t, e := Parse(p.Files[r.Path])
			if e != nil {
				return nil, e
			}
			for _, point := range t.Find("navPoint") {
				cs := point.Find("content")
				ls := point.Find("navLabel")
				if len(cs) > 0 && len(ls) > 0 {
					f, frag, e := Resolve(r.Path, cs[0].Get("src"))
					if e == nil {
						if label := labels[f+"#"+frag]; label != "" {
							ts := ls[0].Find("text")
							if len(ts) > 0 {
								ts[0].Children = []*Node{{Text: label}}
							}
						}
					}
				}
			}
			for _, m := range t.Find("meta") {
				if m.Get("name") == "dtb:uid" {
					m.Set("content", identifier)
				}
			}
			p.Trees[r.Path] = t
		}
	}
	for f, t := range p.Trees {
		preserve := false
		for _, resource := range p.Book.Resources {
			if resource.Path == f && strings.HasPrefix(resource.MediaType, "image/") {
				preserve = true
				break
			}
		}
		if preserve {
			continue
		}
		p.Files[f] = Serialize(t)
	}
	var buf bytes.Buffer
	z := zip.NewWriter(&buf)
	w, e := z.CreateHeader(&zip.FileHeader{Name: "mimetype", Method: zip.Store})
	if e != nil {
		return nil, e
	}
	if _, e = w.Write([]byte("application/epub+zip")); e != nil {
		return nil, e
	}
	names := make([]string, 0, len(p.Files))
	for name := range p.Files {
		if name != "mimetype" {
			names = append(names, name)
		}
	}
	sort.Strings(names)
	for _, name := range names {
		w, e = z.CreateHeader(&zip.FileHeader{Name: name, Method: zip.Deflate})
		if e != nil {
			return nil, e
		}
		if _, e = w.Write(p.Files[name]); e != nil {
			return nil, e
		}
	}
	if e = z.Close(); e != nil {
		return nil, e
	}
	return buf.Bytes(), nil
}
func element(name string, attrs map[string]string) *Node {
	n := &Node{Name: name}
	keys := make([]string, 0, len(attrs))
	for k := range attrs {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	for _, k := range keys {
		n.Attr = append(n.Attr, xml.Attr{Name: xml.Name{Local: k}, Value: attrs[k]})
	}
	return n
}
func relative(base, target string) (string, error) {
	// EPUB paths always use '/', independently of the host OS.
	a := strings.Split(base, "/")
	a = a[:len(a)-1]
	b := strings.Split(target, "/")
	i := 0
	for i < len(a) && i < len(b) && a[i] == b[i] {
		i++
	}
	parts := []string{}
	for j := i; j < len(a); j++ {
		parts = append(parts, "..")
	}
	parts = append(parts, b[i:]...)
	return strings.Join(parts, "/"), nil
}

// VerifyOutput reads the actual ZIP, compares stored targets and DOM markers,
// and checks resource bytes plus image/link occurrences in document order.
func VerifyOutput(source, output []byte, volume string, order int, g domain.Generation, options ...ExportOptions) (domain.Report, error) {
	s, e := Read(source, volume, order)
	if e != nil {
		return domain.Report{}, e
	}
	if len(options) > 0 {
		if e = s.SelectScope(options[0].Chapters); e != nil {
			return domain.Report{}, e
		}
	}
	o, e := Read(output, volume, order)
	if e != nil {
		return domain.Report{}, e
	}
	r := domain.Report{SchemaVersion: 1, SourceHash: s.Book.SourceHash, Revision: g.ID, Extraction: s.Book.TextNodes == s.Book.CoveredTextNodes && s.Book.TextAttributes == s.Book.CoveredTextAttributes, Semantic: "chưa đối chiếu", Format: "chưa chạy EPUBCheck", OutputHash: domain.Hash(output)}
	for _, d := range o.Book.Diagnostics {
		if d.Blocking {
			r.Diagnostics = append(r.Diagnostics, d)
		}
	}
	markers := map[string]*Node{}
	attrs := map[string]*Node{}
	for _, d := range o.Book.Documents {
		if t := o.Trees[d.Path]; t != nil {
			t.Walk(func(n *Node) {
				if id := n.Get("data-source-block"); id != "" {
					if markers[id] != nil {
						r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "duplicate-output-id", Detail: id, Blocking: true})
					}
					markers[id] = n
				}
				for _, a := range n.Attr {
					if strings.HasPrefix(a.Name.Local, "data-source-attr-") {
						attrs[a.Value] = n
					}
				}
			})
		}
	}
	for _, b := range s.Book.Blocks {
		if b.Disposition == "translate" {
			r.Eligible++
		}
		u, ok := g.Units[b.ID]
		if !ok {
			continue
		}
		r.Translated++
		if u.Accepted() {
			r.Accepted++
		}
		n := markers[b.ID]
		if b.Kind == "attribute" {
			n = attrs[b.ID]
			if n == nil || n.Get(b.Tag) != u.Target {
				r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "output-target", Detail: b.ID, Blocking: true})
			}
			continue
		}
		if n == nil {
			r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "missing-output-id", Detail: b.ID, Blocking: true})
			continue
		}
		text, tokens, _ := o.inline(n, b.Document)
		expected := translatedTokenAttributes(b, s.Book, g)
		if text != u.Target || !equivalentTokens(tokens, expected.Tokens) {
			r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "output-target", Detail: b.ID, Blocking: true})
		}
	}
	if len(s.Book.Documents) != len(o.Book.Documents) {
		r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "document-count", Detail: "document count thay đổi", Blocking: true})
	} else {
		actual := map[string]domain.Document{}
		for _, d := range o.Book.Documents {
			actual[d.Path] = d
		}
		aux := auxiliarySpine(s)
		for _, d := range s.Book.Documents {
			od, ok := actual[d.Path]
			expectedSpine := d.Spine
			if promoted, exists := aux[d.Path]; exists {
				expectedSpine = promoted
			}
			if !ok || expectedSpine != od.Spine || d.Linear != od.Linear {
				r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "document-order", Detail: d.Path, Blocking: true})
			}
			var expectedIDs []string
			for _, b := range s.Book.Blocks {
				if b.Document != d.Path || b.Kind == "attribute" {
					continue
				}
				if _, ok := g.Units[b.ID]; ok || b.Kind == "image" {
					expectedIDs = append(expectedIDs, b.ID)
				}
			}
			var actualIDs []string
			if t := o.Trees[d.Path]; t != nil {
				t.Walk(func(n *Node) {
					if id := n.Get("data-source-block"); id != "" {
						actualIDs = append(actualIDs, id)
					}
				})
			}
			if domain.Digest(expectedIDs) != domain.Digest(actualIDs) {
				r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "block-order", Detail: d.Path, Blocking: true})
			}
			if domain.Digest(imageSequence(s.Trees[d.Path])) != domain.Digest(imageSequence(o.Trees[d.Path])) {
				r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "image-order", Detail: d.Path, Blocking: true})
			}
			if domain.Digest(linkSequence(s.Trees[d.Path])) != domain.Digest(linkSequence(o.Trees[d.Path])) {
				r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "link-order", Detail: d.Path, Blocking: true})
			}
		}
	}
	for _, res := range s.Book.Resources {
		if strings.HasPrefix(res.MediaType, "image/") || strings.HasPrefix(res.MediaType, "font/") {
			expectedHash := res.Hash
			if len(options) > 0 && options[0].ImageReplacements != nil {
				if rep, ok := options[0].ImageReplacements[res.Path]; ok {
					expectedHash = domain.Hash(rep)
				}
			}
			if domain.Hash(o.Files[res.Path]) != expectedHash {
				r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "resource-hash", Path: res.Path, Detail: "resource bytes thay đổi", Blocking: true})
			}
		}
	}
	if len(options) > 0 {
		r.Diagnostics = append(r.Diagnostics, verifyImageTranscripts(s, o, options[0].ImageTranscripts)...)
	}
	r.Structure = len(r.Diagnostics) == 0
	if r.Accepted == r.Eligible && r.Eligible > 0 {
		r.Semantic = "đã đối chiếu; không bảo đảm đúng nghĩa 100%"
	}
	if !r.Structure {
		return r, errors.New("đối chiếu file EPUB đầu ra thất bại")
	}
	return r, nil
}
func equivalentTokens(a, b []domain.Token) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		delete(a[i].Attributes, "data-source-block")
		for key := range a[i].Attributes {
			if strings.HasPrefix(key, "data-source-attr-") {
				delete(a[i].Attributes, key)
			}
		}
		for key := range b[i].Attributes {
			if strings.HasPrefix(key, "data-source-attr-") {
				delete(b[i].Attributes, key)
			}
		}
		a[i].Locator = ""
		b[i].Locator = ""
		if domain.Digest(a[i]) != domain.Digest(b[i]) {
			return false
		}
	}
	return true
}
func translatedTokenAttributes(b domain.Block, book domain.Book, g domain.Generation) domain.Block {
	copy := b
	copy.Tokens = append([]domain.Token(nil), b.Tokens...)
	for i, t := range copy.Tokens {
		if t.Kind == "close" {
			continue
		}
		attrs := map[string]string{}
		for k, v := range t.Attributes {
			attrs[k] = v
		}
		for _, ab := range book.Blocks {
			if ab.Document != b.Document || ab.Kind != "attribute" || strings.TrimSuffix(ab.Locator, "@"+ab.Tag) != t.Locator {
				continue
			}
			if u, ok := g.Units[ab.ID]; ok {
				attrs[ab.Tag] = u.Target
				attrs["data-source-attr-"+ab.Tag] = ab.ID
			}
		}
		copy.Tokens[i].Attributes = attrs
	}
	return copy
}
func imageSequence(t *Node) []string {
	var out []string
	if t != nil {
		t.Walk(func(n *Node) {
			if local(n.Name) == "img" {
				out = append(out, n.Get("src"))
			}
			if local(n.Name) == "image" {
				out = append(out, n.GetLocal("href"))
			}
		})
	}
	return out
}
func linkSequence(t *Node) []string {
	var out []string
	if t != nil {
		var visit func(*Node)
		visit = func(n *Node) {
			if n.Get("data-translation-navigation") != "" {
				return
			}
			if local(n.Name) == "a" {
				out = append(out, n.Get("href"))
			}
			for _, child := range n.Children {
				visit(child)
			}
		}
		visit(t)
	}
	return out
}

func findPathToNode(root, target *Node) []*Node {
	if root == target {
		return []*Node{root}
	}
	for _, c := range root.Children {
		if path := findPathToNode(c, target); len(path) > 0 {
			return append([]*Node{root}, path...)
		}
	}
	return nil
}

func insertImageTranscript(tree *Node, imgNode *Node, id, transcript string) error {
	if tree == nil || imgNode == nil || strings.TrimSpace(transcript) == "" {
		return errors.New("thiếu mapping hoặc bản dịch chữ trong ảnh: " + id)
	}
	path := findPathToNode(tree, imgNode)
	if len(path) < 2 {
		return errors.New("không tìm thấy vị trí ảnh: " + id)
	}
	var container *Node
	var containerChild *Node
	for i := len(path) - 2; i >= 0; i-- {
		tag := local(path[i].Name)
		switch tag {
		case "body", "div", "section", "article", "figure", "li", "aside", "td", "th", "blockquote":
			container = path[i]
			containerChild = path[i+1]
			break
		}
		if container != nil {
			break
		}
	}
	if container == nil {
		return errors.New("không có container XHTML cho chú thích ảnh: " + id)
	}
	name := func(tag string) string {
		if i := strings.IndexByte(container.Name, ':'); i >= 0 {
			return container.Name[:i+1] + tag
		}
		return tag
	}

	div := &Node{
		Name: name("div"), NS: container.NS,
		Attr: []xml.Attr{{Name: xml.Name{Local: "class"}, Value: "translation-image-transcript"}, {Name: xml.Name{Local: "data-source-image-transcript"}, Value: id}},
	}
	titleNode := &Node{
		Name: name("div"), NS: container.NS,
		Attr:     []xml.Attr{{Name: xml.Name{Local: "class"}, Value: "translation-image-transcript-title"}},
		Children: []*Node{{Text: "Bản dịch chữ trong tranh:"}},
	}
	div.Children = append(div.Children, titleNode)
	lines := strings.Split(transcript, "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		p := &Node{
			Name: name("p"), NS: container.NS,
			Children: []*Node{{Text: line}},
		}
		div.Children = append(div.Children, p)
	}

	idx := -1
	for i, c := range container.Children {
		if c == containerChild {
			idx = i
			break
		}
	}
	if idx >= 0 {
		// Several inline images can share one paragraph. Keep their captions in
		// source order rather than repeatedly inserting at the same position.
		for idx+1 < len(container.Children) && container.Children[idx+1].Get("data-source-image-transcript") != "" {
			idx++
		}
		newChildren := make([]*Node, 0, len(container.Children)+1)
		newChildren = append(newChildren, container.Children[:idx+1]...)
		newChildren = append(newChildren, div)
		newChildren = append(newChildren, container.Children[idx+1:]...)
		container.Children = newChildren
		return nil
	}
	return errors.New("không tìm thấy container child cho ảnh: " + id)
}

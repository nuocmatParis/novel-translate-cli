package epub

import (
	"archive/zip"
	"bytes"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"net/url"
	"path"
	"regexp"
	"sort"
	"strconv"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

const ParserVersion = "epub-xml-v3"

var generatedTextProperty = regexp.MustCompile(`(?i)\bcontent\s*:`)

type Package struct {
	Book  domain.Book
	Files map[string][]byte
	Trees map[string]*Node
	Nodes map[string]*Node
}

func Resolve(base, href string) (string, string, error) {
	u, e := url.Parse(href)
	if e != nil {
		return "", "", e
	}
	if u.IsAbs() || u.Host != "" {
		return "", "", fmt.Errorf("external URI: %s", href)
	}
	decoded, e := url.PathUnescape(u.EscapedPath())
	if e != nil {
		return "", "", e
	}
	if strings.Contains(decoded, "\\") || strings.HasPrefix(decoded, "/") {
		return "", "", errors.New("EPUB href tuyệt đối/backslash")
	}
	p := base
	if decoded != "" {
		p = path.Join(path.Dir(base), decoded)
	}
	if p == ".." || strings.HasPrefix(p, "../") {
		return "", "", errors.New("EPUB href thoát root")
	}
	return p, u.Fragment, nil
}
func Read(data []byte, volume string, order int) (*Package, error) {
	z, e := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if e != nil {
		return nil, e
	}
	if len(z.File) > 10000 {
		return nil, errors.New("EPUB quá nhiều ZIP entries")
	}
	p := &Package{Files: map[string][]byte{}, Trees: map[string]*Node{}, Nodes: map[string]*Node{}}
	p.Book = domain.Book{SchemaVersion: 1, ParserVersion: ParserVersion, SourceHash: domain.Hash(data), ZIPEntries: len(z.File)}
	var total uint64
	for _, f := range z.File {
		if f.FileInfo().IsDir() {
			continue
		}
		n := f.Name
		if n != path.Clean(n) || strings.Contains(n, "\\") || strings.HasPrefix(n, "/") || strings.HasPrefix(n, "../") || strings.Contains(n, ":") {
			return nil, fmt.Errorf("ZIP path không an toàn: %s", n)
		}
		if _, ok := p.Files[n]; ok {
			return nil, fmt.Errorf("ZIP entry trùng: %s", n)
		}
		total += f.UncompressedSize64
		if f.UncompressedSize64 > 32<<20 || total > 512<<20 {
			return nil, errors.New("EPUB vượt giới hạn size")
		}
		r, e := f.Open()
		if e != nil {
			return nil, e
		}
		b, e := io.ReadAll(io.LimitReader(r, 32<<20+1))
		r.Close()
		if e != nil {
			return nil, e
		}
		if len(b) > 32<<20 {
			return nil, errors.New("ZIP entry vượt giới hạn")
		}
		p.Files[n] = b
	}
	if string(p.Files["mimetype"]) != "application/epub+zip" {
		return nil, errors.New("mimetype EPUB thiếu/sai")
	}
	if len(p.Files["META-INF/encryption.xml"]) > 0 {
		return nil, errors.New("EPUB mã hóa/obfuscated cần xử lý riêng")
	}
	container, e := Parse(p.Files["META-INF/container.xml"])
	if e != nil {
		return nil, fmt.Errorf("container: %w", e)
	}
	roots := container.Find("rootfile")
	containers := container.Find("container")
	if len(containers) != 1 || containers[0].NS != "urn:oasis:names:tc:opendocument:xmlns:container" {
		return nil, errors.New("container namespace không hợp lệ")
	}
	if len(roots) != 1 {
		return nil, errors.New("EPUB phải có đúng một rootfile")
	}
	if roots[0].NS != containers[0].NS {
		return nil, errors.New("rootfile namespace không hợp lệ")
	}
	opf := roots[0].Get("full-path")
	if opf != path.Clean(opf) || strings.HasPrefix(opf, "../") {
		return nil, errors.New("package path không hợp lệ")
	}
	tree, e := Parse(p.Files[opf])
	if e != nil {
		return nil, fmt.Errorf("OPF: %w", e)
	}
	p.Trees[opf] = tree
	p.Book.PackagePath = opf
	pk := tree.Find("package")
	if len(pk) != 1 || pk[0].NS != "http://www.idpf.org/2007/opf" {
		return nil, errors.New("OPF namespace không hợp lệ")
	}
	for _, tag := range []string{"title", "creator", "language"} {
		ns := tree.Find(tag)
		if len(ns) > 0 {
			switch tag {
			case "title":
				p.Book.Title = ns[0].AllText()
			case "creator":
				p.Book.Creator = ns[0].AllText()
			case "language":
				p.Book.Language = ns[0].AllText()
			}
		}
	}
	byID := map[string]domain.Resource{}
	manifestPaths := map[string]bool{}
	for _, n := range tree.Find("item") {
		if n.NS != "http://www.idpf.org/2007/opf" {
			return nil, errors.New("manifest item namespace không hợp lệ")
		}
		r := domain.Resource{ID: n.Get("id"), MediaType: n.Get("media-type"), Properties: n.Get("properties")}
		r.Path, _, e = Resolve(opf, n.Get("href"))
		if e != nil {
			return nil, e
		}
		if r.ID == "" || byID[r.ID].ID != "" || manifestPaths[r.Path] {
			return nil, errors.New("manifest ID/path trùng hoặc thiếu")
		}
		b, ok := p.Files[r.Path]
		if !ok {
			p.diag("missing-resource", r.Path, "manifest không có file ZIP", true)
		}
		r.Hash = domain.Hash(b)
		byID[r.ID] = r
		manifestPaths[r.Path] = true
		p.Book.Resources = append(p.Book.Resources, r)
	}
	spines := tree.Find("spine")
	if len(spines) != 1 {
		return nil, errors.New("OPF spine thiếu/trùng")
	}
	spinePaths := map[string]bool{}
	for i, n := range spines[0].Find("itemref") {
		r, ok := byID[n.Get("idref")]
		if !ok {
			return nil, errors.New("spine idref không tồn tại")
		}
		if spinePaths[r.Path] {
			return nil, errors.New("spine document lặp chưa được hỗ trợ")
		}
		spinePaths[r.Path] = true
		d := domain.Document{Path: r.Path, Spine: i, Linear: n.Get("linear") != "no", Properties: n.Get("properties")}
		if r.MediaType != "application/xhtml+xml" {
			p.diag("unsupported-spine", r.Path, r.MediaType, true)
		}
		p.Book.Documents = append(p.Book.Documents, d)
	}
	for _, r := range p.Book.Resources {
		if (r.MediaType == "application/xhtml+xml" || r.MediaType == "image/svg+xml") && !spinePaths[r.Path] {
			p.Book.Documents = append(p.Book.Documents, domain.Document{Path: r.Path, Spine: -1, Linear: false, Properties: r.Properties})
		}
	}
	// Parse every declared document, including navigation and nonlinear notes.
	for _, d := range p.Book.Documents {
		b, ok := p.Files[d.Path]
		if !ok {
			continue
		}
		t, e := Parse(b)
		if e != nil {
			return nil, fmt.Errorf("%s: %w", d.Path, e)
		}
		p.Trees[d.Path] = t
		unknownNS := map[string]bool{}
		t.Walk(func(n *Node) {
			if n.Name != "" && n.Name != "#root" && n.NS != "http://www.w3.org/1999/xhtml" && n.NS != "http://www.w3.org/2000/svg" && !unknownNS[n.NS] {
				unknownNS[n.NS] = true
				p.diag("unclassified-namespace", d.Path, n.NS+": cần phân loại nội dung", true)
			}
			for _, a := range n.Attr {
				if a.Name.Space == "" && strings.HasPrefix(strings.ToLower(a.Name.Local), "on") {
					p.diag("script", d.Path, "event handler "+a.Name.Local+" chưa được hỗ trợ", true)
				}
			}
			if strings.HasPrefix(strings.ToLower(strings.TrimSpace(n.GetLocal("href"))), "javascript:") {
				p.diag("script", d.Path, "javascript URI chưa được hỗ trợ", true)
			}
		})
		if len(t.Find("script")) > 0 {
			p.diag("script", d.Path, "scripted content chưa được hỗ trợ", true)
		}
		for _, tag := range []string{"math", "audio", "video", "iframe", "object", "embed", "input", "select", "textarea", "canvas"} {
			if len(t.Find(tag)) > 0 {
				p.diag("unsupported-content", d.Path, tag+" cần workflow xử lý riêng", true)
			}
		}
	}
	// Independent inventory counts every significant text node before extraction.
	for _, d := range p.Book.Documents {
		if t := p.Trees[d.Path]; t != nil {
			t.Walk(func(n *Node) {
				for _, a := range []string{"alt", "title", "aria-label"} {
					if n.Get(a) != "" {
						p.Book.TextAttributes++
					}
				}
				if n.Name == "" && strings.TrimSpace(n.Text) != "" {
					p.Book.TextNodes++
				}
			})
		}
	}
	boundaries := p.boundaries(opf, byID)
	current := ""
	chapterIndex := 0
	for i := range p.Book.Documents {
		d := &p.Book.Documents[i]
		t := p.Trees[d.Path]
		if t == nil {
			continue
		}
		if d.Spine < 0 {
			current = ""
		}
		if current == "" {
			chapterIndex++
			current = fmt.Sprintf("%s-c%03d", volume, chapterIndex)
			p.Book.Chapters = append(p.Book.Chapters, domain.Chapter{ID: current, Order: chapterIndex, Title: d.Path, Kind: "auxiliary"})
			if d.Spine >= 0 {
				p.Book.Chapters[len(p.Book.Chapters)-1].Kind = "front-matter"
			}
		}
		var walk func(*Node, bool)
		walk = func(n *Node, hidden bool) {
			tag := local(n.Name)
			hidden = hidden || n.Get("hidden") != "" || strings.Contains(strings.ReplaceAll(n.Get("style"), " ", ""), "display:none")
			key := d.Path + "#" + n.Get("id")
			if title, ok := boundaries[key]; ok && (n.Get("id") != "" || tag == "body") {
				chapterIndex++
				current = fmt.Sprintf("%s-c%03d", volume, chapterIndex)
				p.Book.Chapters = append(p.Book.Chapters, domain.Chapter{ID: current, Order: chapterIndex, Title: title, Kind: chapterKind(title)})
			}
			if tag == "body" {
				if title, ok := boundaries[d.Path+"#"]; ok {
					last := &p.Book.Chapters[len(p.Book.Chapters)-1]
					if len(last.BlockIDs) == 0 {
						last.Title = title
						last.Kind = chapterKind(title)
					} else {
						chapterIndex++
						current = fmt.Sprintf("%s-c%03d", volume, chapterIndex)
						p.Book.Chapters = append(p.Book.Chapters, domain.Chapter{ID: current, Order: chapterIndex, Title: title, Kind: chapterKind(title)})
					}
				}
			}
			if tag == "style" || tag == "script" {
				countText(n, &p.Book.CoveredTextNodes)
				if tag == "style" && generatedTextProperty.MatchString(n.AllText()) {
					p.diag("generated-text", d.Path, "CSS generated content cần xác minh", true)
				}
				return
			}
			if tag == "img" || tag == "image" {
				b := p.block(n, *d, volume, order, current, "image")
				href := n.Get("src")
				if tag == "image" {
					href = n.GetLocal("href")
				}
				b.Resource, _, e = Resolve(d.Path, href)
				if e != nil {
					p.diag("image-href", d.Path, e.Error(), true)
				}
				if _, ok := p.Files[b.Resource]; !ok {
					p.diag("missing-image", d.Path, b.Resource, true)
				}
				b.Disposition = "preserve"
				b.Reason = "bytes hình được bảo toàn; chữ trong hình chưa được dịch"
				p.diag("image-text-candidate", d.Path, b.ID, false)
				p.add(b, n, d)
				p.addAttributes(n, d, volume, order, current)
				return
			}
			if n.Name == "" {
				if strings.TrimSpace(n.Text) != "" {
					b := p.block(n, *d, volume, order, current, "text")
					b.Source = n.Text
					b.SourceHash = domain.Hash([]byte(n.Text))
					if hidden {
						b.Disposition = "unresolved"
						p.diag("hidden-text", d.Path, b.ID, true)
					}
					p.add(b, n, d)
					p.Book.CoveredTextNodes++
				}
				return
			}
			if isTextBlock(tag) {
				b := p.block(n, *d, volume, order, current, "text")
				b.Source, b.Tokens, b.Ruby = p.inline(n, d.Path)
				b.SourceHash = domain.Hash([]byte(b.Source))
				if hidden {
					b.Disposition = "unresolved"
					p.diag("hidden-text", d.Path, b.ID, true)
				}
				if tag == "text" {
					b.Disposition = "unresolved"
					p.diag("svg-text", d.Path, b.ID, true)
				}
				if strings.TrimSpace(b.Source) != "" {
					p.add(b, n, d)
				}
				countText(n, &p.Book.CoveredTextNodes)
				n.Walk(func(x *Node) { p.addAttributes(x, d, volume, order, current) })
				return
			}
			p.addAttributes(n, d, volume, order, current)
			for _, c := range n.Children {
				walk(c, hidden)
			}
		}
		walk(t, false)
	}
	var chapters []domain.Chapter
	for _, c := range p.Book.Chapters {
		if len(c.BlockIDs) > 0 {
			c.Order = len(chapters) + 1
			chapters = append(chapters, c)
		}
	}
	p.Book.Chapters = chapters
	for i := range p.Book.Blocks {
		for _, c := range chapters {
			if c.ID == p.Book.Blocks[i].Chapter {
				p.Book.Blocks[i].Position.Chapter = c.Order
				break
			}
		}
	}
	var extras []string
	for n := range p.Files {
		if !manifestPaths[n] && n != opf && n != "mimetype" && !strings.HasPrefix(n, "META-INF/") {
			extras = append(extras, n)
		}
	}
	sort.Strings(extras)
	for _, n := range extras {
		p.diag("zip-extra", n, "ngoài manifest; chưa phân loại", true)
	}
	for _, r := range p.Book.Resources {
		if r.MediaType == "text/css" && generatedTextProperty.Match(p.Files[r.Path]) {
			p.diag("generated-text", r.Path, "CSS generated content cần xác minh", true)
		}
	}
	if p.Book.TextNodes != p.Book.CoveredTextNodes || p.Book.TextAttributes != p.Book.CoveredTextAttributes {
		p.diag("extraction-coverage", "", "số text nodes trích xuất không khớp", true)
	}
	if len(p.Book.Blocks) == 0 {
		p.diag("no-text", "", "EPUB không có nội dung đọc được", true)
	}
	p.checkLinks()
	sort.Slice(p.Book.Diagnostics, func(i, j int) bool {
		a, b := p.Book.Diagnostics[i], p.Book.Diagnostics[j]
		return a.Path+"|"+a.Code+"|"+a.Detail < b.Path+"|"+b.Code+"|"+b.Detail
	})
	return p, nil
}
func isTextBlock(t string) bool {
	switch t {
	case "p", "h1", "h2", "h3", "h4", "h5", "h6", "title", "li", "dt", "dd", "figcaption", "caption", "td", "th", "text":
		return true
	}
	return false
}

func chapterKind(title string) string {
	title = strings.ToLower(strings.TrimSpace(title))
	for _, prefix := range []string{"プロローグ", "prologue", "序章", "序", "前書き", "mở đầu"} {
		if strings.HasPrefix(title, prefix) {
			return "front-matter"
		}
	}
	for _, prefix := range []string{"エピローグ", "epilogue", "あとがき", "後書き", "lời bạt"} {
		if strings.HasPrefix(title, prefix) {
			return "back-matter"
		}
	}
	for _, prefix := range []string{"目次", "もくじ", "contents", "奥付", "copyright", "mục lục"} {
		if strings.HasPrefix(title, prefix) {
			return "auxiliary"
		}
	}
	return "chapter"
}
func countText(n *Node, count *int) {
	n.Walk(func(c *Node) {
		if c.Name == "" && strings.TrimSpace(c.Text) != "" {
			*count++
		}
	})
}
func (p *Package) diag(code, file, detail string, blocking bool) {
	p.Book.Diagnostics = append(p.Book.Diagnostics, domain.Diagnostic{Code: code, Path: file, Detail: detail, Blocking: blocking})
}
func (p *Package) block(n *Node, d domain.Document, volume string, order int, chapter, kind string) domain.Block {
	id := volume + ":" + domain.Hash([]byte(d.Path + "|" + n.Locator + "|" + p.Book.SourceHash))[:24]
	return domain.Block{ID: id, Document: d.Path, Locator: n.Locator, Sequence: len(p.Book.Blocks), Chapter: chapter, Kind: kind, Tag: local(n.Name), Anchor: n.Get("id"), Disposition: "translate", Position: domain.Position{Volume: order, Block: len(p.Book.Blocks)}}
}
func (p *Package) addAttributes(n *Node, d *domain.Document, volume string, order int, chapter string) {
	for _, tag := range []string{"alt", "title", "aria-label"} {
		if value := n.Get(tag); value != "" {
			b := p.block(n, *d, volume, order, chapter, "attribute")
			b.ID += "-" + tag
			b.Locator += "@" + tag
			b.Tag = tag
			b.Source = value
			b.SourceHash = domain.Hash([]byte(value))
			p.add(b, n, d)
			p.Book.CoveredTextAttributes++
		}
	}
}
func (p *Package) add(b domain.Block, n *Node, d *domain.Document) {
	p.Book.Blocks = append(p.Book.Blocks, b)
	p.Nodes[b.ID] = n
	d.BlockIDs = append(d.BlockIDs, b.ID)
	for i := len(p.Book.Chapters) - 1; i >= 0; i-- {
		if p.Book.Chapters[i].ID == b.Chapter {
			p.Book.Chapters[i].BlockIDs = append(p.Book.Chapters[i].BlockIDs, b.ID)
			break
		}
	}
}
func (p *Package) inline(n *Node, file string) (string, []domain.Token, []domain.Ruby) {
	var out strings.Builder
	var tokens []domain.Token
	var ruby []domain.Ruby
	var walk func(*Node)
	walk = func(c *Node) {
		if c.Name == "" {
			out.WriteString(c.Text)
			return
		}
		tag := local(c.Name)
		if tag == "rt" || tag == "rp" {
			return
		}
		if tag == "ruby" {
			base := strings.Builder{}
			for _, x := range c.Children {
				if local(x.Name) != "rt" && local(x.Name) != "rp" {
					base.WriteString(x.AllText())
				}
			}
			reading := ""
			for _, r := range c.Find("rt") {
				reading += r.AllText()
			}
			index := len(ruby)
			ruby = append(ruby, domain.Ruby{Base: base.String(), Reading: reading, Policy: "review-required", Start: out.Len()})
			for _, x := range c.Children {
				walk(x)
			}
			ruby[index].End = out.Len()
			return
		}
		id := "t" + strconv.Itoa(len(tokens))
		attr := map[string]string{}
		for _, a := range c.Attr {
			attr[qname(a.Name)] = a.Value
		}
		if tag == "img" || tag == "image" {
			href := c.Get("src")
			if tag == "image" {
				href = c.GetLocal("href")
			}
			rp, _, err := Resolve(file, href)
			if err != nil {
				p.diag("image-href", file, err.Error(), true)
			} else if _, ok := p.Files[rp]; !ok {
				p.diag("missing-image", file, rp, true)
			}
			p.diag("inline-image-text-candidate", file, rp, false)
		}
		if tag == "br" || tag == "img" || tag == "image" {
			tokens = append(tokens, domain.Token{ID: id, Kind: "void", Name: c.Name, Attributes: attr, Locator: c.Locator})
			out.WriteString("⟦" + id + "⟧")
			return
		}
		open := id
		tokens = append(tokens, domain.Token{ID: open, Kind: "open", Name: c.Name, Attributes: attr, Locator: c.Locator})
		out.WriteString("⟦" + open + "⟧")
		for _, x := range c.Children {
			walk(x)
		}
		close := "t" + strconv.Itoa(len(tokens))
		tokens = append(tokens, domain.Token{ID: close, Kind: "close", Name: c.Name, Pair: open})
		tokensIndex := 0
		for i, t := range tokens {
			if t.ID == open {
				tokensIndex = i
				break
			}
		}
		tokens[tokensIndex].Pair = close
		out.WriteString("⟦" + close + "⟧")
	}
	for _, c := range n.Children {
		walk(c)
	}
	return out.String(), tokens, ruby
}
func (p *Package) boundaries(opf string, byID map[string]domain.Resource) map[string]string {
	out := map[string]string{}
	for _, r := range p.Book.Resources {
		if r.MediaType == "application/x-dtbncx+xml" {
			t, e := Parse(p.Files[r.Path])
			if e != nil {
				p.diag("ncx", r.Path, e.Error(), true)
				continue
			}
			for _, m := range t.Find("navMap") {
				for _, n := range m.Children {
					if local(n.Name) != "navPoint" {
						continue
					}
					cs := n.Find("content")
					ls := n.Find("navLabel")
					if len(cs) == 0 || len(ls) == 0 {
						continue
					}
					f, a, e := Resolve(r.Path, cs[0].Get("src"))
					if e != nil {
						p.diag("ncx-href", r.Path, e.Error(), true)
						continue
					}
					out[f+"#"+a] = strings.TrimSpace(ls[0].AllText())
				}
			}
		}
	}
	if len(out) > 0 {
		return out
	}
	for _, r := range p.Book.Resources {
		if strings.Contains(r.Properties, "nav") {
			t := p.Trees[r.Path]
			if t == nil {
				continue
			}
			for _, nav := range t.Find("nav") {
				if nav.GetLocal("type") != "toc" {
					continue
				}
				lists := nav.Find("ol")
				if len(lists) == 0 {
					continue
				}
				for _, li := range lists[0].Children {
					if local(li.Name) != "li" {
						continue
					}
					as := li.Find("a")
					if len(as) == 0 {
						continue
					}
					f, a, e := Resolve(r.Path, as[0].Get("href"))
					if e == nil {
						out[f+"#"+a] = strings.TrimSpace(as[0].AllText())
					}
				}
			}
		}
	}
	return out
}
func (p *Package) checkLinks() {
	anchors := map[string]map[string]bool{}
	for f, t := range p.Trees {
		anchors[f] = map[string]bool{}
		t.Walk(func(n *Node) {
			if id := n.Get("id"); id != "" {
				if anchors[f][id] {
					p.diag("duplicate-anchor", f, id, true)
				}
				anchors[f][id] = true
			}
		})
	}
	for f, t := range p.Trees {
		t.Walk(func(n *Node) {
			for _, a := range n.Attr {
				if a.Name.Local != "href" && a.Name.Local != "src" {
					continue
				}
				u, e := url.Parse(a.Value)
				if e != nil {
					p.diag("invalid-link", f, a.Value, true)
					continue
				}
				if u.IsAbs() || u.Host != "" {
					continue
				}
				target, fragment, e := Resolve(f, a.Value)
				if e != nil {
					p.diag("invalid-link", f, a.Value, true)
					continue
				}
				if _, ok := p.Files[target]; !ok {
					p.diag("broken-link", f, a.Value, true)
				} else if fragment != "" && anchors[target] != nil && !anchors[target][fragment] {
					p.diag("broken-anchor", f, a.Value, true)
				}
			}
		})
	}
}

// TokensHTML accepts each known token exactly once, in its original nesting/order.
// This conservative policy preserves link/footnote targets during translation.
func TokensHTML(source domain.Block, target string) (string, error) {
	var out strings.Builder
	cursor := 0
	for _, t := range source.Tokens {
		mark := "⟦" + t.ID + "⟧"
		i := strings.Index(target[cursor:], mark)
		if i < 0 || strings.Count(target, mark) != 1 {
			return "", fmt.Errorf("inline token %s thiếu/trùng", t.ID)
		}
		before := target[cursor : cursor+i]
		if strings.Contains(before, "⟦") || strings.Contains(before, "⟧") {
			return "", errors.New("inline token lạ/đảo thứ tự")
		}
		out.WriteString(Escape(before))
		switch t.Kind {
		case "open", "void":
			out.WriteString("<" + t.Name)
			keys := make([]string, 0, len(t.Attributes))
			for k := range t.Attributes {
				keys = append(keys, k)
			}
			sort.Strings(keys)
			for _, k := range keys {
				out.WriteString(" " + k + "=\"" + Escape(t.Attributes[k]) + "\"")
			}
			if t.Kind == "void" {
				out.WriteString(" />")
			} else {
				out.WriteByte('>')
			}
		case "close":
			out.WriteString("</" + t.Name + ">")
		}
		cursor += i + len(mark)
	}
	tail := target[cursor:]
	if strings.ContainsAny(tail, "⟦⟧") {
		return "", errors.New("inline token lạ")
	}
	out.WriteString(Escape(tail))
	return out.String(), nil
}

// Keep encoding/xml in the reader's dependency set for namespace-safe DTOs.
var _ xml.Name

package epub

import (
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"strings"
)

// repairMetadata preserves malformed source metadata as opaque legacy values
// when its intended target cannot be proven. Narrative/resource data is intact.
func repairMetadata(p *Package) []domain.Diagnostic {
	opf := p.Trees[p.Book.PackagePath]
	var repairs []domain.Diagnostic
	record := func(code, before, after string) {
		repairs = append(repairs, domain.Diagnostic{Code: code, Path: p.Book.PackagePath, Detail: before + " → " + after})
	}
	for file, t := range p.Trees {
		t.Walk(func(n *Node) {
			for i, a := range n.Attr {
				if (a.Name.Space == "xmlns" || a.Name.Local == "xmlns") && a.Value == "http://www.idpf.org/2011/epub" {
					n.Attr[i].Value = "http://www.idpf.org/2007/ops"
					repairs = append(repairs, domain.Diagnostic{Code: "source-epub-namespace", Path: file, Detail: a.Value + " → http://www.idpf.org/2007/ops"})
				}
			}
		})
	}
	ids := map[string]bool{}
	opf.Walk(func(n *Node) {
		if id := n.Get("id"); id != "" {
			ids[id] = true
		}
	})
	for _, m := range opf.Find("meta") {
		ref := strings.TrimPrefix(m.Get("refines"), "#")
		if ref == "" || ids[ref] {
			continue
		}
		before := m.XML()
		resolved := false
		if strings.HasPrefix(ref, "creator") {
			creators := opf.Find("creator")
			if len(creators) == 1 && creators[0].Get("id") != "" {
				duplicate := false
				for _, existing := range opf.Find("meta") {
					if existing != m && existing.Get("refines") == "#"+creators[0].Get("id") && existing.Get("property") == m.Get("property") {
						duplicate = true
					}
				}
				if !duplicate {
					m.Set("refines", "#"+creators[0].Get("id"))
					resolved = true
				}
			}
		}
		if !resolved {
			m.Attr = nil
			m.Children = nil
			m.Set("name", "translation-source-refinement")
			m.Set("content", before)
		}
		record("source-metadata-refinement", before, m.XML())
	}
	for _, m := range opf.Find("meta") {
		if m.Get("name") == "primary-writing-mode" {
			before := m.Get("content")
			m.Set("content", "horizontal-ltr")
			record("writing-mode", before, m.Get("content"))
		}
	}
	for _, spine := range opf.Find("spine") {
		for _, item := range spine.Find("itemref") {
			before := item.Get("properties")
			var kept []string
			for _, property := range strings.Fields(before) {
				if strings.HasPrefix(property, "access:") || strings.HasPrefix(property, "rendition:layout-") || strings.HasPrefix(property, "rendition:page-spread-") || property == "rendition:spread-none" {
					continue
				}
				kept = append(kept, property)
			}
			kept = append(kept, "rendition:layout-reflowable")
			after := strings.Join(kept, " ")
			item.Set("properties", after)
			if before != after {
				record("source-layout-to-reflowable", item.Get("idref")+": "+before, after)
			}
		}
	}
	for _, manifest := range opf.Find("manifest") {
		for _, item := range manifest.Find("item") {
			file, _, e := Resolve(p.Book.PackagePath, item.Get("href"))
			if e != nil {
				continue
			}
			t := p.Trees[file]
			if item.Get("media-type") == "image/svg+xml" && strings.Contains(" "+item.Get("properties")+" ", " svg ") {
				before := item.Get("properties")
				var kept []string
				for _, prop := range strings.Fields(before) {
					if prop != "svg" {
						kept = append(kept, prop)
					}
				}
				item.Set("properties", strings.Join(kept, " "))
				record("source-svg-resource-property", item.Get("id")+": "+before, item.Get("properties"))
			}
			if item.Get("media-type") == "application/xhtml+xml" && t != nil && len(t.Find("svg")) > 0 && !strings.Contains(" "+item.Get("properties")+" ", " svg ") {
				before := item.Get("properties")
				item.Set("properties", strings.TrimSpace(before+" svg"))
				record("source-svg-manifest-property", item.Get("id")+": "+before, item.Get("properties"))
			}
		}
	}
	aux := auxiliarySpine(p)
	for _, d := range p.Book.Documents {
		if _, ok := aux[d.Path]; !ok {
			continue
		}
		for _, item := range opf.Find("item") {
			file, _, e := Resolve(p.Book.PackagePath, item.Get("href"))
			if e != nil || file != d.Path {
				continue
			}
			ref := element("itemref", map[string]string{"idref": item.Get("id"), "linear": "no"})
			for _, spine := range opf.Find("spine") {
				spine.Children = append(spine.Children, ref)
			}
			record("linked-auxiliary-spine", d.Path+": non-spine", "nonlinear spine item")
		}
	}
	return repairs
}
func MetadataRepairs(source []byte, volume string, order int, scope []string) ([]domain.Diagnostic, error) {
	p, e := Read(source, volume, order)
	if e != nil {
		return nil, e
	}
	if e = p.SelectScope(scope); e != nil {
		return nil, e
	}
	return repairMetadata(p), nil
}

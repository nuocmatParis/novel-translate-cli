package epub

import (
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

// Cover returns the image explicitly identified as the book cover by EPUB
// metadata. It does not guess from the first image in the package.
func (p *Package) Cover() (domain.Resource, []byte, bool) {
	if p == nil || p.Book.PackagePath == "" {
		return domain.Resource{}, nil, false
	}

	byID := make(map[string]domain.Resource, len(p.Book.Resources))
	byPath := make(map[string]domain.Resource, len(p.Book.Resources))
	for _, resource := range p.Book.Resources {
		byID[resource.ID] = resource
		byPath[resource.Path] = resource
		if isImage(resource) && hasProperty(resource.Properties, "cover-image") {
			return coverBytes(p, resource)
		}
	}

	opf := p.Trees[p.Book.PackagePath]
	if opf == nil {
		return domain.Resource{}, nil, false
	}
	for _, meta := range opf.Find("meta") {
		if strings.EqualFold(strings.TrimSpace(meta.Get("name")), "cover") {
			if resource, ok := byID[strings.TrimSpace(meta.Get("content"))]; ok && isImage(resource) {
				return coverBytes(p, resource)
			}
		}
	}

	// EPUB 2 identifies a cover document through guide/reference[type=cover].
	// Follow only its declared image, and accept it only when it resolves to a
	// manifest image resource.
	for _, reference := range opf.Find("reference") {
		if !strings.EqualFold(strings.TrimSpace(reference.Get("type")), "cover") {
			continue
		}
		document, _, err := Resolve(p.Book.PackagePath, reference.Get("href"))
		if err != nil {
			continue
		}
		tree := p.Trees[document]
		if tree == nil {
			continue
		}
		var candidate domain.Resource
		ambiguous := false
		tree.Walk(func(node *Node) {
			if node.Name != "img" && node.Name != "image" {
				return
			}
			href := node.Get("src")
			if href == "" {
				href = node.GetLocal("href")
			}
			resourcePath, _, err := Resolve(document, href)
			if err != nil {
				return
			}
			resource, ok := byPath[resourcePath]
			if !ok || !isImage(resource) {
				return
			}
			if candidate.Path != "" && candidate.Path != resource.Path {
				ambiguous = true
				return
			}
			candidate = resource
		})
		if !ambiguous && candidate.Path != "" {
			return coverBytes(p, candidate)
		}
	}

	return domain.Resource{}, nil, false
}

func hasProperty(properties, wanted string) bool {
	for _, property := range strings.Fields(properties) {
		if property == wanted {
			return true
		}
	}
	return false
}

func isImage(resource domain.Resource) bool {
	return strings.HasPrefix(strings.ToLower(resource.MediaType), "image/")
}

func coverBytes(p *Package, resource domain.Resource) (domain.Resource, []byte, bool) {
	data, ok := p.Files[resource.Path]
	if !ok || len(data) == 0 {
		return domain.Resource{}, nil, false
	}
	return resource, data, true
}

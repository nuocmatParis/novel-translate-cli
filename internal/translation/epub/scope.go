package epub

import (
	"errors"
	"fmt"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

// SelectScope keeps exact source nodes in the selected logical chapters, their
// referenced non-spine notes, document metadata and matching navigation entries.
// A link to omitted narrative is rejected rather than silently broken.
func (p *Package) SelectScope(ids []string) error {
	if len(ids) == 0 {
		return nil
	}
	selected := map[string]bool{}
	for _, id := range ids {
		found := false
		for _, ch := range p.Book.Chapters {
			if ch.ID == id {
				found = true
			}
		}
		if !found {
			return fmt.Errorf("chapter không tồn tại: %s", id)
		}
		selected[id] = true
	}
	keep := map[string]bool{}
	docs := map[string]domain.Document{}
	for _, d := range p.Book.Documents {
		docs[d.Path] = d
	}
	for _, b := range p.Book.Blocks {
		if selected[b.Chapter] {
			keep[b.ID] = true
		}
	}
	// Close note references, including backlinks. Their narrative destination must
	// already be in scope; pulling arbitrary omitted chapters into a range is unsafe.
	for changed := true; changed; {
		changed = false
		for _, b := range p.Book.Blocks {
			if !keep[b.ID] {
				continue
			}
			n := p.Nodes[b.ID]
			if n == nil {
				return errors.New("scope source mapping thiếu")
			}
			for _, a := range n.Find("a") {
				file, fragment, e := Resolve(b.Document, a.Get("href"))
				if e != nil {
					continue
				} // external links are preserved
				d, isDoc := docs[file]
				if !isDoc {
					continue
				}
				var targets []*Node
				if fragment == "" {
					targets = p.Trees[file].Find("body")
				} else {
					p.Trees[file].Walk(func(x *Node) {
						if x.Get("id") == fragment {
							targets = append(targets, x)
						}
					})
				}
				matched := false
				for _, target := range targets {
					for _, nb := range p.Book.Blocks {
						if nb.Document != file || nb.Tag == "title" {
							continue
						}
						x := p.Nodes[nb.ID]
						if !within(x, target) && !within(target, x) {
							continue
						}
						if d.Spine >= 0 && !keep[nb.ID] {
							return fmt.Errorf("link range trỏ tới phần truyện bị loại: %s#%s", file, fragment)
						}
						matched = true
						if !keep[nb.ID] {
							keep[nb.ID] = true
							changed = true
						}
					}
				}
				if !matched {
					return fmt.Errorf("không ánh xạ được link phạm vi: %s#%s", file, fragment)
				}
			}
		}
	}
	keptDocs := map[string]bool{}
	for _, r := range p.Book.Resources {
		if r.MediaType == "image/svg+xml" {
			keptDocs[r.Path] = true
		}
	}
	for _, b := range p.Book.Blocks {
		if keep[b.ID] {
			keptDocs[b.Document] = true
		}
	}
	// Navigation nodes are retained only when all destinations survive.
	for _, d := range p.Book.Documents {
		if !strings.Contains(d.Properties, "nav") {
			continue
		}
		keptDocs[d.Path] = true
		for _, b := range p.Book.Blocks {
			if b.Document != d.Path {
				continue
			}
			links := p.Nodes[b.ID].Find("a")
			if len(links) == 0 {
				continue
			}
			valid := true
			for _, a := range links {
				file, fragment, e := Resolve(d.Path, a.Get("href"))
				if e != nil || !keptDocs[file] {
					valid = false
					break
				}
				if fragment != "" {
					found := false
					for _, nb := range p.Book.Blocks {
						if !keep[nb.ID] || nb.Document != file {
							continue
						}
						x := p.Nodes[nb.ID]
						x.Walk(func(n *Node) {
							if n.Get("id") == fragment {
								found = true
							}
						})
						if nb.Anchor == fragment {
							found = true
						}
					}
					if !found {
						valid = false
						break
					}
				}
			}
			if valid {
				keep[b.ID] = true
			}
		}
	}
	for _, b := range p.Book.Blocks {
		if keptDocs[b.Document] && b.Tag == "title" && b.Kind == "text" {
			keep[b.ID] = true
		}
	}
	remove := map[*Node]bool{}
	for _, b := range p.Book.Blocks {
		if !keep[b.ID] && b.Kind != "attribute" {
			remove[p.Nodes[b.ID]] = true
		}
	}
	var prune func(*Node)
	prune = func(n *Node) {
		children := n.Children[:0]
		for _, x := range n.Children {
			if remove[x] {
				continue
			}
			prune(x)
			children = append(children, x)
		}
		n.Children = children
	}
	for f, t := range p.Trees {
		if keptDocs[f] {
			prune(t)
		}
	}
	removedIDs := map[string]bool{}
	for _, res := range p.Book.Resources {
		if res.MediaType == "application/xhtml+xml" && !keptDocs[res.Path] {
			removedIDs[res.ID] = true
			delete(p.Trees, res.Path)
			delete(p.Files, res.Path)
		}
	}
	opf := p.Trees[p.Book.PackagePath]
	for _, manifest := range opf.Find("manifest") {
		children := manifest.Children[:0]
		for _, n := range manifest.Children {
			if local(n.Name) == "item" && removedIDs[n.Get("id")] {
				continue
			}
			children = append(children, n)
		}
		manifest.Children = children
	}
	for _, spine := range opf.Find("spine") {
		children := spine.Children[:0]
		for _, n := range spine.Children {
			if local(n.Name) == "itemref" && removedIDs[n.Get("idref")] {
				continue
			}
			children = append(children, n)
		}
		spine.Children = children
	}
	for _, guide := range opf.Find("guide") {
		children := guide.Children[:0]
		for _, n := range guide.Children {
			if local(n.Name) == "reference" {
				file, _, e := Resolve(p.Book.PackagePath, n.Get("href"))
				if e != nil || !keptDocs[file] {
					continue
				}
			}
			children = append(children, n)
		}
		guide.Children = children
	}
	var removeEmpty func(*Node)
	removeEmpty = func(n *Node) {
		children := n.Children[:0]
		for _, child := range n.Children {
			removeEmpty(child)
			if local(child.Name) == "guide" && len(child.Find("reference")) == 0 {
				continue
			}
			if local(child.Name) == "nav" && len(child.Find("li")) == 0 {
				continue
			}
			children = append(children, child)
		}
		n.Children = children
	}
	removeEmpty(opf)
	for f, t := range p.Trees {
		if keptDocs[f] {
			removeEmpty(t)
		}
	}
	for _, res := range p.Book.Resources {
		if res.MediaType != "application/x-dtbncx+xml" {
			continue
		}
		t, e := Parse(p.Files[res.Path])
		if e != nil {
			return e
		}
		var trim func(*Node)
		trim = func(n *Node) {
			children := n.Children[:0]
			for _, x := range n.Children {
				if local(x.Name) == "navPoint" {
					cs := x.Find("content")
					if len(cs) == 0 {
						continue
					}
					f, frag, e := Resolve(res.Path, cs[0].Get("src"))
					valid := e == nil && keptDocs[f]
					if valid && frag != "" {
						valid = false
						p.Trees[f].Walk(func(n *Node) {
							if n.Get("id") == frag {
								valid = true
							}
						})
					}
					if !valid {
						continue
					}
				}
				trim(x)
				children = append(children, x)
			}
			n.Children = children
		}
		trim(t)
		for i, n := range t.Find("navPoint") {
			n.Set("playOrder", fmt.Sprint(i+1))
		}
		p.Trees[res.Path] = t
		p.Files[res.Path] = Serialize(t)
	}
	book := p.Book
	book.Blocks = nil
	book.Chapters = nil
	book.Documents = nil
	book.Resources = nil
	for _, b := range p.Book.Blocks {
		if keep[b.ID] {
			book.Blocks = append(book.Blocks, b)
		}
	}
	for _, ch := range p.Book.Chapters {
		var blocks []string
		for _, id := range ch.BlockIDs {
			if keep[id] {
				blocks = append(blocks, id)
			}
		}
		if len(blocks) > 0 {
			ch.BlockIDs = blocks
			book.Chapters = append(book.Chapters, ch)
		}
	}
	spine := 0
	for _, d := range p.Book.Documents {
		if !keptDocs[d.Path] {
			continue
		}
		if d.Spine >= 0 {
			d.Spine = spine
			spine++
		}
		d.BlockIDs = nil
		for _, b := range book.Blocks {
			if b.Document == d.Path {
				d.BlockIDs = append(d.BlockIDs, b.ID)
			}
		}
		book.Documents = append(book.Documents, d)
	}
	for _, r := range p.Book.Resources {
		if !removedIDs[r.ID] {
			book.Resources = append(book.Resources, r)
		}
	}
	p.Book = book
	return nil
}
func within(child, parent *Node) bool {
	return child != nil && parent != nil && (child == parent || strings.HasPrefix(child.Locator, parent.Locator+"/"))
}

package epub

import (
	"fmt"
	"strings"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
)

type imageOccurrence struct {
	id, document string
	node         *Node
}

func imageOccurrences(p *Package) []imageOccurrence {
	var images []imageOccurrence
	for _, b := range p.Book.Blocks {
		n := p.Nodes[b.ID]
		if b.Kind == "image" {
			images = append(images, imageOccurrence{b.ID, b.Document, n})
			continue
		}
		if b.Kind == "attribute" || n == nil {
			continue
		}
		var nodes []*Node
		n.Walk(func(child *Node) {
			if tag := local(child.Name); tag == "img" || tag == "image" {
				nodes = append(nodes, child)
			}
		})
		index := 0
		for _, token := range b.Tokens {
			if token.Kind != "void" || (local(token.Name) != "img" && local(token.Name) != "image") {
				continue
			}
			var image *Node
			if index < len(nodes) {
				// Insert outside the translated owner (p/li/td/...) so a caption
				// cannot become part of its target text during readback.
				image = n
			}
			images = append(images, imageOccurrence{b.ID + ":" + token.ID, b.Document, image})
			index++
		}
	}
	return images
}

func writeImageTranscripts(p *Package, transcripts map[string]string) error {
	seen := map[string]bool{}
	for _, image := range imageOccurrences(p) {
		text, ok := transcripts[image.id]
		if !ok {
			continue
		}
		if err := insertImageTranscript(p.Trees[image.document], image.node, image.id, text); err != nil {
			return err
		}
		seen[image.id] = true
	}
	for id := range transcripts {
		if !seen[id] {
			return fmt.Errorf("không tìm thấy image occurrence trong phạm vi xuất: %s", id)
		}
	}
	return nil
}

func transcriptLines(text string) string {
	var lines []string
	for _, line := range strings.Split(text, "\n") {
		if line = strings.TrimSpace(line); line != "" {
			lines = append(lines, line)
		}
	}
	return strings.Join(lines, "\n")
}

func verifyImageTranscripts(source, output *Package, expected map[string]string) []domain.Diagnostic {
	documents := map[string]string{}
	for _, image := range imageOccurrences(source) {
		documents[image.id] = image.document
	}
	seen := map[string]int{}
	var issues []domain.Diagnostic
	for _, document := range output.Book.Documents {
		tree := output.Trees[document.Path]
		if tree == nil {
			continue
		}
		tree.Walk(func(n *Node) {
			id := n.Get("data-source-image-transcript")
			if id == "" {
				return
			}
			seen[id]++
			var lines []string
			for _, paragraph := range n.Find("p") {
				lines = append(lines, paragraph.AllText())
			}
			text, ok := expected[id]
			if !ok || documents[id] != document.Path || transcriptLines(strings.Join(lines, "\n")) != transcriptLines(text) {
				issues = append(issues, domain.Diagnostic{Code: "image-transcript-content", Path: document.Path, Detail: id, Blocking: true})
			}
		})
	}
	for id := range expected {
		if seen[id] != 1 {
			issues = append(issues, domain.Diagnostic{Code: "image-transcript-count", Path: documents[id], Detail: id, Blocking: true})
		}
	}
	return issues
}

package epub

import "strings"

// Referenced XHTML outside the main spine stays nonlinear, but EPUB requires it
// to be declared in the spine so readers can navigate to footnotes/backlinks.
func auxiliarySpine(p *Package) map[string]int {
	ids := map[string]int{}
	eligible := map[string]bool{}
	next := 0
	for _, d := range p.Book.Documents {
		if d.Spine >= 0 {
			next++
		}
	}
	for _, r := range p.Book.Resources {
		if r.MediaType == "application/xhtml+xml" && !strings.Contains(" "+r.Properties+" ", " nav ") {
			eligible[r.Path] = true
		}
	}
	linked := map[string]bool{}
	for _, d := range p.Book.Documents {
		if t := p.Trees[d.Path]; t != nil {
			for _, a := range t.Find("a") {
				file, _, e := Resolve(d.Path, a.Get("href"))
				if e == nil {
					linked[file] = true
				}
			}
		}
	}
	for _, d := range p.Book.Documents {
		if d.Spine < 0 && eligible[d.Path] && linked[d.Path] {
			ids[d.Path] = next
			next++
		}
	}
	return ids
}

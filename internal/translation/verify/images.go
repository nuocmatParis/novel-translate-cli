package verify

import (
	"errors"
	"fmt"
	"net/http"
	"strings"
	"unicode/utf8"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
)

type Image struct {
	ID       string         `json:"id"`
	Resource string         `json:"resource"`
	Hash     string         `json:"hash"`
	Sequence int            `json:"sequence"`
	Decision *ImageDecision `json:"decision,omitempty"`
}
type ImageDecision struct {
	ID              string `json:"id"`
	SourceHash      string `json:"source_hash"`
	ResourceHash    string `json:"resource_hash"`
	Disposition     string `json:"disposition"`
	Reason          string `json:"reason"`
	Replacement     string `json:"replacement,omitempty"`
	ReplacementHash string `json:"replacement_hash,omitempty"`
	Transcript      string `json:"transcript,omitempty"`
}

func Images(book domain.Book) []Image {
	resources := map[string]string{}
	for _, r := range book.Resources {
		resources[r.Path] = r.Hash
	}
	var out []Image
	for _, b := range book.Blocks {
		if b.Kind == "image" {
			out = append(out, Image{ID: b.ID, Resource: b.Resource, Hash: resources[b.Resource], Sequence: b.Sequence})
		}
		for _, t := range b.Tokens {
			name := t.Name[strings.LastIndex(t.Name, ":")+1:]
			if t.Kind == "void" && (name == "img" || name == "image") {
				href := t.Attributes["src"]
				if href == "" {
					href = t.Attributes["href"]
				}
				if href == "" {
					href = t.Attributes["xlink:href"]
				}
				p, _, e := epub.Resolve(b.Document, href)
				if e == nil {
					out = append(out, Image{ID: b.ID + ":" + t.ID, Resource: p, Hash: resources[p], Sequence: b.Sequence})
				}
			}
		}
	}
	return out
}
func ImageGate(book domain.Book, decisions map[string]ImageDecision) []domain.Diagnostic {
	var issues []domain.Diagnostic
	for _, image := range Images(book) {
		d, ok := decisions[image.ID]
		if !ok || d.ID != image.ID || d.SourceHash != book.SourceHash || d.ResourceHash != image.Hash || strings.TrimSpace(d.Reason) == "" {
			issues = append(issues, domain.Diagnostic{Code: "image-text-unresolved", Path: image.Resource, Detail: image.ID + ": cần đánh giá chữ trong hình", Blocking: true})
			continue
		}
		switch d.Disposition {
		case "no-text", "keep-original":
			// Hợp lệ: đã kiểm tra và xác nhận
		case "replace":
			if strings.TrimSpace(d.Replacement) == "" || strings.TrimSpace(d.ReplacementHash) == "" {
				issues = append(issues, domain.Diagnostic{Code: "image-replace-missing", Path: image.Resource, Detail: image.ID + ": thiếu file ảnh thay thế", Blocking: true})
			}
		case "transcript":
			if strings.TrimSpace(d.Transcript) == "" {
				issues = append(issues, domain.Diagnostic{Code: "image-transcript-empty", Path: image.Resource, Detail: image.ID + ": thiếu nội dung transcript bản dịch", Blocking: true})
			}
		default:
			issues = append(issues, domain.Diagnostic{Code: "image-disposition-invalid", Path: image.Resource, Detail: image.ID + ": disposition không hợp lệ (" + d.Disposition + ")", Blocking: true})
		}
	}
	return issues
}
func AssessImage(book domain.Book, id, disposition, reason, replacement, transcript string, replacementBytes []byte) (ImageDecision, error) {
	reason = strings.TrimSpace(reason)
	if reason == "" {
		return ImageDecision{}, errors.New("cần ghi rõ lý do/bằng chứng đánh giá (--reason)")
	}
	disposition = strings.TrimSpace(disposition)
	switch disposition {
	case "no-text", "keep-original":
		// Hợp lệ
	case "replace":
		if strings.TrimSpace(replacement) == "" || len(replacementBytes) == 0 {
			return ImageDecision{}, errors.New("disposition replace cần đường dẫn file ảnh thay thế (--file)")
		}
	case "transcript":
		transcript = strings.TrimPrefix(transcript, "\uFEFF")
		if strings.TrimSpace(transcript) == "" {
			return ImageDecision{}, errors.New("disposition transcript cần nội dung dịch chữ trong ảnh (--transcript hoặc --transcript-file)")
		}
		if !utf8.ValidString(transcript) {
			return ImageDecision{}, errors.New("bản dịch chữ trong ảnh phải là UTF-8")
		}
		for _, r := range transcript {
			if r < 0x20 && r != '\n' && r != '\r' && r != '\t' || r == 0xfffe || r == 0xffff {
				return ImageDecision{}, errors.New("bản dịch chữ trong ảnh chứa ký tự XML không hợp lệ")
			}
		}
	default:
		return ImageDecision{}, errors.New("disposition phải là no-text, keep-original, replace hoặc transcript")
	}
	for _, image := range Images(book) {
		if image.ID == id {
			repHash := ""
			if len(replacementBytes) > 0 {
				repHash = domain.Hash(replacementBytes)
			}
			return ImageDecision{
				ID:              id,
				SourceHash:      book.SourceHash,
				ResourceHash:    image.Hash,
				Disposition:     disposition,
				Reason:          reason,
				Replacement:     replacement,
				ReplacementHash: repHash,
				Transcript:      transcript,
			}, nil
		}
	}
	return ImageDecision{}, errors.New("image occurrence ID không tồn tại")
}

// ImageAssets checks the bytes approved by the user, including shared resource
// decisions. A replacement affects every occurrence of its EPUB resource.
func ImageAssets(book domain.Book, decisions map[string]ImageDecision, read func(string) ([]byte, error)) (map[string][]byte, map[string]string, []domain.Diagnostic) {
	replacements := map[string][]byte{}
	transcripts := map[string]string{}
	issues := ImageGate(book, decisions)
	media := map[string]string{}
	for _, resource := range book.Resources {
		media[resource.Path] = resource.MediaType
	}
	choices := map[string]string{}
	conflicts := map[string]bool{}
	for _, image := range Images(book) {
		d, ok := decisions[image.ID]
		if !ok || d.ID != image.ID || d.SourceHash != book.SourceHash || d.ResourceHash != image.Hash {
			continue
		}
		choice := "original"
		if d.Disposition == "replace" {
			choice = d.ReplacementHash
		}
		if previous, exists := choices[image.Resource]; exists && previous != choice {
			conflicts[image.Resource] = true
			issues = append(issues, domain.Diagnostic{Code: "image-resource-conflict", Path: image.Resource, Detail: "Các lần xuất hiện dùng chung tài nguyên cần thống nhất ảnh gốc hoặc cùng bản thay thế", Blocking: true})
		}
		choices[image.Resource] = choice
		switch d.Disposition {
		case "replace":
			if d.Replacement == "" || d.ReplacementHash == "" {
				continue
			}
			data, err := read(d.Replacement)
			if err == nil && domain.Hash(data) != d.ReplacementHash {
				err = errors.New("ảnh thay thế đã thay đổi; cần image assess lại")
			}
			if err == nil {
				err = validateImageFormat(data, media[image.Resource])
			}
			if err != nil {
				issues = append(issues, domain.Diagnostic{Code: "image-replacement-invalid", Path: image.Resource, Detail: image.ID + ": " + err.Error(), Blocking: true})
				continue
			}
			replacements[image.Resource] = data
		case "transcript":
			if strings.TrimSpace(d.Transcript) != "" {
				transcripts[image.ID] = d.Transcript
			}
		}
	}
	for path := range conflicts {
		delete(replacements, path)
	}
	return replacements, transcripts, issues
}

func validateImageFormat(data []byte, expected string) error {
	if expected == "" {
		return nil
	}
	actual := http.DetectContentType(data)
	if expected == "image/svg+xml" {
		tree, err := epub.Parse(data)
		if err == nil && len(tree.Children) == 1 && (tree.Children[0].Name == "svg" || strings.HasSuffix(tree.Children[0].Name, ":svg")) && tree.Children[0].NS == "http://www.w3.org/2000/svg" {
			return nil
		}
	} else if actual == expected {
		return nil
	}
	return fmt.Errorf("ảnh thay thế phải cùng định dạng %s với nguồn (nhận %s)", expected, actual)
}

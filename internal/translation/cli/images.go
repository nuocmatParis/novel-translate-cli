package cli

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/verify"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func exportImages(out io.Writer, c workspace.Config, book domain.Book, raw []byte, directory string) error {
	p, err := epub.Read(raw, "images", 1)
	if err != nil {
		return err
	}
	type exported struct {
		verify.Image
		File string `json:"file"`
	}
	images := []exported{}
	for _, image := range verify.Images(book) {
		data, ok := p.Files[image.Resource]
		if !ok || domain.Hash(data) != image.Hash {
			return fmt.Errorf("tài nguyên ảnh không khớp: %s", image.Resource)
		}
		path, err := c.WritePath(filepath.Join(directory, image.Hash+"-"+filepath.Base(image.Resource)))
		if err != nil {
			return err
		}
		existing, readErr := os.ReadFile(path)
		if readErr == nil && !bytes.Equal(existing, data) {
			return fmt.Errorf("không ghi đè ảnh đã chỉnh sửa: %s; chọn --directory khác", path)
		}
		if readErr != nil {
			if !errors.Is(readErr, os.ErrNotExist) {
				return readErr
			}
			if err = state.Atomic(path, data); err != nil {
				return err
			}
		}
		images = append(images, exported{Image: image, File: path})
	}
	return printJSON(out, images)
}

package verify

import (
	"context"
	"errors"
	"fmt"
	"os/exec"
	"path/filepath"
	"strings"
)

func EPUBCheck(ctx context.Context, checker, book string) (string, error) {
	if checker == "" {
		return "chưa cấu hình EPUBCheck", errors.New("cần output.epubcheck: executable hoặc epubcheck.jar")
	}
	var cmd *exec.Cmd
	absoluteBook, e := filepath.Abs(book)
	if e != nil {
		return "", e
	}
	argument := filepath.Base(absoluteBook)
	if strings.HasSuffix(strings.ToLower(checker), ".jar") {
		checker, e = filepath.Abs(checker)
		if e != nil {
			return "", e
		}
		cmd = exec.CommandContext(ctx, "java", "-Dfile.encoding=UTF-8", "-jar", checker, argument)
	} else {
		if strings.ContainsAny(checker, "/\\") {
			checker, e = filepath.Abs(checker)
			if e != nil {
				return "", e
			}
		}
		cmd = exec.CommandContext(ctx, checker, argument)
	}
	// JDKs on Windows can encode argv using the active ANSI code page. Starting
	// in the export directory avoids corrupting Unicode workspace names in argv.
	cmd.Dir = filepath.Dir(absoluteBook)
	b, e := cmd.CombinedOutput()
	if e != nil {
		return string(b), fmt.Errorf("EPUBCheck: %w", e)
	}
	return string(b), nil
}

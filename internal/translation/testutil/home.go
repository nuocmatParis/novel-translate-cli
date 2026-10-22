package testutil

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// IsolatedHome prevents tests from creating or editing real shared user skills.
func IsolatedHome(m *testing.M) int {
	base, err := filepath.Abs(os.TempDir())
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return 1
	}
	dir, err := os.MkdirTemp(base, "novel-translate-test-home-")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return 1
	}
	defer func() {
		rel, err := filepath.Rel(base, dir)
		if err == nil && rel != "." && rel != ".." && !strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
			_ = os.RemoveAll(dir)
		}
	}()
	for _, name := range []string{"HOME", "USERPROFILE"} {
		old, ok := os.LookupEnv(name)
		defer func(name, old string, ok bool) {
			if ok {
				_ = os.Setenv(name, old)
			} else {
				_ = os.Unsetenv(name)
			}
		}(name, old, ok)
		if err := os.Setenv(name, dir); err != nil {
			fmt.Fprintln(os.Stderr, err)
			return 1
		}
	}
	return m.Run()
}

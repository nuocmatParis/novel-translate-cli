//go:build !windows

package state

import (
	"os"
	"path/filepath"
)

func replace(from, to string) error {
	if e := os.Rename(from, to); e != nil {
		return e
	}
	f, e := os.Open(filepath.Dir(to))
	if e != nil {
		return e
	}
	defer f.Close()
	return f.Sync()
}

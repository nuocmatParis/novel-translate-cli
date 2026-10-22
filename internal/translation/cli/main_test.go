package cli

import (
	"github.com/voocel/ainovel-cli/internal/translation/testutil"
	"os"
	"testing"
)

func TestMain(m *testing.M) { os.Exit(testutil.IsolatedHome(m)) }

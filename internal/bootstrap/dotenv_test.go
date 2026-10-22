package bootstrap

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestDotEnvCredentialPrecedenceAndWorkspaceIsolation(t *testing.T) {
	const name = "NOVEL_TRANSLATE_TEST_DOTENV_KEY"
	t.Setenv(name, "")
	cwd, provider, first, second := t.TempDir(), t.TempDir(), t.TempDir(), t.TempDir()
	t.Chdir(cwd)
	write := func(dir, value string) {
		t.Helper()
		if err := os.WriteFile(filepath.Join(dir, ".env"), []byte("\xef\xbb\xbf# saved locally\n"+name+"='"+value+"'\n"), 0600); err != nil {
			t.Fatal(err)
		}
	}
	write(cwd, "cwd-key")
	write(provider, "provider-key")
	write(first, "first-$key#value")
	write(second, "second-key")
	config := filepath.Join(provider, "config.json")
	if err := os.WriteFile(config, []byte(`{"provider":"test","model":"fake","providers":{"test":{"type":"openai","api_key_env":"`+name+`","models":["fake"]}}}`), 0600); err != nil {
		t.Fatal(err)
	}
	check := func(root, want string) {
		t.Helper()
		c, err := LoadConfigFile(config, root)
		if err != nil || c.DefaultProviderConfig().APIKey != want {
			t.Fatalf("wrong resolved key or error: %v", err)
		}
	}
	check(first, "first-$key#value")
	check(second, "second-key")
	if os.Getenv(name) != "" {
		t.Fatal("workspace key leaked into process environment")
	}
	t.Setenv(name, "process-key")
	check(first, "process-key")
	t.Setenv(name, "")
	check(t.TempDir(), "provider-key")
	if err := os.Remove(filepath.Join(provider, ".env")); err != nil {
		t.Fatal(err)
	}
	check(t.TempDir(), "cwd-key")
}

func TestDotEnvParseErrorsDoNotExposeSecrets(t *testing.T) {
	path := filepath.Join(t.TempDir(), ".env")
	const secret = "fake-secret-should-never-appear"
	if err := os.WriteFile(path, []byte("KEY='"+secret), 0600); err != nil {
		t.Fatal(err)
	}
	_, err := ReadDotEnv(path)
	if err == nil || strings.Contains(err.Error(), secret) {
		t.Fatal("missing or unredacted dotenv error")
	}
}

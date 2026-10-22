package bootstrap

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/joho/godotenv"
)

// DefaultConfigPath is a read-only credential fallback. Workspace configuration
// takes precedence and never depends on the command's current directory.
func DefaultConfigPath() string {
	home, e := os.UserHomeDir()
	if e != nil {
		return ""
	}
	return filepath.Join(home, ".novel-translate", "config.json")
}

// ReadConfigFile parses configuration without resolving credentials.
func ReadConfigFile(path string) (Config, error) {
	var c Config
	b, e := os.ReadFile(path)
	if e != nil {
		return c, e
	}
	// Windows editors may save UTF-8 with a BOM. It is not part of JSON data.
	b = bytes.TrimPrefix(b, []byte{0xef, 0xbb, 0xbf})
	d := json.NewDecoder(bytes.NewReader(stripJSONComments(b)))
	d.DisallowUnknownFields()
	if e = d.Decode(&c); e != nil {
		return c, fmt.Errorf("provider config: %w", e)
	}
	var extra any
	if e = d.Decode(&extra); e != io.EOF {
		return c, fmt.Errorf("provider config cần một JSON object")
	}
	return c, nil
}

// LoadConfigFile resolves keys without changing os.Environ. Explicit workspace
// directories precede the provider directory and the current directory.
func LoadConfigFile(path string, envDirectories ...string) (Config, error) {
	c, err := ReadConfigFile(path)
	if err != nil {
		return c, err
	}
	for name, p := range c.Providers {
		if p.APIKeyEnv != "" {
			if p.APIKey != "" {
				return c, fmt.Errorf("provider %s chỉ dùng api_key hoặc api_key_env", name)
			}
			p.APIKey, err = LookupAPIKey(p.APIKeyEnv, append(envDirectories, filepath.Dir(path))...)
			if err != nil {
				return c, err
			}
			if p.APIKey == "" {
				return c, fmt.Errorf("thiếu %s cho provider %s; nhập key trong TUI hoặc lưu vào .env", p.APIKeyEnv, name)
			}
			c.Providers[name] = p
		}
	}
	return c, nil
}

func ReadDotEnv(path string) (map[string]string, error) {
	data, err := os.ReadFile(path)
	if errors.Is(err, os.ErrNotExist) {
		return map[string]string{}, nil
	}
	if err != nil {
		return nil, fmt.Errorf("không đọc được .env: %s", path)
	}
	data = bytes.TrimPrefix(data, []byte{0xef, 0xbb, 0xbf})
	values, err := godotenv.Unmarshal(string(data))
	if err != nil {
		// Parser errors can contain an entire secret-bearing line.
		return nil, fmt.Errorf("cú pháp .env không hợp lệ: %s", path)
	}
	return values, nil
}

func LookupAPIKey(name string, directories ...string) (string, error) {
	if value := os.Getenv(name); value != "" {
		return value, nil
	}
	if cwd, err := os.Getwd(); err == nil {
		directories = append(directories, cwd)
	}
	seen := map[string]bool{}
	for _, directory := range directories {
		path, err := filepath.Abs(filepath.Join(directory, ".env"))
		if err != nil || seen[path] {
			continue
		}
		seen[path] = true
		values, err := ReadDotEnv(path)
		if err != nil {
			return "", err
		}
		if value := values[name]; value != "" {
			return value, nil
		}
	}
	return "", nil
}
func cloneMap(m map[string]any) map[string]any {
	if m == nil {
		return nil
	}
	copy := map[string]any{}
	for k, v := range m {
		copy[k] = v
	}
	return copy
}
func CloneConfig(c Config) Config {
	copy := c
	copy.Providers = map[string]ProviderConfig{}
	for k, p := range c.Providers {
		p.Models = append([]ModelConfig(nil), p.Models...)
		p.Extra = cloneMap(p.Extra)
		p.ExtraBody = cloneMap(p.ExtraBody)
		copy.Providers[k] = p
	}
	copy.Roles = map[string]RoleConfig{}
	for k, r := range c.Roles {
		r.Fallbacks = append([]ModelRef(nil), r.Fallbacks...)
		copy.Roles[k] = r
	}
	return copy
}
func stripJSONComments(data []byte) []byte {
	out := []byte{}
	quoted, escape := false, false
	for i := 0; i < len(data); i++ {
		b := data[i]
		if escape {
			out = append(out, b)
			escape = false
			continue
		}
		if quoted {
			out = append(out, b)
			if b == '\\' {
				escape = true
			} else if b == '"' {
				quoted = false
			}
			continue
		}
		if b == '"' {
			quoted = true
			out = append(out, b)
			continue
		}
		if b == '/' && i+1 < len(data) && data[i+1] == '/' {
			for i < len(data) && data[i] != '\n' {
				i++
			}
			if i < len(data) {
				out = append(out, '\n')
			}
			continue
		}
		out = append(out, b)
	}
	return out
}

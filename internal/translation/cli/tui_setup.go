package cli

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"slices"
	"strconv"
	"strings"

	"github.com/joho/godotenv"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

func providerPath(c workspace.Config) string {
	if c.ProviderConfig != "" {
		return c.ReadPath(c.ProviderConfig)
	}
	return bootstrap.DefaultConfigPath()
}

func (m *translationTUI) saveModelSetup(values []string) func(context.Context, io.Writer, io.Writer) error {
	root := m.config.Root
	return func(ctx context.Context, out, progress io.Writer) error {
		if len(values) != modelContextField+1 {
			return errors.New("thiếu thông tin thiết lập model")
		}
		unlock, err := (state.Store{Root: root}).Lock(ctx)
		if err != nil {
			return err
		}
		defer unlock()
		c, err := workspace.Load(root)
		if err != nil {
			return err
		}
		protocol, modelName, baseURL := strings.TrimSpace(values[modelProviderField]), strings.TrimSpace(values[modelVersionField]), strings.TrimSpace(values[modelURLField])
		if !slices.Contains(modelProviders, protocol) {
			return errors.New("nhà cung cấp: OpenAI, Gemini, DeepSeek, Claude, OpenRouter hoặc Ollama")
		}
		if modelName == "" {
			return errors.New("cần tên model")
		}
		effort := strings.TrimSpace(values[modelEffortField])
		if !slices.Contains(modelEfforts(protocol, modelName), effort) {
			return errors.New("mức suy luận không được hỗ trợ cho model này; chọn lại hoặc dùng mặc định API")
		}
		window, err := strconv.Atoi(strings.TrimSpace(values[modelContextField]))
		if err != nil || window < 0 {
			return errors.New("context tokens phải là số >= 0")
		}
		keyName := "NOVEL_TRANSLATE_API_KEY"
		old, _ := bootstrap.ReadConfigFile(providerPath(c))
		pc := old.DefaultProviderConfig()
		oldType, _ := pc.ProviderType(old.Provider)
		sameProvider := oldType == protocol
		if sameProvider && pc.APIKeyEnv != "" {
			keyName = pc.APIKeyEnv
		}
		if old.Provider != "" && !sameProvider {
			keyName = "NOVEL_TRANSLATE_" + strings.ToUpper(protocol) + "_API_KEY"
		}
		key := strings.TrimSpace(values[modelKeyField])
		if key == "" && sameProvider {
			key = pc.APIKey
		}
		if key == "" {
			key, err = bootstrap.LookupAPIKey(keyName, root, filepath.Dir(providerPath(c)))
			if err != nil {
				return err
			}
		}
		if key == "" && protocol != "ollama" {
			return errors.New("nhập API key hoặc lưu key vào .env trước")
		}
		pc = bootstrap.ProviderConfig{Type: protocol, BaseURL: baseURL, Models: []bootstrap.ModelConfig{{Name: modelName, ContextWindow: window}}}
		if protocol == "openai" {
			pc.API = strings.TrimSpace(values[modelAPIField])
		}
		if key != "" {
			pc.APIKeyEnv = keyName
		}
		cfg := bootstrap.Config{Provider: protocol, ModelName: modelName, Providers: map[string]bootstrap.ProviderConfig{protocol: pc}, ReasoningEffort: effort, Budget: old.Budget}
		// Validate using a private copy, never persist the resolved key in JSON.
		validated := bootstrap.CloneConfig(cfg)
		target := validated.Providers[cfg.Provider]
		target.APIKey = key
		validated.Providers[cfg.Provider] = target
		if err := validated.ValidateBase(); err != nil {
			return err
		}
		data, err := json.MarshalIndent(cfg, "", "  ")
		if err != nil {
			return err
		}
		configPath, err := c.WritePath(filepath.Join(".translation_work", "provider.json"))
		if err != nil {
			return err
		}
		envPath, err := c.WritePath(".env")
		if err != nil {
			return err
		}
		writes := []localWrite{}
		if key != "" {
			values, err := bootstrap.ReadDotEnv(envPath)
			if err != nil {
				return err
			}
			values[keyName] = key
			text, err := godotenv.Marshal(values)
			if err != nil {
				return errors.New("không thể tạo .env")
			}
			writes = append(writes, localWrite{path: envPath, data: []byte(text + "\n")})
		}
		writes = append(writes, localWrite{path: configPath, data: append(data, '\n')})
		c.ProviderConfig = filepath.ToSlash(filepath.Join(".translation_work", "provider.json"))
		if err := c.Validate(); err != nil {
			return err
		}
		if err := saveLocalFiles(writes, c.Save); err != nil {
			return err
		}
		fmt.Fprintln(out, "Đã lưu model trong workspace; lần mở sau tự đọc key từ .env.")
		if os.Getenv(keyName) != "" {
			fmt.Fprintln(out, "Terminal đang đặt key cùng tên và được ưu tiên hơn .env.")
		}
		return nil
	}
}

type localWrite struct {
	path string
	data []byte
}

// Roll back local setup files if updating workspace.yaml fails midway.
func saveLocalFiles(writes []localWrite, commit func() error) error {
	backups := make([]localWrite, 0, len(writes))
	existed := map[string]bool{}
	for _, w := range writes {
		data, err := os.ReadFile(w.path)
		if err != nil && !errors.Is(err, os.ErrNotExist) {
			return err
		}
		existed[w.path] = err == nil
		backups = append(backups, localWrite{w.path, data})
	}
	applied := 0
	var err error
	for _, w := range writes {
		if err = state.Atomic(w.path, w.data); err != nil {
			break
		}
		applied++
	}
	if err == nil {
		err = commit()
	}
	if err == nil {
		return nil
	}
	for i := applied - 1; i >= 0; i-- {
		var rollback error
		if existed[backups[i].path] {
			rollback = state.Atomic(backups[i].path, backups[i].data)
		} else {
			rollback = os.Remove(backups[i].path)
		}
		err = errors.Join(err, rollback)
	}
	return err
}

func (m *translationTUI) saveRunSettings(values []string) func(context.Context, io.Writer, io.Writer) error {
	root := m.config.Root
	return func(ctx context.Context, out, progress io.Writer) error {
		unlock, err := (state.Store{Root: root}).Lock(ctx)
		if err != nil {
			return err
		}
		defer unlock()
		c, err := workspace.Load(root)
		if err != nil {
			return err
		}
		c.Run.MaxRequests, err = strconv.Atoi(values[0])
		if err != nil {
			return errors.New("số requests phải là số nguyên")
		}
		c.Run.RequestTimeoutSeconds, err = strconv.Atoi(values[1])
		if err != nil {
			return errors.New("timeout phải là số nguyên")
		}
		c.Output.EPUBCheck, c.Output.FontDirectory = strings.Trim(values[2], "\"'"), strings.Trim(values[3], "\"'")
		return c.Save()
	}
}

package app

import (
	"context"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"slices"
	"strconv"
	"strings"

	"github.com/joho/godotenv"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type ServiceOptions struct {
	NewModel           func(workspace.Config) (model.Caller, error)
	WorkspaceDirectory string
}

type AppService struct {
	Config             *workspace.Config
	Store              state.Store
	NewModel           func(workspace.Config) (model.Caller, error)
	WorkspaceDirectory string
}

func NewService(opts ServiceOptions) *AppService {
	directory := opts.WorkspaceDirectory
	if directory == "" {
		directory = "workspace"
	}
	if absolute, err := filepath.Abs(directory); err == nil {
		directory = absolute
	}
	return &AppService{
		NewModel:           opts.NewModel,
		WorkspaceDirectory: directory,
	}
}

func (s *AppService) LoadWorkspace(path string) (*workspace.Config, error) {
	c, err := workspace.Load(path)
	if err != nil {
		return nil, err
	}
	s.Config = &c
	s.Store = state.Store{Root: c.Root}
	s.AddRecentWorkspace(c.Root)
	return s.Config, nil
}

func (s *AppService) InitWorkspace(root, series, title string) (*workspace.Config, error) {
	c, err := workspace.Create(root, series, title)
	if err != nil {
		return nil, err
	}
	if err := skills.InstallDefaults(c); err != nil {
		return nil, err
	}
	s.Config = &c
	s.Store = state.Store{Root: c.Root}
	s.AddRecentWorkspace(c.Root)
	return s.Config, nil
}

func (s *AppService) AddVolume(id, sourcePath string, order int, prevID, dir string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()

	latest, err := workspace.Load(s.Config.Root)
	if err != nil {
		return err
	}
	if dir == "" {
		dir = id
	}
	v := workspace.Volume{
		ID:        id,
		Order:     order,
		Source:    sourcePath,
		Previous:  prevID,
		Directory: dir,
	}
	if err := latest.Add(v); err != nil {
		return err
	}
	s.Config = &latest
	return nil
}

func (s *AppService) Lock(ctx context.Context) (func(), error) {
	if s.Config == nil {
		return nil, errors.New("chưa mở workspace")
	}
	return s.Store.Lock(ctx)
}

func (s *AppService) RecentWorkspaces() []string {
	userDir, err := os.UserConfigDir()
	if err != nil {
		return nil
	}
	historyFile := filepath.Join(userDir, "novel-translate-cli", "recent_workspaces.json")
	data, err := os.ReadFile(historyFile)
	if err != nil {
		return nil
	}
	var paths []string
	if err := json.Unmarshal(data, &paths); err != nil {
		return nil
	}
	var existing []string
	for _, p := range paths {
		if fi, err := os.Stat(filepath.Join(p, "workspace.yaml")); err == nil && !fi.IsDir() {
			existing = append(existing, p)
		}
	}
	return existing
}

func (s *AppService) AddRecentWorkspace(path string) {
	abs, err := filepath.Abs(path)
	if err != nil {
		abs = path
	}
	userDir, err := os.UserConfigDir()
	if err != nil {
		return
	}
	dir := filepath.Join(userDir, "novel-translate-cli")
	_ = os.MkdirAll(dir, 0700)
	historyFile := filepath.Join(dir, "recent_workspaces.json")

	var paths []string
	if data, err := os.ReadFile(historyFile); err == nil {
		_ = json.Unmarshal(data, &paths)
	}
	paths = slices.DeleteFunc(paths, func(p string) bool { return strings.EqualFold(p, abs) })
	paths = append([]string{abs}, paths...)
	if len(paths) > 10 {
		paths = paths[:10]
	}
	data, err := json.MarshalIndent(paths, "", "  ")
	if err == nil {
		_ = os.WriteFile(historyFile, data, 0600)
	}
}

type localWrite struct {
	path string
	data []byte
}

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

func (s *AppService) updateWorkspaceConfig(update func(*workspace.Config) error) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	root := s.Config.Root
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()
	c, err := workspace.Load(root)
	if err != nil {
		return err
	}
	if err := update(&c); err != nil {
		return err
	}
	if err := c.Validate(); err != nil {
		return err
	}
	if err := c.Save(); err != nil {
		return err
	}
	s.Config = &c
	return nil
}

func (s *AppService) SaveRunSettings(values []string) error {
	if len(values) != 4 {
		return errors.New("thiếu thông tin giới hạn tác vụ")
	}
	maxRequests, err := strconv.Atoi(strings.TrimSpace(values[0]))
	if err != nil || maxRequests < 0 {
		return errors.New("giới hạn request phải là số >= 0")
	}
	timeout, err := strconv.Atoi(strings.TrimSpace(values[1]))
	if err != nil || timeout < 1 {
		return errors.New("timeout mỗi request phải là số >= 1 giây")
	}
	inputBytes, err := strconv.Atoi(strings.TrimSpace(values[2]))
	if err != nil || inputBytes < 1024 {
		return errors.New("giới hạn input phải từ 1024 byte")
	}
	outputTokens, err := strconv.Atoi(strings.TrimSpace(values[3]))
	if err != nil || outputTokens < 256 {
		return errors.New("giới hạn output phải từ 256 token")
	}
	return s.updateWorkspaceConfig(func(c *workspace.Config) error {
		c.Run.MaxRequests = maxRequests
		c.Run.RequestTimeoutSeconds = timeout
		c.Run.MaxInputBytes = inputBytes
		c.Run.MaxOutputTokens = outputTokens
		return nil
	})
}

func (s *AppService) SaveOutputSettings(values []string) error {
	if len(values) != 3 {
		return errors.New("thiếu thông tin xuất EPUB")
	}
	return s.updateWorkspaceConfig(func(c *workspace.Config) error {
		c.Output.Directory = strings.TrimSpace(values[0])
		c.Output.EPUBCheck = strings.TrimSpace(values[1])
		c.Output.FontDirectory = strings.TrimSpace(values[2])
		return nil
	})
}

func (s *AppService) SaveModelSetup(values []string) error {
	if s.Config == nil {
		return errors.New("chưa mở workspace")
	}
	if len(values) < 7 {
		return errors.New("thiếu thông tin thiết lập model")
	}
	root := s.Config.Root
	unlock, err := s.Store.Lock(context.Background())
	if err != nil {
		return err
	}
	defer unlock()

	c, err := workspace.Load(root)
	if err != nil {
		return err
	}
	protocol := strings.TrimSpace(values[0])
	modelName := strings.TrimSpace(values[1])
	effort := strings.TrimSpace(values[2])
	key := strings.TrimSpace(values[3])
	baseURL := strings.TrimSpace(values[4])
	api := strings.TrimSpace(values[5])
	window, err := strconv.Atoi(strings.TrimSpace(values[6]))
	if err != nil || window < 0 {
		return errors.New("context tokens phải là số >= 0")
	}

	providerPath := filepath.Join(c.Root, "provider.jsonc")
	if c.ProviderConfig != "" {
		providerPath = c.ReadPath(c.ProviderConfig)
	}
	old, _ := bootstrap.ReadConfigFile(providerPath)
	pc := old.DefaultProviderConfig()
	oldType, _ := pc.ProviderType(old.Provider)
	sameProvider := oldType == protocol

	keyName := "NOVEL_TRANSLATE_API_KEY"
	if sameProvider && pc.APIKeyEnv != "" {
		keyName = pc.APIKeyEnv
	} else if old.Provider != "" && !sameProvider {
		keyName = "NOVEL_TRANSLATE_" + strings.ToUpper(protocol) + "_API_KEY"
	}

	if key == "" && sameProvider {
		key = pc.APIKey
	}
	if key == "" {
		key, err = bootstrap.LookupAPIKey(keyName, root, filepath.Dir(providerPath))
		if err != nil {
			return err
		}
	}
	if key == "" && protocol != "ollama" {
		return errors.New("nhập API key hoặc lưu key vào .env trước")
	}

	pc = bootstrap.ProviderConfig{
		Type:    protocol,
		BaseURL: baseURL,
		Models:  []bootstrap.ModelConfig{{Name: modelName, ContextWindow: window}},
	}
	if protocol == "openai" {
		pc.API = api
	}
	if key != "" {
		pc.APIKeyEnv = keyName
	}
	cfg := bootstrap.Config{
		Provider:        protocol,
		ModelName:       modelName,
		Providers:       map[string]bootstrap.ProviderConfig{protocol: pc},
		ReasoningEffort: effort,
		Budget:          old.Budget,
	}

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
	targetConfigPath, err := c.WritePath(filepath.Join(".translation_work", "provider.json"))
	if err != nil {
		return err
	}
	envPath, err := c.WritePath(".env")
	if err != nil {
		return err
	}

	writes := []localWrite{}
	if key != "" {
		envValues, err := bootstrap.ReadDotEnv(envPath)
		if err != nil {
			return err
		}
		envValues[keyName] = key
		text, err := godotenv.Marshal(envValues)
		if err != nil {
			return errors.New("không thể tạo .env")
		}
		writes = append(writes, localWrite{path: envPath, data: []byte(text + "\n")})
	}
	writes = append(writes, localWrite{path: targetConfigPath, data: append(data, '\n')})

	c.ProviderConfig = filepath.ToSlash(filepath.Join(".translation_work", "provider.json"))
	if err := c.Validate(); err != nil {
		return err
	}
	if err := saveLocalFiles(writes, c.Save); err != nil {
		return err
	}
	s.Config = &c
	return nil
}

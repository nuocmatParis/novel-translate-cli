package screens

import (
	"fmt"
	"path/filepath"
	"strconv"

	"github.com/charmbracelet/bubbles/textinput"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/tui/components"
	"github.com/voocel/ainovel-cli/internal/translation/tui/theme"
)

type SettingsScreen struct {
	Service      *app.AppService
	Inputs       []textinput.Model
	ModelInputs  []textinput.Model
	RunInputs    []textinput.Model
	ExportInputs []textinput.Model
	FocusedField int
	ActiveTab    int
	Message      string
	Error        string
}

func NewSettingsScreen(svc *app.AppService) *SettingsScreen {
	s := &SettingsScreen{Service: svc}
	modelValues := []string{"", "", "", "", "", "", "0"}
	if svc != nil && svc.Config != nil {
		providerPath := filepath.Join(svc.Config.Root, "provider.jsonc")
		if svc.Config.ProviderConfig != "" {
			providerPath = svc.Config.ReadPath(svc.Config.ProviderConfig)
		}
		if cfg, err := bootstrap.ReadConfigFile(providerPath); err == nil {
			provider := cfg.Provider
			pc := cfg.DefaultProviderConfig()
			protocol, _ := pc.ProviderType(provider)
			window, _ := cfg.ResolveContextWindow(provider, cfg.ModelName)
			modelValues = []string{protocol, cfg.ModelName, cfg.ReasoningEffort, "", pc.BaseURL, pc.API, strconv.Itoa(window)}
		}
	}
	if modelValues[0] == "" {
		modelValues[0] = "openai"
	}
	runValues := []string{"0", "180", "16000", "12000"}
	exportValues := []string{"Output", "", ""}
	if svc != nil && svc.Config != nil {
		c := svc.Config
		runValues = []string{
			strconv.Itoa(c.Run.MaxRequests),
			strconv.Itoa(c.Run.RequestTimeoutSeconds),
			strconv.Itoa(c.Run.MaxInputBytes),
			strconv.Itoa(c.Run.MaxOutputTokens),
		}
		exportValues = []string{c.Output.Directory, c.Output.EPUBCheck, c.Output.FontDirectory}
	}
	s.ModelInputs = makeInputs(modelValues, 3)
	s.RunInputs = makeInputs(runValues, -1)
	s.ExportInputs = makeInputs(exportValues, -1)
	s.SetTab(0)
	return s
}

func makeInputs(values []string, secretIndex int) []textinput.Model {
	inputs := make([]textinput.Model, len(values))
	for i, value := range values {
		inputs[i] = textinput.New()
		inputs[i].SetValue(value)
		inputs[i].CharLimit = 4096
		if i == secretIndex {
			inputs[i].EchoMode = textinput.EchoPassword
			inputs[i].EchoCharacter = '*'
		}
	}
	return inputs
}

func (s *SettingsScreen) Tabs() []string {
	return []string{"Provider & Model", "Giới hạn tác vụ", "Xuất & EPUBCheck"}
}

func (s *SettingsScreen) SetTab(index int) {
	s.ActiveTab = (index + len(s.Tabs())) % len(s.Tabs())
	if s.ActiveTab == 0 {
		s.Inputs = s.ModelInputs
	} else if s.ActiveTab == 1 {
		s.Inputs = s.RunInputs
	} else {
		s.Inputs = s.ExportInputs
	}
	s.FocusedField = 0
	for i := range s.Inputs {
		if i == 0 {
			s.Inputs[i].Focus()
		} else {
			s.Inputs[i].Blur()
		}
	}
}

func (s *SettingsScreen) Save() error {
	if s.Service == nil || s.Service.Config == nil {
		s.Error = "mở workspace trước khi lưu cấu hình"
		return fmt.Errorf("%s", s.Error)
	}
	values := make([]string, len(s.Inputs))
	for i, input := range s.Inputs {
		values[i] = input.Value()
	}
	var err error
	switch s.ActiveTab {
	case 0:
		err = s.Service.SaveModelSetup(values)
	case 1:
		err = s.Service.SaveRunSettings(values)
	case 2:
		err = s.Service.SaveOutputSettings(values)
	}
	if err != nil {
		s.Error = err.Error()
		return err
	}
	s.Message = "Đã lưu cấu hình trong workspace."
	s.Error = ""
	return nil
}

func (s *SettingsScreen) View(th theme.Theme, width, height int, focused bool) string {
	w := max(40, width)
	h := max(15, height)
	var lines []string
	for i, label := range s.Tabs() {
		style := th.ButtonSecondary
		if i == s.ActiveTab {
			style = th.ButtonPrimary
		}
		lines = append(lines, style.Render(label))
	}
	lines = append(lines, "")

	var labels []string
	switch s.ActiveTab {
	case 0:
		labels = []string{
			"Provider (openai, gemini, anthropic, deepseek, openrouter, ollama):",
			"Tên model:",
			"Mức suy luận (off, low, medium, high, max):",
			"API key (để trống để giữ key đã lưu):",
			"Base URL (có thể để trống):",
			"API OpenAI (chat hoặc responses):",
			"Context tokens (0: tự nhận theo model):",
		}
		lines = append(lines, th.TextMuted.Render("Key được che khi nhập và lưu trong .env của workspace."))
	case 1:
		labels = []string{"Giới hạn requests (0: không giới hạn):", "Timeout mỗi request (giây):", "Input tối đa (byte):", "Output tối đa (token):"}
		lines = append(lines, th.TextMuted.Render("Mỗi lượt chạy vẫn tuân theo giới hạn và policy của workspace."))
	case 2:
		labels = []string{"Thư mục EPUB xuất (trong workspace):", "Đường dẫn EPUBCheck (có thể để trống):", "Thư mục font (có thể để trống):"}
		lines = append(lines, th.TextMuted.Render("EPUBCheck chỉ được xem là đạt sau khi chạy trên file xuất."))
	}
	lines = append(lines, "")
	for i, label := range labels {
		prefix := "  "
		style := th.Text
		if i == s.FocusedField {
			prefix = "› "
			style = th.TextAccent
		}
		lines = append(lines, style.Render(prefix+label))
		if i < len(s.Inputs) {
			s.Inputs[i].Width = max(1, w-8)
			lines = append(lines, "    "+s.Inputs[i].View())
		}
	}
	shortcut := "Tab: trường tiếp · Ctrl+←/→: nhóm cấu hình · Ctrl+S: lưu"
	if s.ActiveTab == 2 {
		shortcut += " · Ctrl+O: chọn file/thư mục"
	}
	lines = append(lines, "", th.TextMuted.Render(shortcut))
	if s.Message != "" {
		lines = append(lines, "", th.TextSuccess.Render(cleanDisplayText(s.Message)))
	}
	if s.Error != "" {
		lines = append(lines, "", th.TextError.Render("Lỗi: "+cleanDisplayText(s.Error)))
	}
	return components.RenderPanel(th, components.Panel{Title: "Cấu hình", Focused: focused, Width: w, Height: h, Lines: lines})
}

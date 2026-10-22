package cli

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
	"unicode"

	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/charmbracelet/x/term"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/translation/app"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/tui"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

const tuiHelp = `Mở giao diện terminal dịch EPUB Nhật → Việt.

novel-translate-cli tui [--workspace <path>] [--volume <id>]
novel-translate-cli --tui [--workspace <path>]

Không truyền workspace: mở workspace tại thư mục hiện tại nếu có,
hoặc mở danh sách workspace gần đây. Workspace mới chỉ cần tên bộ truyện;
ID và thư mục được tạo tự động trong workspace/.
Tab đổi panel; ↑/↓ hoặc j/k chọn; Enter thực hiện; PgUp/PgDn cuộn danh sách.
Esc quay lại; Ctrl+C dừng tác vụ; q thoát khi rảnh; ? trợ giúp theo màn.
n/o: tạo/mở workspace; a: đăng ký tập; i: import snapshot EPUB.
p: xuất preview; f: xuất Final khi đủ cổng; Ctrl+S lưu sửa tay/cấu hình.
Ctrl+O mở bộ chọn file/thư mục tại các trường đường dẫn.
Tab Ghi chú trong Bản dịch lưu ghi chú riêng theo khối, không đưa vào prompt.
Tác vụ model bắt đầu từ scope đã chọn; mở màn hoặc đổi panel không gọi model.
Pause/resume (p) chỉ áp dụng cho dịch/review tại ranh giới khối.
API key nhập trong Cấu hình được che và lưu vào .env của workspace.
`

func runTUI(ctx context.Context, args []string, out, errout io.Writer, options Options) error {
	f := flag.NewFlagSet("tui", flag.ContinueOnError)
	f.SetOutput(errout)
	path := f.String("workspace", "", "workspace path")
	volume := f.String("volume", "", "selected volume ID")
	f.Usage = func() { fmt.Fprint(errout, tuiHelp) }
	if err := f.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return nil
		}
		return err
	}
	if f.NArg() != 0 {
		return errors.New("tui chỉ nhận --workspace và --volume")
	}
	if *path == "" {
		if _, err := os.Stat("workspace.yaml"); err == nil {
			*path = "."
		}
	}
	svc := app.NewService(app.ServiceOptions{NewModel: options.NewModel})
	if *path != "" {
		c, err := svc.LoadWorkspace(*path)
		if err != nil {
			return err
		}
		if *volume != "" {
			if _, err := c.Volume(*volume); err != nil {
				return err
			}
		}
	} else if *volume != "" {
		return errors.New("--volume cần một workspace")
	}
	input := options.Input
	if input == nil {
		if !term.IsTerminal(os.Stdin.Fd()) {
			return errors.New("TUI cần terminal tương tác; mở PowerShell/Terminal rồi chạy novel-translate-cli tui")
		}
		input = os.Stdin
	}
	// Child jobs are also canceled if the terminal/program closes unexpectedly.
	runCtx, cancel := context.WithCancel(ctx)
	defer cancel()

	appInstance := tui.NewApp(svc)
	appInstance.SetJobContext(runCtx)
	if *volume != "" && svc.Config != nil {
		if err := appInstance.SetSelectedVolumeID(*volume); err != nil {
			return err
		}
		appInstance.OpenScreen("content")
	}
	_, err := tea.NewProgram(appInstance, tea.WithInput(input), tea.WithOutput(out), tea.WithContext(runCtx), tea.WithAltScreen()).Run()
	return err
}

type tuiAction struct {
	label   string
	command []string
	form    string
	editor  string
	paid    bool
	confirm bool
}

func (m *translationTUI) actions() []tuiAction {
	common := []tuiAction{{label: "Mở workspace", form: "open"}, {label: "Tạo workspace mới", form: "create"}}
	if m.config == nil {
		return common
	}
	actions := []tuiAction{
		{label: "Làm mới tiến độ", command: []string{"status"}},
		{label: "Thêm tập / EPUB nguồn", form: "add"},
		{label: "Thiết lập model / API key [m]", form: "model"},
		{label: "Chọn file cấu hình API", form: "provider"},
		{label: "Giới hạn chạy / EPUBCheck / font [c]", form: "settings"},
		{label: "Chọn thư viện skill chung (mọi bộ) [l]", form: "library"},
		{label: "Đọc / sửa skill chung (mọi bộ)", editor: "shared"},
		{label: "Chọn skill chung dùng cho bộ này", form: "enabled-skills"},
		{label: "Sửa quy ước riêng của bộ truyện [s]", editor: "skill"},
	}
	if len(m.config.Volumes) > 0 {
		actions = append(actions,
			tuiAction{label: "Nhập EPUB của tập đang chọn", command: []string{"import"}},
			tuiAction{label: "Dịch / tiếp tục tập đang chọn", command: []string{"resume"}, paid: true},
			tuiAction{label: "Review bản dịch", command: []string{"review"}, paid: true},
			tuiAction{label: "Đọc / sửa bản dịch [e]", editor: "translation"},
			tuiAction{label: "Đồng bộ bản sửa tay", command: []string{"sync"}},
			tuiAction{label: "Kiểm tra độ đầy đủ", command: []string{"verify"}},
			tuiAction{label: "Xuất EPUB xem trước", command: []string{"export", "--preview"}},
			tuiAction{label: "Xuất EPUB hoàn chỉnh", command: []string{"export"}, confirm: true},
			tuiAction{label: "Danh sách ảnh / trạng thái xử lý chữ", command: []string{"image", "list"}},
			tuiAction{label: "Xuất ảnh gốc để xem / tự edit", command: []string{"image", "export"}},
			tuiAction{label: "Lưu quyết định xử lý ảnh", form: "image"},
		)
	}
	actions = append(actions, tuiAction{label: "Xem skills / quy ước", command: []string{"skill", "list"}})
	return append(actions, common...)
}

type tuiVolumeStatus struct {
	ID             string   `json:"id"`
	Accepted       int      `json:"accepted"`
	Eligible       int      `json:"eligible"`
	Translated     int      `json:"translated"`
	Imported       bool     `json:"source_imported"`
	Stale          []string `json:"stale"`
	FreshnessError string   `json:"freshness_error"`
	Images         int      `json:"images"`
	ImageIssues    int      `json:"image_issues"`
}

type tuiSnapshot struct {
	Volumes []tuiVolumeStatus  `json:"volumes"`
	Usage   model.UsageSummary `json:"usage"`
}

type tuiField struct {
	label, value              string
	input                     textinput.Model
	secret, browse, directory bool
	choices                   []string
	extensions                []string
	selectOnly, custom        bool
}
type tuiForm struct {
	kind      string
	fields    []tuiField
	index     int
	selection *tuiChoices
}
type tuiLogMsg string
type tuiDoneMsg struct {
	config      *workspace.Config
	snapshot    tuiSnapshot
	err         error
	snapshotErr error
}
type tuiTickMsg struct{}

type translationTUI struct {
	ctx           context.Context
	options       Options
	config        *workspace.Config
	snapshot      tuiSnapshot
	volume        int
	selected      int
	width         int
	height        int
	logs          []string
	scroll        int
	form          *tuiForm
	pending       *tuiAction
	busy          bool
	jobLabel      string
	notice        string
	cancel        context.CancelFunc
	events        chan tea.Msg
	frame         int
	quitOnDone    bool
	editor        *tuiEditor
	picker        *tuiPicker
	showHelp      bool
	submittedForm *tuiForm
}

func newTUI(ctx context.Context, options Options) *translationTUI {
	return &translationTUI{ctx: ctx, options: options, width: 90, height: 30, notice: "Chọn workspace để bắt đầu. Mở giao diện không gọi API."}
}

func (m *translationTUI) Init() tea.Cmd {
	if m.config != nil {
		return m.start("Đọc tiến độ", []string{"status"}, m.config.Root)
	}
	return nil
}

func (m *translationTUI) addLog(text string) {
	for _, line := range strings.Split(strings.ReplaceAll(text, "\r\n", "\n"), "\n") {
		if line = cleanTUIText(line); strings.TrimSpace(line) != "" {
			m.logs = append(m.logs, ansi.Truncate(line, 2000, "…"))
		}
	}
	if len(m.logs) > 500 {
		m.logs = append([]string(nil), m.logs[len(m.logs)-500:]...)
	}
}

// Source text, provider errors and file names must never inject terminal escapes.
func cleanTUIText(s string) string {
	return strings.Map(func(r rune) rune {
		if unicode.IsControl(r) {
			return -1
		}
		return r
	}, ansi.Strip(s))
}

func (m *translationTUI) wait() tea.Cmd {
	events, ctx := m.events, m.ctx
	return func() tea.Msg {
		select {
		case event := <-events:
			return event
		case <-ctx.Done():
			return nil
		}
	}
}

func tuiTick() tea.Cmd {
	return tea.Tick(150*time.Millisecond, func(time.Time) tea.Msg { return tuiTickMsg{} })
}

// stdout is bounded: import may print an entire book. Progress is streamed to
// the UI instead of writing directly over Bubble Tea's renderer.
type tuiOutput struct{ data []byte }

func (w *tuiOutput) Write(p []byte) (int, error) {
	const limit = 24000
	n := len(p)
	if len(p) > limit {
		p = p[len(p)-limit:]
	}
	w.data = append(w.data, p...)
	if len(w.data) > limit {
		w.data = append([]byte(nil), w.data[len(w.data)-limit:]...)
	}
	return n, nil
}

type tuiProgress struct {
	ctx    context.Context
	events chan tea.Msg
}

func (w tuiProgress) Write(p []byte) (int, error) {
	select {
	case w.events <- tuiLogMsg(string(p)):
		return len(p), nil
	case <-w.ctx.Done():
		return 0, w.ctx.Err()
	}
}

func (m *translationTUI) start(label string, args []string, path string) tea.Cmd {
	command := append([]string(nil), args...)
	if len(command) > 0 && command[0] == "status" {
		return m.startOperation(label, path, nil)
	}
	if len(command) > 0 && command[0] != "init" {
		command = append(command, "--workspace", path)
	}
	options := m.options
	return m.startOperation(label, path, func(ctx context.Context, out, progress io.Writer) error {
		if command[0] == "import" {
			if err := execute(ctx, command, io.Discard, progress, options); err != nil {
				return err
			}
			_, err := fmt.Fprintln(out, "Đã nhập EPUB. Chọn Dịch / tiếp tục để bắt đầu.")
			return err
		}
		return execute(ctx, command, out, progress, options)
	})
}

func (m *translationTUI) startOperation(label, path string, operation func(context.Context, io.Writer, io.Writer) error) tea.Cmd {
	if m.busy {
		return nil
	}
	m.busy, m.jobLabel, m.notice, m.scroll = true, label, "", 0
	m.addLog("› " + label)
	jobCtx, cancel := context.WithCancel(m.ctx)
	m.cancel = cancel
	m.events = make(chan tea.Msg, 64)
	events, rootCtx := m.events, m.ctx
	go func() {
		defer cancel()
		var output tuiOutput
		var err error
		progress := tuiProgress{ctx: jobCtx, events: events}
		if operation != nil {
			err = operation(jobCtx, &output, progress)
			if len(output.data) > 0 {
				_, _ = progress.Write(output.data)
			}
		}
		result := tuiDoneMsg{err: err}
		if err == nil || mPathExists(path) {
			c, loadErr := workspace.Load(path)
			if loadErr == nil {
				result.config = &c
				var raw bytes.Buffer
				result.snapshotErr = status(&raw, c, state.Store{Root: c.Root}, true)
				if result.snapshotErr == nil {
					result.snapshotErr = json.Unmarshal(raw.Bytes(), &result.snapshot)
				}
			} else {
				result.snapshotErr = loadErr
			}
		}
		// Even canceled jobs report completion: q waits for the writer to stop.
		select {
		case events <- result:
		case <-rootCtx.Done():
		}
	}()
	return tea.Batch(m.wait(), tuiTick())
}

func mPathExists(path string) bool {
	name, err := workspace.ConfigPath(path)
	if err != nil {
		return false
	}
	_, err = os.Stat(name)
	return err == nil
}

func (m *translationTUI) Update(message tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := message.(type) {
	case tea.WindowSizeMsg:
		m.width, m.height = msg.Width, msg.Height
		if m.form != nil {
			for i := range m.form.fields {
				input := &m.form.fields[i].input
				input.Width = max(1, m.width-6)
				input.SetCursor(input.Position())
			}
		}
		if m.editor != nil {
			m.editor.resize(msg.Width, msg.Height)
		}
	case tuiEditorLoadedMsg:
		m.editor = msg.editor
		if m.editor.stage == "edit" {
			return m, tea.Batch(m.wait(), m.editor.initInput(m.width, m.height))
		}
		return m, m.wait()
	case tuiEditorSavedMsg:
		if m.editor != nil {
			m.editor.base = msg.text
			if msg.generation != nil {
				m.editor.generation = *msg.generation
			}
			if msg.hash != "" {
				m.editor.fileHash = msg.hash
			}
		}
		return m, m.wait()
	case tuiTickMsg:
		if m.busy {
			m.frame++
			return m, tuiTick()
		}
	case tuiLogMsg:
		m.addLog(string(msg))
		return m, m.wait()
	case tuiDoneMsg:
		m.busy = false
		if m.cancel != nil {
			m.cancel()
			m.cancel = nil
		}
		if msg.config != nil {
			m.config, m.snapshot = msg.config, msg.snapshot
			m.volume = min(m.volume, max(0, len(m.config.Volumes)-1))
			m.selected = min(m.selected, len(m.actions())-1)
		}
		m.notice = "Đã xong: " + m.jobLabel
		if msg.err != nil {
			m.notice = "Lỗi: " + msg.err.Error()
			if errors.Is(msg.err, engine.ErrUnresolved) {
				m.notice = "Còn vấn đề cần xử lý: " + msg.err.Error()
			}
		} else if msg.snapshotErr != nil {
			m.notice = "Không đọc được tiến độ: " + msg.snapshotErr.Error()
		}
		m.addLog(m.notice)
		if m.submittedForm != nil {
			if msg.err != nil {
				m.form = m.submittedForm
			} else {
				for i := range m.submittedForm.fields {
					field := &m.submittedForm.fields[i]
					if field.secret {
						field.value = ""
						field.input.SetValue("")
					}
				}
			}
			m.submittedForm = nil
		}
		if m.quitOnDone {
			return m, tea.Quit
		}
	case tea.KeyMsg:
		key := msg.String()
		if m.busy {
			switch key {
			case "ctrl+c", "esc", "q":
				m.cancel()
				m.notice = "Đang dừng và chờ tác vụ kết thúc…"
				m.quitOnDone = key == "q"
			case "pgup":
				m.scroll += 5
			case "pgdown":
				m.scroll = max(0, m.scroll-5)
			}
			return m, nil
		}
		if m.showHelp {
			m.showHelp = false
			return m, nil
		}
		if m.picker != nil {
			return m, m.updatePicker(message)
		}
		if m.editor != nil {
			return m, m.updateEditor(message)
		}
		if m.form != nil {
			return m, m.updateForm(msg)
		}
		if m.pending != nil {
			switch key {
			case "y":
				action := *m.pending
				m.pending = nil
				return m, m.runAction(action)
			case "n", "esc", "ctrl+c":
				m.pending = nil
			}
			return m, nil
		}
		switch key {
		case "?":
			m.showHelp = true
		case "n":
			m.openForm("create")
			return m, textinput.Blink
		case "o":
			m.openForm("open")
			return m, textinput.Blink
		case "a", "m", "c", "s", "e", "l":
			if m.config == nil {
				m.notice = "Mở hoặc tạo workspace trước."
				return m, nil
			}
			switch key {
			case "l":
				m.openForm("library")
				return m, textinput.Blink
			case "a":
				m.openForm("add")
				return m, textinput.Blink
			case "m":
				m.openForm("model")
				return m, textinput.Blink
			case "c":
				m.openForm("settings")
				return m, textinput.Blink
			case "s":
				return m, m.openEditor("skill")
			case "e":
				if len(m.config.Volumes) > 0 {
					return m, m.openEditor("translation")
				}
			}
		case "q", "ctrl+c":
			return m, tea.Quit
		case "up", "k":
			m.selected = (m.selected + len(m.actions()) - 1) % len(m.actions())
		case "down", "j", "tab":
			m.selected = (m.selected + 1) % len(m.actions())
		case "left", "right":
			if m.config != nil && len(m.config.Volumes) > 0 {
				delta := 1
				if key == "left" {
					delta = -1
				}
				m.volume = (m.volume + delta + len(m.config.Volumes)) % len(m.config.Volumes)
			}
		case "pgup":
			m.scroll += 5
		case "pgdown":
			m.scroll = max(0, m.scroll-5)
		case "r":
			if m.config != nil {
				return m, m.start("Đọc tiến độ", []string{"status"}, m.config.Root)
			}
		case "enter":
			action := m.actions()[m.selected]
			if action.editor != "" {
				return m, m.openEditor(action.editor)
			} else if action.form != "" {
				m.openForm(action.form)
				return m, textinput.Blink
			} else if action.paid || action.confirm {
				m.pending = &action
			} else {
				return m, m.runAction(action)
			}
		}
	}
	if m.picker != nil {
		return m, m.updatePicker(message)
	}
	if m.editor != nil && !m.busy {
		return m, m.updateEditor(message)
	}
	if m.form != nil && !m.busy {
		field := &m.form.fields[m.form.index]
		var cmd tea.Cmd
		field.input, cmd = field.input.Update(message)
		field.value = field.input.Value()
		return m, cmd
	}
	return m, nil
}

func (m *translationTUI) runAction(action tuiAction) tea.Cmd {
	args := append([]string(nil), action.command...)
	if args[0] != "status" && args[0] != "skill" {
		args = append(args, "--volume", m.config.Volumes[m.volume].ID)
	}
	return m.start(action.label, args, m.config.Root)
}

func (m *translationTUI) openForm(kind string) {
	f := &tuiForm{kind: kind}
	switch kind {
	case "open":
		f.fields = []tuiField{{label: "Đường dẫn workspace", browse: true, directory: true}}
	case "create":
		f.fields = []tuiField{{label: "Thư mục workspace mới", value: "workspace/bo-truyen", browse: true, directory: true}, {label: "Series ID", value: "bo-truyen"}, {label: "Tên bộ truyện"}}
	case "add":
		order := 1
		previous := ""
		for _, v := range m.config.Volumes {
			if v.Order >= order {
				order, previous = v.Order+1, v.ID
			}
		}
		f.fields = []tuiField{{label: "ID tập", value: fmt.Sprintf("vol%02d", order)}, {label: "Thứ tự tập", value: strconv.Itoa(order)}, {label: "Đường dẫn EPUB Nhật", browse: true, extensions: []string{".epub", ".EPUB"}}, {label: "Tập trước (để trống nếu chưa có)", value: previous}}
	case "provider":
		path := m.config.ProviderConfig
		if path == "" {
			path = bootstrap.DefaultConfigPath()
		}
		f.fields = []tuiField{{label: "File cấu hình provider JSON/JSONC", value: path, browse: true, extensions: []string{".json", ".jsonc"}}}
	case "library":
		path, err := skills.SharedDirectory()
		if err != nil {
			path, _ = skills.DefaultSharedDirectory()
		}
		f.fields = []tuiField{{label: "Thư mục skill chung (áp dụng mọi bộ)", value: path, browse: true, directory: true}}
	case "enabled-skills":
		f.fields = []tuiField{{label: "Skill ID dùng cho bộ này, cách nhau bằng dấu phẩy", value: strings.Join(m.config.EnabledSkills, ", ")}}
	case "model":
		cfg, _ := bootstrap.ReadConfigFile(providerPath(*m.config))
		f.fields = newModelForm(cfg)
	case "settings":
		f.fields = []tuiField{{label: "Giới hạn requests (0: không giới hạn)", value: strconv.Itoa(m.config.Run.MaxRequests)}, {label: "Timeout mỗi request (giây)", value: strconv.Itoa(m.config.Run.RequestTimeoutSeconds)}, {label: "EPUBCheck executable/JAR", value: m.config.Output.EPUBCheck, browse: true}, {label: "Thư mục font (có thể bỏ trống)", value: m.config.Output.FontDirectory, browse: true, directory: true}}
	case "image":
		f.fields = []tuiField{{label: "Image occurrence ID"}, {label: "Cách xử lý (Ctrl+N đổi)", value: "no-text", choices: []string{"no-text", "keep-original", "replace", "transcript"}}, {label: "Lý do / bằng chứng"}, {label: "File ảnh đã edit (chỉ replace)", browse: true}, {label: "File bản dịch UTF-8 (chỉ transcript)", browse: true}}
	}
	for i := range f.fields {
		field := &f.fields[i]
		field.input = textinput.New()
		field.input.Width = max(1, m.width-6)
		field.input.SetValue(field.value)
		field.input.Prompt = ""
		if field.secret {
			field.input.EchoMode = textinput.EchoPassword
			field.input.EchoCharacter = '•'
		}
	}
	if kind == "model" {
		f.refreshModelChoices(false)
	}
	f.fields[0].input.Focus()
	m.notice = ""
	if kind == "model" {
		m.notice = "Enter chọn nhà cung cấp → phiên bản → suy luận. Danh sách theo tài liệu; quyền dùng tùy API key. Một model cho cả ba role."
	}
	if kind == "library" {
		m.notice = "Thư viện dùng chung cho mọi workspace mặc định. Skills/SKILL.md vẫn là quy ước riêng của từng bộ."
	}
	m.form = f
}

func (m *translationTUI) updateForm(key tea.KeyMsg) tea.Cmd {
	f := m.form
	if f.selection != nil {
		return m.updateChoices(key)
	}
	field := &f.fields[f.index]
	if field.input.Value() != field.value {
		field.input.SetValue(field.value)
	}
	switch key.String() {
	case "esc", "ctrl+c":
		m.form = nil
	case "tab", "down":
		return f.focusNext(1)
	case "shift+tab", "up":
		return f.focusNext(-1)
	case "ctrl+u":
		if field.selectOnly && !field.custom {
			return nil
		}
		field.value = ""
		field.input.SetValue("")
		if f.kind == "model" && f.index == modelVersionField {
			f.refreshModelChoices(false)
		}
	case "ctrl+o":
		return m.openPicker()
	case "ctrl+n":
		if field.selectOnly {
			return m.openChoices()
		}
		if len(field.choices) > 0 {
			index := -1
			for i, value := range field.choices {
				if value == field.value {
					index = i
				}
			}
			field.value = field.choices[(index+1)%len(field.choices)]
			field.input.SetValue(field.value)
			field.input.CursorEnd()
		}
	case "enter":
		if field.selectOnly && !field.custom {
			return m.openChoices()
		}
		if f.index < len(f.fields)-1 {
			return f.focusNext(1)
		}
		return m.submitForm()
	default:
		if field.selectOnly && !field.custom {
			if key.Type == tea.KeyRunes || key.String() == " " {
				return m.openChoices()
			}
			return nil
		}
		var cmd tea.Cmd
		field.input, cmd = field.input.Update(key)
		field.value = field.input.Value()
		if f.kind == "model" && f.index == modelVersionField {
			f.refreshModelChoices(false)
		}
		return cmd
	}
	return nil
}

func (m *translationTUI) submitForm() tea.Cmd {
	f := m.form
	value := func(i int) string { return strings.Trim(strings.TrimSpace(f.fields[i].value), "\"'") }
	var args []string
	path := ""
	switch f.kind {
	case "model", "settings", "library", "enabled-skills":
		values := make([]string, len(f.fields))
		for i, field := range f.fields {
			values[i] = field.value
		}
		operation := m.saveRunSettings
		label := "Lưu cấu hình chạy"
		if f.kind == "model" {
			operation = m.saveModelSetup
			label = "Lưu model / API key"
		}
		if f.kind == "library" {
			operation = m.saveLibraryForm
			label = "Lưu thư viện skill chung"
		}
		if f.kind == "enabled-skills" {
			operation = m.saveEnabledSkills
			label = "Lưu skill cho bộ truyện"
		}
		m.submittedForm, m.form = f, nil
		return m.startOperation(label, m.config.Root, operation(values))
	case "open":
		if value(0) == "" || !mPathExists(value(0)) {
			m.notice = "Không tìm thấy workspace.yaml tại đường dẫn đã nhập."
			return nil
		}
		path, args = value(0), []string{"status"}
	case "create":
		if value(0) == "" || !workspace.ValidID(value(1)) || value(2) == "" {
			m.notice = "Cần đường dẫn, Series ID hợp lệ và tên bộ truyện."
			return nil
		}
		path = value(0)
		args = []string{"init", path, "--series", value(1), "--title", value(2)}
	case "add":
		path = m.config.Root
		args = []string{"volume", "add", "--id", value(0), "--order", value(1), "--source", value(2)}
		if value(3) != "" {
			args = append(args, "--previous", value(3))
		}
	case "image":
		path = m.config.Root
		args = []string{"image", "assess", "--volume", m.config.Volumes[m.volume].ID, "--id", value(0), "--disposition", value(1), "--reason", value(2)}
		if value(1) == "replace" {
			args = append(args, "--file", value(3))
		}
		if value(1) == "transcript" {
			args = append(args, "--transcript-file", value(4))
		}
	case "provider":
		// Never read/display keys in the TUI. Only persist the config file path,
		// under the same workspace lock as other configuration mutations.
		path := m.config.ReadPath(value(0))
		if value(0) == "" {
			m.notice = "Cần đường dẫn file cấu hình."
			return nil
		}
		if info, err := os.Stat(path); err != nil || info.IsDir() {
			m.notice = "Không tìm thấy file cấu hình provider."
			return nil
		}
		root := m.config.Root
		m.submittedForm, m.form = f, nil
		return m.startOperation("Lưu đường dẫn cấu hình API", root, func(ctx context.Context, out, progress io.Writer) error {
			unlock, err := (state.Store{Root: root}).Lock(ctx)
			if err != nil {
				return err
			}
			defer unlock()
			c, err := workspace.Load(root)
			if err != nil {
				return err
			}
			c.ProviderConfig = filepath.Clean(path)
			return c.Save()
		})
	}
	m.submittedForm, m.form = f, nil
	return m.start("Workspace: "+f.kind, args, path)
}

func (m *translationTUI) View() string {
	if m.form != nil && m.form.selection != nil {
		return m.form.choicesView(m.width, m.height)
	}
	if m.showHelp {
		return "PHÍM TẮT TUI\n\nn: tạo workspace · o: mở workspace · a: thêm tập\nm: thiết lập model/API key · c: giới hạn/checker/font\ne: đọc/sửa bản dịch · s: quy ước riêng · l: thư viện skill chung\nSkill chung dùng cho nhiều bộ; quy ước riêng luôn ưu tiên trong bộ đó.\n\nTrong ô nhập: Ctrl+O chọn file/thư mục · Ctrl+N đổi lựa chọn\n←/→ Home/End di chuyển con trỏ · Ctrl+U xóa ô\nTrong trình sửa: Ctrl+S lưu · Tab chuyển nguồn/bản dịch\nEsc quay lại; thay đổi chưa lưu có xác nhận trước khi bỏ.\n\nAPI key có thể lưu trong .env; không cần export mỗi lần.\nLinux/macOS: sh scripts/run.sh\n\nNhấn phím bất kỳ để quay lại."
	}
	if m.picker != nil {
		return m.picker.view(m.width, m.height)
	}
	if m.editor != nil {
		return m.editor.view(m.width, m.height, m.notice, m.busy)
	}
	width := max(1, m.width-2)
	accent := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("86"))
	muted := lipgloss.NewStyle().Foreground(lipgloss.Color("245"))
	selected := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("230")).Background(lipgloss.Color("24"))
	line := func(s string) string { return ansi.Truncate(s, width, "…") }
	var content []string
	content = append(content, accent.Render("NOVEL TRANSLATE  /  Nhật → Việt"))
	if m.config != nil {
		content = append(content, line(cleanTUIText(m.config.Series.Title)+"  ·  "+cleanTUIText(m.config.Root)))
		provider := m.config.ProviderConfig
		if provider == "" {
			provider = bootstrap.DefaultConfigPath()
		}
		content = append(content, muted.Render(line("API config: "+cleanTUIText(provider))))
		if len(m.config.Volumes) == 0 {
			content = append(content, "Chưa có tập. Chọn Thêm tập / EPUB nguồn.")
		} else {
			v := m.config.Volumes[m.volume]
			text := fmt.Sprintf("◀ Tập %d · %s ▶  [%d/%d]  Chưa nhập EPUB", v.Order, v.ID, m.volume+1, len(m.config.Volumes))
			detail := ""
			for _, s := range m.snapshot.Volumes {
				if s.ID == v.ID && s.Imported {
					text = fmt.Sprintf("◀ Tập %d · %s ▶  Dịch %d/%d · Đối chiếu %d · Cần xem lại %d", v.Order, v.ID, s.Translated, s.Eligible, s.Accepted, len(s.Stale))
					detail = fmt.Sprintf("Ảnh: %d · Vấn đề ảnh: %d", s.Images, s.ImageIssues)
					if s.FreshnessError != "" {
						detail += " · Nguồn/bản thảo cần xử lý: " + cleanTUIText(s.FreshnessError)
					}
				}
			}
			content = append(content, accent.Render(line(text)))
			if detail != "" {
				content = append(content, line(detail))
			}
		}
		u := m.snapshot.Usage
		content = append(content, muted.Render(line(fmt.Sprintf("%d requests · Chi phí đã biết $%.4f · %d requests chưa rõ giá", u.Requests, u.KnownCostUSD, u.UnknownCostRequests))))
	} else {
		content = append(content, "Mỗi workspace là một bộ truyện, chứa nhiều tập EPUB.")
	}
	content = append(content, "")
	if m.form != nil {
		content = append(content, accent.Render("Nhập thông tin"))
		visible := max(1, m.height-len(content)-9)
		start := max(0, m.form.index-visible+1)
		for i := start; i < min(len(m.form.fields), start+visible); i++ {
			if !m.form.fieldVisible(i) {
				continue
			}
			f := m.form.fields[i]
			value := f.value
			if f.selectOnly {
				value = m.form.choiceLabel(i, value)
			}
			if f.secret {
				value = strings.Repeat("•", len([]rune(value)))
			}
			text := "  " + f.label + ": " + cleanTUIText(value)
			if i == m.form.index {
				content = append(content, selected.Render(line("› "+f.label+":")))
				text = line("  " + f.input.View())
				if f.selectOnly && !f.custom {
					text = line("  " + cleanTUIText(value) + "  ▾ Enter chọn")
				}
			} else {
				text = line(text)
			}
			content = append(content, text)
		}
		content = append(content, muted.Render(line("Tab đổi ô · Enter chọn/tiếp/lưu · Ctrl+O chọn file · Ctrl+N danh sách · Esc hủy")))
	} else if m.pending != nil {
		content = append(content, accent.Render(line("Xác nhận: "+m.pending.label)))
		if m.pending.paid {
			content = append(content, line("Gọi API thật, có thể tính phí; tự chạy tiếp theo giới hạn workspace."))
		} else {
			content = append(content, line("Xuất vào Output; có thể thay file xuất trước đó khi mọi kiểm tra đạt."))
		}
		content = append(content, "Nhấn y để chạy · n/Esc hủy")
	} else {
		actions := m.actions()
		visible := max(1, min(len(actions), m.height-len(content)-8))
		start := max(0, min(m.selected-visible+1, len(actions)-visible))
		for i := start; i < min(len(actions), start+visible); i++ {
			text := "  " + actions[i].label
			if i == m.selected {
				text = selected.Render(line("› " + actions[i].label))
			} else {
				text = line(text)
			}
			content = append(content, text)
		}
		if visible < len(actions) {
			content = append(content, muted.Render(fmt.Sprintf("  Thao tác %d/%d · ↑/↓ để xem thêm", m.selected+1, len(actions))))
		}
	}
	content = append(content, "")
	if m.busy {
		content = append(content, accent.Render(line([]string{"⠋", "⠙", "⠹", "⠸"}[m.frame%4]+" "+m.jobLabel)))
	}
	if m.notice != "" {
		content = append(content, line(cleanTUIText(m.notice)))
	}
	logHeight := max(1, m.height-len(content)-3)
	content = append(content, muted.Render("── Nhật ký ──"))
	end := max(0, len(m.logs)-min(m.scroll, max(0, len(m.logs)-logHeight)))
	for _, entry := range m.logs[max(0, end-logHeight):end] {
		content = append(content, line(entry))
	}
	footer := "↑/↓ chọn · ←/→ đổi tập · Enter chạy · n/o workspace · m model · e sửa · ? trợ giúp · q thoát"
	if m.busy {
		footer = "Ctrl+C/Esc dừng tác vụ · q dừng rồi thoát · PgUp/PgDn cuộn nhật ký"
	}
	content = append(content, muted.Render(line(footer)))
	for i := range content {
		content[i] = line(content[i])
	}
	return strings.Join(content, "\n")
}

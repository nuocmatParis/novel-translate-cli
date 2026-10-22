package cli

import (
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/charmbracelet/x/term"
	"github.com/voocel/ainovel-cli/internal/bootstrap"
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/epub"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/research"
	"github.com/voocel/ainovel-cli/internal/translation/revision"
	"github.com/voocel/ainovel-cli/internal/translation/skills"
	"github.com/voocel/ainovel-cli/internal/translation/state"
	"github.com/voocel/ainovel-cli/internal/translation/verify"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

const Help = `novel-translate-cli — dịch EPUB Nhật → Việt

tui [--workspace <path>] [--volume <id>]
init <workspace> --series <id> [--title <text>]
volume add --workspace <path> --id <id> --order <n> --source <epub> [--previous <id>] [--directory <relative>]
import|translate|resume|sync|verify --workspace <path> --volume <id>
status --workspace <path> [--json]
review --workspace <path> --volume <id> [--chapter <id>]
review --workspace <path> --volume <id> --keep <block-id> --reason <text>
retranslate plan --workspace <path> --volume <id>
retranslate apply --workspace <path> --plan <id>
restore --workspace <path> --volume <id> --generation <id>
source plan --workspace <path> --volume <id>
source apply --workspace <path> --volume <id> --plan <id>
scene set --workspace <path> --volume <id> --id <first-block> --end <last-block> --quote <source-evidence> --reason <text> [--story-volume <n> --story-chapter <n> --story-block <n> | --uncertain]
scene clear --workspace <path> --volume <id>
skill list --workspace <path>
skill import --workspace <path> --source <markdown-or-directory> [--from-volume <n> --until-volume <exclusive-n>]
skill generate --workspace <path> --volume <id>
skill resolve --workspace <path> --id <rule-id> --decision confirm|reject --reason <text>
skill research --workspace <path> --volume <id> --term <text>
image list --workspace <path> --volume <id>
image export --workspace <path> --volume <id> [--directory <workspace-relative-path>]
image assess --workspace <path> --volume <id> --id <occurrence-id> --disposition no-text|keep-original|replace|transcript --reason <evidence> [--file <path>] [--transcript <text>] [--transcript-file <path>]
export --workspace <path> --volume <id> [--preview] [--from <chapter> --to <chapter>]

translate/resume tự chạy tiếp, hai chương logic mỗi đợt. Lệnh chỉ đọc/import
không cần API key. provider_config trong workspace.yaml trỏ tới JSON provider.
Không truyền lệnh trong terminal: mở TUI. TUI hỗ trợ tạo workspace,
thiết lập model/key, chọn file và sửa bản dịch. Key tự đọc từ .env.
Exit: 0 thành công, 1 lỗi vận hành/config, 2 còn vấn đề chưa giải quyết.
`

type Options struct {
	NewModel func(workspace.Config) (model.Caller, error)
	// Input defaults to os.Stdin; interactive callers can supply their terminal.
	Input io.Reader
}

func defaultModel(c workspace.Config) (model.Caller, error) {
	a, e := model.New(c)
	if e != nil {
		return nil, e
	}
	if e = a.RestoreUsage(); e != nil {
		return nil, e
	}
	return a, nil
}
func Run(ctx context.Context, args []string, stdout, stderr io.Writer, options Options) int {
	if len(args) == 0 && (options.Input != nil || term.IsTerminal(os.Stdin.Fd())) {
		args = []string{"tui"}
	}
	if len(args) == 0 || args[0] == "help" || args[0] == "--help" || args[0] == "-h" {
		fmt.Fprint(stdout, Help)
		return 0
	}
	if options.NewModel == nil {
		options.NewModel = defaultModel
	}
	var e error
	if args[0] == "tui" || args[0] == "--tui" {
		e = runTUI(ctx, args[1:], stdout, stderr, options)
	} else {
		e = execute(ctx, args, stdout, stderr, options)
	}
	if e == nil || errors.Is(e, flag.ErrHelp) {
		return 0
	}
	fmt.Fprintln(stderr, "Lỗi:", e)
	if errors.Is(e, engine.ErrUnresolved) {
		return 2
	}
	return 1
}
func printJSON(w io.Writer, v any) error {
	enc := json.NewEncoder(w)
	enc.SetIndent("", "  ")
	return enc.Encode(v)
}
func execute(ctx context.Context, args []string, out, progress io.Writer, options Options) error {
	if len(args) == 2 && (args[1] == "--help" || args[1] == "-h") {
		fmt.Fprint(out, Help)
		return nil
	}
	command := args[0]
	args = args[1:]
	sub := ""
	if command == "volume" || command == "skill" || command == "retranslate" || command == "image" || command == "source" || command == "scene" {
		if len(args) == 0 {
			return errors.New("thiếu subcommand")
		}
		sub = args[0]
		args = args[1:]
	}
	initPath := ""
	if command == "init" {
		if len(args) == 0 {
			return errors.New("init cần đường dẫn workspace")
		}
		initPath = args[0]
		args = args[1:]
	}
	f := flag.NewFlagSet(command, flag.ContinueOnError)
	f.SetOutput(progress)
	wp := f.String("workspace", "", "workspace path")
	series := f.String("series", "", "series ID")
	title := f.String("title", "", "series title")
	volume := f.String("volume", "", "volume ID")
	id := f.String("id", "", "volume ID")
	decision := f.String("decision", "", "confirm/reject rule candidate")
	order := f.Int("order", 0, "volume order")
	source := f.String("source", "", "source path")
	previous := f.String("previous", "", "previous volume ID")
	directory := f.String("directory", "", "volume directory")
	jsonOut := f.Bool("json", false, "JSON output")
	chapter := f.String("chapter", "", "chapter ID")
	preview := f.Bool("preview", false, "incomplete preview")
	from := f.String("from", "", "first chapter")
	to := f.String("to", "", "last chapter")
	planID := f.String("plan", "", "plan ID")
	generation := f.String("generation", "", "generation ID")
	term := f.String("term", "", "research term")
	keep := f.String("keep", "", "keep unit after manual decision")
	reason := f.String("reason", "", "manual decision evidence/reason")
	disposition := f.String("disposition", "", "image text disposition")
	file := f.String("file", "", "image replacement file path")
	transcript := f.String("transcript", "", "image translation transcript text")
	transcriptFile := f.String("transcript-file", "", "image translation transcript file")
	fromVolume := f.Int("from-volume", 0, "reference effective volume")
	untilVolume := f.Int("until-volume", 0, "reference exclusive end volume")
	end := f.String("end", "", "scene last block")
	quote := f.String("quote", "", "source evidence quote")
	storyVolume := f.Int("story-volume", 0, "scene story-time volume")
	storyChapter := f.Int("story-chapter", 0, "scene story-time chapter")
	storyBlock := f.Int("story-block", 0, "scene story-time block")
	uncertain := f.Bool("uncertain", false, "uncertain story time; no inherited pair context")
	if e := f.Parse(args); e != nil {
		return e
	}
	if f.NArg() > 0 {
		return errors.New("tham số vị trí không hợp lệ")
	}
	if command == "init" {
		c, e := workspace.Create(initPath, *series, *title)
		if e != nil {
			return e
		}
		if e = skills.InstallDefaults(c); e != nil {
			return e
		}
		return printJSON(out, map[string]string{"workspace": c.Root, "series": c.Series.ID})
	}
	if *wp == "" {
		return errors.New("cần --workspace")
	}
	c, e := workspace.Load(*wp)
	if e != nil {
		return e
	}
	store := state.Store{Root: c.Root}
	if command == "status" {
		return status(out, c, store, *jsonOut)
	}
	unlock, e := store.Lock(ctx)
	if e != nil {
		return e
	}
	defer unlock()
	if command == "volume" {
		if sub != "add" {
			return errors.New("volume chỉ có add")
		}
		if *directory == "" {
			*directory = *id + "_translation"
		}
		if *source == "" {
			return errors.New("cần --source")
		}
		sourceAbs, e := filepath.Abs(*source)
		if e != nil {
			return e
		}
		if e = c.Add(workspace.Volume{ID: *id, Order: *order, Previous: *previous, Directory: *directory, Source: sourceAbs}); e != nil {
			return e
		}
		return printJSON(out, c.Volumes)
	}
	if command == "skill" && sub == "list" {
		s, e := skills.Load(c)
		if e != nil {
			return e
		}
		for i := range s.Skills {
			s.Skills[i].Body = ""
		}
		return printJSON(out, s)
	}
	if command == "skill" && sub == "import" {
		if *source == "" {
			return errors.New("cần --source")
		}
		abs, e := filepath.Abs(*source)
		if e != nil {
			return e
		}
		id, e := skills.Import(c, abs, skills.ImportScope{FromVolume: *fromVolume, UntilVolume: *untilVolume})
		if e != nil {
			return e
		}
		return printJSON(out, map[string]string{"import": id, "status": "scoped references; unknown scope quarantined"})
	}
	if command == "skill" && sub == "resolve" {
		digest, e := skills.ResolveCandidate(c, *id, *decision, *reason)
		if e != nil {
			return e
		}
		return printJSON(out, map[string]string{"rules_revision": digest, "next": "retranslate plan để đối chiếu vùng ảnh hưởng"})
	}
	if command == "retranslate" && sub == "apply" {
		if !workspace.ValidID(*planID) {
			return errors.New("plan ID không hợp lệ")
		}
		var p revision.Impact
		if e = state.ReadJSON(store.Path("plans", *planID+".json"), &p); e != nil {
			return e
		}
		*volume = p.Volume
		v, e := c.Volume(*volume)
		if e != nil {
			return e
		}
		_, book, e := engine.Source(c, v, true)
		if e != nil {
			return e
		}
		g, e := store.Load(v.ID)
		if e != nil {
			return e
		}
		if e = revision.Check(c, v, book, g); e != nil {
			return e
		}
		s, e := skills.Load(c)
		if e != nil {
			return e
		}
		if s.Digest != p.Skills || book.SourceHash != p.Source {
			return errors.New("plan inputs stale")
		}
		currentPlan, e := engine.ImpactPlan(c, v, store, book, g, s)
		if e != nil {
			return e
		}
		if currentPlan.ID != p.ID {
			return errors.New("plan dependencies stale; lập plan mới")
		}
		if e = revision.Apply(&g, p); e != nil {
			return e
		}
		if e = revision.CaptureViews(c, v, book, &g); e != nil {
			return e
		}
		if e = store.Commit(v.ID, &g, g.ID); e != nil {
			return e
		}
		if e = revision.Materialize(c, v, book, g); e != nil {
			return e
		}
		caller, e := options.NewModel(c)
		if e != nil {
			return e
		}
		return engine.Runner{Config: c, Volume: v, Store: store, Model: caller, Progress: func(s string) { fmt.Fprintln(progress, s) }}.Run(ctx, false, "")
	}
	v, e := c.Volume(*volume)
	if e != nil {
		return e
	}
	if command == "import" {
		book, e := engine.Import(c, v)
		if e != nil {
			return e
		}
		return printJSON(out, book)
	}
	if command == "source" {
		if sub == "plan" {
			p, e := engine.PlanSourceRevision(c, v)
			if e != nil {
				return e
			}
			return printJSON(out, p)
		}
		if sub == "apply" {
			if !workspace.ValidID(*planID) {
				return errors.New("source plan ID sai")
			}
			var p engine.SourceRevision
			if e = state.ReadJSON(store.Path("source_plans", *planID+".json"), &p); e != nil {
				return e
			}
			return engine.ApplySourceRevision(c, v, p)
		}
		return errors.New("source chỉ có plan/apply")
	}
	raw, book, e := engine.Source(c, v, true)
	if e != nil {
		return e
	}
	g, e := store.Load(v.ID)
	if e != nil {
		return e
	}
	switch command {
	case "scene":
		if sub == "clear" {
			return state.WriteJSON(store.Path("volumes", v.ID, "scenes.json"), []engine.Scene{})
		}
		if sub != "set" {
			return errors.New("scene chỉ có set/clear")
		}
		return engine.SetScene(c, v, book, *id, *end, &domain.Position{Volume: *storyVolume, Chapter: *storyChapter, Block: *storyBlock}, *quote, *reason, *uncertain)
	case "image":
		if sub == "export" {
			dest := *directory
			if dest == "" {
				dest = filepath.Join(v.Directory, "images", "source")
			}
			return exportImages(out, c, book, raw, dest)
		}
		var decisions map[string]verify.ImageDecision
		imagePath := store.Path("volumes", v.ID, "image-decisions.json")
		if e = state.ReadJSON(imagePath, &decisions); e != nil && !errors.Is(e, os.ErrNotExist) {
			return e
		}
		if decisions == nil {
			decisions = map[string]verify.ImageDecision{}
		}
		if sub == "list" {
			images := verify.Images(book)
			for i := range images {
				if d, ok := decisions[images[i].ID]; ok {
					images[i].Decision = &d
				}
			}
			return printJSON(out, images)
		}
		if sub != "assess" {
			return errors.New("image chỉ có list/export/assess")
		}
		transcriptText := *transcript
		if *transcriptFile != "" {
			b, err := os.ReadFile(c.ReadPath(*transcriptFile))
			if err != nil {
				return fmt.Errorf("không đọc được transcript-file %s: %w", *transcriptFile, err)
			}
			transcriptText = string(b)
		}
		var repBytes []byte
		if *file != "" {
			var err error
			repBytes, err = os.ReadFile(c.ReadPath(*file))
			if err != nil {
				return fmt.Errorf("không đọc được file ảnh thay thế %s: %w", *file, err)
			}
		}
		decision, e := verify.AssessImage(book, *id, *disposition, *reason, *file, transcriptText, repBytes)
		if e != nil {
			return e
		}
		decisions[*id] = decision
		return state.WriteJSON(imagePath, decisions)
	case "sync":
		count, e := revision.Sync(c, v, book, &g)
		if e != nil {
			return e
		}
		if count > 0 {
			if e = revision.CaptureViews(c, v, book, &g); e != nil {
				return e
			}
			if e = store.Commit(v.ID, &g, g.ID); e != nil {
				return e
			}
		}
		return printJSON(out, map[string]any{"manual_revisions": count, "generation": g.ID})
	case "restore":
		if !workspace.ValidID(*generation) {
			return errors.New("generation ID không hợp lệ")
		}
		if e = revision.Check(c, v, book, g); e != nil {
			return e
		}
		previous, e := store.History(v.ID, *generation)
		if e != nil {
			return e
		}
		if previous.ID != *generation || previous.SourceHash != book.SourceHash {
			return errors.New("restore source/ID không khớp")
		}
		expected := g.ID
		if e = revision.CaptureViews(c, v, book, &g); e != nil {
			return e
		}
		bases := g.ViewBases
		g = previous
		g.ViewBases = bases
		if e = store.Commit(v.ID, &g, expected); e != nil {
			return e
		}
		return revision.Materialize(c, v, book, g)
	case "retranslate":
		if sub != "plan" {
			return errors.New("subcommand retranslate không hợp lệ")
		}
		s, e := skills.Load(c)
		if e != nil {
			return e
		}
		p, e := engine.ImpactPlan(c, v, store, book, g, s)
		if e != nil {
			return e
		}
		if e = state.WriteJSON(store.Path("plans", p.ID+".json"), p); e != nil {
			return e
		}
		return printJSON(out, p)
	case "translate", "resume", "review":
		if *keep != "" {
			if command != "review" || strings.TrimSpace(*reason) == "" {
				return errors.New("--keep cần review và --reason")
			}
			u, ok := g.Units[*keep]
			if !ok {
				return errors.New("block chưa có bản dịch")
			}
			if e = revision.Check(c, v, book, g); e != nil {
				return e
			}
			u.Review = &domain.Review{SourceHash: u.SourceHash, TargetHash: domain.Hash([]byte(u.Target)), Decision: "user-kept", Model: "manual decision: " + *reason}
			var block domain.Block
			for _, b := range book.Blocks {
				if b.ID == u.ID {
					block = b
				}
			}
			if block.ID == "" || u.SourceHash != block.SourceHash {
				return errors.New("manual decision source stale")
			}
			if _, e = epub.TokensHTML(block, u.Target); e != nil {
				return e
			}
			snapshot, e := skills.Load(c)
			if e != nil {
				return e
			}
			rules, conflicts, e := engine.ResolvedRules(c, v, store, g, snapshot, block.Position)
			if e != nil {
				return e
			}
			if len(conflicts) > 0 {
				return fmt.Errorf("rule conflicts cần xử lý trước: %v", conflicts)
			}
			events, e := engine.ContextEvents(c, v, store, g, block.Position, nil)
			if e != nil {
				return e
			}
			u.Dependencies.Source = block.SourceHash
			u.Dependencies.Skills = snapshot.Fingerprint(block.Position, block.Source)
			u.Dependencies.Context, e = engine.ContextDigest(c, v, events, block.Position)
			if e != nil {
				return e
			}
			u.Dependencies.Rules = map[string]string{}
			for _, rule := range rules {
				u.Dependencies.Rules[rule.ID] = domain.Digest(rule)
			}
			u.Review.DecisionReason = *reason
			u.RefreshDependencies = false
			u.StaleReason = ""
			u.Events = nil
			u.Rules = nil
			g.Units[u.ID] = u
			return store.Commit(v.ID, &g, g.ID)
		}
		caller, e := options.NewModel(c)
		if e != nil {
			return e
		}
		runner := engine.Runner{Config: c, Volume: v, Store: store, Model: caller, Progress: func(s string) { fmt.Fprintln(progress, s) }}
		if e = runner.Run(ctx, command == "review", *chapter); e != nil {
			return e
		}
		return printJSON(out, map[string]string{"volume": v.ID, "status": "đã đối chiếu phạm vi được hỗ trợ"})
	case "skill":
		switch sub {
		case "research":
			evs, e := research.New(c).Research(ctx, *term, book)
			if pe := printJSON(out, evs); pe != nil {
				return pe
			}
			return e
		case "generate":
			snapshot, e := skills.Load(c)
			if e != nil {
				return e
			}
			impact, e := engine.ImpactPlan(c, v, store, book, g, snapshot)
			if e != nil {
				return e
			}
			if len(impact.Units) > 0 {
				return fmt.Errorf("%w: bản dịch stale; xử lý retranslate plan trước khi sinh skill", engine.ErrUnresolved)
			}
			caller, e := options.NewModel(c)
			if e != nil {
				return e
			}
			var evs []research.Evidence
			files, _ := filepath.Glob(store.Path("research", "evidence", "*.json"))
			for _, f := range files {
				var ev research.Evidence
				if e = state.ReadJSON(f, &ev); e != nil {
					return e
				}
				evs = append(evs, ev)
			}
			id, e := skills.Generate(ctx, c, book, g, caller, evs)
			if e != nil {
				return e
			}
			s, e := skills.Load(c)
			if e != nil {
				return e
			}
			p, e := engine.ImpactPlan(c, v, store, book, g, s)
			if e != nil {
				return e
			}
			if e = state.WriteJSON(store.Path("plans", p.ID+".json"), p); e != nil {
				return e
			}
			return printJSON(out, map[string]string{"rules_revision": id, "impact_plan": p.ID})
		}
		return errors.New("skill subcommand không hợp lệ")
	case "verify":
		report, e := check(c, v, book, g)
		if pe := printJSON(out, report); pe != nil {
			return pe
		}
		return e
	case "export":
		var chapters []string
		if *from != "" || *to != "" {
			if *from == "" || *to == "" {
				return errors.New("range cần cả --from và --to")
			}
			start, end := -1, -1
			for i, ch := range book.Chapters {
				if ch.ID == *from {
					start = i
				}
				if ch.ID == *to {
					end = i
				}
			}
			if start < 0 || end < start {
				return errors.New("range chapter không hợp lệ")
			}
			for _, ch := range book.Chapters[start : end+1] {
				chapters = append(chapters, ch.ID)
			}
		}
		report, e := check(c, v, book, g, chapters)
		if e != nil && !*preview {
			return e
		}
		fonts := map[string][]byte{}
		if c.Output.FontDirectory != "" {
			for _, name := range []string{"LiberationSerif-Regular.ttf", "LiberationSerif-Bold.ttf", "LiberationSerif-Italic.ttf", "LiberationSerif-BoldItalic.ttf"} {
				b, e := os.ReadFile(filepath.Join(c.ReadPath(c.Output.FontDirectory), name))
				if e != nil {
					return e
				}
				fonts[name] = b
			}
		}
		var imageDecisions map[string]verify.ImageDecision
		if e = state.ReadJSON(store.Path("volumes", v.ID, "image-decisions.json"), &imageDecisions); e != nil && !errors.Is(e, os.ErrNotExist) {
			return e
		}
		imageBook := book
		if len(chapters) > 0 {
			selected, err := epub.Read(raw, v.ID, v.Order)
			if err != nil {
				return err
			}
			if err = selected.SelectScope(chapters); err != nil {
				return err
			}
			imageBook = selected.Book
		}
		imageReplacements, imageTranscripts, imageIssues := verify.ImageAssets(imageBook, imageDecisions, func(path string) ([]byte, error) { return os.ReadFile(c.ReadPath(path)) })
		if len(imageIssues) > 0 && !*preview {
			return fmt.Errorf("%w: kiểm tra lại quyết định/file ảnh", engine.ErrUnresolved)
		}
		exportOptions := epub.ExportOptions{
			Preview:           *preview,
			Fonts:             fonts,
			Chapters:          chapters,
			Title:             fmt.Sprintf("%s — Tập %d", c.Series.Title, v.Order),
			ImageReplacements: imageReplacements,
			ImageTranscripts:  imageTranscripts,
		}
		data, e := epub.Write(raw, v.ID, v.Order, g, exportOptions)
		if e != nil {
			return e
		}
		verified, ve := epub.VerifyOutput(raw, data, v.ID, v.Order, g, exportOptions)
		report.Structure = verified.Structure
		report.Diagnostics = append(report.Diagnostics, verified.Diagnostics...)
		report.Preview = *preview
		report.OutputHash = domain.Hash(data)
		report.Repairs, e = epub.MetadataRepairs(raw, v.ID, v.Order, chapters)
		if e != nil {
			return e
		}
		if ve != nil && !*preview {
			return ve
		}
		tmp := store.Path("exports", domain.Hash(data)+".epub")
		if e = state.Atomic(tmp, data); e != nil {
			return e
		}
		checker := c.Output.EPUBCheck
		if strings.ContainsAny(checker, "/\\") {
			checker = c.ReadPath(checker)
		}
		checkCtx, cancel := context.WithTimeout(ctx, 2*time.Minute)
		log, ce := verify.EPUBCheck(checkCtx, checker, tmp)
		cancel()
		if ce == nil {
			report.Format = "EPUBCheck passed"
		} else {
			report.Format = "EPUBCheck chưa pass: " + ce.Error()
		}
		if e = state.Atomic(tmp+".epubcheck.txt", []byte(log)); e != nil {
			return e
		}
		if e = state.WriteJSON(tmp+".report.json", report); e != nil {
			return e
		}
		if ce != nil && !*preview {
			return fmt.Errorf("%w: %v", engine.ErrUnresolved, ce)
		}
		name := v.ID + "_VI.epub"
		if len(chapters) > 0 {
			name = v.ID + "_VI_" + *from + "-" + *to + ".epub"
		}
		if *preview {
			name = strings.TrimSuffix(name, ".epub") + ".preview.epub"
		}
		dest, e := c.WritePath(filepath.Join(c.Output.Directory, name))
		if e != nil {
			return e
		}
		if e = state.Atomic(dest, data); e != nil {
			return e
		}
		if e = state.WriteJSON(dest+".report.json", report); e != nil {
			return e
		}
		return printJSON(out, map[string]any{"output": dest, "report": report})
	default:
		return fmt.Errorf("lệnh không hợp lệ: %s", command)
	}
}
func check(c workspace.Config, v workspace.Volume, book domain.Book, g domain.Generation, scope ...[]string) (domain.Report, error) {
	r := domain.Report{SchemaVersion: 1, SourceHash: book.SourceHash, Revision: g.ID, Extraction: book.TextNodes == book.CoveredTextNodes && book.TextAttributes == book.CoveredTextAttributes, Semantic: "chưa đối chiếu", Format: "chưa chạy EPUBCheck"}
	s, e := skills.Load(c)
	if e != nil {
		return r, e
	}
	if e = revision.Check(c, v, book, g); e != nil {
		return r, e
	}
	if len(scope) > 0 && len(scope[0]) > 0 {
		raw, _, e := engine.Source(c, v, true)
		if e != nil {
			return r, e
		}
		p, e := epub.Read(raw, v.ID, v.Order)
		if e != nil {
			return r, e
		}
		if e = p.SelectScope(scope[0]); e != nil {
			return r, e
		}
		book = p.Book
		r.RequestedScope = append([]string(nil), scope[0]...)
	}
	for _, ch := range book.Chapters {
		r.Scope = append(r.Scope, ch.ID)
	}
	for _, d := range book.Diagnostics {
		r.Diagnostics = append(r.Diagnostics, d)
	}
	var imageDecisions map[string]verify.ImageDecision
	if e = state.ReadJSON(state.Store{Root: c.Root}.Path("volumes", v.ID, "image-decisions.json"), &imageDecisions); e != nil && !errors.Is(e, os.ErrNotExist) {
		return r, e
	}
	_, _, imageIssues := verify.ImageAssets(book, imageDecisions, func(path string) ([]byte, error) { return os.ReadFile(c.ReadPath(path)) })
	r.Diagnostics = append(r.Diagnostics, imageIssues...)
	for _, b := range book.Blocks {
		if b.Disposition != "translate" {
			continue
		}
		r.Eligible++
		u, ok := g.Units[b.ID]
		if ok && u.SourceHash == b.SourceHash {
			r.Translated++
			contextEvents, contextErr := engine.ContextEvents(c, v, state.Store{Root: c.Root}, g, b.Position, nil)
			contextHash, contextDigestErr := engine.ContextDigest(c, v, contextEvents, b.Position)
			rules, conflicts, ruleErr := engine.ResolvedRules(c, v, state.Store{Root: c.Root}, g, s, b.Position, b.Source)
			ruleHashes := map[string]string{}
			for _, rule := range rules {
				ruleHashes[rule.ID] = domain.Digest(rule)
			}
			if u.Accepted() && u.Dependencies.Skills == s.Fingerprint(b.Position, b.Source) && contextErr == nil && contextDigestErr == nil && ruleErr == nil && len(conflicts) == 0 && u.Dependencies.Context == contextHash && domain.Digest(u.Dependencies.Rules) == domain.Digest(ruleHashes) {
				r.Accepted++
			} else {
				r.Diagnostics = append(r.Diagnostics, domain.Diagnostic{Code: "unreviewed-or-stale", Detail: b.ID, Blocking: true})
			}
		}
	}
	r.Structure = false
	blocking := false
	for _, d := range r.Diagnostics {
		if d.Blocking {
			blocking = true
		}
	}
	if blocking || r.Accepted != r.Eligible {
		return r, engine.ErrUnresolved
	}
	r.Semantic = "đã đối chiếu; không bảo đảm đúng nghĩa 100%"
	return r, nil
}
func status(out io.Writer, c workspace.Config, s state.Store, jsonOut bool) error {
	snapshot, e := skills.Load(c)
	if e != nil {
		return e
	}
	usage, e := model.ReadUsage(s)
	if e != nil {
		return e
	}
	budget := map[string]any{"max_requests": c.Run.MaxRequests, "scope": "workspace lifetime", "known_cost_usd": usage.KnownCostUSD, "unknown_cost_requests": usage.UnknownCostRequests}
	providerPath := bootstrap.DefaultConfigPath()
	if c.ProviderConfig != "" {
		providerPath = c.ReadPath(c.ProviderConfig)
	}
	if cfg, e := bootstrap.LoadConfigFile(providerPath, c.Root); e == nil {
		budget["max_cost_usd"] = cfg.Budget.BookUSD
		budget["cost_limit_enabled"] = cfg.Budget.Enabled()
	}
	volumes := []map[string]any{}
	for _, v := range c.Volumes {
		g, e := s.Load(v.ID)
		if e != nil {
			return e
		}
		_, book, se := engine.Source(c, v, false)
		freshnessError := ""
		affected := map[string]string{}
		if se == nil {
			if _, _, e := engine.Source(c, v, true); e != nil {
				freshnessError = e.Error()
			} else if e = revision.Check(c, v, book, g); e != nil {
				freshnessError = e.Error()
			} else {
				impact, e := engine.ImpactPlan(c, v, s, book, g, snapshot)
				if e != nil {
					freshnessError = e.Error()
				} else {
					for _, a := range impact.Units {
						affected[a.ID] = a.Reason
					}
				}
			}
		}
		eligible, accepted := 0, 0
		issues := []domain.Issue{}
		stale := []string{}
		for _, b := range book.Blocks {
			if b.Disposition == "translate" {
				eligible++
			}
			if u, ok := g.Units[b.ID]; ok {
				if u.Accepted() && freshnessError == "" && affected[u.ID] == "" && b.Disposition == "translate" {
					accepted++
				}
				if u.Review != nil {
					issues = append(issues, u.Review.Issues...)
				}
				if u.StaleReason != "" || affected[u.ID] != "" || freshnessError != "" {
					stale = append(stale, u.ID)
				}
			}
		}
		var decisions map[string]verify.ImageDecision
		if e := state.ReadJSON(s.Path("volumes", v.ID, "image-decisions.json"), &decisions); e != nil && !errors.Is(e, os.ErrNotExist) {
			return e
		}
		_, _, imageIssues := verify.ImageAssets(book, decisions, func(path string) ([]byte, error) { return os.ReadFile(c.ReadPath(path)) })
		entry := map[string]any{"id": v.ID, "generation": g.ID, "eligible": eligible, "translated": len(g.Units), "accepted": accepted, "issues": issues, "stale": stale, "stale_reasons": affected, "freshness_error": freshnessError, "chapters": book.Chapters, "batches": engine.PlanBatches(book.Chapters), "source_diagnostics": book.Diagnostics, "source_imported": se == nil, "images": len(verify.Images(book)), "image_issues": len(imageIssues), "image_diagnostics": imageIssues}
		volumes = append(volumes, entry)
		if !jsonOut {
			fmt.Fprintf(out, "%s: %d/%d khối đã đối chiếu; %d stale; %d vấn đề ảnh; generation %s\n", v.ID, accepted, eligible, len(stale), len(imageIssues), g.ID)
		}
	}
	if jsonOut {
		return printJSON(out, map[string]any{"schema_version": 1, "series": c.Series, "volumes": volumes, "usage": usage, "budget": budget, "live_acceptance": "chưa xác nhận; cần evidence từ lượt dịch/review thật"})
	}
	fmt.Fprintf(out, "Requests: %d; chi phí đã biết: %.6f USD; %d request chưa rõ chi phí\n", usage.Requests, usage.KnownCostUSD, usage.UnknownCostRequests)
	return nil
}

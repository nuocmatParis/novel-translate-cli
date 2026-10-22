package app

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"github.com/voocel/ainovel-cli/internal/translation/engine"
	"github.com/voocel/ainovel-cli/internal/translation/model"
	"github.com/voocel/ainovel-cli/internal/translation/progress"
	"github.com/voocel/ainovel-cli/internal/translation/workspace"
)

type JobState string

const (
	StateIdle       JobState = "idle"
	StateRunning    JobState = "running"
	StatePaused     JobState = "paused"
	StateCancelling JobState = "cancelling"
	StateDone       JobState = "done"
	StateFailed     JobState = "failed"
)

type JobStatus struct {
	State       JobState      `json:"state"`
	Kind        string        `json:"kind"`
	Label       string        `json:"label"`
	VolumeID    string        `json:"volume_id,omitempty"`
	Phase       string        `json:"phase"`
	Message     string        `json:"message"`
	ActiveUnit  string        `json:"active_unit"`
	ActiveModel string        `json:"active_model"`
	Elapsed     time.Duration `json:"elapsed"`
	Error       string        `json:"error,omitempty"`
}

type JobManager struct {
	service     *AppService
	broadcaster *progress.Broadcaster

	mu          sync.RWMutex
	state       JobState
	kind        string
	label       string
	volumeID    string
	phase       string
	message     string
	activeUnit  string
	activeModel string
	startTime   time.Time
	err         error

	cancelCtx context.CancelFunc
	resumeCh  chan struct{}
	pauseFlag bool
	pauseMu   sync.RWMutex
	logs      []string
}

func NewJobManager(service *AppService) *JobManager {
	jm := &JobManager{
		service:     service,
		broadcaster: progress.NewBroadcaster(),
		state:       StateIdle,
	}
	return jm
}

func (jm *JobManager) Broadcaster() *progress.Broadcaster {
	return jm.broadcaster
}

func (jm *JobManager) Status() JobStatus {
	jm.mu.RLock()
	defer jm.mu.RUnlock()

	var elapsed time.Duration
	if !jm.startTime.IsZero() && (jm.state == StateRunning || jm.state == StatePaused) {
		elapsed = time.Since(jm.startTime)
	}

	errStr := ""
	if jm.err != nil {
		errStr = jm.err.Error()
	}

	return JobStatus{
		State:       jm.state,
		Kind:        jm.kind,
		Label:       jm.label,
		VolumeID:    jm.volumeID,
		Phase:       jm.phase,
		Message:     jm.message,
		ActiveUnit:  jm.activeUnit,
		ActiveModel: jm.activeModel,
		Elapsed:     elapsed,
		Error:       errStr,
	}
}

func (jm *JobManager) IsBusy() bool {
	jm.mu.RLock()
	defer jm.mu.RUnlock()
	return jm.state == StateRunning || jm.state == StatePaused || jm.state == StateCancelling
}

func (jm *JobManager) Pause() {
	jm.pauseMu.Lock()
	jm.pauseFlag = true
	jm.pauseMu.Unlock()

	jm.mu.Lock()
	if jm.state == StateRunning {
		jm.message = "Đang hoàn tất request hiện tại; sẽ dừng tại ranh giới khối."
	}
	jm.mu.Unlock()
}

func (jm *JobManager) Resume() {
	wasPaused := jm.isPaused()
	jm.pauseMu.Lock()
	jm.pauseFlag = false
	jm.pauseMu.Unlock()

	jm.mu.Lock()
	if jm.state == StatePaused {
		jm.state = StateRunning
		jm.message = "Đang tiếp tục tác vụ..."
	}
	resumeCh := jm.resumeCh
	jm.mu.Unlock()
	if wasPaused && resumeCh != nil {
		select {
		case resumeCh <- struct{}{}:
		default:
		}
	}
}

func (jm *JobManager) Cancel() {
	jm.mu.Lock()
	if jm.cancelCtx != nil {
		jm.state = StateCancelling
		jm.message = "Đang dừng tác vụ..."
		jm.cancelCtx()
	}
	jm.mu.Unlock()
}

func (jm *JobManager) Logs() []string {
	jm.mu.RLock()
	defer jm.mu.RUnlock()
	return append([]string(nil), jm.logs...)
}

func (jm *JobManager) AddLog(line string) {
	jm.mu.Lock()
	defer jm.mu.Unlock()
	jm.logs = append(jm.logs, line)
	if len(jm.logs) > 500 {
		jm.logs = jm.logs[len(jm.logs)-500:]
	}
}

func (jm *JobManager) isPaused() bool {
	jm.pauseMu.RLock()
	defer jm.pauseMu.RUnlock()
	return jm.pauseFlag
}

func (jm *JobManager) StartTranslate(ctx context.Context, volumeID string, scope engine.RunScope, policy engine.ReviewPolicy) error {
	if jm.IsBusy() {
		return errors.New("tác vụ khác đang chạy")
	}
	if jm.service.Config == nil {
		return errors.New("chưa mở workspace")
	}
	v, err := jm.service.Config.Volume(volumeID)
	if err != nil {
		return err
	}

	jobCtx, cancel := context.WithCancel(ctx)
	jm.mu.Lock()
	jm.state = StateRunning
	jm.kind = "translate"
	jm.label = "Dịch / tiếp tục tập " + volumeID
	if policy.ReviewOnly {
		jm.label = "Review lại tập " + volumeID
	}
	jm.volumeID = volumeID
	jm.phase = "prepare"
	jm.message = "Đang khởi tạo runner..."
	jm.activeUnit = ""
	jm.activeModel = ""
	jm.logs = nil
	jm.startTime = time.Now()
	jm.cancelCtx = cancel
	jm.resumeCh = make(chan struct{}, 1)
	jm.err = nil
	jm.mu.Unlock()
	jm.pauseMu.Lock()
	jm.pauseFlag = false
	jm.pauseMu.Unlock()

	go func() {
		defer cancel()
		defer func() {
			jm.mu.Lock()
			if jm.err != nil {
				jm.state = StateFailed
			} else {
				jm.state = StateDone
			}
			jm.cancelCtx = nil
			jm.resumeCh = nil
			jm.mu.Unlock()
		}()

		unlock, err := jm.service.Store.Lock(jobCtx)
		if err != nil {
			jm.mu.Lock()
			jm.err = err
			jm.mu.Unlock()
			return
		}
		defer unlock()

		modelCaller := jm.service.NewModel
		if modelCaller == nil {
			modelCaller = func(c workspace.Config) (model.Caller, error) {
				return model.New(c)
			}
		}
		caller, err := modelCaller(*jm.service.Config)
		if err != nil {
			jm.mu.Lock()
			jm.err = err
			jm.mu.Unlock()
			return
		}

		completed := make(map[string]bool)
		runner := engine.Runner{
			Config: *jm.service.Config,
			Volume: v,
			Store:  jm.service.Store,
			Model:  caller,
			Progress: func(s string) {
				jm.AddLog(s)
			},
			Observer: progress.ObserverFunc(func(e progress.Event) {
				if e.UnitID != "" {
					switch e.Kind {
					case progress.EventReviewCommitted:
						completed[e.UnitID] = true
					case progress.EventRepairStarted:
						delete(completed, e.UnitID)
					case progress.EventUnitAccepted:
						completed[e.UnitID] = true
					}
				}
				if e.Message != "" {
					jm.AddLog(e.Message)
				}
				jm.mu.Lock()
				jm.phase = e.Phase
				jm.message = e.Message
				if e.UnitID != "" {
					jm.activeUnit = e.UnitID
				}
				if e.Model != "" {
					jm.activeModel = e.Model
				}
				jm.mu.Unlock()
				jm.broadcaster.Broadcast(e)
			}),
		}
		currentScope := scope
		for {
			err = runner.RunWithOptions(jobCtx, engine.RunOptions{
				Policy:   policy,
				Scope:    currentScope,
				IsPaused: jm.isPaused,
				PauseNotify: func() {
					jm.mu.Lock()
					jm.state = StatePaused
					jm.message = "Đã tạm dừng tại ranh giới khối."
					jm.mu.Unlock()
				},
			})
			if !errors.Is(err, engine.ErrPaused) {
				break
			}
			jm.mu.RLock()
			resumeCh := jm.resumeCh
			jm.mu.RUnlock()
			select {
			case <-jobCtx.Done():
				err = jobCtx.Err()
				break
			case <-resumeCh:
				jm.pauseMu.Lock()
				jm.pauseFlag = false
				jm.pauseMu.Unlock()
				currentScope.SkipUnitIDs = currentScope.SkipUnitIDs[:0]
				for id := range completed {
					currentScope.SkipUnitIDs = append(currentScope.SkipUnitIDs, id)
				}
				jm.mu.Lock()
				jm.state = StateRunning
				jm.message = "Tiếp tục từ ranh giới đã lưu."
				jm.mu.Unlock()
				continue
			}
			break
		}

		jm.mu.Lock()
		jm.err = err
		jm.mu.Unlock()
	}()

	return nil
}

func (jm *JobManager) StartImport(ctx context.Context, volumeID string) error {
	if jm.IsBusy() {
		return errors.New("tác vụ khác đang chạy")
	}
	if jm.service == nil || jm.service.Config == nil {
		return errors.New("chưa mở workspace")
	}
	jobCtx, cancel := context.WithCancel(ctx)
	jm.mu.Lock()
	jm.state = StateRunning
	jm.kind = "import"
	jm.label = "Import EPUB tập " + volumeID
	jm.volumeID = volumeID
	jm.phase = "import"
	jm.message = "Đang đọc EPUB và tạo snapshot nguồn…"
	jm.activeUnit = ""
	jm.activeModel = ""
	jm.logs = nil
	jm.startTime = time.Now()
	jm.cancelCtx = cancel
	jm.err = nil
	jm.mu.Unlock()
	jm.pauseMu.Lock()
	jm.pauseFlag = false
	jm.pauseMu.Unlock()
	go func() {
		defer cancel()
		_, err := jm.service.ImportVolume(jobCtx, volumeID)
		jm.mu.Lock()
		jm.err = err
		if err != nil {
			jm.phase = "failed"
			jm.message = "Import thất bại."
		} else {
			jm.phase = "done"
			jm.message = "Đã import snapshot EPUB."
		}
		if err != nil {
			jm.state = StateFailed
		} else {
			jm.state = StateDone
		}
		jm.cancelCtx = nil
		jm.mu.Unlock()
		kind := progress.EventRunFinished
		message := "Đã import snapshot EPUB."
		if err != nil {
			kind = progress.EventRunFailed
			message = err.Error()
			jm.AddLog("Import lỗi: " + err.Error())
		}
		jm.broadcaster.Broadcast(progress.Event{
			SchemaVersion: 1,
			EventID:       domain.Digest(struct{ Volume, Time string }{volumeID, time.Now().UTC().Format(time.RFC3339Nano)})[:24],
			RunID:         domain.Digest(struct{ Volume, Kind string }{volumeID, "import"})[:24],
			Time:          time.Now().UTC(),
			Kind:          kind,
			VolumeID:      volumeID,
			Phase:         "import",
			Message:       message,
			Error:         errorString(err),
		})
	}()
	return nil
}

func (jm *JobManager) StartExport(ctx context.Context, volumeID string, preview bool) error {
	if jm.IsBusy() {
		return errors.New("tác vụ khác đang chạy")
	}
	if jm.service == nil || jm.service.Config == nil {
		return errors.New("chưa mở workspace")
	}
	jobCtx, cancel := context.WithCancel(ctx)
	label := "Xuất EPUB hoàn chỉnh · " + volumeID
	if preview {
		label = "Xuất EPUB xem trước · " + volumeID
	}
	jm.mu.Lock()
	jm.state = StateRunning
	jm.kind = "export"
	jm.label = label
	jm.volumeID = volumeID
	jm.phase = "validate"
	jm.message = "Đang chuẩn bị kiểm tra đầu ra…"
	jm.activeUnit = ""
	jm.activeModel = ""
	jm.logs = nil
	jm.startTime = time.Now()
	jm.cancelCtx = cancel
	jm.err = nil
	jm.mu.Unlock()
	jm.pauseMu.Lock()
	jm.pauseFlag = false
	jm.pauseMu.Unlock()
	go func() {
		defer cancel()
		var sequence int
		emit := func(kind progress.EventKind, phase, message string, eventErr error) {
			sequence++
			jm.mu.Lock()
			jm.phase = phase
			jm.message = message
			jm.mu.Unlock()
			if message != "" {
				jm.AddLog(message)
			}
			jm.broadcaster.Broadcast(progress.Event{
				SchemaVersion: 1,
				EventID:       domain.Digest(struct{ Volume, Kind, Seq string }{volumeID, string(kind), fmt.Sprint(sequence)})[:24],
				RunID:         domain.Digest(struct{ Volume, Label string }{volumeID, label})[:24],
				Sequence:      sequence,
				Time:          time.Now().UTC(),
				Kind:          kind,
				VolumeID:      volumeID,
				Phase:         phase,
				Message:       message,
				Error:         errorString(eventErr),
			})
		}
		result, err := jm.service.ExportVolume(jobCtx, volumeID, preview, func(message string) {
			phase := "export"
			if strings.Contains(message, "EPUBCheck") {
				phase = "epubcheck"
			} else if strings.Contains(message, "đọc lại") {
				phase = "readback"
			} else if strings.Contains(message, "Đọc và kiểm tra") {
				phase = "validate"
			}
			emit(progress.EventExportStage, phase, message, nil)
		})
		if err == nil {
			emit(progress.EventExportResult, "done", "Đã xuất EPUB: "+result.Path, nil)
		} else {
			emit(progress.EventExportResult, "failed", "Xuất EPUB thất bại: "+err.Error(), err)
		}
		jm.mu.Lock()
		jm.err = err
		if err != nil {
			jm.state = StateFailed
		} else {
			jm.state = StateDone
		}
		jm.cancelCtx = nil
		jm.mu.Unlock()
	}()
	return nil
}

func errorString(err error) string {
	if err == nil {
		return ""
	}
	return err.Error()
}

func (jm *JobManager) StartReview(ctx context.Context, volumeID string, scope engine.RunScope, force bool) error {
	policy := engine.ReviewPolicy{
		ReviewOnly: true,
		Force:      force,
		AutoRepair: false,
	}
	return jm.StartTranslate(ctx, volumeID, scope, policy)
}

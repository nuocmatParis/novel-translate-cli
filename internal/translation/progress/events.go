package progress

import (
	"sync"
	"time"
)

type EventKind string

const (
	EventRunStarted       EventKind = "run_started"
	EventRunFinished      EventKind = "run_finished"
	EventRunFailed        EventKind = "run_failed"
	EventRunCancelled     EventKind = "run_cancelled"
	EventBatchStarted     EventKind = "batch_started"
	EventBatchFinished    EventKind = "batch_finished"
	EventUnitStarted      EventKind = "unit_started"
	EventContextPrepared  EventKind = "context_prepared"
	EventRequestStarted   EventKind = "request_started"
	EventRequestRetried   EventKind = "request_retried"
	EventRequestFallback  EventKind = "request_fallback"
	EventRequestCompleted EventKind = "request_completed"
	EventDraftCommitted   EventKind = "draft_committed"
	EventReviewStarted    EventKind = "review_started"
	EventReviewValidated  EventKind = "review_validated"
	EventReviewCommitted  EventKind = "review_committed"
	EventRepairStarted    EventKind = "repair_started"
	EventRepairProposed   EventKind = "repair_proposed"
	EventRepairCommitted  EventKind = "repair_committed"
	EventUnitAccepted     EventKind = "unit_accepted"
	EventPauseRequested   EventKind = "pause_requested"
	EventPaused           EventKind = "paused"
	EventResumed          EventKind = "resumed"
	EventExportStage      EventKind = "export_stage"
	EventExportResult     EventKind = "export_result"
)

type Event struct {
	SchemaVersion int           `json:"schema_version"`
	EventID       string        `json:"event_id"`
	RunID         string        `json:"run_id"`
	Sequence      int           `json:"sequence"`
	Time          time.Time     `json:"time"`
	Kind          EventKind     `json:"kind"`
	VolumeID      string        `json:"volume_id,omitempty"`
	ChapterID     string        `json:"chapter_id,omitempty"`
	UnitID        string        `json:"unit_id,omitempty"`
	ChildID       string        `json:"child_id,omitempty"`
	AttemptID     string        `json:"attempt_id,omitempty"`
	GenerationID  string        `json:"generation_id,omitempty"`
	Phase         string        `json:"phase,omitempty"`
	Message       string        `json:"message,omitempty"`
	Model         string        `json:"model,omitempty"`
	Role          string        `json:"role,omitempty"`
	Attempt       int           `json:"attempt,omitempty"`
	RepairRound   int           `json:"repair_round,omitempty"`
	Elapsed       time.Duration `json:"elapsed,omitempty"`
	Error         string        `json:"error,omitempty"`
	Data          any           `json:"data,omitempty"`
}

type Observer interface {
	OnEvent(Event)
}

type ObserverFunc func(Event)

func (f ObserverFunc) OnEvent(e Event) {
	if f != nil {
		f(e)
	}
}

type Broadcaster struct {
	mu        sync.RWMutex
	observers []Observer
}

func NewBroadcaster() *Broadcaster {
	return &Broadcaster{}
}

func (b *Broadcaster) Subscribe(obs Observer) {
	if obs == nil {
		return
	}
	b.mu.Lock()
	defer b.mu.Unlock()
	b.observers = append(b.observers, obs)
}

func (b *Broadcaster) Broadcast(e Event) {
	b.mu.RLock()
	defer b.mu.RUnlock()
	for _, o := range b.observers {
		o.OnEvent(e)
	}
}

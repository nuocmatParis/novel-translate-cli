use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

pub const DIAGNOSTIC_CAPACITY: usize = 256;

static NEXT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticComponent {
    Application,
    Cli,
    Terminal,
    Tui,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticEvent {
    StartupFailed,
    ArgumentsInvalid,
    TerminalUnavailable,
    TerminalSetupFailed,
    EventReadFailed,
    RenderFailed,
    TerminalCleanupFailed,
    Panic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticRecord {
    pub sequence: u64,
    pub severity: DiagnosticSeverity,
    pub component: DiagnosticComponent,
    pub event: DiagnosticEvent,
    pub occurrences: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticFinalError {
    pub primary_component: DiagnosticComponent,
    pub primary_event: DiagnosticEvent,
    pub cleanup: Option<(DiagnosticComponent, DiagnosticEvent)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticSnapshot {
    pub records: Vec<DiagnosticRecord>,
    pub dropped_count: u64,
    pub final_error: Option<DiagnosticFinalError>,
}

#[derive(Clone, Debug, Default)]
pub struct DiagnosticStore {
    state: Arc<Mutex<DiagnosticState>>,
}

#[derive(Debug, Default)]
struct DiagnosticState {
    records: VecDeque<DiagnosticRecord>,
    dropped_count: u64,
    final_error: Option<DiagnosticFinalError>,
}

impl DiagnosticStore {
    pub fn record(
        &self,
        severity: DiagnosticSeverity,
        component: DiagnosticComponent,
        event: DiagnosticEvent,
    ) {
        let mut state = self.lock();
        if let Some(last) = state.records.back_mut()
            && last.severity == severity
            && last.component == component
            && last.event == event
        {
            last.occurrences = last.occurrences.saturating_add(1);
            return;
        }

        let record = DiagnosticRecord {
            sequence: next_sequence(),
            severity,
            component,
            event,
            occurrences: 1,
        };
        if state.records.len() == DIAGNOSTIC_CAPACITY {
            state.records.pop_front();
            state.dropped_count = state.dropped_count.saturating_add(1);
        }
        state.records.push_back(record);
    }

    pub fn record_final_error(&self, error: DiagnosticFinalError) {
        self.lock().final_error = Some(error);
    }

    pub fn snapshot(&self) -> DiagnosticSnapshot {
        let state = self.lock();
        DiagnosticSnapshot {
            records: state.records.iter().copied().collect(),
            dropped_count: state.dropped_count,
            final_error: state.final_error,
        }
    }

    fn lock(&self) -> MutexGuard<'_, DiagnosticState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn next_sequence() -> u64 {
    loop {
        let current = NEXT_SEQUENCE.load(Ordering::Relaxed);
        let next = current.saturating_add(1);
        if NEXT_SEQUENCE
            .compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            return current;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DIAGNOSTIC_CAPACITY, DiagnosticComponent, DiagnosticEvent, DiagnosticFinalError,
        DiagnosticSeverity, DiagnosticStore,
    };

    #[test]
    fn coalesces_repeated_safe_events_without_storing_messages() {
        let store = DiagnosticStore::default();
        store.record(
            DiagnosticSeverity::Warn,
            DiagnosticComponent::Tui,
            DiagnosticEvent::RenderFailed,
        );
        store.record(
            DiagnosticSeverity::Warn,
            DiagnosticComponent::Tui,
            DiagnosticEvent::RenderFailed,
        );

        let snapshot = store.snapshot();
        assert_eq!(snapshot.records.len(), 1);
        assert_eq!(snapshot.records[0].occurrences, 2);
        assert_eq!(snapshot.dropped_count, 0);
    }

    #[test]
    fn bounds_burst_and_counts_dropped_old_records() {
        let store = DiagnosticStore::default();
        for index in 0..(DIAGNOSTIC_CAPACITY + 44) {
            let event = if index % 2 == 0 {
                DiagnosticEvent::RenderFailed
            } else {
                DiagnosticEvent::EventReadFailed
            };
            store.record(DiagnosticSeverity::Warn, DiagnosticComponent::Tui, event);
        }

        let snapshot = store.snapshot();
        assert_eq!(snapshot.records.len(), DIAGNOSTIC_CAPACITY);
        assert_eq!(snapshot.dropped_count, 44);
        assert!(
            snapshot
                .records
                .windows(2)
                .all(|pair| { pair[0].sequence < pair[1].sequence })
        );
    }

    #[test]
    fn final_error_survives_ring_overflow_as_typed_codes() {
        let store = DiagnosticStore::default();
        let final_error = DiagnosticFinalError {
            primary_component: DiagnosticComponent::Terminal,
            primary_event: DiagnosticEvent::RenderFailed,
            cleanup: Some((
                DiagnosticComponent::Terminal,
                DiagnosticEvent::TerminalCleanupFailed,
            )),
        };
        store.record_final_error(final_error);
        for index in 0..(DIAGNOSTIC_CAPACITY + 1) {
            let event = if index % 2 == 0 {
                DiagnosticEvent::Panic
            } else {
                DiagnosticEvent::StartupFailed
            };
            store.record(
                DiagnosticSeverity::Error,
                DiagnosticComponent::Application,
                event,
            );
        }

        assert_eq!(store.snapshot().final_error, Some(final_error));
    }
}

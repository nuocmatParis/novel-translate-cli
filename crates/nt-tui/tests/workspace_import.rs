use nt_application::workspace::{WorkspaceError, WorkspaceMetadata};
use nt_tui::WorkspaceCommands;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operation {
    Create,
    Open,
}

#[derive(Default)]
struct RecordingCommands {
    calls: Mutex<Vec<(Operation, PathBuf)>>,
}

impl WorkspaceCommands for RecordingCommands {
    fn create_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((Operation::Create, path.to_path_buf()));
        Err(WorkspaceError::StorageFailure)
    }

    fn open_workspace(&self, path: &Path) -> Result<WorkspaceMetadata, WorkspaceError> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((Operation::Open, path.to_path_buf()));
        Err(WorkspaceError::NotFound)
    }
}

fn assert_send_sync_static<T: Send + Sync + 'static + ?Sized>() {}

#[test]
fn public_workspace_command_port_can_run_off_thread_with_unicode_paths() {
    assert_send_sync_static::<dyn WorkspaceCommands>();

    let recorder = Arc::new(RecordingCommands::default());
    let service: Arc<dyn WorkspaceCommands> = recorder.clone();
    let path = PathBuf::from("C:\\Novel 世界\\Series 01");
    let worker = thread::spawn(move || service.open_workspace(&path));

    assert_eq!(
        worker.join().expect("workspace operation thread joins"),
        Err(WorkspaceError::NotFound)
    );
    assert_eq!(
        recorder
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_slice(),
        &[(Operation::Open, PathBuf::from("C:\\Novel 世界\\Series 01"))]
    );
}

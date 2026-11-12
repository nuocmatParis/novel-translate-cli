use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::thread::{self, JoinHandle};

use nt_application::generation::GenerationRepository;
use nt_application::object_store::ImmutableObjectStore;
use nt_application::workspace::{WorkspaceError, WorkspaceRepository};
use nt_domain::workspace::{
    GenerationCommitError, GenerationCommitResult, GenerationWriteRequest, OperationId, SeriesId,
};
use rusqlite::{Connection, OpenFlags};

use crate::FileObjectStore;
use crate::generation::{commit_request, fingerprint_request, lookup_operation};
use crate::workspace::{
    SCHEMA_VERSION, SqliteWorkspaceRepository, configure_existing_database,
    open_read_only_workspace,
};

const WRITER_QUEUE_CAPACITY: usize = 4;
const DATABASE_FILE: &str = "series.sqlite3";

enum WriterMessage {
    Commit {
        request: GenerationWriteRequest,
        fingerprint: [u8; 32],
        reply: SyncSender<Result<GenerationCommitResult, GenerationCommitError>>,
    },
    Shutdown,
}

/// One bounded, serial SQLite writer for a workspace Series.
pub struct SqliteGenerationWriter {
    inner: Arc<WriterInner>,
}

struct WriterInner {
    sender: SyncSender<WriterMessage>,
    worker: Mutex<Option<JoinHandle<()>>>,
    database_path: std::path::PathBuf,
    series_id: SeriesId,
}

static WRITER_REGISTRY: OnceLock<Mutex<HashMap<std::path::PathBuf, Weak<WriterInner>>>> =
    OnceLock::new();

impl SqliteGenerationWriter {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, GenerationCommitError> {
        let root = root.as_ref();
        let (read_only_connection, preflight) =
            open_read_only_workspace(root).map_err(map_workspace_error)?;
        drop(read_only_connection);
        let metadata = if preflight.schema_version == SCHEMA_VERSION {
            preflight
        } else {
            SqliteWorkspaceRepository
                .open(root)
                .map_err(map_workspace_error)?
        };
        let canonical_root =
            std::fs::canonicalize(root).map_err(|_| GenerationCommitError::StorageFailure)?;
        let database_path = std::fs::canonicalize(canonical_root.join(DATABASE_FILE))
            .map_err(|_| GenerationCommitError::StorageFailure)?;
        let registry = WRITER_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
        let mut registry = registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(inner) = registry.get(&database_path).and_then(Weak::upgrade) {
            if inner.series_id != metadata.series.id {
                return Err(GenerationCommitError::CorruptWorkspace);
            }
            return Ok(Self { inner });
        }
        registry.remove(&database_path);

        let store =
            FileObjectStore::new(root).map_err(|_| GenerationCommitError::StorageFailure)?;
        let series_id = metadata.series.id;
        let worker_series_id = series_id.clone();
        let (sender, receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(0);
        let worker_database_path = database_path.clone();

        let worker = thread::Builder::new()
            .name("nt-series-writer".to_owned())
            .spawn(move || {
                let initialization = open_writer_connection(&worker_database_path);
                match initialization {
                    Ok(mut connection) => {
                        if ready_sender.send(Ok(())).is_err() {
                            return;
                        }
                        writer_loop(&mut connection, &store, &worker_series_id, receiver);
                    }
                    Err(error) => {
                        let _ = ready_sender.send(Err(error));
                    }
                }
            })
            .map_err(|_| GenerationCommitError::WorkerUnavailable)?;

        match ready_receiver.recv() {
            Ok(Ok(())) => {
                let inner = Arc::new(WriterInner {
                    sender,
                    worker: Mutex::new(Some(worker)),
                    database_path: database_path.clone(),
                    series_id,
                });
                registry.insert(database_path, Arc::downgrade(&inner));
                Ok(Self { inner })
            }
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => {
                let _ = worker.join();
                Err(GenerationCommitError::WorkerUnavailable)
            }
        }
    }

    fn reconcile_unknown(
        inner: &WriterInner,
        operation_id: &OperationId,
        fingerprint: [u8; 32],
    ) -> Result<GenerationCommitResult, GenerationCommitError> {
        let connection = Connection::open_with_flags(
            &inner.database_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| GenerationCommitError::CommitOutcomeUnknown)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|_| GenerationCommitError::CommitOutcomeUnknown)?;
        lookup_operation(&connection, &inner.series_id, operation_id, fingerprint)
            .map_err(|error| match error {
                GenerationCommitError::OperationConflict
                | GenerationCommitError::CorruptWorkspace => error,
                _ => GenerationCommitError::CommitOutcomeUnknown,
            })?
            .ok_or(GenerationCommitError::CommitOutcomeUnknown)
    }
}

impl GenerationRepository for SqliteGenerationWriter {
    fn commit_generation(
        &self,
        request: GenerationWriteRequest,
    ) -> Result<GenerationCommitResult, GenerationCommitError> {
        let operation_id = request.operation_id().clone();
        let fingerprint = fingerprint_request(&self.inner.series_id, &request);
        let (reply_sender, reply_receiver) = mpsc::sync_channel(1);
        let message = WriterMessage::Commit {
            request,
            fingerprint,
            reply: reply_sender,
        };
        match self.inner.sender.try_send(message) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(GenerationCommitError::Busy),
            Err(TrySendError::Disconnected(_)) => {
                return Err(GenerationCommitError::WorkerUnavailable);
            }
        }

        match reply_receiver.recv() {
            Ok(result) => result,
            Err(_) => Self::reconcile_unknown(&self.inner, &operation_id, fingerprint),
        }
    }
}

impl Drop for WriterInner {
    fn drop(&mut self) {
        let _ = self.sender.send(WriterMessage::Shutdown);
        if let Ok(mut worker) = self.worker.lock()
            && let Some(worker) = worker.take()
        {
            let _ = worker.join();
        }
    }
}

fn open_writer_connection(path: &Path) -> Result<Connection, GenerationCommitError> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| GenerationCommitError::StorageFailure)?;
    configure_existing_database(&connection).map_err(map_workspace_error)?;
    let metadata = crate::workspace::validate_database(&connection).map_err(map_workspace_error)?;
    if metadata.schema_version != crate::workspace::SCHEMA_VERSION {
        return Err(GenerationCommitError::CorruptWorkspace);
    }
    Ok(connection)
}

fn writer_loop(
    connection: &mut Connection,
    store: &impl ImmutableObjectStore,
    series_id: &SeriesId,
    receiver: Receiver<WriterMessage>,
) {
    while let Ok(message) = receiver.recv() {
        match message {
            WriterMessage::Commit {
                request,
                fingerprint,
                reply,
            } => {
                let result = commit_request(connection, store, series_id, request, fingerprint);
                let _ = reply.send(result);
            }
            WriterMessage::Shutdown => break,
        }
    }
}

fn map_workspace_error(error: WorkspaceError) -> GenerationCommitError {
    match error {
        WorkspaceError::Busy => GenerationCommitError::Busy,
        WorkspaceError::CorruptWorkspace => GenerationCommitError::CorruptWorkspace,
        _ => GenerationCommitError::StorageFailure,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Condvar, Mutex, mpsc};
    use std::time::Duration;

    use nt_application::generation::GenerationRepository;
    use nt_application::object_store::{ImmutableObjectStore, ObjectInventory, ObjectStoreError};
    use nt_application::workspace::{WorkspaceRepository, create_workspace};
    use nt_domain::object::{ObjectHash, ObjectReference};
    use nt_domain::workspace::{
        GenerationCommitError, GenerationId, GenerationWriteRequest, OperationId,
    };
    use rusqlite::Connection;

    use super::{
        DATABASE_FILE, SqliteGenerationWriter, WRITER_QUEUE_CAPACITY, WriterInner, WriterMessage,
        open_writer_connection, writer_loop,
    };
    use crate::{FileObjectStore, SqliteWorkspaceRepository};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);
            let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nt-storage-p02-writer-{}-{sequence}",
                std::process::id()
            ));
            std::fs::create_dir(&path).expect("create isolated writer directory");
            Self(path)
        }

        fn workspace(&self) -> PathBuf {
            let root = self.0.join("series");
            create_workspace(&SqliteWorkspaceRepository, &root)
                .expect("create schema v2 workspace");
            root
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    struct FirstReadGate {
        entered: mpsc::SyncSender<()>,
        released: Mutex<bool>,
        changed: Condvar,
        used: AtomicBool,
    }

    impl FirstReadGate {
        fn wait(&self) {
            if !self.used.swap(true, Ordering::AcqRel) {
                let _ = self.entered.send(());
                let mut released = self.released.lock().expect("lock test gate");
                while !*released {
                    released = self.changed.wait(released).expect("wait for test gate");
                }
            }
        }

        fn release(&self) {
            *self.released.lock().expect("lock test gate") = true;
            self.changed.notify_all();
        }
    }

    struct GatedObjectStore {
        inner: FileObjectStore,
        first_read: FirstReadGate,
    }

    impl ImmutableObjectStore for GatedObjectStore {
        fn put(&self, source: &mut dyn std::io::Read) -> Result<ObjectReference, ObjectStoreError> {
            self.inner.put(source)
        }

        fn open_verified(
            &self,
            hash: ObjectHash,
        ) -> Result<Box<dyn std::io::Read + Send>, ObjectStoreError> {
            self.first_read.wait();
            self.inner.open_verified(hash)
        }

        fn inventory(
            &self,
            referenced: &BTreeSet<ObjectHash>,
        ) -> Result<ObjectInventory, ObjectStoreError> {
            self.inner.inventory(referenced)
        }
    }

    fn request(
        operation_id: usize,
        generation_id: usize,
        object: ObjectReference,
    ) -> GenerationWriteRequest {
        GenerationWriteRequest::new(
            OperationId::parse(format!("{operation_id:032x}")).expect("valid operation ID"),
            GenerationId::parse("g0").expect("valid parent generation"),
            GenerationId::parse(format!("{generation_id:032x}")).expect("valid generation ID"),
            vec![object],
        )
        .expect("valid generation write request")
    }

    #[test]
    fn dropping_writer_drains_all_accepted_requests_and_reports_full_queue_as_busy() {
        let temp = TestDirectory::new();
        let root = temp.workspace();
        let inner_store = FileObjectStore::new(&root).expect("open immutable object store");
        let object = inner_store
            .put(&mut &b"queued generation object"[..])
            .expect("publish immutable object");
        let metadata = SqliteWorkspaceRepository
            .open(&root)
            .expect("open workspace metadata");
        let (entered_sender, entered_receiver) = mpsc::sync_channel(0);
        let gate = FirstReadGate {
            entered: entered_sender,
            released: Mutex::new(false),
            changed: Condvar::new(),
            used: AtomicBool::new(false),
        };
        let store = Arc::new(GatedObjectStore {
            inner: inner_store,
            first_read: gate,
        });
        let worker_store = Arc::clone(&store);
        let (sender, receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY);
        let mut connection =
            open_writer_connection(&root.join(DATABASE_FILE)).expect("open writer connection");
        let worker_series_id = metadata.series.id.clone();
        let worker = std::thread::spawn(move || {
            writer_loop(
                &mut connection,
                worker_store.as_ref(),
                &worker_series_id,
                receiver,
            );
        });
        let writer = SqliteGenerationWriter {
            inner: Arc::new(WriterInner {
                sender,
                worker: Mutex::new(Some(worker)),
                database_path: root.join(DATABASE_FILE),
                series_id: metadata.series.id,
            }),
        };

        let first_request = request(1, 1, object);
        let first_fingerprint =
            super::super::fingerprint_request(&writer.inner.series_id, &first_request);
        let (first_reply_sender, first_reply_receiver) = mpsc::sync_channel(1);
        writer
            .inner
            .sender
            .try_send(WriterMessage::Commit {
                request: first_request,
                fingerprint: first_fingerprint,
                reply: first_reply_sender,
            })
            .expect("accept first request");
        if entered_receiver
            .recv_timeout(Duration::from_secs(3))
            .is_err()
        {
            store.first_read.release();
            panic!("worker did not reach the controlled pause");
        }
        assert!(matches!(
            first_reply_receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        let before_commit = Connection::open_with_flags(
            root.join(DATABASE_FILE),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("open read-only connection during the controlled pause");
        let current_before_commit: String = before_commit
            .query_row(
                "SELECT generation_id FROM current_generation WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .expect("read current generation before releasing the writer");
        assert_eq!(current_before_commit, "g0");
        drop(before_commit);

        let mut queued_replies = Vec::new();
        for index in 0..WRITER_QUEUE_CAPACITY {
            let queued_request = request(index + 2, index + 2, object);
            let fingerprint =
                super::super::fingerprint_request(&writer.inner.series_id, &queued_request);
            let (reply_sender, reply_receiver) = mpsc::sync_channel(1);
            writer
                .inner
                .sender
                .try_send(WriterMessage::Commit {
                    request: queued_request,
                    fingerprint,
                    reply: reply_sender,
                })
                .expect("accept request into bounded queue");
            queued_replies.push(reply_receiver);
        }

        assert_eq!(
            writer.commit_generation(request(100, 100, object)),
            Err(GenerationCommitError::Busy),
            "a request rejected by the full queue must report busy"
        );

        store.first_read.release();
        drop(writer);

        let first_result = first_reply_receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("first accepted request receives a reply");
        assert_eq!(
            first_result
                .expect("first accepted request commits")
                .generation_id
                .as_str(),
            "00000000000000000000000000000001"
        );
        for reply in queued_replies {
            assert_eq!(
                reply
                    .recv_timeout(Duration::from_secs(3))
                    .expect("queued accepted request receives a reply"),
                Err(GenerationCommitError::StaleGeneration),
                "queued requests are drained and checked against the committed parent"
            );
        }

        let connection = Connection::open(root.join(DATABASE_FILE)).expect("reopen workspace");
        let current: String = connection
            .query_row(
                "SELECT generation_id FROM current_generation WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .expect("read current generation after shutdown");
        let operation_count: i64 = connection
            .query_row("SELECT count(*) FROM generation_operations", [], |row| {
                row.get(0)
            })
            .expect("count committed requests after shutdown");
        assert_eq!(current, "00000000000000000000000000000001");
        assert_eq!(operation_count, 1, "rejected requests leave no ledger rows");
    }

    #[test]
    fn reopening_a_database_in_process_shares_one_writer_and_queue() {
        let temp = TestDirectory::new();
        let root = temp.workspace();
        let writer_one = SqliteGenerationWriter::open(&root).expect("open first writer");
        let writer_two = SqliteGenerationWriter::open(root.join("."))
            .expect("open the same canonical database through a path alias");
        assert!(
            Arc::ptr_eq(&writer_one.inner, &writer_two.inner),
            "same database handles share the single worker and bounded queue"
        );

        let worker = Arc::downgrade(&writer_one.inner);
        drop(writer_one);
        assert!(
            worker.upgrade().is_some(),
            "the second handle retains the worker"
        );
        drop(writer_two);
        assert!(
            worker.upgrade().is_none(),
            "last handle shuts down and releases worker"
        );
    }

    #[test]
    fn unreadable_ledger_after_lost_ack_stays_unknown_and_replays_after_recovery() {
        let temp = TestDirectory::new();
        let root = temp.workspace();
        let store = FileObjectStore::new(&root).expect("open immutable object store");
        let object = store
            .put(&mut &b"commit whose ledger lookup is temporarily unavailable"[..])
            .expect("publish immutable object");
        let metadata = SqliteWorkspaceRepository
            .open(&root)
            .expect("open workspace metadata");
        let worker_store = store;
        let worker_series_id = metadata.series.id.clone();
        let worker_database_path = root.join(DATABASE_FILE);
        let (sender, receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY);
        let worker = std::thread::spawn(move || {
            let mut connection =
                open_writer_connection(&worker_database_path).expect("open writer connection");
            if let Ok(WriterMessage::Commit {
                request,
                fingerprint,
                reply,
            }) = receiver.recv()
            {
                super::super::commit_request(
                    &mut connection,
                    &worker_store,
                    &worker_series_id,
                    request,
                    fingerprint,
                )
                .expect("commit before simulating the lost acknowledgement");
                connection
                    .execute_batch(
                        "ALTER TABLE generation_operations RENAME TO generation_operations_saved",
                    )
                    .expect("inject ledger lookup failure after the commit");
                drop(reply);
            }
        });
        let writer = SqliteGenerationWriter {
            inner: Arc::new(WriterInner {
                sender,
                worker: Mutex::new(Some(worker)),
                database_path: root.join(DATABASE_FILE),
                series_id: metadata.series.id,
            }),
        };

        assert_eq!(
            writer.commit_generation(request(302, 302, object)),
            Err(GenerationCommitError::CommitOutcomeUnknown),
            "a ledger read failure after lost ACK must not masquerade as a definite failure"
        );
        drop(writer);

        let connection = Connection::open(root.join(DATABASE_FILE)).expect("reopen workspace");
        connection
            .execute_batch(
                "ALTER TABLE generation_operations_saved RENAME TO generation_operations",
            )
            .expect("restore the committed operation ledger");
        drop(connection);

        let retry = SqliteGenerationWriter::open(&root).expect("reopen writer after recovery");
        let replay = retry
            .commit_generation(request(302, 302, object))
            .expect("same ID and fingerprint reconcile the committed result");
        assert!(replay.replayed);
        assert_eq!(
            replay.generation_id.as_str(),
            "0000000000000000000000000000012e"
        );
        drop(retry);

        let connection = Connection::open(root.join(DATABASE_FILE)).expect("inspect replay");
        let generation_count: i64 = connection
            .query_row("SELECT count(*) FROM generations", [], |row| row.get(0))
            .expect("count generations after replay");
        let operation_count: i64 = connection
            .query_row("SELECT count(*) FROM generation_operations", [], |row| {
                row.get(0)
            })
            .expect("count operations after replay");
        assert_eq!(generation_count, 2);
        assert_eq!(operation_count, 1);
    }

    #[test]
    fn lost_ack_is_reconciled_from_the_committed_operation_id() {
        let temp = TestDirectory::new();
        let root = temp.workspace();
        let store = FileObjectStore::new(&root).expect("open immutable object store");
        let object = store
            .put(&mut &b"commit whose acknowledgement is lost"[..])
            .expect("publish immutable object");
        let metadata = SqliteWorkspaceRepository
            .open(&root)
            .expect("open workspace metadata");
        let worker_store = store;
        let worker_series_id = metadata.series.id.clone();
        let worker_root = root.clone();
        let (sender, receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY);
        let worker = std::thread::spawn(move || {
            let mut connection = open_writer_connection(&worker_root.join(DATABASE_FILE))
                .expect("open writer connection");
            if let Ok(WriterMessage::Commit {
                request,
                fingerprint,
                reply,
            }) = receiver.recv()
            {
                super::super::commit_request(
                    &mut connection,
                    &worker_store,
                    &worker_series_id,
                    request,
                    fingerprint,
                )
                .expect("commit before simulating the lost acknowledgement");
                drop(reply);
            }
        });
        let writer = SqliteGenerationWriter {
            inner: Arc::new(WriterInner {
                sender,
                worker: Mutex::new(Some(worker)),
                database_path: root.join(DATABASE_FILE),
                series_id: metadata.series.id,
            }),
        };

        let result = writer
            .commit_generation(request(300, 300, object))
            .expect("recover the result from the operation ledger");
        assert_eq!(
            result.generation_id.as_str(),
            "0000000000000000000000000000012c"
        );
        assert!(result.replayed);
        drop(writer);

        let connection = Connection::open(root.join(DATABASE_FILE)).expect("reopen workspace");
        let operation_count: i64 = connection
            .query_row("SELECT count(*) FROM generation_operations", [], |row| {
                row.get(0)
            })
            .expect("count reconciled operation");
        assert_eq!(operation_count, 1);
    }
}

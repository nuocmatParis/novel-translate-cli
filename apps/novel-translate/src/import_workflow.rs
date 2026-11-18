//! Capture a selected EPUB for the importer without reopening the user's path.

mod classification_history;
mod eligibility;
mod persisted;
mod prepare;
mod prepare_source;
mod publication;
mod readiness;
mod scope;
mod scope_change;
mod scope_proof;

pub use eligibility::BlockingSource;
pub use persisted::{ImportFlowError, SavedVolume, read_saved_volume, read_volume_scope};
pub use prepare::prepare_import;
pub use publication::{ImportTarget, PreparedImport, Publication, prepare_staged_import};
pub use readiness::{ImportReadiness, ReviewIssue};
pub use scope_change::{Classification, ScopeChange, prepare_scope_change};

#[cfg(windows)]
use std::fs::{File, OpenOptions};
#[cfg(windows)]
use std::io::{self, Read};
#[cfg(windows)]
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

#[cfg(windows)]
use nt_application::object_store::ImmutableObjectStore;
use nt_application::object_store::ObjectStoreError;
use nt_application::volume_import::ObjectReference;
use nt_storage::FileObjectStore;

#[derive(Debug)]
pub enum CaptureError {
    SourceUnavailable,
    InvalidSource,
    SourceChanged,
    UnsupportedFilesystem,
    Storage(ObjectStoreError),
}

/// Capture one pinned regular file. The caller uses the returned content
/// reference to open the sealed, seekable object for all subsequent EPUB IO.
#[cfg(windows)]
pub fn capture_epub(store: &FileObjectStore, path: &Path) -> Result<ObjectReference, CaptureError> {
    let source = open_source(path)?;
    capture_open_source(store, source)
}

#[cfg(not(windows))]
pub fn capture_epub(
    _store: &FileObjectStore,
    _path: &Path,
) -> Result<ObjectReference, CaptureError> {
    Err(CaptureError::UnsupportedFilesystem)
}

#[cfg(windows)]
fn open_source(path: &Path) -> Result<File, CaptureError> {
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(source_error)
}

#[cfg(windows)]
fn source_error(error: io::Error) -> CaptureError {
    match error.raw_os_error() {
        Some(32 | 33) => CaptureError::SourceChanged,
        _ => CaptureError::SourceUnavailable,
    }
}

#[cfg(windows)]
fn capture_open_source(
    store: &FileObjectStore,
    source: File,
) -> Result<ObjectReference, CaptureError> {
    let before = source_stamp(&source)?;
    // The handle remains pinned for both observations and the whole transfer.
    // Read at most the original length plus one byte to detect growth.
    let mut reader = source.take(before.length + 1);
    let object = store.put(&mut reader).map_err(CaptureError::Storage)?;
    if source_stamp(reader.get_ref())? != before || object.byte_len != before.length {
        return Err(CaptureError::SourceChanged);
    }
    Ok(object)
}

#[cfg(windows)]
#[derive(Eq, PartialEq)]
struct SourceStamp {
    length: u64,
    created: u64,
    written: u64,
}

#[cfg(windows)]
fn source_stamp(source: &File) -> Result<SourceStamp, CaptureError> {
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    let meta = source.metadata().map_err(source_error)?;
    let length = meta.len();
    if !meta.is_file()
        || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || length == 0
        || length > i64::MAX as u64
    {
        return Err(CaptureError::InvalidSource);
    }
    Ok(SourceStamp {
        length,
        created: meta.creation_time(),
        written: meta.last_write_time(),
    })
}

#[cfg(all(test, windows))]
mod tests {
    use std::fs;
    use std::io::Read;
    use std::sync::atomic::{AtomicU64, Ordering};

    use nt_application::workspace::create_workspace;
    use nt_storage::SqliteWorkspaceRepository;

    use super::*;

    #[test]
    fn capture_rejects_existing_writer_and_keeps_the_sealed_bytes_after_path_changes() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        struct TestDirectory(std::path::PathBuf);
        impl Drop for TestDirectory {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }

        let root = std::env::temp_dir().join(format!(
            "nt-cp05-capture-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let temp = TestDirectory(root);
        let workspace = temp.0.join("workspace");
        create_workspace(&SqliteWorkspaceRepository, &workspace).unwrap();
        let store = FileObjectStore::new(&workspace).unwrap();
        let source_path = temp.0.join("巻 1.epub");
        let original = b"captured source bytes";
        fs::write(&source_path, original).unwrap();

        let competing_writer = OpenOptions::new().write(true).open(&source_path).unwrap();
        assert!(matches!(
            capture_epub(&store, &source_path),
            Err(CaptureError::SourceChanged)
        ));
        drop(competing_writer);

        let pinned = open_source(&source_path).unwrap();
        assert!(fs::write(&source_path, b"parallel edit").is_err());
        assert!(fs::rename(&source_path, temp.0.join("replacement.epub")).is_err());
        let object = capture_open_source(&store, pinned).unwrap();
        assert_eq!(object.byte_len, original.len() as u64);

        fs::write(&source_path, b"later source change").unwrap();
        let mut capture = store.open_import_object(object).unwrap();
        let mut actual = Vec::new();
        capture.read_to_end(&mut actual).unwrap();
        assert_eq!(&actual, original);
        assert_eq!(fs::read(&source_path).unwrap(), b"later source change");
    }
}

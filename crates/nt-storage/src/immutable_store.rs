use std::collections::BTreeSet;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use nt_application::object_store::{ImmutableObjectStore, ObjectInventory, ObjectStoreError};
use nt_domain::object::{ObjectHash, ObjectReference};
use sha2::{Digest, Sha256};

const OBJECT_DIRECTORY: &str = "objects";
const OBJECT_SUFFIX: &str = ".blob";
const TEMP_PREFIX: &str = ".nt-object-tmp-";
const TEMP_ATTEMPTS: u32 = 128;
const COPY_BUFFER_SIZE: usize = 64 * 1024;
#[cfg(windows)]
const WINDOWS_FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
#[cfg(windows)]
const WINDOWS_FILE_SHARE_READ: u32 = 0x0000_0001;
#[cfg(windows)]
const WINDOWS_FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

/// Filesystem adapter for immutable, content-addressed source objects.
pub struct FileObjectStore {
    workspace_root: PathBuf,
    files: Arc<dyn FileOperations>,
}

impl FileObjectStore {
    pub fn new(workspace_root: impl AsRef<Path>) -> Result<Self, ObjectStoreError> {
        let workspace_root = canonical_workspace_root(workspace_root.as_ref())?;
        Ok(Self {
            workspace_root,
            files: Arc::new(SystemFileOperations),
        })
    }

    #[cfg(test)]
    fn with_file_operations(
        workspace_root: impl AsRef<Path>,
        files: Arc<dyn FileOperations>,
    ) -> Result<Self, ObjectStoreError> {
        let workspace_root = canonical_workspace_root(workspace_root.as_ref())?;
        Ok(Self {
            workspace_root,
            files,
        })
    }

    fn ensure_object_directory(&self) -> Result<PathBuf, ObjectStoreError> {
        self.validate_workspace_root()?;
        let path = self.workspace_root.join(OBJECT_DIRECTORY);
        match fs::create_dir(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(map_io_error(error)),
        }
        self.validate_object_directory(&path)
    }

    fn existing_object_directory(&self) -> Result<Option<PathBuf>, ObjectStoreError> {
        self.validate_workspace_root()?;
        let path = self.workspace_root.join(OBJECT_DIRECTORY);
        match fs::symlink_metadata(&path) {
            Ok(_) => self.validate_object_directory(&path).map(Some),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(map_io_error(error)),
        }
    }

    fn validate_workspace_root(&self) -> Result<(), ObjectStoreError> {
        validate_no_reparse_components(&self.workspace_root)?;
        let metadata = fs::symlink_metadata(&self.workspace_root).map_err(map_io_error)?;
        if is_reparse_point(&metadata) || !metadata.is_dir() {
            return Err(ObjectStoreError::InvalidWorkspaceRoot);
        }
        let canonical = fs::canonicalize(&self.workspace_root).map_err(map_io_error)?;
        if canonical != self.workspace_root {
            return Err(ObjectStoreError::InvalidWorkspaceRoot);
        }
        Ok(())
    }

    fn validate_object_directory(&self, path: &Path) -> Result<PathBuf, ObjectStoreError> {
        let metadata = fs::symlink_metadata(path).map_err(map_io_error)?;
        if is_reparse_point(&metadata) || !metadata.is_dir() {
            return Err(ObjectStoreError::UnsupportedFilesystem);
        }
        validate_no_reparse_components(path)?;
        let canonical = fs::canonicalize(path).map_err(map_io_error)?;
        if !canonical.starts_with(&self.workspace_root) || canonical == self.workspace_root {
            return Err(ObjectStoreError::UnsupportedFilesystem);
        }
        Ok(canonical)
    }

    fn object_path(&self, object_directory: &Path, hash: ObjectHash) -> PathBuf {
        object_directory.join(format!("{hash}{OBJECT_SUFFIX}"))
    }

    fn create_temp_file(&self, object_directory: &Path) -> Result<TempFile, ObjectStoreError> {
        for _ in 0..TEMP_ATTEMPTS {
            let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
            let name = format!("{TEMP_PREFIX}{}-{sequence}.part", std::process::id());
            let path = object_directory.join(name);
            match self.files.create_temp(&path) {
                Ok(file) => {
                    return Ok(TempFile::new(path, file, Arc::clone(&self.files)));
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(map_io_error(error)),
            }
        }
        Err(ObjectStoreError::Io(io::Error::new(
            ErrorKind::AlreadyExists,
            "could not allocate a unique temporary object name",
        )))
    }

    fn open_verified_path(
        &self,
        path: &Path,
        expected_hash: ObjectHash,
    ) -> Result<(File, ObjectReference), ObjectStoreError> {
        let path_metadata = fs::symlink_metadata(path).map_err(map_read_error)?;
        if is_reparse_point(&path_metadata) || !path_metadata.is_file() {
            return Err(ObjectStoreError::CorruptObject);
        }

        let mut file = self.files.open_read(path).map_err(map_read_error)?;
        let file_metadata = file.metadata().map_err(map_io_error)?;
        if is_reparse_point(&file_metadata) || !file_metadata.is_file() {
            return Err(ObjectStoreError::CorruptObject);
        }

        let (actual_hash, byte_len) = hash_reader(&mut file)?;
        if actual_hash != expected_hash {
            return Err(ObjectStoreError::CorruptObject);
        }
        file.seek(SeekFrom::Start(0)).map_err(map_io_error)?;
        Ok((
            file,
            ObjectReference {
                hash: actual_hash,
                byte_len,
            },
        ))
    }

    fn compare_with_temp(
        &self,
        existing: &mut File,
        temp: &mut File,
    ) -> Result<bool, ObjectStoreError> {
        existing.seek(SeekFrom::Start(0)).map_err(map_io_error)?;
        temp.seek(SeekFrom::Start(0)).map_err(map_io_error)?;
        files_equal(existing, temp).map_err(map_io_error)
    }

    fn verify_published_object(
        &self,
        path: &Path,
        hash: ObjectHash,
        byte_len: u64,
        temp: &mut File,
    ) -> Result<ObjectReference, ObjectStoreError> {
        let (mut published, reference) = self.open_verified_path(path, hash)?;
        if reference.byte_len != byte_len || !self.compare_with_temp(&mut published, temp)? {
            return Err(ObjectStoreError::ObjectConflict);
        }
        Ok(reference)
    }

    fn verify_existing_destination(
        &self,
        path: &Path,
        hash: ObjectHash,
        byte_len: u64,
        temp: &mut File,
    ) -> Result<ObjectReference, ObjectStoreError> {
        let (mut existing, reference) =
            self.open_verified_path(path, hash)
                .map_err(|error| match error {
                    ObjectStoreError::ObjectNotFound
                    | ObjectStoreError::CorruptObject
                    | ObjectStoreError::UnsupportedFilesystem => ObjectStoreError::ObjectConflict,
                    other => other,
                })?;
        if reference.byte_len != byte_len || !self.compare_with_temp(&mut existing, temp)? {
            return Err(ObjectStoreError::ObjectConflict);
        }
        Ok(reference)
    }

    fn inventory_object_directory(
        &self,
        object_directory: &Path,
        referenced_hashes: &BTreeSet<ObjectHash>,
    ) -> Result<ObjectInventory, ObjectStoreError> {
        let mut inventory = ObjectInventory::default();
        let mut verified_hashes = BTreeSet::new();

        for entry in fs::read_dir(object_directory).map_err(map_io_error)? {
            let entry = entry.map_err(map_io_error)?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let metadata = fs::symlink_metadata(&path).map_err(map_io_error)?;
            if is_reparse_point(&metadata) || !metadata.is_file() {
                inventory.invalid_entries.push(name);
                continue;
            }

            if name.starts_with(TEMP_PREFIX) {
                inventory.incomplete_temps.push(name);
                continue;
            }

            let Some(hash_text) = name.strip_suffix(OBJECT_SUFFIX) else {
                inventory.invalid_entries.push(name);
                continue;
            };
            let Ok(hash) = ObjectHash::parse(hash_text) else {
                inventory.invalid_entries.push(name);
                continue;
            };

            match self.open_verified_path(&path, hash) {
                Ok((_, reference)) => {
                    verified_hashes.insert(hash);
                    if referenced_hashes.contains(&hash) {
                        inventory.referenced.push(reference);
                    } else {
                        inventory.unreferenced.push(reference);
                    }
                }
                Err(ObjectStoreError::CorruptObject | ObjectStoreError::UnsupportedFilesystem) => {
                    inventory.invalid_entries.push(name);
                }
                Err(error) => return Err(error),
            }
        }

        inventory.missing_references = referenced_hashes
            .difference(&verified_hashes)
            .copied()
            .collect();
        inventory.referenced.sort_by_key(|reference| reference.hash);
        inventory
            .unreferenced
            .sort_by_key(|reference| reference.hash);
        inventory.incomplete_temps.sort();
        inventory.invalid_entries.sort();
        Ok(inventory)
    }
}

impl ImmutableObjectStore for FileObjectStore {
    fn put(&self, source: &mut dyn Read) -> Result<ObjectReference, ObjectStoreError> {
        let object_directory = self.ensure_object_directory()?;
        let mut temp = self.create_temp_file(&object_directory)?;
        let (hash, byte_len) = stream_to_temp(source, temp.file_mut(), self.files.as_ref())?;
        self.files.sync_all(temp.file_mut()).map_err(map_io_error)?;
        temp.close();
        let (sealed_temp, temp_reference) = self.open_verified_path(temp.path(), hash)?;
        if temp_reference.byte_len != byte_len {
            return Err(ObjectStoreError::CorruptObject);
        }
        temp.set_file(sealed_temp);

        let destination = self.object_path(&object_directory, hash);
        match self.files.hard_link(temp.path(), &destination) {
            Ok(()) => {}
            Err(link_error) => match fs::symlink_metadata(&destination) {
                Ok(_) => {
                    let reference = self.verify_existing_destination(
                        &destination,
                        hash,
                        byte_len,
                        temp.file_mut(),
                    )?;
                    temp.remove()?;
                    return Ok(reference);
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    return Err(map_hard_link_error(link_error));
                }
                Err(error) => return Err(map_io_error(error)),
            },
        }

        let reference =
            self.verify_published_object(&destination, hash, byte_len, temp.file_mut())?;
        temp.remove()?;
        Ok(reference)
    }

    fn open_verified(&self, hash: ObjectHash) -> Result<Box<dyn Read + Send>, ObjectStoreError> {
        let object_directory = self
            .existing_object_directory()?
            .ok_or(ObjectStoreError::ObjectNotFound)?;
        let path = self.object_path(&object_directory, hash);
        let (file, _) = self.open_verified_path(&path, hash)?;
        Ok(Box::new(file))
    }

    fn inventory(
        &self,
        referenced: &BTreeSet<ObjectHash>,
    ) -> Result<ObjectInventory, ObjectStoreError> {
        match self.existing_object_directory()? {
            Some(object_directory) => {
                self.inventory_object_directory(&object_directory, referenced)
            }
            None => Ok(ObjectInventory {
                missing_references: referenced.iter().copied().collect(),
                ..ObjectInventory::default()
            }),
        }
    }
}

trait FileOperations: Send + Sync {
    fn create_temp(&self, path: &Path) -> io::Result<File>;
    fn write(&self, file: &mut File, bytes: &[u8]) -> io::Result<usize>;
    fn sync_all(&self, file: &File) -> io::Result<()>;
    fn hard_link(&self, source: &Path, destination: &Path) -> io::Result<()>;
    fn open_read(&self, path: &Path) -> io::Result<File>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
}

struct SystemFileOperations;

impl FileOperations for SystemFileOperations {
    fn create_temp(&self, path: &Path) -> io::Result<File> {
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options
                .share_mode(WINDOWS_FILE_SHARE_READ)
                .custom_flags(WINDOWS_FILE_FLAG_OPEN_REPARSE_POINT);
        }
        options.open(path)
    }

    fn write(&self, file: &mut File, bytes: &[u8]) -> io::Result<usize> {
        file.write(bytes)
    }

    fn sync_all(&self, file: &File) -> io::Result<()> {
        file.sync_all()
    }

    fn hard_link(&self, source: &Path, destination: &Path) -> io::Result<()> {
        fs::hard_link(source, destination)
    }

    fn open_read(&self, path: &Path) -> io::Result<File> {
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options
                .share_mode(WINDOWS_FILE_SHARE_READ)
                .custom_flags(WINDOWS_FILE_FLAG_OPEN_REPARSE_POINT);
        }
        options.open(path)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}

struct TempFile {
    path: PathBuf,
    file: Option<File>,
    files: Arc<dyn FileOperations>,
    armed: bool,
}

impl TempFile {
    fn new(path: PathBuf, file: File, files: Arc<dyn FileOperations>) -> Self {
        Self {
            path,
            file: Some(file),
            files,
            armed: true,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn file_mut(&mut self) -> &mut File {
        self.file.as_mut().expect("temporary file is still open")
    }

    fn close(&mut self) {
        self.file.take();
    }

    fn set_file(&mut self, file: File) {
        debug_assert!(self.file.is_none());
        self.file = Some(file);
    }

    fn remove(&mut self) -> Result<(), ObjectStoreError> {
        self.file.take();
        match self.files.remove_file(&self.path) {
            Ok(()) => {
                self.armed = false;
                Ok(())
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                self.armed = false;
                Ok(())
            }
            Err(error) => Err(map_io_error(error)),
        }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        self.file.take();
        if self.armed {
            let _ = self.files.remove_file(&self.path);
        }
    }
}

fn canonical_workspace_root(path: &Path) -> Result<PathBuf, ObjectStoreError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(map_io_error)?.join(path)
    };
    if absolute
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ObjectStoreError::InvalidWorkspaceRoot);
    }
    validate_no_reparse_components(&absolute)
        .map_err(|_| ObjectStoreError::InvalidWorkspaceRoot)?;
    let metadata =
        fs::symlink_metadata(&absolute).map_err(|_| ObjectStoreError::InvalidWorkspaceRoot)?;
    if is_reparse_point(&metadata) || !metadata.is_dir() {
        return Err(ObjectStoreError::InvalidWorkspaceRoot);
    }
    fs::canonicalize(absolute).map_err(|_| ObjectStoreError::InvalidWorkspaceRoot)
}

fn validate_no_reparse_components(path: &Path) -> Result<(), ObjectStoreError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => current.push(prefix.as_os_str()),
            Component::RootDir => current.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => return Err(ObjectStoreError::UnsupportedFilesystem),
            Component::Normal(part) => {
                current.push(part);
                let metadata = fs::symlink_metadata(&current).map_err(map_io_error)?;
                if is_reparse_point(&metadata) || !metadata.is_dir() {
                    return Err(ObjectStoreError::UnsupportedFilesystem);
                }
            }
        }
    }
    Ok(())
}

fn is_reparse_point(metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & WINDOWS_FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn stream_to_temp(
    source: &mut dyn Read,
    destination: &mut File,
    files: &dyn FileOperations,
) -> Result<(ObjectHash, u64), ObjectStoreError> {
    let mut hasher = Sha256::new();
    let mut byte_len = 0_u64;
    let mut buffer = [0_u8; COPY_BUFFER_SIZE];

    loop {
        let read = match source.read(&mut buffer) {
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(map_io_error(error)),
        };
        if read == 0 {
            break;
        }

        byte_len = byte_len
            .checked_add(read as u64)
            .ok_or_else(|| ObjectStoreError::Io(io::Error::other("object length overflow")))?;
        hasher.update(&buffer[..read]);
        write_all(files, destination, &buffer[..read]).map_err(map_io_error)?;
    }

    let digest = hasher.finalize();
    let mut hash_bytes = [0; 32];
    hash_bytes.copy_from_slice(&digest);
    Ok((ObjectHash::from_bytes(hash_bytes), byte_len))
}

fn write_all(
    files: &dyn FileOperations,
    destination: &mut File,
    mut bytes: &[u8],
) -> io::Result<()> {
    while !bytes.is_empty() {
        match files.write(destination, bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    ErrorKind::WriteZero,
                    "failed to write object",
                ));
            }
            Ok(written) => bytes = &bytes[written..],
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn hash_reader(reader: &mut dyn Read) -> Result<(ObjectHash, u64), ObjectStoreError> {
    let mut hasher = Sha256::new();
    let mut byte_len = 0_u64;
    let mut buffer = [0_u8; COPY_BUFFER_SIZE];

    loop {
        let read = match reader.read(&mut buffer) {
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(map_io_error(error)),
        };
        if read == 0 {
            break;
        }
        byte_len = byte_len
            .checked_add(read as u64)
            .ok_or_else(|| ObjectStoreError::Io(io::Error::other("object length overflow")))?;
        hasher.update(&buffer[..read]);
    }

    let digest = hasher.finalize();
    let mut hash_bytes = [0; 32];
    hash_bytes.copy_from_slice(&digest);
    Ok((ObjectHash::from_bytes(hash_bytes), byte_len))
}

fn files_equal(left: &mut dyn Read, right: &mut dyn Read) -> io::Result<bool> {
    let mut left_buffer = [0_u8; COPY_BUFFER_SIZE];
    let mut right_buffer = [0_u8; COPY_BUFFER_SIZE];

    loop {
        let left_read = read_retry(left, &mut left_buffer)?;
        let right_read = read_retry(right, &mut right_buffer)?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
    }
}

fn read_retry(reader: &mut dyn Read, buffer: &mut [u8]) -> io::Result<usize> {
    loop {
        match reader.read(buffer) {
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            result => return result,
        }
    }
}

fn map_read_error(error: io::Error) -> ObjectStoreError {
    if error.kind() == ErrorKind::NotFound {
        ObjectStoreError::ObjectNotFound
    } else {
        map_io_error(error)
    }
}

fn map_hard_link_error(error: io::Error) -> ObjectStoreError {
    if error.kind() == ErrorKind::Unsupported {
        ObjectStoreError::UnsupportedCapability
    } else {
        map_io_error(error)
    }
}

fn map_io_error(error: io::Error) -> ObjectStoreError {
    if error.kind() == ErrorKind::Unsupported {
        ObjectStoreError::UnsupportedCapability
    } else {
        ObjectStoreError::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{FileObjectStore, FileOperations, SystemFileOperations};
    use nt_application::object_store::{ImmutableObjectStore, ObjectStoreError};
    use std::fs::{self, File};
    use std::io::{self, ErrorKind};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(0);

    #[derive(Clone, Copy)]
    enum Failure {
        Write,
        Sync,
        Link,
    }

    struct FaultingFileOperations {
        failure: Failure,
        system: SystemFileOperations,
    }

    impl FileOperations for FaultingFileOperations {
        fn create_temp(&self, path: &Path) -> io::Result<File> {
            self.system.create_temp(path)
        }

        fn write(&self, file: &mut File, bytes: &[u8]) -> io::Result<usize> {
            if matches!(self.failure, Failure::Write) {
                return Err(io::Error::other("injected write failure"));
            }
            self.system.write(file, bytes)
        }

        fn sync_all(&self, file: &File) -> io::Result<()> {
            if matches!(self.failure, Failure::Sync) {
                return Err(io::Error::other("injected sync failure"));
            }
            self.system.sync_all(file)
        }

        fn hard_link(&self, source: &Path, destination: &Path) -> io::Result<()> {
            if matches!(self.failure, Failure::Link) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "injected unsupported hard link",
                ));
            }
            self.system.hard_link(source, destination)
        }

        fn open_read(&self, path: &Path) -> io::Result<File> {
            self.system.open_read(path)
        }

        fn remove_file(&self, path: &Path) -> io::Result<()> {
            self.system.remove_file(path)
        }
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "nt-object-store-unit-{}-{}",
                std::process::id(),
                NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("create test workspace root");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn write_failure_removes_only_the_operation_temp() {
        assert_fault_cleans_temp(Failure::Write, false);
    }

    #[test]
    fn sync_failure_removes_only_the_operation_temp() {
        assert_fault_cleans_temp(Failure::Sync, false);
    }

    #[test]
    fn unsupported_hard_link_returns_without_publishing_a_reference() {
        assert_fault_cleans_temp(Failure::Link, true);
    }

    fn assert_fault_cleans_temp(failure: Failure, unsupported: bool) {
        let root = TestDirectory::new();
        let outside = std::env::temp_dir().join(format!(
            "nt-object-store-sentinel-{}-{}",
            std::process::id(),
            NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&outside, b"keep outside bytes").expect("create outside sentinel");

        let files = Arc::new(FaultingFileOperations {
            failure,
            system: SystemFileOperations,
        });
        let store = FileObjectStore::with_file_operations(&root.0, files).unwrap();
        let error = store.put(&mut &b"object payload"[..]).unwrap_err();
        if unsupported {
            assert!(matches!(error, ObjectStoreError::UnsupportedCapability));
        } else {
            assert!(matches!(error, ObjectStoreError::Io(_)));
        }

        let object_dir = root.0.join("objects");
        let entries = fs::read_dir(&object_dir)
            .expect("object directory exists")
            .collect::<Result<Vec<_>, _>>()
            .expect("read object directory");
        assert!(
            entries.is_empty(),
            "operation temp or object was left behind"
        );
        assert_eq!(fs::read(&outside).unwrap(), b"keep outside bytes");
        fs::remove_file(outside).unwrap();
    }
}

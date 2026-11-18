//! Snapshot-scoped image copies. Each open source and parent directory remains
//! pinned while the returned handle is retained by the publication caller.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use nt_application::object_store::{ImmutableObjectStore, ObjectStoreError};
use nt_domain::object::{ObjectHash, ObjectReference};
use nt_domain::volume_import::{MAX_IMPORT_PAYLOAD_BYTES, StagedPayload};
use nt_domain::workspace::VolumeId;
use sha2::{Digest, Sha256};

use super::{
    FileObjectStore, canonical_workspace_root, hash_reader, is_reparse_point, map_io_error,
    map_read_error,
};

static NEXT_SOURCE_TEMP: AtomicU64 = AtomicU64::new(0);

pub struct FileImportSourceStore {
    root: PathBuf,
}

/// Keeping this value alive pins the verified file and its parent directories.
pub struct PinnedImportSource {
    pub relative_path: String,
    _file: File,
    _directories: Vec<File>,
}

struct PinnedImportObject {
    file: File,
    _directories: [File; 2],
}

impl Read for PinnedImportObject {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.file.read(bytes)
    }
}

impl Seek for PinnedImportObject {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.file.seek(position)
    }
}

impl FileObjectStore {
    /// Keep the verified object and its parent directories pinned for ZIP
    /// readers that seek back to the exact bytes they previously validated.
    pub fn open_import_object(
        &self,
        expected: ObjectReference,
    ) -> Result<impl Read + Seek + Send + use<>, ObjectStoreError> {
        let (directory, pins) = self.pin_import_directory(false)?;
        let path = self.object_path(&directory, expected.hash);
        let mut file = self.files.open_read(&path).map_err(map_read_error)?;
        let metadata = file.metadata().map_err(map_io_error)?;
        if is_reparse_point(&metadata) || !metadata.is_file() || metadata.len() != expected.byte_len
        {
            return Err(ObjectStoreError::CorruptObject);
        }
        let (hash, length) = hash_reader(&mut file)?;
        if hash != expected.hash || length != expected.byte_len {
            return Err(ObjectStoreError::CorruptObject);
        }
        file.rewind().map_err(map_io_error)?;
        Ok(PinnedImportObject {
            file,
            _directories: pins,
        })
    }

    /// Stream a canonical payload into temporary storage, then seal it through
    /// the existing immutable put path. A failed encoding publishes no object.
    pub fn stage_encoded_import(
        &self,
        encode: impl FnOnce(&mut dyn Write) -> io::Result<()>,
    ) -> Result<StagedPayload, ObjectStoreError> {
        let (directory, _pins) = self.pin_import_directory(true)?;
        let mut spool = self.create_temp_file(&directory)?;
        let length = {
            let mut writer = BufWriter::with_capacity(
                64 * 1024,
                PayloadWriter {
                    destination: spool.file_mut(),
                    length: 0,
                },
            );
            encode(&mut writer).map_err(map_io_error)?;
            writer.flush().map_err(map_io_error)?;
            writer.get_ref().length
        };
        if length == 0 {
            return Err(map_io_error(io::Error::new(
                io::ErrorKind::InvalidData,
                "Empty import payload.",
            )));
        }
        spool.file_mut().rewind().map_err(map_io_error)?;
        let object = self.put(spool.file_mut())?;
        if object.byte_len != length {
            return Err(ObjectStoreError::CorruptObject);
        }
        spool.remove()?;
        Ok(StagedPayload { object })
    }

    fn pin_import_directory(&self, create: bool) -> Result<(PathBuf, [File; 2]), ObjectStoreError> {
        self.validate_workspace_root()?;
        let root = open_directory(&self.workspace_root)?;
        let directory = if create {
            self.ensure_object_directory()?
        } else {
            self.existing_object_directory()?
                .ok_or(ObjectStoreError::ObjectNotFound)?
        };
        let objects = open_directory(&directory)?;
        self.validate_object_directory(&directory)?;
        Ok((directory, [root, objects]))
    }
}

struct PayloadWriter<'a> {
    destination: &'a mut File,
    length: u64,
}

impl Write for PayloadWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .length
            .checked_add(bytes.len() as u64)
            .is_none_or(|length| length > MAX_IMPORT_PAYLOAD_BYTES)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Import payload byte limit exceeded.",
            ));
        }
        let written = self.destination.write(bytes)?;
        self.length += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.destination.flush()
    }
}

impl FileImportSourceStore {
    pub(crate) fn validate_resource_path(path: &str) -> Result<(), ObjectStoreError> {
        validate_relative_path(path)
    }

    pub fn new(root: impl AsRef<Path>) -> Result<Self, ObjectStoreError> {
        Ok(Self {
            root: canonical_workspace_root(root.as_ref())?,
        })
    }

    pub fn source_relative_path(
        volume: &VolumeId,
        epub_sha256: [u8; 32],
        canonical_path: &str,
    ) -> Result<String, ObjectStoreError> {
        validate_relative_path(canonical_path)?;
        let snapshot = ObjectHash::from_bytes(epub_sha256);
        Ok(format!(
            "volumes/{}/snapshots/{snapshot}/images/source/{canonical_path}",
            volume.as_str()
        ))
    }

    /// Materialize into a newly created inode, never into the object-store inode.
    /// A pre-existing destination is only reused when its verified bytes match.
    pub fn materialize(
        &self,
        volume: &VolumeId,
        epub_sha256: [u8; 32],
        canonical_path: &str,
        expected: ObjectReference,
        source: &mut dyn Read,
    ) -> Result<PinnedImportSource, ObjectStoreError> {
        let relative = Self::source_relative_path(volume, epub_sha256, canonical_path)?;
        let (parent, name) = split_relative(&relative)?;
        let (directory, pins) = self.pin_parent(parent, true)?;
        ensure_exact_case(&directory, name)?;
        let destination = directory.join(name);
        if destination.exists() {
            return verify_destination(destination, relative, expected, pins);
        }

        let (temporary, mut file) = create_temp(&directory)?;
        let result = (|| {
            let mut hash = Sha256::new();
            let mut total = 0_u64;
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let count = source.read(&mut buffer).map_err(map_io_error)?;
                if count == 0 {
                    break;
                }
                total = total
                    .checked_add(count as u64)
                    .ok_or(ObjectStoreError::CorruptObject)?;
                if total > expected.byte_len {
                    return Err(ObjectStoreError::ObjectConflict);
                }
                hash.update(&buffer[..count]);
                file.write_all(&buffer[..count]).map_err(map_io_error)?;
            }
            file.sync_all().map_err(map_io_error)?;
            let digest: [u8; 32] = hash.finalize().into();
            if digest != *expected.hash.as_bytes() || total != expected.byte_len {
                return Err(ObjectStoreError::ObjectConflict);
            }
            match fs::hard_link(&temporary, &destination) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(map_io_error(error)),
            }
            Ok(())
        })();
        drop(file);
        let _ = fs::remove_file(&temporary);
        result?;
        ensure_exact_case(&directory, name)?;
        verify_destination(destination, relative, expected, pins)
    }

    pub fn open_verified(
        &self,
        volume: &VolumeId,
        epub_sha256: [u8; 32],
        canonical_path: &str,
        expected: ObjectReference,
    ) -> Result<PinnedImportSource, ObjectStoreError> {
        let relative = Self::source_relative_path(volume, epub_sha256, canonical_path)?;
        let (parent, name) = split_relative(&relative)?;
        let (directory, pins) = self.pin_parent(parent, false)?;
        ensure_exact_case(&directory, name)?;
        verify_destination(directory.join(name), relative, expected, pins)
    }

    fn pin_parent(
        &self,
        parent: &str,
        create: bool,
    ) -> Result<(PathBuf, Vec<File>), ObjectStoreError> {
        let mut path = self.root.clone();
        let mut pins = vec![open_directory(&path)?];
        for component in parent.split('/') {
            ensure_exact_case(&path, component)?;
            path.push(component);
            if create {
                match fs::create_dir(&path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(map_io_error(error)),
                }
            }
            let pin = open_directory(&path)?;
            let canonical = fs::canonicalize(&path).map_err(map_io_error)?;
            if !canonical.starts_with(&self.root) || canonical == self.root {
                return Err(ObjectStoreError::UnsupportedFilesystem);
            }
            pins.push(pin);
        }
        Ok((path, pins))
    }
}

fn split_relative(relative: &str) -> Result<(&str, &str), ObjectStoreError> {
    relative
        .rsplit_once('/')
        .ok_or(ObjectStoreError::UnsupportedFilesystem)
}

fn validate_relative_path(path: &str) -> Result<(), ObjectStoreError> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return Err(ObjectStoreError::UnsupportedFilesystem);
    }
    for part in path.split('/') {
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|ch| ch <= '\u{1f}' || matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        {
            return Err(ObjectStoreError::UnsupportedFilesystem);
        }
        let stem = part
            .split('.')
            .next()
            .unwrap_or_default()
            .trim_end_matches([' ', '.'])
            .to_ascii_uppercase();
        if matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
        ) || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(
                    suffix,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        }) {
            return Err(ObjectStoreError::UnsupportedFilesystem);
        }
    }
    Ok(())
}

fn ensure_exact_case(parent: &Path, desired: &str) -> Result<(), ObjectStoreError> {
    let key = desired.to_lowercase();
    for entry in fs::read_dir(parent).map_err(map_io_error)? {
        let entry = entry.map_err(map_io_error)?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(ObjectStoreError::UnsupportedFilesystem);
        };
        if name.to_lowercase() == key && name != desired {
            return Err(ObjectStoreError::ObjectConflict);
        }
    }
    Ok(())
}

fn verify_destination(
    path: PathBuf,
    relative_path: String,
    expected: ObjectReference,
    directories: Vec<File>,
) -> Result<PinnedImportSource, ObjectStoreError> {
    let metadata = fs::symlink_metadata(&path).map_err(map_io_error)?;
    if is_reparse_point(&metadata) || !metadata.is_file() {
        return Err(ObjectStoreError::UnsupportedFilesystem);
    }
    let mut file = open_source(&path)?;
    let metadata = file.metadata().map_err(map_io_error)?;
    if is_reparse_point(&metadata) || !metadata.is_file() {
        return Err(ObjectStoreError::UnsupportedFilesystem);
    }
    let (hash, byte_len) = hash_reader(&mut file)?;
    if hash != expected.hash || byte_len != expected.byte_len {
        return Err(ObjectStoreError::ObjectConflict);
    }
    file.seek(SeekFrom::Start(0)).map_err(map_io_error)?;
    Ok(PinnedImportSource {
        relative_path,
        _file: file,
        _directories: directories,
    })
}

fn create_temp(directory: &Path) -> Result<(PathBuf, File), ObjectStoreError> {
    for _ in 0..128 {
        let sequence = NEXT_SOURCE_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = directory.join(format!(".nt-source-tmp-{}-{sequence}", std::process::id()));
        match open_temp(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(map_io_error(error)),
        }
    }
    Err(ObjectStoreError::UnsupportedFilesystem)
}

#[cfg(windows)]
fn open_directory(path: &Path) -> Result<File, ObjectStoreError> {
    use std::os::windows::fs::OpenOptionsExt;
    let file = OpenOptions::new()
        .read(true)
        .share_mode(0x0000_0003)
        .custom_flags(0x0200_0000 | 0x0020_0000)
        .open(path)
        .map_err(map_io_error)?;
    let metadata = file.metadata().map_err(map_io_error)?;
    if is_reparse_point(&metadata) || !metadata.is_dir() {
        return Err(ObjectStoreError::UnsupportedFilesystem);
    }
    Ok(file)
}
#[cfg(not(windows))]
fn open_directory(_path: &Path) -> Result<File, ObjectStoreError> {
    Err(ObjectStoreError::UnsupportedFilesystem)
}

#[cfg(windows)]
fn open_source(path: &Path) -> Result<File, ObjectStoreError> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .share_mode(0x0000_0001)
        .custom_flags(0x0020_0000)
        .open(path)
        .map_err(map_io_error)
}
#[cfg(not(windows))]
fn open_source(_path: &Path) -> Result<File, ObjectStoreError> {
    Err(ObjectStoreError::UnsupportedFilesystem)
}

#[cfg(windows)]
fn open_temp(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .share_mode(0x0000_0003)
        .custom_flags(0x0020_0000)
        .open(path)
}
#[cfg(not(windows))]
fn open_temp(_path: &Path) -> std::io::Result<File> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Windows materialization required",
    ))
}

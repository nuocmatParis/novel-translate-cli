use std::collections::HashMap;
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicU64, Ordering};

use zip::CompressionMethod;
use zip::read::{ArchiveOffset, Config, ZipArchive};

use crate::path::{ArchivePathError, canonical_archive_path, windows_materialization_key};

const EOCD_SIGNATURE: u32 = 0x0605_4b50;
const ZIP64_EOCD_SIGNATURE: u32 = 0x0606_4b50;
const ZIP64_LOCATOR_SIGNATURE: u32 = 0x0706_4b50;
const CENTRAL_DIRECTORY_SIGNATURE: u32 = 0x0201_4b50;
const CENTRAL_DIRECTORY_DIGITAL_SIGNATURE: u32 = 0x0505_4b50;
const EOCD_FIXED_BYTES: usize = 22;
const ZIP64_LOCATOR_BYTES: usize = 20;
const ZIP64_EOCD_FIXED_BYTES: usize = 56;
const CENTRAL_DIRECTORY_FIXED_BYTES: usize = 46;
const MAX_EOCD_SCAN_BYTES: usize = EOCD_FIXED_BYTES + u16::MAX as usize;
const DEFAULT_MAX_CENTRAL_DIRECTORY_BYTES: u64 = 64 * 1024 * 1024;
const READ_BUFFER_BYTES: usize = 64 * 1024;

static NEXT_ARCHIVE_ID: AtomicU64 = AtomicU64::new(1);

/// Resource limits for one archive read session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveLimits {
    pub max_entries: usize,
    pub max_entry_bytes: u64,
    pub max_total_bytes: u64,
    pub max_central_directory_bytes: u64,
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_entry_bytes: 128 * 1024 * 1024,
            max_total_bytes: 1024 * 1024 * 1024,
            max_central_directory_bytes: DEFAULT_MAX_CENTRAL_DIRECTORY_BYTES,
        }
    }
}

/// Identity for one central-directory entry in one [`EpubZip`] instance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EntryId {
    archive_id: u64,
    index: usize,
}

/// Validated metadata for one ZIP entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveEntry {
    id: EntryId,
    name: String,
    canonical_path: String,
    declared_size: u64,
    is_directory: bool,
}

impl ArchiveEntry {
    pub fn id(&self) -> EntryId {
        self.id
    }

    /// The decoded name from the ZIP central directory, preserving its case.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The archive-relative path after collapsing empty and `.` components.
    pub fn canonical_path(&self) -> &str {
        &self.canonical_path
    }

    /// The uncompressed size claimed by the ZIP central directory.
    pub fn declared_size(&self) -> u64 {
        self.declared_size
    }

    pub fn is_directory(&self) -> bool {
        self.is_directory
    }
}

/// A bounded reader over one EPUB ZIP container.
pub struct EpubZip<R> {
    archive: ZipArchive<R>,
    entries: Vec<ArchiveEntry>,
    by_canonical_path: HashMap<String, EntryId>,
    limits: ArchiveLimits,
    archive_id: u64,
    expanded_bytes_consumed: u64,
    failed_read_entry: Option<String>,
}

struct ContainerMetadata {
    entries: u64,
    archive_offset: u64,
}

struct Zip64EndRecord {
    entries: u64,
    directory_bytes: u64,
    directory_offset: u64,
    record_offset: u64,
    physical_offset: u64,
}

/// A container, path, limit, or entry read failure.
#[derive(Debug)]
pub enum ArchiveError {
    Io(io::Error),
    Zip(zip::result::ZipError),
    InvalidContainer(&'static str),
    MultiDiskArchive,
    TooManyEntries {
        actual: u64,
        limit: usize,
    },
    CentralDirectoryTooLarge {
        actual: u64,
        limit: u64,
    },
    EntryPath {
        entry: String,
        source: ArchivePathError,
    },
    DuplicateCanonicalPath {
        first: String,
        duplicate: String,
    },
    WindowsMaterializationCollision {
        first: String,
        second: String,
    },
    UnsupportedCompression {
        entry: String,
        method: CompressionMethod,
    },
    UnknownEntry(String),
    ForeignEntryId,
    EntryDeclaredTooLarge {
        entry: String,
        declared: u64,
        limit: u64,
    },
    TotalDeclaredTooLarge {
        entry: String,
        declared: u64,
        limit: u64,
    },
    EntryLimitExceeded {
        entry: String,
        limit: u64,
        observed: u64,
    },
    TotalLimitExceeded {
        entry: String,
        limit: u64,
        observed: u64,
    },
    EntrySizeMismatch {
        entry: String,
        declared: u64,
        actual: u64,
    },
    EntryRead {
        entry: String,
        source: io::Error,
    },
    ReadSessionFailed {
        failed_entry: String,
        attempted_entry: String,
    },
    AllocationFailed {
        entry: String,
    },
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "reading ZIP container: {error}"),
            Self::Zip(error) => write!(f, "reading ZIP container: {error}"),
            Self::InvalidContainer(reason) => write!(f, "invalid ZIP container: {reason}"),
            Self::MultiDiskArchive => f.write_str("multi-disk ZIP archives are not supported"),
            Self::TooManyEntries { actual, limit } => {
                write!(f, "ZIP has {actual} entries; limit is {limit}")
            }
            Self::CentralDirectoryTooLarge { actual, limit } => write!(
                f,
                "ZIP central directory has {actual} bytes; limit is {limit}"
            ),
            Self::EntryPath { entry, source } => {
                write!(f, "invalid ZIP entry `{entry}`: {source}")
            }
            Self::DuplicateCanonicalPath { first, duplicate } => write!(
                f,
                "ZIP entries `{first}` and `{duplicate}` resolve to the same archive path"
            ),
            Self::WindowsMaterializationCollision { first, second } => write!(
                f,
                "ZIP entries `{first}` and `{second}` collide on Windows when materialized"
            ),
            Self::UnsupportedCompression { entry, method } => {
                write!(
                    f,
                    "ZIP entry `{entry}` uses unsupported compression {method:?}"
                )
            }
            Self::UnknownEntry(entry) => write!(f, "ZIP entry `{entry}` does not exist"),
            Self::ForeignEntryId => f.write_str("entry identity belongs to another ZIP archive"),
            Self::EntryDeclaredTooLarge {
                entry,
                declared,
                limit,
            } => write!(
                f,
                "ZIP entry `{entry}` declares {declared} bytes; per-entry limit is {limit}"
            ),
            Self::TotalDeclaredTooLarge {
                entry,
                declared,
                limit,
            } => write!(
                f,
                "ZIP entry `{entry}` declares {declared} bytes; remaining import limit is {limit}"
            ),
            Self::EntryLimitExceeded {
                entry,
                limit,
                observed,
            } => write!(
                f,
                "ZIP entry `{entry}` expands beyond its {limit}-byte limit (observed at least {observed})"
            ),
            Self::TotalLimitExceeded {
                entry,
                limit,
                observed,
            } => write!(
                f,
                "ZIP entry `{entry}` exceeds the {limit}-byte total import limit (observed at least {observed})"
            ),
            Self::EntrySizeMismatch {
                entry,
                declared,
                actual,
            } => write!(
                f,
                "ZIP entry `{entry}` declares {declared} bytes but expands to {actual}"
            ),
            Self::EntryRead { entry, source } => {
                write!(f, "reading ZIP entry `{entry}`: {source}")
            }
            Self::ReadSessionFailed {
                failed_entry,
                attempted_entry,
            } => write!(
                f,
                "ZIP read session stopped after `{failed_entry}` failed; cannot read `{attempted_entry}`"
            ),
            Self::AllocationFailed { entry } => {
                write!(
                    f,
                    "cannot allocate the bounded buffer for ZIP entry `{entry}`"
                )
            }
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) | Self::EntryRead { source: error, .. } => Some(error),
            Self::Zip(error) => Some(error),
            Self::InvalidContainer(_)
            | Self::MultiDiskArchive
            | Self::TooManyEntries { .. }
            | Self::CentralDirectoryTooLarge { .. }
            | Self::EntryPath { .. }
            | Self::DuplicateCanonicalPath { .. }
            | Self::WindowsMaterializationCollision { .. }
            | Self::UnsupportedCompression { .. }
            | Self::UnknownEntry(_)
            | Self::ForeignEntryId
            | Self::EntryDeclaredTooLarge { .. }
            | Self::TotalDeclaredTooLarge { .. }
            | Self::EntryLimitExceeded { .. }
            | Self::TotalLimitExceeded { .. }
            | Self::EntrySizeMismatch { .. }
            | Self::ReadSessionFailed { .. }
            | Self::AllocationFailed { .. } => None,
        }
    }
}

impl<R: Read + Seek> EpubZip<R> {
    pub fn new(reader: R) -> Result<Self, ArchiveError> {
        Self::with_limits(reader, ArchiveLimits::default())
    }

    pub fn with_limits(mut reader: R, limits: ArchiveLimits) -> Result<Self, ArchiveError> {
        let preflight = inspect_container(&mut reader, &limits)?;
        let mut archive = ZipArchive::with_config(
            Config {
                archive_offset: ArchiveOffset::Known(preflight.archive_offset),
            },
            reader,
        )
        .map_err(ArchiveError::Zip)?;
        if archive.len() as u64 != preflight.entries {
            return Err(ArchiveError::InvalidContainer(
                "central-directory count differs from the end record",
            ));
        }

        let archive_id = NEXT_ARCHIVE_ID.fetch_add(1, Ordering::Relaxed);
        let mut entries: Vec<ArchiveEntry> = Vec::with_capacity(archive.len());
        let mut by_canonical_path: HashMap<String, EntryId> = HashMap::with_capacity(archive.len());

        for index in 0..archive.len() {
            let file = archive.by_index_raw(index).map_err(ArchiveError::Zip)?;
            let name = file.name().map_err(ArchiveError::Zip)?.into_owned();
            let canonical_path =
                canonical_archive_path(&name).map_err(|source| ArchiveError::EntryPath {
                    entry: name.clone(),
                    source,
                })?;
            if let Some(first_id) = by_canonical_path.get(&canonical_path) {
                let first = entries[first_id.index].name.clone();
                return Err(ArchiveError::DuplicateCanonicalPath {
                    first,
                    duplicate: name,
                });
            }

            if !matches!(
                file.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            ) {
                return Err(ArchiveError::UnsupportedCompression {
                    entry: name,
                    method: file.compression(),
                });
            }

            let id = EntryId { archive_id, index };
            let entry = ArchiveEntry {
                id,
                name,
                canonical_path: canonical_path.clone(),
                declared_size: file.size(),
                is_directory: file.is_dir(),
            };
            by_canonical_path.insert(canonical_path, id);
            entries.push(entry);
        }

        Ok(Self {
            archive,
            entries,
            by_canonical_path,
            limits,
            archive_id,
            expanded_bytes_consumed: 0,
            failed_read_entry: None,
        })
    }

    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    pub fn entry_by_path(&self, path: &str) -> Result<Option<&ArchiveEntry>, ArchiveError> {
        let canonical = canonical_archive_path(path).map_err(|source| ArchiveError::EntryPath {
            entry: path.to_owned(),
            source,
        })?;
        Ok(self
            .by_canonical_path
            .get(&canonical)
            .and_then(|id| self.entries.get(id.index)))
    }

    /// Validate archive paths against case-insensitive Windows destinations,
    /// including conflicts where a file occupies another entry's parent path.
    /// Call this before any consumer materializes multiple entries to disk.
    pub fn validate_windows_materialization(&self) -> Result<(), ArchiveError> {
        let mut by_destination =
            HashMap::<String, &ArchiveEntry>::with_capacity(self.entries.len());
        for entry in &self.entries {
            let key = windows_materialization_key(&entry.canonical_path);
            if let Some(first) = by_destination.get(&key) {
                if first.canonical_path != entry.canonical_path {
                    return Err(ArchiveError::WindowsMaterializationCollision {
                        first: first.name.clone(),
                        second: entry.name.clone(),
                    });
                }
            } else {
                by_destination.insert(key, entry);
            }
        }

        for entry in &self.entries {
            let mut prefix = String::new();
            let components: Vec<_> = entry.canonical_path.split('/').collect();
            for component in components.iter().take(components.len().saturating_sub(1)) {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(component);
                if let Some(parent) = by_destination.get(&windows_materialization_key(&prefix))
                    && !parent.is_directory
                {
                    return Err(ArchiveError::WindowsMaterializationCollision {
                        first: parent.name.clone(),
                        second: entry.name.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Read one complete entry. On any read-time error the partial buffer is
    /// discarded and the archive read session is closed. This prevents retrying
    /// a failed entry or switching entries to bypass cumulative limits.
    pub fn read_entry(&mut self, id: EntryId) -> Result<Vec<u8>, ArchiveError> {
        if let Some(failed_entry) = &self.failed_read_entry {
            let attempted_entry = if id.archive_id == self.archive_id {
                self.entries
                    .get(id.index)
                    .map(|entry| entry.name.clone())
                    .unwrap_or_else(|| format!("entry index {}", id.index))
            } else {
                "foreign entry identity".to_owned()
            };
            return Err(ArchiveError::ReadSessionFailed {
                failed_entry: failed_entry.clone(),
                attempted_entry,
            });
        }
        if id.archive_id != self.archive_id {
            return Err(ArchiveError::ForeignEntryId);
        }
        let entry = self
            .entries
            .get(id.index)
            .cloned()
            .ok_or_else(|| ArchiveError::UnknownEntry(id.index.to_string()))?;

        if entry.declared_size > self.limits.max_entry_bytes {
            return Err(ArchiveError::EntryDeclaredTooLarge {
                entry: entry.name,
                declared: entry.declared_size,
                limit: self.limits.max_entry_bytes,
            });
        }

        let remaining_total = self
            .limits
            .max_total_bytes
            .saturating_sub(self.expanded_bytes_consumed);
        if entry.declared_size > remaining_total {
            return Err(ArchiveError::TotalDeclaredTooLarge {
                entry: entry.name,
                declared: entry.declared_size,
                limit: remaining_total,
            });
        }

        let total_is_tighter = remaining_total < self.limits.max_entry_bytes;
        let read_limit = remaining_total.min(self.limits.max_entry_bytes);
        let name = entry.name.clone();
        let declared_size = entry.declared_size;

        let read_result = match self.archive.by_index(id.index) {
            Ok(mut file) => read_bounded(
                &mut file,
                read_limit,
                total_is_tighter,
                declared_size,
                &name,
            ),
            Err(error) => (
                Err(ArchiveError::EntryRead {
                    entry: name.clone(),
                    source: io::Error::other(error),
                }),
                0,
            ),
        };

        let (result, consumed) = read_result;
        self.expanded_bytes_consumed = self.expanded_bytes_consumed.saturating_add(consumed);
        if result.is_err() {
            self.failed_read_entry = Some(name);
        }
        result
    }

    /// Bytes surfaced by entry readers to the wrapper. If the ZIP library
    /// consumes bytes internally and returns an error before surfacing them,
    /// the read session is closed and those hidden bytes cannot be retried.
    pub fn expanded_bytes_consumed(&self) -> u64 {
        self.expanded_bytes_consumed
    }

    pub fn limits(&self) -> &ArchiveLimits {
        &self.limits
    }
}

fn read_bounded<R: Read>(
    reader: &mut R,
    limit: u64,
    total_is_tighter: bool,
    declared_size: u64,
    entry: &str,
) -> (Result<Vec<u8>, ArchiveError>, u64) {
    let mut bytes = Vec::new();
    let mut consumed = 0_u64;
    let mut chunk = [0_u8; READ_BUFFER_BYTES];

    loop {
        let allowance = limit.saturating_sub(consumed);
        let read_size = if allowance == 0 {
            1
        } else {
            allowance.min(chunk.len() as u64) as usize
        };
        match reader.read(&mut chunk[..read_size]) {
            Ok(0) => break,
            Ok(read) => {
                consumed = consumed.saturating_add(read as u64);
                if allowance == 0 {
                    let error = if total_is_tighter {
                        ArchiveError::TotalLimitExceeded {
                            entry: entry.to_owned(),
                            limit,
                            observed: consumed,
                        }
                    } else {
                        ArchiveError::EntryLimitExceeded {
                            entry: entry.to_owned(),
                            limit,
                            observed: consumed,
                        }
                    };
                    return (Err(error), consumed);
                }
                if bytes.try_reserve(read).is_err() {
                    return (
                        Err(ArchiveError::AllocationFailed {
                            entry: entry.to_owned(),
                        }),
                        consumed,
                    );
                }
                bytes.extend_from_slice(&chunk[..read]);
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(source) => {
                return (
                    Err(ArchiveError::EntryRead {
                        entry: entry.to_owned(),
                        source,
                    }),
                    consumed,
                );
            }
        }
    }

    if consumed != declared_size {
        return (
            Err(ArchiveError::EntrySizeMismatch {
                entry: entry.to_owned(),
                declared: declared_size,
                actual: consumed,
            }),
            consumed,
        );
    }

    (Ok(bytes), consumed)
}

fn inspect_container<R: Read + Seek>(
    reader: &mut R,
    limits: &ArchiveLimits,
) -> Result<ContainerMetadata, ArchiveError> {
    let original_position = reader.stream_position().map_err(ArchiveError::Io)?;
    let result = inspect_container_at_start(reader, limits);
    reader
        .seek(SeekFrom::Start(original_position))
        .map_err(ArchiveError::Io)?;
    result
}

fn inspect_container_at_start<R: Read + Seek>(
    reader: &mut R,
    limits: &ArchiveLimits,
) -> Result<ContainerMetadata, ArchiveError> {
    let file_bytes = reader.seek(SeekFrom::End(0)).map_err(ArchiveError::Io)?;
    if file_bytes < EOCD_FIXED_BYTES as u64 {
        return Err(ArchiveError::InvalidContainer(
            "end-of-central-directory record is missing",
        ));
    }

    let tail_len = file_bytes.min(MAX_EOCD_SCAN_BYTES as u64) as usize;
    let tail_start = file_bytes - tail_len as u64;
    reader
        .seek(SeekFrom::Start(tail_start))
        .map_err(ArchiveError::Io)?;
    let mut tail = vec![0_u8; tail_len];
    reader.read_exact(&mut tail).map_err(ArchiveError::Io)?;

    let mut selected_eocd = None;
    for index in 0..=tail.len() - EOCD_FIXED_BYTES {
        let has_signature = read_u32(&tail, index) == Some(EOCD_SIGNATURE);
        let comment_reaches_end = read_u16(&tail, index + 20)
            .is_some_and(|comment| index + EOCD_FIXED_BYTES + comment as usize == tail.len());
        if has_signature && comment_reaches_end && selected_eocd.replace(index).is_some() {
            return Err(ArchiveError::InvalidContainer(
                "multiple end-of-central-directory records reach the end of the file",
            ));
        }
    }
    let eocd_index = selected_eocd.ok_or(ArchiveError::InvalidContainer(
        "end-of-central-directory record is missing or malformed",
    ))?;
    let comment_start = eocd_index + EOCD_FIXED_BYTES;
    if tail[comment_start..]
        .windows(4)
        .any(|window| window == EOCD_SIGNATURE.to_le_bytes())
    {
        return Err(ArchiveError::InvalidContainer(
            "end-of-central-directory signature is not allowed in its comment",
        ));
    }
    let eocd_offset = tail_start + eocd_index as u64;

    let disk_number = read_u16(&tail, eocd_index + 4).unwrap_or(u16::MAX);
    let central_directory_disk = read_u16(&tail, eocd_index + 6).unwrap_or(u16::MAX);
    let entries_on_disk = read_u16(&tail, eocd_index + 8).unwrap_or(u16::MAX);
    let entries_total = read_u16(&tail, eocd_index + 10).unwrap_or(u16::MAX);
    let central_directory_bytes = read_u32(&tail, eocd_index + 12).unwrap_or(u32::MAX);
    let central_directory_offset = read_u32(&tail, eocd_index + 16).unwrap_or(u32::MAX);

    if disk_number != 0 || central_directory_disk != 0 {
        return Err(ArchiveError::MultiDiskArchive);
    }

    let needs_zip64 = entries_on_disk == u16::MAX
        || entries_total == u16::MAX
        || central_directory_bytes == u32::MAX
        || central_directory_offset == u32::MAX;

    let (entries, directory_bytes, directory_offset, directory_end, zip64_record) = if needs_zip64 {
        let zip64 = inspect_zip64_end_record(reader, eocd_offset)?;
        if (entries_on_disk != u16::MAX && entries_on_disk as u64 != zip64.entries)
            || (entries_total != u16::MAX && entries_total as u64 != zip64.entries)
            || (central_directory_bytes != u32::MAX
                && central_directory_bytes as u64 != zip64.directory_bytes)
            || (central_directory_offset != u32::MAX
                && central_directory_offset as u64 != zip64.directory_offset)
        {
            return Err(ArchiveError::InvalidContainer(
                "legacy and ZIP64 end records differ",
            ));
        }
        (
            zip64.entries,
            zip64.directory_bytes,
            zip64.directory_offset,
            zip64.physical_offset,
            Some(zip64),
        )
    } else {
        if entries_on_disk != entries_total {
            return Err(ArchiveError::InvalidContainer(
                "entry counts in the end record differ",
            ));
        }
        (
            entries_total as u64,
            central_directory_bytes as u64,
            central_directory_offset as u64,
            eocd_offset,
            None,
        )
    };

    if entries > limits.max_entries as u64 {
        return Err(ArchiveError::TooManyEntries {
            actual: entries,
            limit: limits.max_entries,
        });
    }
    if directory_bytes > limits.max_central_directory_bytes {
        return Err(ArchiveError::CentralDirectoryTooLarge {
            actual: directory_bytes,
            limit: limits.max_central_directory_bytes,
        });
    }

    let directory_start =
        directory_end
            .checked_sub(directory_bytes)
            .ok_or(ArchiveError::InvalidContainer(
                "central directory range starts before the archive",
            ))?;
    let archive_offset =
        directory_start
            .checked_sub(directory_offset)
            .ok_or(ArchiveError::InvalidContainer(
                "central directory offset precedes the archive start",
            ))?;
    if let Some(zip64) = zip64_record {
        let expected_zip64_record_offset =
            zip64.physical_offset.checked_sub(archive_offset).ok_or(
                ArchiveError::InvalidContainer("ZIP64 end record precedes the archive start"),
            )?;
        if expected_zip64_record_offset != zip64.record_offset {
            return Err(ArchiveError::InvalidContainer(
                "ZIP64 locator offset does not match the end record",
            ));
        }
    }

    let directory_end =
        directory_start
            .checked_add(directory_bytes)
            .ok_or(ArchiveError::InvalidContainer(
                "central directory range overflows",
            ))?;
    inspect_central_directory(reader, directory_start, directory_end, entries)?;
    Ok(ContainerMetadata {
        entries,
        archive_offset,
    })
}

fn inspect_central_directory<R: Read + Seek>(
    reader: &mut R,
    directory_start: u64,
    directory_end: u64,
    expected_entries: u64,
) -> Result<(), ArchiveError> {
    let mut cursor = directory_start;
    for _ in 0..expected_entries {
        let remaining = directory_end
            .checked_sub(cursor)
            .ok_or(ArchiveError::InvalidContainer(
                "central directory record exceeds its declared extent",
            ))?;
        if remaining < CENTRAL_DIRECTORY_FIXED_BYTES as u64 {
            return Err(ArchiveError::InvalidContainer(
                "central directory entry header is truncated",
            ));
        }

        reader
            .seek(SeekFrom::Start(cursor))
            .map_err(ArchiveError::Io)?;
        let mut fixed = [0_u8; CENTRAL_DIRECTORY_FIXED_BYTES];
        read_exact_container(
            reader,
            &mut fixed,
            "central directory entry header is truncated",
        )?;
        if read_u32(&fixed, 0) != Some(CENTRAL_DIRECTORY_SIGNATURE) {
            return Err(ArchiveError::InvalidContainer(
                "central directory entry signature is invalid",
            ));
        }
        if read_u16(&fixed, 34).is_some_and(|disk| disk != 0) {
            return Err(ArchiveError::MultiDiskArchive);
        }

        let name_bytes = read_u16(&fixed, 28).unwrap_or(u16::MAX) as u64;
        let extra_bytes = read_u16(&fixed, 30).unwrap_or(u16::MAX) as u64;
        let comment_bytes = read_u16(&fixed, 32).unwrap_or(u16::MAX) as u64;
        let record_bytes = (CENTRAL_DIRECTORY_FIXED_BYTES as u64)
            .checked_add(name_bytes)
            .and_then(|size| size.checked_add(extra_bytes))
            .and_then(|size| size.checked_add(comment_bytes))
            .ok_or(ArchiveError::InvalidContainer(
                "central directory entry length overflows",
            ))?;
        cursor = cursor
            .checked_add(record_bytes)
            .ok_or(ArchiveError::InvalidContainer(
                "central directory record offset overflows",
            ))?;
        if cursor > directory_end {
            return Err(ArchiveError::InvalidContainer(
                "central directory entry exceeds its declared extent",
            ));
        }
    }

    let remaining = directory_end - cursor;
    if remaining == 0 {
        return Ok(());
    }
    if remaining < 6 {
        return Err(ArchiveError::InvalidContainer(
            "central directory has trailing bytes",
        ));
    }

    reader
        .seek(SeekFrom::Start(cursor))
        .map_err(ArchiveError::Io)?;
    let mut digital_signature = [0_u8; 6];
    read_exact_container(
        reader,
        &mut digital_signature,
        "central directory digital signature is truncated",
    )?;
    let signature_size = read_u16(&digital_signature, 4).unwrap_or(u16::MAX) as u64;
    if read_u32(&digital_signature, 0) != Some(CENTRAL_DIRECTORY_DIGITAL_SIGNATURE)
        || signature_size.checked_add(6) != Some(remaining)
    {
        return Err(ArchiveError::InvalidContainer(
            "central directory has an invalid trailing record",
        ));
    }
    Ok(())
}

fn read_exact_container<R: Read>(
    reader: &mut R,
    bytes: &mut [u8],
    truncated_reason: &'static str,
) -> Result<(), ArchiveError> {
    match reader.read_exact(bytes) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
            Err(ArchiveError::InvalidContainer(truncated_reason))
        }
        Err(error) => Err(ArchiveError::Io(error)),
    }
}

fn inspect_zip64_end_record<R: Read + Seek>(
    reader: &mut R,
    eocd_offset: u64,
) -> Result<Zip64EndRecord, ArchiveError> {
    let locator_offset = eocd_offset
        .checked_sub(ZIP64_LOCATOR_BYTES as u64)
        .ok_or(ArchiveError::InvalidContainer("ZIP64 locator is missing"))?;
    reader
        .seek(SeekFrom::Start(locator_offset))
        .map_err(ArchiveError::Io)?;
    let mut locator = [0_u8; ZIP64_LOCATOR_BYTES];
    read_exact_container(reader, &mut locator, "ZIP64 locator is truncated")?;
    if read_u32(&locator, 0) != Some(ZIP64_LOCATOR_SIGNATURE) {
        return Err(ArchiveError::InvalidContainer("ZIP64 locator is missing"));
    }
    let disk_with_record = read_u32(&locator, 4).unwrap_or(u32::MAX);
    let record_offset = read_u64(&locator, 8).unwrap_or(u64::MAX);
    let disk_count = read_u32(&locator, 16).unwrap_or(u32::MAX);
    if disk_with_record != 0 || disk_count != 1 {
        return Err(ArchiveError::MultiDiskArchive);
    }

    let physical_offset = locator_offset
        .checked_sub(ZIP64_EOCD_FIXED_BYTES as u64)
        .ok_or(ArchiveError::InvalidContainer(
            "ZIP64 end record is missing",
        ))?;
    reader
        .seek(SeekFrom::Start(physical_offset))
        .map_err(ArchiveError::Io)?;
    let mut record = [0_u8; ZIP64_EOCD_FIXED_BYTES];
    read_exact_container(reader, &mut record, "ZIP64 end record is truncated")?;
    if read_u32(&record, 0) != Some(ZIP64_EOCD_SIGNATURE) {
        return Err(ArchiveError::InvalidContainer(
            "ZIP64 end record is missing or has extensible data",
        ));
    }
    let record_payload_bytes = read_u64(&record, 4).unwrap_or(0);
    if record_payload_bytes != 44 {
        return Err(ArchiveError::InvalidContainer(
            "ZIP64 extensible data is not supported",
        ));
    }

    let disk_number = read_u32(&record, 16).unwrap_or(u32::MAX);
    let central_directory_disk = read_u32(&record, 20).unwrap_or(u32::MAX);
    let entries_on_disk = read_u64(&record, 24).unwrap_or(u64::MAX);
    let entries = read_u64(&record, 32).unwrap_or(u64::MAX);
    let directory_bytes = read_u64(&record, 40).unwrap_or(u64::MAX);
    let directory_offset = read_u64(&record, 48).unwrap_or(u64::MAX);
    if disk_number != 0 || central_directory_disk != 0 {
        return Err(ArchiveError::MultiDiskArchive);
    }
    if entries_on_disk != entries {
        return Err(ArchiveError::InvalidContainer(
            "ZIP64 entry counts in the end record differ",
        ));
    }

    Ok(Zip64EndRecord {
        entries,
        directory_bytes,
        directory_offset,
        record_offset,
        physical_offset,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let pair: [u8; 2] = bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?;
    Some(u16::from_le_bytes(pair))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let quad: [u8; 4] = bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(quad))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    let octet: [u8; 8] = bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?;
    Some(u64::from_le_bytes(octet))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{ArchiveError, read_bounded};

    #[test]
    fn bounds_actual_bytes_below_at_and_above_entry_limit() {
        for limit in [2, 3, 4] {
            let (result, consumed) = read_bounded(
                &mut Cursor::new(b"abc"),
                limit,
                false,
                3,
                "OPS/chapter.xhtml",
            );
            assert_eq!(consumed, if limit == 2 { limit + 1 } else { 3 });
            if limit == 2 {
                assert!(matches!(
                    result,
                    Err(ArchiveError::EntryLimitExceeded {
                        limit: 2,
                        observed: 3,
                        ..
                    })
                ));
            } else {
                assert_eq!(result.unwrap(), b"abc");
            }
        }
    }
}

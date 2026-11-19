//! Opaque bounded payload IO and snapshot reads for complete volume states.

#[path = "import_io.rs"]
mod payload_io;
pub use payload_io::{ImportDigestSink, stage_import_payload};
#[path = "import_registration.rs"]
mod volume_registration;
pub(crate) use volume_registration::{
    check_registration, register_volume, verify_registration_replay,
};

use std::collections::BTreeSet;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use nt_application::object_store::ImmutableObjectStore;
use nt_application::volume_import::{ImportReadError, ImportWriteError};
use nt_domain::object::{ObjectHash, ObjectReference};
use nt_domain::source_scope::ScopeMode;
use nt_domain::volume_import::{
    ImportedVolumeState, MAX_IMPORT_PAYLOAD_BYTES, MAX_IMPORT_WORKING_BYTES, StagedPayload,
    VolumeImportVersion, VolumeResource, VolumeState,
};
use nt_domain::workspace::{GenerationCommitError, GenerationId, SeriesId, VolumeId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

use crate::generation::{map_commit_sqlite_error, map_object_error};
use crate::{FileImportSourceStore, FileObjectStore, PinnedImportSource};

pub struct SqliteVolumeImportReader {
    connection: Connection,
    series_id: SeriesId,
    root: PathBuf,
}

pub struct VolumeImportSnapshot<'connection> {
    transaction: Transaction<'connection>,
    generation_id: GenerationId,
    series_id: SeriesId,
    _pins: ActivePins,
}

/// SQLite's incremental BLOB handle borrows the same read transaction as the
/// enclosing snapshot. The digest is verified before any bytes are exposed.
pub struct VerifiedImportPayload<'snapshot> {
    blob: rusqlite::blob::Blob<'snapshot>,
    object: ObjectReference,
}

impl VerifiedImportPayload<'_> {
    pub fn object(&self) -> ObjectReference {
        self.object
    }
}
impl Read for VerifiedImportPayload<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.blob.read(bytes)
    }
}
impl Seek for VerifiedImportPayload<'_> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.blob.seek(position)
    }
}

impl SqliteVolumeImportReader {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ImportReadError> {
        let root = root.as_ref();
        let (connection, metadata) =
            crate::workspace::open_read_only_workspace(root).map_err(map_workspace)?;
        Ok(Self {
            connection,
            series_id: metadata.series.id,
            root: root.to_path_buf(),
        })
    }

    pub fn begin_snapshot(&mut self) -> Result<VolumeImportSnapshot<'_>, ImportReadError> {
        self.begin(None)
    }

    pub fn begin_generation_snapshot(
        &mut self,
        generation: &GenerationId,
    ) -> Result<VolumeImportSnapshot<'_>, ImportReadError> {
        self.begin(Some(generation))
    }

    fn begin(
        &mut self,
        generation: Option<&GenerationId>,
    ) -> Result<VolumeImportSnapshot<'_>, ImportReadError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .map_err(sql)?;
        let current = current_generation(&transaction, &self.series_id)?;
        let generation_id = generation.unwrap_or(&current).clone();
        // Reading current above pins the snapshot before any object/payload IO.
        validate_chain(&transaction, &self.series_id, &generation_id)?;
        let store = FileObjectStore::new(&self.root).map_err(object_error)?;
        let pins = pin_active(&transaction, &store, &self.series_id, &generation_id)?;
        Ok(VolumeImportSnapshot {
            transaction,
            generation_id,
            series_id: self.series_id.clone(),
            _pins: pins,
        })
    }
}

impl VolumeImportSnapshot<'_> {
    pub fn generation_id(&self) -> &GenerationId {
        &self.generation_id
    }
    pub fn series_id(&self) -> &SeriesId {
        &self.series_id
    }

    pub fn series_default(&self) -> Result<ScopeMode, ImportReadError> {
        read_default(&self.transaction, &self.series_id, &self.generation_id)
    }

    pub fn volume(&self, volume: &VolumeId) -> Result<VolumeImportVersion, ImportReadError> {
        read_volume(
            &self.transaction,
            &self.series_id,
            &self.generation_id,
            volume,
        )
    }

    pub fn open_payload(
        &self,
        volume: &VolumeId,
    ) -> Result<Option<VerifiedImportPayload<'_>>, ImportReadError> {
        let state = find_volume_row(
            &self.transaction,
            &self.series_id,
            &self.generation_id,
            volume,
        )?;
        match state {
            Some((rowid, _, VolumeState::Imported(state))) => {
                verified_blob(&self.transaction, rowid, state.payload.object).map(Some)
            }
            _ => {
                volume_registration::ensure_visible(
                    &self.transaction,
                    &self.series_id,
                    &self.generation_id,
                    volume,
                )?;
                Ok(None)
            }
        }
    }

    /// Find the closest ancestor version with the exact decision context.
    /// This read never switches away from the pinned SQLite transaction.
    pub fn open_payload_for_context(
        &self,
        volume: &VolumeId,
        epub_sha256: &[u8; 32],
        mapping_version: &[u8; 32],
    ) -> Result<Option<(VolumeImportVersion, VerifiedImportPayload<'_>)>, ImportReadError> {
        volume_exists(&self.transaction, &self.series_id, volume)?;
        volume_registration::ensure_visible(
            &self.transaction,
            &self.series_id,
            &self.generation_id,
            volume,
        )?;
        if schema_version(&self.transaction)? < 3 {
            return Ok(None);
        }
        let mut statement = self
            .transaction
            .prepare(
                "SELECT rowid FROM cp05_volume_versions
             WHERE series_id=?1 AND generation_id=?2 AND volume_id=?3
             AND state='imported' AND epub_sha256=?4 AND mapping_version=?5",
            )
            .map_err(sql)?;
        let selected = ancestors(
            &self.transaction,
            &self.series_id,
            &self.generation_id,
            |at| {
                let rowid = statement
                    .query_row(
                        (
                            self.series_id.as_str(),
                            at.as_str(),
                            volume.as_str(),
                            epub_sha256.as_slice(),
                            mapping_version.as_slice(),
                        ),
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(sql)?;
                Ok(rowid.map(|id| (id, at.clone())))
            },
        )?;
        let Some((rowid, generation)) = selected else {
            return Ok(None);
        };
        let version = read_volume(&self.transaction, &self.series_id, &generation, volume)?;
        let VolumeState::Imported(state) = &version.state else {
            return Err(corrupt());
        };
        if version.generation_id.as_ref() != Some(&generation)
            || &state.epub_sha256 != epub_sha256
            || &state.mapping_version != mapping_version
        {
            return Err(corrupt());
        }
        let payload = verified_blob(&self.transaction, rowid, state.payload.object)?;
        Ok(Some((version, payload)))
    }

    pub fn finish(self) -> Result<(), ImportReadError> {
        self.transaction.commit().map_err(sql)
    }
}

pub(crate) fn sql(error: rusqlite::Error) -> ImportWriteError {
    map_commit_sqlite_error(error).into()
}
pub(crate) fn corrupt() -> ImportWriteError {
    GenerationCommitError::CorruptWorkspace.into()
}
pub(crate) fn object_error(
    error: nt_application::object_store::ObjectStoreError,
) -> ImportWriteError {
    map_object_error(error).into()
}
fn map_workspace(error: nt_application::workspace::WorkspaceError) -> ImportWriteError {
    match error {
        nt_application::workspace::WorkspaceError::Busy => GenerationCommitError::Busy.into(),
        nt_application::workspace::WorkspaceError::CorruptWorkspace => corrupt(),
        _ => GenerationCommitError::StorageFailure.into(),
    }
}

pub(crate) fn mode_text(mode: ScopeMode) -> &'static str {
    match mode {
        ScopeMode::AllText => "all-text",
        ScopeMode::Story => "story",
    }
}
fn parse_mode(value: &str) -> Result<ScopeMode, ImportWriteError> {
    match value {
        "all-text" => Ok(ScopeMode::AllText),
        "story" => Ok(ScopeMode::Story),
        _ => Err(corrupt()),
    }
}
pub(crate) fn current_generation(
    connection: &Connection,
    series: &SeriesId,
) -> Result<GenerationId, ImportWriteError> {
    let value: String = connection
        .query_row(
            "SELECT generation_id FROM current_generation WHERE singleton=1 AND series_id=?1",
            [series.as_str()],
            |row| row.get(0),
        )
        .map_err(sql)?;
    GenerationId::parse(value).map_err(|_| corrupt())
}
fn schema_version(connection: &Connection) -> Result<u32, ImportWriteError> {
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(sql)
}

/// Constant-memory ancestry traversal. The actual number of generation rows
/// bounds traversal, so even a corrupt cycle terminates without a growing set.
fn ancestors<T>(
    connection: &Connection,
    series: &SeriesId,
    generation: &GenerationId,
    mut visit: impl FnMut(&GenerationId) -> Result<Option<T>, ImportWriteError>,
) -> Result<Option<T>, ImportWriteError> {
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM generations WHERE series_id=?1",
            [series.as_str()],
            |row| row.get(0),
        )
        .map_err(sql)?;
    let mut current = generation.clone();
    for _ in 0..count {
        let parent: Option<String> = connection.query_row(
            "SELECT parent_generation_id FROM generations WHERE series_id=?1 AND generation_id=?2",
            (series.as_str(), current.as_str()), |row| row.get(0))
            .optional().map_err(sql)?.ok_or_else(corrupt)?;
        if let Some(result) = visit(&current)? {
            return Ok(Some(result));
        }
        match parent {
            Some(parent) => current = GenerationId::parse(parent).map_err(|_| corrupt())?,
            None if current.as_str() == "g0" => return Ok(None),
            None => return Err(corrupt()),
        }
    }
    Err(corrupt())
}
pub(crate) fn validate_chain(
    connection: &Connection,
    series: &SeriesId,
    generation: &GenerationId,
) -> Result<(), ImportWriteError> {
    ancestors::<()>(connection, series, generation, |_| Ok(None)).map(|_| ())
}
pub(crate) fn read_default(
    connection: &Connection,
    series: &SeriesId,
    generation: &GenerationId,
) -> Result<ScopeMode, ImportWriteError> {
    if schema_version(connection)? < 3 {
        return Ok(ScopeMode::AllText);
    }
    Ok(ancestors(connection, series, generation, |at| {
        let mode: Option<String> = connection
            .query_row(
                "SELECT mode FROM cp05_series_defaults WHERE series_id=?1 AND generation_id=?2",
                (series.as_str(), at.as_str()),
                |row| row.get(0),
            )
            .optional()
            .map_err(sql)?;
        mode.as_deref().map(parse_mode).transpose()
    })?
    .unwrap_or(ScopeMode::AllText))
}

fn volume_exists(
    connection: &Connection,
    series: &SeriesId,
    volume: &VolumeId,
) -> Result<(), ImportWriteError> {
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM volumes WHERE series_id=?1 AND volume_id=?2)",
            (series.as_str(), volume.as_str()),
            |row| row.get(0),
        )
        .map_err(sql)?;
    if exists {
        Ok(())
    } else {
        Err(ImportWriteError::InvalidVolume)
    }
}
type VolumeRow = (i64, GenerationId, VolumeState);
fn find_volume_row(
    connection: &Connection,
    series: &SeriesId,
    generation: &GenerationId,
    volume: &VolumeId,
) -> Result<Option<VolumeRow>, ImportWriteError> {
    volume_exists(connection, series, volume)?;
    if schema_version(connection)? < 3 {
        return Ok(None);
    }
    ancestors(connection, series, generation, |at| {
        let mut statement = connection
            .prepare(
                "SELECT rowid,state,mode,epub_sha256,mapping_version,length(payload),payload_sha256
             FROM cp05_volume_versions WHERE series_id=?1 AND generation_id=?2 AND volume_id=?3",
            )
            .map_err(sql)?;
        let mut rows = statement
            .query((series.as_str(), at.as_str(), volume.as_str()))
            .map_err(sql)?;
        let Some(row) = rows.next().map_err(sql)? else {
            return Ok(None);
        };
        let state = row
            .get_ref(1)
            .map_err(sql)?
            .as_str()
            .map_err(|_| corrupt())?;
        let mode = parse_mode(
            row.get_ref(2)
                .map_err(sql)?
                .as_str()
                .map_err(|_| corrupt())?,
        )?;
        let value = match state {
            "unimported"
                if (3..7).all(|index| {
                    matches!(row.get_ref(index), Ok(rusqlite::types::ValueRef::Null))
                }) =>
            {
                VolumeState::Unimported { mode }
            }
            "imported" => {
                let length =
                    u64::try_from(row.get::<_, i64>(5).map_err(sql)?).map_err(|_| corrupt())?;
                if length == 0 || length > MAX_IMPORT_PAYLOAD_BYTES {
                    return Err(corrupt());
                }
                let bytes = |index| -> Result<[u8; 32], ImportWriteError> {
                    row.get_ref(index)
                        .map_err(sql)?
                        .as_blob()
                        .map_err(|_| corrupt())?
                        .try_into()
                        .map_err(|_| corrupt())
                };
                VolumeState::Imported(ImportedVolumeState {
                    mode,
                    epub_sha256: bytes(3)?,
                    mapping_version: bytes(4)?,
                    payload: StagedPayload {
                        object: ObjectReference {
                            hash: ObjectHash::from_bytes(bytes(6)?),
                            byte_len: length,
                        },
                    },
                })
            }
            _ => return Err(corrupt()),
        };
        Ok(Some((row.get(0).map_err(sql)?, at.clone(), value)))
    })
}
pub(crate) fn read_volume(
    connection: &Connection,
    series: &SeriesId,
    generation: &GenerationId,
    volume: &VolumeId,
) -> Result<VolumeImportVersion, ImportWriteError> {
    read_volume_with_budget(
        connection,
        series,
        generation,
        volume,
        &mut Budget::default(),
    )
}

pub(crate) fn read_volume_with_budget(
    connection: &Connection,
    series: &SeriesId,
    generation: &GenerationId,
    volume: &VolumeId,
    budget: &mut Budget,
) -> Result<VolumeImportVersion, ImportWriteError> {
    let row = find_volume_row(connection, series, generation, volume)?;
    let (generation_id, state, resources) = match row {
        Some((_, at, state)) => {
            let resources = read_resources(connection, series, &at, volume, budget)?;
            if matches!(state, VolumeState::Unimported { .. }) && !resources.is_empty() {
                return Err(corrupt());
            }
            (Some(at), state, resources)
        }
        None => {
            volume_registration::ensure_visible(connection, series, generation, volume)?;
            (
                None,
                VolumeState::Unimported {
                    mode: ScopeMode::AllText,
                },
                Vec::new(),
            )
        }
    };
    Ok(VolumeImportVersion {
        volume_id: volume.clone(),
        generation_id,
        state,
        resources,
    })
}

fn read_resources(
    connection: &Connection,
    series: &SeriesId,
    generation: &GenerationId,
    volume: &VolumeId,
    budget: &mut Budget,
) -> Result<Vec<VolumeResource>, ImportWriteError> {
    let mut statement = connection.prepare(
        "SELECT r.role,r.canonical_path,r.object_hash,r.byte_len,r.source_relative_path,o.byte_len
         FROM cp05_resource_refs r LEFT JOIN generation_object_refs o
         ON o.series_id=r.series_id AND o.generation_id=r.generation_id AND o.object_hash=r.object_hash
         WHERE r.series_id=?1 AND r.generation_id=?2 AND r.volume_id=?3 ORDER BY r.role,r.canonical_path").map_err(sql)?;
    let mut rows = statement
        .query((series.as_str(), generation.as_str(), volume.as_str()))
        .map_err(sql)?;
    let mut resources = Vec::new();
    while let Some(row) = rows.next().map_err(sql)? {
        let text = |index| -> Result<&str, ImportWriteError> {
            row.get_ref(index)
                .map_err(sql)?
                .as_str()
                .map_err(|_| corrupt())
        };
        let role = text(0)?;
        let path = text(1)?;
        FileImportSourceStore::validate_resource_path(path).map_err(|_| corrupt())?;
        let source = match row.get_ref(4).map_err(sql)? {
            rusqlite::types::ValueRef::Null => None,
            value => Some(value.as_str().map_err(|_| corrupt())?),
        };
        for bytes in [
            std::mem::size_of::<VolumeResource>(),
            role.len(),
            path.len(),
            source.map_or(0, str::len),
        ] {
            budget.add(bytes)?;
        }
        let object = ObjectReference {
            hash: ObjectHash::parse(text(2)?).map_err(|_| corrupt())?,
            byte_len: u64::try_from(row.get::<_, i64>(3).map_err(sql)?).map_err(|_| corrupt())?,
        };
        let referenced_length = row
            .get::<_, Option<i64>>(5)
            .map_err(sql)?
            .map(u64::try_from)
            .transpose()
            .map_err(|_| corrupt())?;
        if referenced_length != Some(object.byte_len) {
            return Err(corrupt());
        }
        resources
            .try_reserve_exact(1)
            .map_err(|_| ImportWriteError::ResourceLimit)?;
        resources.push(VolumeResource {
            role: role.to_owned(),
            canonical_path: path.to_owned(),
            object,
            source_relative_path: source.map(str::to_owned),
        });
    }
    Ok(resources)
}

fn verified_blob(
    connection: &Connection,
    rowid: i64,
    object: ObjectReference,
) -> Result<VerifiedImportPayload<'_>, ImportWriteError> {
    let mut blob = connection
        .blob_open("main", "cp05_volume_versions", "payload", rowid, true)
        .map_err(sql)?;
    if blob.len() as u64 != object.byte_len || object.byte_len > MAX_IMPORT_PAYLOAD_BYTES {
        return Err(corrupt());
    }
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = blob.read(&mut buffer).map_err(|_| corrupt())?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    let digest: [u8; 32] = hash.finalize().into();
    if digest != *object.hash.as_bytes() {
        return Err(corrupt());
    }
    blob.seek(SeekFrom::Start(0)).map_err(|_| corrupt())?;
    Ok(VerifiedImportPayload { blob, object })
}

#[derive(Default)]
pub(crate) struct Budget(usize);
impl Budget {
    pub(crate) fn add(&mut self, bytes: usize) -> Result<(), ImportWriteError> {
        self.0 = self
            .0
            .checked_add(bytes)
            .filter(|value| *value <= MAX_IMPORT_WORKING_BYTES)
            .ok_or(ImportWriteError::ResourceLimit)?;
        Ok(())
    }
}
#[derive(Default)]
pub(crate) struct ActivePins {
    objects: Vec<Box<dyn Read + Send>>,
    sources: Vec<PinnedImportSource>,
}
pub(crate) fn pin_active(
    connection: &Connection,
    store: &impl ImmutableObjectStore,
    series: &SeriesId,
    generation: &GenerationId,
) -> Result<ActivePins, ImportWriteError> {
    pin_active_with_budget(
        connection,
        store,
        series,
        generation,
        &mut Budget::default(),
    )
}

pub(crate) fn pin_active_with_budget(
    connection: &Connection,
    store: &impl ImmutableObjectStore,
    series: &SeriesId,
    generation: &GenerationId,
    budget: &mut Budget,
) -> Result<ActivePins, ImportWriteError> {
    let mut pins = ActivePins::default();
    let mut checked = BTreeSet::new();
    pin_generation_objects(connection, store, series, generation, &mut pins, budget)?;
    checked.insert(generation.as_str().to_owned());
    if schema_version(connection)? < 3 {
        return Ok(pins);
    }
    let root = Path::new(connection.path().ok_or_else(corrupt)?)
        .parent()
        .ok_or_else(corrupt)?;
    let source_store = FileImportSourceStore::new(root).map_err(object_error)?;
    let mut statement = connection
        .prepare("SELECT volume_id FROM volumes WHERE series_id=?1 ORDER BY volume_id")
        .map_err(sql)?;
    let mut rows = statement.query([series.as_str()]).map_err(sql)?;
    while let Some(row) = rows.next().map_err(sql)? {
        let volume =
            VolumeId::parse(row.get::<_, String>(0).map_err(sql)?).map_err(|_| corrupt())?;
        let Some((rowid, at, state)) = find_volume_row(connection, series, generation, &volume)?
        else {
            continue;
        };
        let VolumeState::Imported(state) = state else {
            continue;
        };
        drop(verified_blob(connection, rowid, state.payload.object)?);
        if !checked.contains(at.as_str()) {
            budget.add(256)?;
            pin_generation_objects(connection, store, series, &at, &mut pins, budget)?;
            checked.insert(at.as_str().to_owned());
        }
        let original_present: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM generation_object_refs WHERE series_id=?1 AND generation_id=?2 AND object_hash=?3)",
            (series.as_str(), at.as_str(), ObjectHash::from_bytes(state.epub_sha256).to_hex()), |row| row.get(0)).map_err(sql)?;
        if !original_present {
            return Err(corrupt());
        }
        for resource in read_resources(connection, series, &at, &volume, budget)? {
            if let Some(relative) = &resource.source_relative_path {
                let expected = FileImportSourceStore::source_relative_path(
                    &volume,
                    state.epub_sha256,
                    &resource.canonical_path,
                )
                .map_err(object_error)?;
                if relative != &expected {
                    return Err(corrupt());
                }
                budget.add(
                    expected
                        .len()
                        .checked_mul(8)
                        .and_then(|bytes| bytes.checked_add(512))
                        .ok_or(ImportWriteError::ResourceLimit)?,
                )?;
                let pin = source_store
                    .open_verified(
                        &volume,
                        state.epub_sha256,
                        &resource.canonical_path,
                        resource.object,
                    )
                    .map_err(object_error)?;
                pins.sources
                    .try_reserve_exact(1)
                    .map_err(|_| ImportWriteError::ResourceLimit)?;
                pins.sources.push(pin);
            }
        }
    }
    Ok(pins)
}

fn pin_generation_objects(
    connection: &Connection,
    store: &impl ImmutableObjectStore,
    series: &SeriesId,
    generation: &GenerationId,
    pins: &mut ActivePins,
    budget: &mut Budget,
) -> Result<(), ImportWriteError> {
    let mut statement = connection.prepare(
        "SELECT object_hash,byte_len FROM generation_object_refs WHERE series_id=?1 AND generation_id=?2 ORDER BY object_hash").map_err(sql)?;
    let mut rows = statement
        .query((series.as_str(), generation.as_str()))
        .map_err(sql)?;
    while let Some(row) = rows.next().map_err(sql)? {
        budget.add(512)?;
        let hash =
            ObjectHash::parse(&row.get::<_, String>(0).map_err(sql)?).map_err(|_| corrupt())?;
        let expected = u64::try_from(row.get::<_, i64>(1).map_err(sql)?).map_err(|_| corrupt())?;
        let mut reader = store.open_verified(hash).map_err(object_error)?;
        let actual = std::io::copy(&mut reader, &mut std::io::sink()).map_err(|_| corrupt())?;
        if actual != expected {
            return Err(corrupt());
        }
        pins.objects
            .try_reserve_exact(1)
            .map_err(|_| ImportWriteError::ResourceLimit)?;
        pins.objects.push(reader);
    }
    Ok(())
}

pub(crate) fn pin_active_for_generic(
    connection: &Connection,
    store: &impl ImmutableObjectStore,
    series: &SeriesId,
) -> Result<ActivePins, GenerationCommitError> {
    let check = || {
        let current = current_generation(connection, series)?;
        validate_chain(connection, series, &current)?;
        pin_active(connection, store, series, &current)
    };
    check().map_err(|error| match error {
        ImportWriteError::Generation(error) => error,
        _ => GenerationCommitError::CorruptWorkspace,
    })
}

/// Validate current imported ancestry from a single read-only database snapshot.
/// Merely opening an older schema still preserves its published reader behavior.
pub(crate) fn verify_import_integrity(
    root: &Path,
    connection: &Connection,
    series: &SeriesId,
) -> Result<(), nt_application::workspace::WorkspaceError> {
    use nt_application::workspace::WorkspaceError;
    if schema_version(connection).map_err(|_| WorkspaceError::CorruptWorkspace)? < 3 {
        return Ok(());
    }
    let snapshot = connection
        .unchecked_transaction()
        .map_err(|_| WorkspaceError::Busy)?;
    let store = FileObjectStore::new(root).map_err(|_| WorkspaceError::CorruptWorkspace)?;
    let pins = pin_active_for_generic(&snapshot, &store, series).map_err(|error| match error {
        GenerationCommitError::Busy => WorkspaceError::Busy,
        GenerationCommitError::CorruptWorkspace => WorkspaceError::CorruptWorkspace,
        _ => WorkspaceError::StorageFailure,
    })?;
    snapshot
        .commit()
        .map_err(|_| WorkspaceError::StorageFailure)?;
    drop(pins);
    Ok(())
}

use std::sync::mpsc::{self, TrySendError};

use nt_application::volume_import::{ImportWriteError, VolumeImportRepository};
use nt_domain::source_scope::ScopeMode;
use nt_domain::volume_import::{ImportOperation, VolumeImportWriteRequest, VolumeState};
use nt_domain::workspace::{GenerationCommitError, GenerationCommitResult};
use sha2::{Digest, Sha256};

use super::{SqliteGenerationWriter, WriterMessage};
use crate::import::{self, corrupt, current_generation, mode_text, object_error, sql};
use nt_application::object_store::ImmutableObjectStore;
use nt_domain::workspace::SeriesId;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::io::{Read, Write};

impl SqliteGenerationWriter {
    /// Identity used by both submission and acknowledgement reconciliation.
    pub fn import_fingerprint(request: &VolumeImportWriteRequest) -> [u8; 32] {
        fingerprint_import(request)
    }
}

impl VolumeImportRepository for SqliteGenerationWriter {
    fn commit_volume_import(
        &self,
        request: VolumeImportWriteRequest,
    ) -> Result<GenerationCommitResult, ImportWriteError> {
        if request.series_id() != &self.inner.series_id {
            return Err(ImportWriteError::InvalidVolume);
        }
        let operation_id = request.operation_id().clone();
        let fingerprint = fingerprint_import(&request);
        let (reply, receiver) = mpsc::sync_channel(1);
        match self.inner.sender.try_send(WriterMessage::CommitImport {
            request,
            fingerprint,
            reply,
        }) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(GenerationCommitError::Busy.into()),
            Err(TrySendError::Disconnected(_)) => {
                return Err(GenerationCommitError::WorkerUnavailable.into());
            }
        }
        match receiver.recv() {
            Ok(result) => result,
            Err(_) => {
                Self::reconcile_unknown(&self.inner, &operation_id, fingerprint).map_err(Into::into)
            }
        }
    }

    fn reconcile_volume_import(
        &self,
        request: &VolumeImportWriteRequest,
    ) -> Result<GenerationCommitResult, ImportWriteError> {
        if request.series_id() != &self.inner.series_id {
            return Err(ImportWriteError::InvalidVolume);
        }
        let connection = Connection::open_with_flags(
            &self.inner.database_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| GenerationCommitError::CommitOutcomeUnknown)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|_| GenerationCommitError::CommitOutcomeUnknown)?;
        let snapshot = connection
            .unchecked_transaction()
            .map_err(|_| GenerationCommitError::CommitOutcomeUnknown)?;
        let mut budget = import::Budget::default();
        budget.add(request.working_bytes())?;
        let result = lookup_import(
            &snapshot,
            &self.inner.series_id,
            request,
            fingerprint_import(request),
            &mut budget,
        )
        .map_err(|error| match error {
            ImportWriteError::Generation(
                GenerationCommitError::OperationConflict | GenerationCommitError::CorruptWorkspace,
            ) => error,
            _ => GenerationCommitError::CommitOutcomeUnknown.into(),
        })?
        .ok_or_else(|| ImportWriteError::from(GenerationCommitError::CommitOutcomeUnknown))?;
        snapshot
            .commit()
            .map_err(|_| GenerationCommitError::CommitOutcomeUnknown)?;
        Ok(result)
    }
}

fn field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_be_bytes());
    hash.update(value);
}

fn mode_byte(mode: ScopeMode) -> u8 {
    match mode {
        ScopeMode::AllText => 1,
        ScopeMode::Story => 2,
    }
}

fn scope_only_digest(mode: ScopeMode) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"NT-CP05-SCOPE-ONLY-V1\0");
    hash.update([mode_byte(mode)]);
    hash.finalize().into()
}

pub(super) fn commit_import_request(
    connection: &mut Connection,
    store: &impl ImmutableObjectStore,
    series: &SeriesId,
    request: VolumeImportWriteRequest,
    fingerprint: [u8; 32],
) -> Result<GenerationCommitResult, ImportWriteError> {
    commit_with_hook(connection, store, series, request, fingerprint, |_| Ok(()))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommitStage {
    BeforeTransaction,
    AfterReferences,
    AfterState,
    BeforeCommit,
}

fn commit_with_hook(
    connection: &mut Connection,
    store: &impl ImmutableObjectStore,
    series: &SeriesId,
    request: VolumeImportWriteRequest,
    fingerprint: [u8; 32],
    mut hook: impl FnMut(CommitStage) -> Result<(), ImportWriteError>,
) -> Result<GenerationCommitResult, ImportWriteError> {
    if request.series_id() != series {
        return Err(ImportWriteError::InvalidVolume);
    }
    let mut budget = import::Budget::default();
    budget.add(request.working_bytes())?;
    if let Some(result) = lookup_import(connection, series, &request, fingerprint, &mut budget)? {
        return Ok(result);
    }

    let preflight = connection
        .transaction_with_behavior(TransactionBehavior::Deferred)
        .map_err(sql)?;
    if let Some(result) = lookup_import(&preflight, series, &request, fingerprint, &mut budget)? {
        return Ok(result);
    }
    let current = current_generation(&preflight, series)?;
    if &current != request.expected_parent() {
        return Err(GenerationCommitError::StaleGeneration.into());
    }
    import::validate_chain(&preflight, series, &current)?;
    let _active_pins =
        import::pin_active_with_budget(&preflight, store, series, &current, &mut budget)?;
    let root = std::path::Path::new(preflight.path().ok_or_else(corrupt)?)
        .parent()
        .ok_or_else(corrupt)?;
    let source_store = crate::FileImportSourceStore::new(root).map_err(object_error)?;

    let (volume, state, resources, default) = match request.operation() {
        ImportOperation::Publish {
            volume_id,
            state,
            resources,
            default_delta,
        } => {
            import::read_volume_with_budget(&preflight, series, &current, volume_id, &mut budget)?;
            (
                Some(volume_id),
                Some(VolumeState::Imported(state.clone())),
                resources.as_slice(),
                *default_delta,
            )
        }
        ImportOperation::SetVolumeScope {
            volume_id,
            state,
            resources,
            default_delta,
        } => {
            let previous = import::read_volume_with_budget(
                &preflight,
                series,
                &current,
                volume_id,
                &mut budget,
            )?;
            let compatible = match (&previous.state, state) {
                (VolumeState::Unimported { .. }, VolumeState::Unimported { .. }) => true,
                (VolumeState::Imported(before), VolumeState::Imported(after)) => {
                    before.epub_sha256 == after.epub_sha256
                        && before.mapping_version == after.mapping_version
                        && previous.resources == *resources
                }
                _ => false,
            };
            if !compatible {
                return Err(ImportWriteError::InvalidRequest);
            }
            (
                Some(volume_id),
                Some(state.clone()),
                resources.as_slice(),
                *default_delta,
            )
        }
        ImportOperation::SetSeriesDefault { mode } => (None, None, &[][..], Some(*mode)),
    };

    let mut pins = Vec::new();
    let mut source_pins = Vec::new();
    for object in request.objects() {
        budget.add(512)?;
        let mut reader = store.open_verified(object.hash).map_err(object_error)?;
        let length = std::io::copy(&mut reader, &mut std::io::sink()).map_err(|_| corrupt())?;
        if length != object.byte_len {
            return Err(ImportWriteError::PayloadMismatch);
        }
        pins.try_reserve_exact(1)
            .map_err(|_| ImportWriteError::ResourceLimit)?;
        pins.push(reader);
    }
    let mut payload = None;
    if let Some(VolumeState::Imported(state)) = &state {
        if !request
            .objects()
            .iter()
            .any(|object| object.hash.as_bytes() == &state.epub_sha256)
        {
            return Err(ImportWriteError::InvalidRequest);
        }
        payload = Some(
            store
                .open_verified(state.payload.object.hash)
                .map_err(object_error)?,
        );
        let volume = volume.ok_or(ImportWriteError::InvalidRequest)?;
        for resource in resources {
            let expected = crate::FileImportSourceStore::source_relative_path(
                volume,
                state.epub_sha256,
                &resource.canonical_path,
            )
            .map_err(|_| ImportWriteError::InvalidRequest)?;
            if let Some(relative) = &resource.source_relative_path {
                if relative != &expected {
                    return Err(ImportWriteError::InvalidRequest);
                }
                budget.add(
                    expected
                        .len()
                        .checked_mul(8)
                        .and_then(|size| size.checked_add(512))
                        .ok_or(ImportWriteError::ResourceLimit)?,
                )?;
                let pin = source_store
                    .open_verified(
                        volume,
                        state.epub_sha256,
                        &resource.canonical_path,
                        resource.object,
                    )
                    .map_err(object_error)?;
                source_pins
                    .try_reserve_exact(1)
                    .map_err(|_| ImportWriteError::ResourceLimit)?;
                source_pins.push(pin);
            }
        }
    }
    // All source and object reads are prepared under a consistent read view.
    // Hold their verified handles through publication, then acquire the sole
    // write transaction and recheck the immutable parent and replay ledger.
    preflight.commit().map_err(sql)?;
    hook(CommitStage::BeforeTransaction)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(sql)?;
    if let Some(result) = lookup_import(&transaction, series, &request, fingerprint, &mut budget)? {
        return Ok(result);
    }
    if current_generation(&transaction, series)? != current {
        return Err(GenerationCommitError::StaleGeneration.into());
    }
    if let Some(volume) = volume {
        import::read_volume_with_budget(&transaction, series, &current, volume, &mut budget)?;
    }
    let timestamp = super::super::now_utc_ms()?;
    transaction.execute(
        "INSERT INTO generations(series_id,generation_id,parent_generation_id,created_at_utc_ms) VALUES (?1,?2,?3,?4)",
        (series.as_str(), request.generation_id().as_str(), current.as_str(), timestamp)).map_err(sql)?;
    for reference in request.objects() {
        transaction.execute(
            "INSERT INTO generation_object_refs(series_id,generation_id,object_hash,byte_len) VALUES (?1,?2,?3,?4)",
            (series.as_str(), request.generation_id().as_str(), reference.hash.to_hex(),
                i64::try_from(reference.byte_len).map_err(|_| ImportWriteError::ResourceLimit)?)).map_err(sql)?;
    }
    hook(CommitStage::AfterReferences)?;
    if let (Some(volume), Some(state)) = (volume, &state) {
        match state {
            VolumeState::Unimported { mode } => {
                transaction.execute(
                    "INSERT INTO cp05_volume_versions(series_id,generation_id,volume_id,state,mode) VALUES (?1,?2,?3,'unimported',?4)",
                    (series.as_str(), request.generation_id().as_str(), volume.as_str(), mode_text(*mode))).map_err(sql)?;
            }
            VolumeState::Imported(state) => {
                let size = i32::try_from(state.payload.object.byte_len)
                    .map_err(|_| ImportWriteError::ResourceLimit)?;
                transaction.execute(
                    "INSERT INTO cp05_volume_versions(series_id,generation_id,volume_id,state,mode,epub_sha256,mapping_version,payload,payload_sha256)
                     VALUES (?1,?2,?3,'imported',?4,?5,?6,?7,?8)",
                    params![series.as_str(), request.generation_id().as_str(), volume.as_str(), mode_text(state.mode),
                        state.epub_sha256.as_slice(), state.mapping_version.as_slice(), rusqlite::blob::ZeroBlob(size),
                        state.payload.object.hash.as_bytes().as_slice()]).map_err(sql)?;
                let rowid = transaction.last_insert_rowid();
                let mut blob = transaction
                    .blob_open("main", "cp05_volume_versions", "payload", rowid, false)
                    .map_err(sql)?;
                let mut source = payload.take().ok_or(ImportWriteError::InvalidRequest)?;
                let mut hash = Sha256::new();
                let mut total = 0_u64;
                let mut buffer = [0_u8; 64 * 1024];
                loop {
                    let count = source
                        .read(&mut buffer)
                        .map_err(|_| ImportWriteError::PayloadMismatch)?;
                    if count == 0 {
                        break;
                    }
                    total = total
                        .checked_add(count as u64)
                        .ok_or(ImportWriteError::ResourceLimit)?;
                    if total > state.payload.object.byte_len {
                        return Err(ImportWriteError::PayloadMismatch);
                    }
                    hash.update(&buffer[..count]);
                    blob.write_all(&buffer[..count])
                        .map_err(|_| GenerationCommitError::StorageFailure)?;
                }
                let digest: [u8; 32] = hash.finalize().into();
                if total != state.payload.object.byte_len
                    || digest != *state.payload.object.hash.as_bytes()
                {
                    return Err(ImportWriteError::PayloadMismatch);
                }
                blob.close().map_err(sql)?;
                pins.push(source);
            }
        }
        for resource in resources {
            transaction.execute(
                "INSERT INTO cp05_resource_refs(series_id,generation_id,volume_id,role,canonical_path,object_hash,byte_len,source_relative_path)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![series.as_str(), request.generation_id().as_str(), volume.as_str(), resource.role,
                    resource.canonical_path, resource.object.hash.to_hex(),
                    i64::try_from(resource.object.byte_len).map_err(|_| ImportWriteError::ResourceLimit)?,
                    resource.source_relative_path]).map_err(sql)?;
        }
    }
    if let Some(default) = default {
        transaction
            .execute(
                "INSERT INTO cp05_series_defaults(series_id,generation_id,mode) VALUES (?1,?2,?3)",
                (
                    series.as_str(),
                    request.generation_id().as_str(),
                    mode_text(default),
                ),
            )
            .map_err(sql)?;
    }
    hook(CommitStage::AfterState)?;
    transaction.execute(
        "INSERT INTO generation_operations(series_id,operation_id,request_fingerprint,expected_parent_generation_id,result_generation_id,committed_at_utc_ms)
         VALUES (?1,?2,?3,?4,?5,?6)",
        (series.as_str(), request.operation_id().as_str(), fingerprint.as_slice(), current.as_str(), request.generation_id().as_str(), timestamp)).map_err(sql)?;
    let changed = transaction.execute(
        "UPDATE current_generation SET generation_id=?1 WHERE singleton=1 AND series_id=?2 AND generation_id=?3",
        (request.generation_id().as_str(), series.as_str(), current.as_str())).map_err(sql)?;
    if changed != 1 {
        return Err(GenerationCommitError::StaleGeneration.into());
    }
    hook(CommitStage::BeforeCommit)?;
    let result = transaction.commit();
    if result.is_ok() {
        return Ok(GenerationCommitResult {
            generation_id: request.generation_id().clone(),
            replayed: false,
        });
    }
    lookup_import(connection, series, &request, fingerprint, &mut budget)
        .map_err(|error| match error {
            ImportWriteError::Generation(
                GenerationCommitError::OperationConflict | GenerationCommitError::CorruptWorkspace,
            ) => error,
            _ => GenerationCommitError::CommitOutcomeUnknown.into(),
        })?
        .ok_or_else(|| GenerationCommitError::CommitOutcomeUnknown.into())
}

fn lookup_import(
    connection: &Connection,
    series: &SeriesId,
    request: &VolumeImportWriteRequest,
    fingerprint: [u8; 32],
    budget: &mut import::Budget,
) -> Result<Option<GenerationCommitResult>, ImportWriteError> {
    let result =
        super::super::lookup_operation(connection, series, request.operation_id(), fingerprint)?;
    let Some(result) = result else {
        return Ok(None);
    };
    // The digest pins the opaque codec. Redundant typed columns and resource
    // rows must also match, so a malformed caller cannot reuse an ID while
    // changing a typed field without changing its asserted payload digest.
    let (volume, state, resources, default) = match request.operation() {
        ImportOperation::Publish {
            volume_id,
            state,
            resources,
            default_delta,
        } => (
            Some(volume_id),
            Some(VolumeState::Imported(state.clone())),
            resources.as_slice(),
            *default_delta,
        ),
        ImportOperation::SetVolumeScope {
            volume_id,
            state,
            resources,
            default_delta,
        } => (
            Some(volume_id),
            Some(state.clone()),
            resources.as_slice(),
            *default_delta,
        ),
        ImportOperation::SetSeriesDefault { mode } => (None, None, &[][..], Some(*mode)),
    };
    if let (Some(volume), Some(state)) = (volume, state) {
        let previous = import::read_volume_with_budget(
            connection,
            series,
            &result.generation_id,
            volume,
            budget,
        )?;
        if previous.generation_id.as_ref() != Some(&result.generation_id)
            || previous.state != state
            || previous.resources != resources
        {
            return Err(GenerationCommitError::OperationConflict.into());
        }
    }
    let recorded: Option<String> = connection
        .query_row(
            "SELECT mode FROM cp05_series_defaults WHERE series_id=?1 AND generation_id=?2",
            (series.as_str(), result.generation_id.as_str()),
            |row| row.get(0),
        )
        .optional()
        .map_err(sql)?;
    if recorded.as_deref() != default.map(mode_text) {
        return Err(GenerationCommitError::OperationConflict.into());
    }
    Ok(Some(result))
}

/// Fingerprint domains and length prefixes are pinned in the CP05 decision.
pub(super) fn fingerprint_import(request: &VolumeImportWriteRequest) -> [u8; 32] {
    let mut hash = Sha256::new();
    let (domain, volume, payload, mode) = match request.operation() {
        ImportOperation::Publish {
            volume_id,
            state,
            default_delta,
            ..
        } => (
            b"nt-cp05-import-v1".as_slice(),
            Some(volume_id),
            Some(*state.payload.object.hash.as_bytes()),
            default_delta.map_or(0, mode_byte),
        ),
        ImportOperation::SetVolumeScope {
            volume_id,
            state,
            default_delta,
            ..
        } => (
            b"nt-cp05-volume-scope-v1".as_slice(),
            Some(volume_id),
            Some(match state {
                VolumeState::Unimported { mode } => scope_only_digest(*mode),
                VolumeState::Imported(state) => *state.payload.object.hash.as_bytes(),
            }),
            default_delta.map_or(0, mode_byte),
        ),
        ImportOperation::SetSeriesDefault { mode } => (
            b"nt-cp05-series-default-v1".as_slice(),
            None,
            None,
            mode_byte(*mode),
        ),
    };
    field(&mut hash, domain);
    field(&mut hash, request.series_id().as_str().as_bytes());
    if let Some(volume) = volume {
        field(&mut hash, volume.as_str().as_bytes());
    }
    field(&mut hash, request.operation_id().as_str().as_bytes());
    field(&mut hash, request.expected_parent().as_str().as_bytes());
    field(&mut hash, request.generation_id().as_str().as_bytes());
    if let Some(payload) = payload {
        field(&mut hash, &payload);
    }
    field(&mut hash, &[mode]);
    hash.update((request.objects().len() as u64).to_be_bytes());
    for object in request.objects() {
        hash.update(object.hash.as_bytes());
        hash.update(object.byte_len.to_be_bytes());
    }
    hash.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::{fingerprint_import, scope_only_digest};
    use nt_domain::object::{ObjectHash, ObjectReference};
    use nt_domain::source_scope::ScopeMode;
    use nt_domain::volume_import::{
        ImportOperation, ImportedVolumeState, StagedPayload, VolumeImportWriteRequest, VolumeState,
    };
    use nt_domain::workspace::{GenerationId, OperationId, SeriesId, VolumeId};

    fn request(operation: ImportOperation, refs: Vec<ObjectReference>) -> VolumeImportWriteRequest {
        VolumeImportWriteRequest::new(
            SeriesId::parse("11111111111111111111111111111111").unwrap(),
            OperationId::parse("33333333333333333333333333333333").unwrap(),
            GenerationId::parse("g0").unwrap(),
            GenerationId::parse("44444444444444444444444444444444").unwrap(),
            operation,
            refs,
        )
        .unwrap()
    }

    #[test]
    fn decision_fingerprint_vectors_match_independent_bytes() {
        let volume = VolumeId::parse("22222222222222222222222222222222").unwrap();
        let refs = vec![
            ObjectReference {
                hash: ObjectHash::from_bytes([0x22; 32]),
                byte_len: 258,
            },
            ObjectReference {
                hash: ObjectHash::from_bytes([0x11; 32]),
                byte_len: 17,
            },
        ];
        let import = request(
            ImportOperation::Publish {
                volume_id: volume.clone(),
                state: ImportedVolumeState {
                    mode: ScopeMode::Story,
                    epub_sha256: [0x77; 32],
                    mapping_version: [0x88; 32],
                    payload: StagedPayload {
                        object: ObjectReference {
                            hash: ObjectHash::from_bytes([0; 32]),
                            byte_len: 1,
                        },
                    },
                },
                resources: vec![],
                default_delta: Some(ScopeMode::AllText),
            },
            refs.clone(),
        );
        assert_eq!(
            ObjectHash::from_bytes(fingerprint_import(&import)).to_string(),
            "20b9a87546d9b6533cce3fec179ba2f89e375a887809134405c7fab6152d9b14"
        );
        let scope = request(
            ImportOperation::SetVolumeScope {
                volume_id: volume,
                state: VolumeState::Unimported {
                    mode: ScopeMode::Story,
                },
                resources: vec![],
                default_delta: None,
            },
            vec![],
        );
        assert_eq!(
            ObjectHash::from_bytes(scope_only_digest(ScopeMode::Story)).to_string(),
            "efdbcf02b7f1002ac67b0a281278671b4b97caf1572727e5dde9ae63f8cc3e72"
        );
        assert_eq!(
            ObjectHash::from_bytes(fingerprint_import(&scope)).to_string(),
            "05613f80398a2f974e3bc730a006606b55aafac31586677813843a6bd291b973"
        );
        let default = request(
            ImportOperation::SetSeriesDefault {
                mode: ScopeMode::Story,
            },
            refs,
        );
        assert_eq!(
            ObjectHash::from_bytes(fingerprint_import(&default)).to_string(),
            "8b68375a6899b61b0f6dd3340ffe929b70ab5ba4fc50e2768714c51b02d75e68"
        );
    }
}

#[cfg(test)]
#[path = "import_writer/fault_tests.rs"]
mod fault_tests;

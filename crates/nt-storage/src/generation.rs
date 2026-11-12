#[path = "writer.rs"]
mod writer;

pub use writer::SqliteGenerationWriter;

use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use nt_application::object_store::{ImmutableObjectStore, ObjectStoreError};
use nt_domain::object::ObjectReference;
use nt_domain::workspace::{
    GenerationCommitError, GenerationCommitResult, GenerationId, GenerationWriteRequest,
    OperationId, SeriesId,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

pub struct SqliteGenerationReader {
    connection: Connection,
    series_id: SeriesId,
}

pub struct CurrentGenerationSnapshot<'connection> {
    _transaction: Transaction<'connection>,
    generation_id: GenerationId,
    objects: Vec<ObjectReference>,
}

impl SqliteGenerationReader {
    pub fn open(root: impl AsRef<std::path::Path>) -> Result<Self, GenerationCommitError> {
        let root = root.as_ref();
        let (connection, metadata) =
            crate::workspace::open_read_only_workspace(root).map_err(map_workspace_error)?;
        if metadata.schema_version != crate::workspace::SCHEMA_VERSION {
            return Err(GenerationCommitError::CorruptWorkspace);
        }
        Ok(Self {
            connection,
            series_id: metadata.series.id,
        })
    }

    pub fn begin_snapshot(
        &mut self,
    ) -> Result<CurrentGenerationSnapshot<'_>, GenerationCommitError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .map_err(map_commit_sqlite_error)?;
        let generation = transaction
            .query_row(
                "SELECT generation_id FROM current_generation WHERE singleton = 1 AND series_id = ?1",
                [self.series_id.as_str()],
                |row| row.get::<_, String>(0),
            )
            .map_err(map_commit_sqlite_error)?;
        let generation_id =
            GenerationId::parse(generation).map_err(|_| GenerationCommitError::CorruptWorkspace)?;
        let mut statement = transaction
            .prepare(
                "WITH RECURSIVE generation_chain(generation_id, parent_generation_id) AS ( \
                     SELECT generation_id, parent_generation_id FROM generations \
                     WHERE series_id = ?1 AND generation_id = ?2 \
                     UNION \
                     SELECT generations.generation_id, generations.parent_generation_id \
                     FROM generations JOIN generation_chain \
                       ON generations.generation_id = generation_chain.parent_generation_id \
                     WHERE generations.series_id = ?1 \
                 ) \
                 SELECT DISTINCT refs.object_hash, refs.byte_len \
                 FROM generation_object_refs refs \
                 JOIN generation_chain chain ON chain.generation_id = refs.generation_id \
                 WHERE refs.series_id = ?1 AND refs.generation_id = ?2 ORDER BY refs.object_hash",
            )
            .map_err(map_commit_sqlite_error)?;
        let rows = statement
            .query_map((self.series_id.as_str(), generation_id.as_str()), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(map_commit_sqlite_error)?;
        let mut objects = Vec::new();
        for row in rows {
            let (hash, byte_len) = row.map_err(map_commit_sqlite_error)?;
            let hash = nt_domain::object::ObjectHash::parse(&hash)
                .map_err(|_| GenerationCommitError::CorruptWorkspace)?;
            let byte_len =
                u64::try_from(byte_len).map_err(|_| GenerationCommitError::CorruptWorkspace)?;
            objects.push(ObjectReference { hash, byte_len });
        }
        drop(statement);
        Ok(CurrentGenerationSnapshot {
            _transaction: transaction,
            generation_id,
            objects,
        })
    }
}

impl CurrentGenerationSnapshot<'_> {
    pub fn generation_id(&self) -> &GenerationId {
        &self.generation_id
    }

    pub fn objects(&self) -> &[ObjectReference] {
        &self.objects
    }

    pub fn finish(self) -> Result<(), GenerationCommitError> {
        self._transaction
            .commit()
            .map_err(|_| GenerationCommitError::StorageFailure)
    }
}

const FINGERPRINT_DOMAIN: &[u8] = b"nt-generation-write-v1";

pub(super) fn fingerprint_request(
    series_id: &SeriesId,
    request: &GenerationWriteRequest,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hash_field(&mut hasher, FINGERPRINT_DOMAIN);
    hash_field(&mut hasher, series_id.as_str().as_bytes());
    hash_field(&mut hasher, request.operation_id().as_str().as_bytes());
    hash_field(&mut hasher, request.expected_parent().as_str().as_bytes());
    hash_field(&mut hasher, request.generation_id().as_str().as_bytes());
    hasher.update((request.objects().len() as u64).to_be_bytes());
    for reference in request.objects() {
        hash_field(&mut hasher, reference.hash.as_bytes());
        hasher.update(reference.byte_len.to_be_bytes());
    }
    hasher.finalize().into()
}

fn hash_field(hasher: &mut Sha256, field: &[u8]) {
    hasher.update((field.len() as u64).to_be_bytes());
    hasher.update(field);
}

pub(super) fn commit_request(
    connection: &mut Connection,
    store: &impl ImmutableObjectStore,
    series_id: &SeriesId,
    request: GenerationWriteRequest,
    fingerprint: [u8; 32],
) -> Result<GenerationCommitResult, GenerationCommitError> {
    if let Some(result) =
        lookup_operation(connection, series_id, request.operation_id(), fingerprint)?
    {
        return Ok(result);
    }

    verify_objects(store, request.objects())?;

    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(map_commit_sqlite_error)?;

    let previous = transaction
        .query_row(
            "SELECT request_fingerprint, result_generation_id \
             FROM generation_operations WHERE series_id = ?1 AND operation_id = ?2",
            (series_id.as_str(), request.operation_id().as_str()),
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(map_commit_sqlite_error)?;
    if let Some((previous_fingerprint, previous_generation)) = previous {
        if previous_fingerprint.as_slice() != fingerprint {
            return Err(GenerationCommitError::OperationConflict);
        }
        let generation_id = GenerationId::parse(previous_generation)
            .map_err(|_| GenerationCommitError::CorruptWorkspace)?;
        drop(transaction);
        return Ok(GenerationCommitResult {
            generation_id,
            replayed: true,
        });
    }

    let current: String = transaction
        .query_row(
            "SELECT generation_id FROM current_generation WHERE singleton = 1 AND series_id = ?1",
            [series_id.as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(map_commit_sqlite_error)?
        .ok_or(GenerationCommitError::CorruptWorkspace)?;
    if current != request.expected_parent().as_str() {
        return Err(GenerationCommitError::StaleGeneration);
    }

    let committed_at = now_utc_ms()?;
    transaction
        .execute(
            "INSERT INTO generations(series_id, generation_id, parent_generation_id, created_at_utc_ms) \
             VALUES (?1, ?2, ?3, ?4)",
            (
                series_id.as_str(),
                request.generation_id().as_str(),
                request.expected_parent().as_str(),
                committed_at,
            ),
        )
        .map_err(map_commit_sqlite_error)?;

    for reference in request.objects() {
        let byte_len =
            i64::try_from(reference.byte_len).map_err(|_| GenerationCommitError::StorageFailure)?;
        transaction
            .execute(
                "INSERT INTO generation_object_refs(series_id, generation_id, object_hash, byte_len) \
                 VALUES (?1, ?2, ?3, ?4)",
                (
                    series_id.as_str(),
                    request.generation_id().as_str(),
                    reference.hash.to_hex(),
                    byte_len,
                ),
            )
            .map_err(map_commit_sqlite_error)?;
    }

    transaction
        .execute(
            "INSERT INTO generation_operations( \
                 series_id, operation_id, request_fingerprint, expected_parent_generation_id, \
                 result_generation_id, committed_at_utc_ms \
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                series_id.as_str(),
                request.operation_id().as_str(),
                fingerprint.as_slice(),
                request.expected_parent().as_str(),
                request.generation_id().as_str(),
                committed_at,
            ),
        )
        .map_err(map_commit_sqlite_error)?;

    let updated = transaction
        .execute(
            "UPDATE current_generation SET generation_id = ?1 \
             WHERE singleton = 1 AND series_id = ?2 AND generation_id = ?3",
            (
                request.generation_id().as_str(),
                series_id.as_str(),
                request.expected_parent().as_str(),
            ),
        )
        .map_err(map_commit_sqlite_error)?;
    if updated != 1 {
        return Err(GenerationCommitError::StaleGeneration);
    }

    let commit_result = transaction.commit();
    resolve_commit_result(commit_result, request.generation_id(), || {
        lookup_operation(connection, series_id, request.operation_id(), fingerprint)
    })
}

fn resolve_commit_result(
    commit_result: rusqlite::Result<()>,
    requested_generation_id: &GenerationId,
    lookup: impl FnOnce() -> Result<Option<GenerationCommitResult>, GenerationCommitError>,
) -> Result<GenerationCommitResult, GenerationCommitError> {
    if commit_result.is_ok() {
        return Ok(GenerationCommitResult {
            generation_id: requested_generation_id.clone(),
            replayed: false,
        });
    }

    lookup()
        .map_err(|error| match error {
            GenerationCommitError::OperationConflict | GenerationCommitError::CorruptWorkspace => {
                error
            }
            _ => GenerationCommitError::CommitOutcomeUnknown,
        })?
        .ok_or(GenerationCommitError::CommitOutcomeUnknown)
}

pub(super) fn lookup_operation(
    connection: &Connection,
    series_id: &SeriesId,
    operation_id: &OperationId,
    fingerprint: [u8; 32],
) -> Result<Option<GenerationCommitResult>, GenerationCommitError> {
    let previous = connection
        .query_row(
            "SELECT request_fingerprint, result_generation_id \
             FROM generation_operations WHERE series_id = ?1 AND operation_id = ?2",
            (series_id.as_str(), operation_id.as_str()),
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(map_commit_sqlite_error)?;
    let Some((previous_fingerprint, previous_generation)) = previous else {
        return Ok(None);
    };
    if previous_fingerprint.as_slice() != fingerprint {
        return Err(GenerationCommitError::OperationConflict);
    }
    let generation_id = GenerationId::parse(previous_generation)
        .map_err(|_| GenerationCommitError::CorruptWorkspace)?;
    Ok(Some(GenerationCommitResult {
        generation_id,
        replayed: true,
    }))
}

fn verify_objects(
    store: &impl ImmutableObjectStore,
    references: &[ObjectReference],
) -> Result<(), GenerationCommitError> {
    for reference in references {
        if i64::try_from(reference.byte_len).is_err() {
            return Err(GenerationCommitError::StorageFailure);
        }
        let mut reader = store
            .open_verified(reference.hash)
            .map_err(map_object_error)?;
        let length = io::copy(&mut reader, &mut io::sink())
            .map_err(|_| GenerationCommitError::StorageFailure)?;
        if length != reference.byte_len {
            return Err(GenerationCommitError::CorruptWorkspace);
        }
    }
    Ok(())
}

pub(super) fn map_object_error(error: ObjectStoreError) -> GenerationCommitError {
    match error {
        ObjectStoreError::ObjectNotFound
        | ObjectStoreError::ObjectConflict
        | ObjectStoreError::CorruptObject => GenerationCommitError::CorruptWorkspace,
        _ => GenerationCommitError::StorageFailure,
    }
}

fn map_workspace_error(error: nt_application::workspace::WorkspaceError) -> GenerationCommitError {
    match error {
        nt_application::workspace::WorkspaceError::Busy => GenerationCommitError::Busy,
        nt_application::workspace::WorkspaceError::CorruptWorkspace => {
            GenerationCommitError::CorruptWorkspace
        }
        _ => GenerationCommitError::StorageFailure,
    }
}

pub(super) fn map_commit_sqlite_error(error: rusqlite::Error) -> GenerationCommitError {
    match &error {
        rusqlite::Error::SqliteFailure(code, _)
            if matches!(
                code.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            ) =>
        {
            GenerationCommitError::Busy
        }
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            GenerationCommitError::CorruptWorkspace
        }
        _ => GenerationCommitError::StorageFailure,
    }
}

fn now_utc_ms() -> Result<i64, GenerationCommitError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| GenerationCommitError::StorageFailure)?;
    i64::try_from(duration.as_millis()).map_err(|_| GenerationCommitError::StorageFailure)
}

#[cfg(test)]
mod tests {
    use super::{
        GenerationCommitError, GenerationCommitResult, GenerationId, resolve_commit_result,
    };

    #[test]
    fn failed_commit_with_unreadable_ledger_stays_unknown() {
        let requested_generation_id =
            GenerationId::parse("0000000000000000000000000000002a").expect("valid generation ID");
        let mut lookup_calls = 0;
        let result = resolve_commit_result(
            Err(rusqlite::Error::InvalidQuery),
            &requested_generation_id,
            || {
                lookup_calls += 1;
                Err(GenerationCommitError::StorageFailure)
            },
        );

        assert_eq!(lookup_calls, 1);
        assert_eq!(result, Err(GenerationCommitError::CommitOutcomeUnknown));
    }

    #[test]
    fn failed_commit_reconciliation_preserves_confirmed_errors() {
        let requested_generation_id =
            GenerationId::parse("0000000000000000000000000000002a").expect("valid generation ID");

        for error in [
            GenerationCommitError::Busy,
            GenerationCommitError::StorageFailure,
            GenerationCommitError::WorkerUnavailable,
        ] {
            let result = resolve_commit_result(
                Err(rusqlite::Error::InvalidQuery),
                &requested_generation_id,
                || Err(error),
            );
            assert_eq!(result, Err(GenerationCommitError::CommitOutcomeUnknown));
        }

        for error in [
            GenerationCommitError::OperationConflict,
            GenerationCommitError::CorruptWorkspace,
        ] {
            let result = resolve_commit_result(
                Err(rusqlite::Error::InvalidQuery),
                &requested_generation_id,
                || Err(error),
            );
            assert_eq!(result, Err(error));
        }

        assert_eq!(
            resolve_commit_result(
                Err(rusqlite::Error::InvalidQuery),
                &requested_generation_id,
                || Ok(None),
            ),
            Err(GenerationCommitError::CommitOutcomeUnknown),
        );
    }

    #[test]
    fn failed_commit_with_confirmed_ledger_result_replays_that_generation() {
        let requested_generation_id =
            GenerationId::parse("0000000000000000000000000000002a").expect("valid generation ID");
        let committed_generation_id =
            GenerationId::parse("00000000000000000000000000000029").expect("valid generation ID");
        let committed_result = GenerationCommitResult {
            generation_id: committed_generation_id,
            replayed: true,
        };

        assert_eq!(
            resolve_commit_result(
                Err(rusqlite::Error::InvalidQuery),
                &requested_generation_id,
                || Ok(Some(committed_result.clone())),
            ),
            Ok(committed_result),
        );
    }

    #[test]
    fn successful_commit_does_not_reconcile_the_ledger() {
        let requested_generation_id =
            GenerationId::parse("0000000000000000000000000000002a").expect("valid generation ID");

        let result = resolve_commit_result(Ok(()), &requested_generation_id, || {
            panic!("a successful commit does not need a recovery lookup")
        });

        assert_eq!(
            result,
            Ok(GenerationCommitResult {
                generation_id: requested_generation_id,
                replayed: false,
            }),
        );
    }
}

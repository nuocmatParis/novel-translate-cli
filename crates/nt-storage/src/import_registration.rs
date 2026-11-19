use nt_application::volume_import::ImportWriteError;
use nt_domain::source_scope::ScopeMode;
use nt_domain::volume_import::VolumeRegistration;
use nt_domain::workspace::{
    GenerationCommitError, GenerationId, OperationId, SeriesId, VolumeId, VolumeMetadata,
};
use rusqlite::{Connection, OptionalExtension, Transaction};

use super::{VolumeImportSnapshot, ancestors, corrupt, read_default, schema_version, sql};

impl VolumeImportSnapshot<'_> {
    /// Metadata and membership belong to the same read snapshot as its state.
    pub fn volumes(&self) -> Result<Vec<VolumeMetadata>, ImportWriteError> {
        let mut statement = self
            .transaction
            .prepare(
                "SELECT volume_id, display_title, created_at_utc_ms
                 FROM volumes WHERE series_id = ?1 ORDER BY created_at_utc_ms, volume_id",
            )
            .map_err(sql)?;
        let mut rows = statement.query([self.series_id.as_str()]).map_err(sql)?;
        let mut volumes = Vec::new();
        let mut budget = super::Budget::default();
        while let Some(row) = rows.next().map_err(sql)? {
            let id =
                VolumeId::parse(row.get::<_, String>(0).map_err(sql)?).map_err(|_| corrupt())?;
            match ensure_visible(&self.transaction, &self.series_id, &self.generation_id, &id) {
                Ok(()) => {}
                Err(ImportWriteError::InvalidVolume) => continue,
                Err(error) => return Err(error),
            }
            let title: Option<&str> = row
                .get_ref(1)
                .map_err(sql)?
                .as_str_or_null()
                .map_err(|_| corrupt())?;
            budget.add(256 + title.map_or(0, str::len))?;
            volumes
                .try_reserve_exact(1)
                .map_err(|_| ImportWriteError::ResourceLimit)?;
            volumes.push(VolumeMetadata {
                id,
                series_id: self.series_id.clone(),
                display_title: title.map(str::to_owned),
                created_at_utc_ms: row.get(2).map_err(sql)?,
            });
        }
        Ok(volumes)
    }

    /// Allocate an opaque identity without writing anything to the workspace.
    pub fn new_volume_id(&self) -> Result<VolumeId, ImportWriteError> {
        VolumeId::parse(random_id(&self.transaction)?).map_err(|_| corrupt())
    }

    pub fn new_write_ids(&self) -> Result<(OperationId, GenerationId), ImportWriteError> {
        let operation = OperationId::parse(random_id(&self.transaction)?).map_err(|_| corrupt())?;
        let generation =
            GenerationId::parse(random_id(&self.transaction)?).map_err(|_| corrupt())?;
        Ok((operation, generation))
    }
}

fn random_id(connection: &Connection) -> Result<String, ImportWriteError> {
    connection
        .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
        .map_err(sql)
}

fn registered_at(
    connection: &Connection,
    series: &SeriesId,
    volume: &VolumeId,
) -> Result<Option<GenerationId>, ImportWriteError> {
    if schema_version(connection)? < 4 {
        return Ok(None);
    }
    let generation: Option<String> = connection
        .query_row(
            "SELECT generation_id FROM cp06_volume_registrations
             WHERE series_id = ?1 AND volume_id = ?2",
            (series.as_str(), volume.as_str()),
            |row| row.get(0),
        )
        .optional()
        .map_err(sql)?;
    generation
        .map(|value| GenerationId::parse(value).map_err(|_| corrupt()))
        .transpose()
}

pub(super) fn ensure_visible(
    connection: &Connection,
    series: &SeriesId,
    generation: &GenerationId,
    volume: &VolumeId,
) -> Result<(), ImportWriteError> {
    let Some(first) = registered_at(connection, series, volume)? else {
        return Ok(());
    };
    if ancestors(connection, series, generation, |at| {
        Ok((at == &first).then_some(()))
    })?
    .is_none()
    {
        return Err(ImportWriteError::InvalidVolume);
    }
    Ok(())
}

pub(crate) fn check_registration(
    connection: &Connection,
    series: &SeriesId,
    parent: &GenerationId,
    volume: &VolumeId,
    mode: ScopeMode,
) -> Result<(), ImportWriteError> {
    match super::volume_exists(connection, series, volume) {
        Ok(()) => return Err(ImportWriteError::InvalidVolume),
        Err(ImportWriteError::InvalidVolume) => {}
        Err(error) => return Err(error),
    }
    if read_default(connection, series, parent)? != mode {
        return Err(ImportWriteError::InvalidRequest);
    }
    Ok(())
}

pub(crate) fn register_volume(
    transaction: &Transaction<'_>,
    series: &SeriesId,
    volume: &VolumeId,
    timestamp: i64,
    registration: &VolumeRegistration,
) -> Result<(), ImportWriteError> {
    transaction
        .execute(
            "INSERT INTO volumes (series_id, volume_id, display_title, created_at_utc_ms)
             VALUES (?1, ?2, ?3, ?4)",
            (
                series.as_str(),
                volume.as_str(),
                registration.display_title.as_deref(),
                timestamp,
            ),
        )
        .map_err(sql)?;
    Ok(())
}

pub(crate) fn verify_registration_replay(
    connection: &Connection,
    series: &SeriesId,
    volume: &VolumeId,
    generation: &GenerationId,
    registration: Option<&VolumeRegistration>,
) -> Result<(), ImportWriteError> {
    let first = registered_at(connection, series, volume)?;
    if let Some(registration) = registration {
        if first.as_ref() != Some(generation) {
            return Err(corrupt());
        }
        let matches: bool = connection
            .query_row(
                "SELECT display_title IS ?3
                 FROM volumes WHERE series_id = ?1 AND volume_id = ?2",
                (
                    series.as_str(),
                    volume.as_str(),
                    registration.display_title.as_deref(),
                ),
                |row| row.get(0),
            )
            .map_err(sql)?;
        if !matches {
            return Err(corrupt());
        }
    } else if first.as_ref() == Some(generation) {
        return Err(GenerationCommitError::OperationConflict.into());
    }
    Ok(())
}

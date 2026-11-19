use std::path::Path;

use novel_translate::import_workflow::{
    ImportFlowError, ImportTarget, PreparedImport, Publication, ScopeChange, prepare_import,
    prepare_registered_import, prepare_scope_change,
};
use nt_application::volume_import::{
    GenerationCommitError, GenerationId, ImportWriteError, SeriesId, VolumeId,
};
use nt_storage::{SqliteVolumeImportReader, VolumeImportSnapshot};

pub(crate) enum ImportDestination {
    Selected(VolumeId),
    New,
}

pub(crate) enum ImportCommandError {
    InvalidExtension,
    Import(ImportFlowError),
}

impl From<ImportFlowError> for ImportCommandError {
    fn from(error: ImportFlowError) -> Self {
        Self::Import(error)
    }
}

pub(crate) fn import(
    workspace: &Path,
    expected: Option<(&SeriesId, &GenerationId)>,
    epub: &Path,
    destination: ImportDestination,
) -> Result<(SeriesId, Publication), ImportCommandError> {
    if !epub
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("epub"))
    {
        return Err(ImportCommandError::InvalidExtension);
    }

    let mut reader = SqliteVolumeImportReader::open(workspace).map_err(ImportFlowError::from)?;
    let snapshot = reader.begin_snapshot().map_err(ImportFlowError::from)?;
    check_view(&snapshot, expected)?;
    let (id, register) = match destination {
        ImportDestination::Selected(id) => {
            let register = match snapshot.volume(&id) {
                Ok(_) => false,
                Err(ImportWriteError::InvalidVolume) => true,
                Err(error) => return Err(ImportFlowError::from(error).into()),
            };
            (id, register)
        }
        ImportDestination::New => (
            snapshot.new_volume_id().map_err(ImportFlowError::from)?,
            true,
        ),
    };
    let target = write_target(&snapshot, id)?;
    let series = target.series.clone();
    snapshot.finish().map_err(ImportFlowError::from)?;

    let prepared = if register {
        prepare_registered_import(workspace, epub, target)?
    } else {
        prepare_import(workspace, epub, target)?
    };
    Ok((series, submit(&prepared)?))
}

pub(crate) fn scope(
    workspace: &Path,
    expected: Option<(&SeriesId, &GenerationId)>,
    volume: VolumeId,
    change: ScopeChange,
) -> Result<(SeriesId, Publication), ImportFlowError> {
    let mut reader = SqliteVolumeImportReader::open(workspace)?;
    let snapshot = reader.begin_snapshot()?;
    check_view(&snapshot, expected)?;
    let target = write_target(&snapshot, volume)?;
    let series = target.series.clone();
    snapshot.finish()?;
    Ok((
        series,
        submit(&prepare_scope_change(workspace, target, change)?)?,
    ))
}

pub(crate) fn check_view(
    snapshot: &VolumeImportSnapshot<'_>,
    expected: Option<(&SeriesId, &GenerationId)>,
) -> Result<(), ImportFlowError> {
    if let Some((series, parent)) = expected {
        if snapshot.series_id() != series {
            return Err(ImportFlowError::InvalidWorkspace);
        }
        if snapshot.generation_id() != parent {
            return Err(ImportWriteError::from(GenerationCommitError::StaleGeneration).into());
        }
    }
    Ok(())
}

pub(crate) fn write_target(
    snapshot: &VolumeImportSnapshot<'_>,
    volume: VolumeId,
) -> Result<ImportTarget, ImportFlowError> {
    let (operation, generation) = snapshot.new_write_ids()?;
    Ok(ImportTarget {
        series: snapshot.series_id().clone(),
        volume,
        expected_parent: snapshot.generation_id().clone(),
        operation,
        generation,
    })
}

pub(crate) fn submit(prepared: &PreparedImport) -> Result<Publication, ImportFlowError> {
    let result = prepared.submit()?;
    if matches!(result, Publication::CommitOutcomeUnknown { .. })
        && let Ok(resolved) = prepared.reconcile()
    {
        return Ok(resolved);
    }
    Ok(result)
}

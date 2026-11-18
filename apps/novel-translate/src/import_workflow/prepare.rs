use std::path::Path;

use nt_application::chapter_map::build_candidate_map;
use nt_application::source_scope::{ScopeWorking, evaluate_scope_within};
use nt_application::volume_import::{
    CapturedSourceError, GenerationCommitError, ImportWriteError, MAX_IMPORT_WORKING_BYTES,
    chapter_map::ConfirmedBoundary,
};
use nt_epub::{
    EpubInventory, EpubZip, PARSER_VERSION, ReferenceIndexLimits, SourceExtraction,
    SourceExtractionError, index_references,
};
use nt_storage::{FileImportSourceStore, FileObjectStore};

use crate::import_codec::{Digest, ResourceFiles, SnapshotDraft, SnapshotInputs, VolumeRecord};

use super::classification_history::PreviousClassifications;
use super::eligibility::{check_dependencies, check_spine};
use super::persisted::read_scope;
use super::prepare_source::SourceReservation;
use super::scope::scope_input;
use super::{
    ImportFlowError, ImportTarget, PreparedImport, capture_epub, prepare_staged_import,
    read_saved_volume,
};

const MAPPING_RULE: &str = "nt-chapter-map/1";
const SCOPE_RULE: &str = "nt-content-scope/1";

/// Capture the selected book and stage a fully checked import. The returned
/// request is the only input to publish or retry this operation.
pub fn prepare_import(
    workspace: &Path,
    source_path: &Path,
    target: ImportTarget,
) -> Result<PreparedImport, ImportFlowError> {
    let before = read_saved_volume(workspace, &target.series, &target.volume, None)?;
    if before.generation != target.expected_parent {
        return Err(ImportWriteError::from(GenerationCommitError::StaleGeneration).into());
    }
    let mode = before.version.state.mode();
    let objects = FileObjectStore::new(workspace).map_err(ImportFlowError::Files)?;
    let images = FileImportSourceStore::new(workspace).map_err(ImportFlowError::Files)?;
    let original = capture_epub(&objects, source_path).map_err(ImportFlowError::Capture)?;
    let epub = Digest(*original.hash.as_bytes());
    let (confirmed, history, previous_bytes) = match before.record {
        Some(mut record) => {
            let previous_bytes = record.reimport_history_bytes()?;
            let scope = read_scope(
                workspace,
                &target.series,
                &target.volume,
                &before.generation,
                &mut record,
            )?;
            let history = PreviousClassifications::capture(&record.source_index, scope)?;
            (
                previous_boundaries(Some(record), epub)?,
                history,
                previous_bytes,
            )
        }
        None => (Vec::new(), PreviousClassifications::default(), 0),
    };

    let file = objects
        .open_import_object(original)
        .map_err(ImportFlowError::Files)?;
    let mut archive = EpubZip::new(file).map_err(ImportFlowError::Archive)?;
    archive
        .validate_windows_materialization()
        .map_err(ImportFlowError::Archive)?;
    let inventory_memory = MAX_IMPORT_WORKING_BYTES
        .checked_sub(previous_bytes)
        .ok_or(CapturedSourceError::ResourceLimit)?;
    let inventory = EpubInventory::inspect_captured(&mut archive, inventory_memory)?;
    check_spine(&inventory).map_err(ImportFlowError::Blocked)?;
    let measured = SourceReservation::measure(&mut archive, &inventory, epub.0, previous_bytes)?;
    let navigation_memory = measured.remaining()?;
    let mut source =
        SourceExtraction::extract_captured(&mut archive, &inventory, epub.0, navigation_memory)
            .map_err(|error| match error {
                SourceExtractionError::WorkingMemoryLimit => {
                    CapturedSourceError::ResourceLimit.into()
                }
                other => ImportFlowError::Extraction(other),
            })?;
    let navigation_bytes = crate::import_codec::memory::working(&source.navigation, 0)?;
    let navigation_bytes =
        crate::import_codec::memory::working(&source.navigation_diagnostics, navigation_bytes)?;
    let reference_memory = navigation_memory
        .checked_sub(navigation_bytes)
        .ok_or(CapturedSourceError::ResourceLimit)?;
    let limits = ReferenceIndexLimits::for_working_bytes(reference_memory);
    let references = index_references(&mut archive, &inventory, &source, limits)
        .map_err(ImportFlowError::References)?;
    let reference_bytes = crate::import_codec::memory::working(&references, 0)?;
    let navigation_space = reference_memory
        .checked_sub(reference_bytes)
        .ok_or(CapturedSourceError::ResourceLimit)?;
    check_dependencies(&inventory, &references, &source.diagnostics)
        .map_err(ImportFlowError::Blocked)?;

    // The CP04 adapter copies its block vectors when supplied. Transfer them
    // for the map and scope pass so the expanded text exists only once.
    let blocks: Vec<_> = source
        .documents
        .iter_mut()
        .map(|doc| std::mem::take(&mut doc.blocks))
        .collect();
    let navigation = std::mem::take(&mut source.navigation);
    let navigation_diagnostics = std::mem::take(&mut source.navigation_diagnostics);
    let mut input = source.chapter_map_input(confirmed);
    input.navigation = navigation;
    input.diagnostics = navigation_diagnostics;
    for (document, blocks) in input.documents.iter_mut().zip(blocks) {
        document.blocks = blocks;
    }
    let mapping = build_candidate_map(&input).map_err(ImportFlowError::Mapping)?;
    source.navigation = std::mem::take(&mut input.navigation);
    source.navigation_diagnostics = std::mem::take(&mut input.diagnostics);
    let navigation_labels = nt_epub::SourceExtraction::read_captured_navigation_labels(
        &mut archive,
        &source.navigation,
        navigation_space,
    )?;
    let live = measured
        .retained_bytes()?
        .checked_add(navigation_bytes)
        .and_then(|bytes| bytes.checked_add(reference_bytes))
        .ok_or(CapturedSourceError::ResourceLimit)?;
    let base_reservation = crate::import_codec::memory::working(&navigation_labels, live)?;
    let mut scope_space = ScopeWorking::new(
        MAX_IMPORT_WORKING_BYTES
            .checked_sub(base_reservation)
            .ok_or(CapturedSourceError::ResourceLimit)?,
    );
    let mut observed = scope_input(
        &source,
        &input,
        &mapping,
        &measured.placements,
        &references,
        &navigation_labels,
        &mut scope_space,
    )
    .map_err(ImportFlowError::Scope)?;
    history.apply(&mut observed, &input, &mut scope_space)?;
    for (destination, source) in source.documents.iter_mut().zip(&mut input.documents) {
        destination.blocks = std::mem::take(&mut source.blocks);
    }
    let scope_bytes = observed
        .retained_working_bytes()
        .map_err(ImportFlowError::Scope)?;
    let reserved = base_reservation
        .checked_add(scope_bytes)
        .ok_or(CapturedSourceError::ResourceLimit)?;
    drop(measured);
    let files = ResourceFiles {
        objects: &objects,
        source_images: &images,
        volume: &target.volume,
        epub,
    };
    let mut draft = SnapshotDraft::capture(
        SnapshotInputs {
            original,
            series: &target.series,
            inventory: &inventory,
            source: &source,
            mapping: &mapping,
            confirmed_boundaries: &input.confirmed_boundaries,
            references: &references,
            mapping_rule_version: MAPPING_RULE,
            classification_rule_version: SCOPE_RULE,
        },
        &files,
        &mut archive,
        reserved,
    )
    .map_err(ImportFlowError::Stage)?;
    drop(source);
    drop(input);
    drop(references);
    drop(inventory);
    drop(mapping);
    drop(archive);

    let mut evaluation = ScopeWorking::new(draft.remaining_working_bytes());
    let scope = evaluate_scope_within(
        observed.with_context(draft.decision_context().clone(), mode),
        &mut evaluation,
    )
    .map_err(ImportFlowError::Scope)?;
    draft.reserve_scope_evaluation(evaluation.used())?;
    let staged = draft
        .stage(&scope, &objects)
        .map_err(ImportFlowError::Stage)?;
    drop(scope);
    prepare_staged_import(workspace, target, staged)
}

fn previous_boundaries(
    previous: Option<VolumeRecord>,
    epub: Digest,
) -> Result<Vec<ConfirmedBoundary>, ImportFlowError> {
    let Some(record) = previous else {
        return Ok(Vec::new());
    };
    if record.source.epub_sha256 != epub
        || record.source.parser_version != PARSER_VERSION
        || record.source.mapping_rule_version != MAPPING_RULE
    {
        return Ok(Vec::new());
    }
    if record.mapping.confirmed_boundaries.len()
        > MAX_IMPORT_WORKING_BYTES / size_of::<ConfirmedBoundary>()
    {
        return Err(CapturedSourceError::ResourceLimit.into());
    }
    let (_, boundaries) = record.mapping.into_map()?;
    Ok(boundaries)
}

use nt_application::object_store::ObjectStoreError;
use nt_application::volume_import::{ObjectHash, ObjectReference, VolumeResource};
use nt_epub::{
    ArchiveError, EpubInventory, EpubZip, InventoryCoverage, ReferenceIndex, ReferenceIndexError,
    ReferenceIndexLimits, ResourceEncryption, SourceDiagnostic,
};

use super::super::{ArchiveSourceError, ReplayedSource, ResourceFileError, ResourceFiles};
use super::{
    Budget, CodecError, SourceInputDiagnostic, SourceIssue, StoredInventory, VolumeDiagnostics,
    VolumeRecord,
};
use crate::import_codec::memory::working;

#[derive(Debug)]
pub enum CaptureVerificationError {
    Archive(ArchiveError),
    Object(ObjectStoreError),
    Codec(CodecError),
    Source(ArchiveSourceError),
    References(ReferenceIndexError),
    Resources(ResourceFileError),
    EvidenceMismatch,
}

impl From<CodecError> for CaptureVerificationError {
    fn from(error: CodecError) -> Self {
        Self::Codec(error)
    }
}

pub struct CaptureEvidence {
    pub(crate) working_bytes: usize,
    pub inventory: EpubInventory,
    pub references: ReferenceIndex,
    pub diagnostics: Vec<SourceDiagnostic>,
    pub navigation_labels: Vec<nt_application::volume_import::CapturedNavigationLabel>,
    pub placements: std::collections::HashMap<
        nt_application::volume_import::source_range::SourceBlockId,
        nt_application::volume_import::CapturedBlockPlacement,
    >,
}

impl VolumeRecord {
    /// Validate the stored source and every indexed use against the pinned
    /// original. The caller verifies the database version and scope context.
    pub fn verify_capture(
        &self,
        files: &ResourceFiles<'_>,
        rows: &[VolumeResource],
        reserved_working_bytes: usize,
    ) -> Result<CaptureEvidence, CaptureVerificationError> {
        let record_working = working(self, reserved_working_bytes)?;
        self.validate_structure()?;
        if files.epub != self.source.epub_sha256 {
            return Err(CaptureVerificationError::EvidenceMismatch);
        }
        let original = files
            .objects
            .open_import_object(ObjectReference {
                hash: ObjectHash::from_bytes(self.source.original_object_sha256.0),
                byte_len: self.source.original_object_byte_len,
            })
            .map_err(CaptureVerificationError::Object)?;
        let mut archive = EpubZip::new(original).map_err(CaptureVerificationError::Archive)?;
        archive
            .validate_windows_materialization()
            .map_err(CaptureVerificationError::Archive)?;
        let inventory = EpubInventory::inspect_captured(
            &mut archive,
            Budget::new(record_working)?.remaining_bytes(),
        )
        .map_err(|error| CaptureVerificationError::Source(ArchiveSourceError::XmlProof(error)))?;
        let inventory_working = working(&inventory, record_working)?;
        let actual = StoredInventory::capture(&inventory, inventory_working)?;
        if self.inventory != actual {
            return Err(CaptureVerificationError::EvidenceMismatch);
        }
        drop(actual);

        let source = self
            .source_index
            .replay_capture(
                &mut archive,
                &inventory,
                &self.resources,
                self.source.epub_sha256,
                &self.source.parser_version,
                inventory_working,
            )
            .map_err(CaptureVerificationError::Source)?;
        let diagnostics = original_diagnostics(&inventory, &source.document_diagnostics);
        let mut budget = Budget::new(working(&diagnostics, source.working_bytes)?)?;
        if !self.check_source_evidence(&source, &diagnostics, &mut budget)? {
            return Err(CaptureVerificationError::EvidenceMismatch);
        }
        let references = ReferenceIndex::from_replayed(
            &mut archive,
            &inventory,
            &source.images,
            &diagnostics,
            ReferenceIndexLimits::for_working_bytes(budget.remaining_bytes()),
        )
        .map_err(CaptureVerificationError::References)?;
        budget.charge(working(&references, 0)?, 0)?;
        if VolumeDiagnostics::reference_issues(&references, &mut budget)?
            != self.diagnostics.unresolved_resource_uses
        {
            return Err(CaptureVerificationError::EvidenceMismatch);
        }
        let navigation_labels = nt_epub::SourceExtraction::read_captured_navigation_labels(
            &mut archive,
            &source.navigation,
            budget.remaining_bytes(),
        )
        .map_err(|error| CaptureVerificationError::Source(ArchiveSourceError::XmlProof(error)))?;
        let ReplayedSource { placements, .. } = source;
        let live = working(&navigation_labels, budget.used_bytes())?;
        files
            .verify(
                &mut archive,
                &inventory,
                &references,
                &self.resources,
                rows,
                live,
            )
            .map_err(CaptureVerificationError::Resources)?;
        Ok(CaptureEvidence {
            working_bytes: live - record_working,
            inventory,
            references,
            diagnostics,
            navigation_labels,
            placements,
        })
    }

    fn check_source_evidence(
        &self,
        source: &ReplayedSource,
        diagnostics: &[SourceDiagnostic],
        budget: &mut Budget,
    ) -> Result<bool, CodecError> {
        if !self
            .mapping
            .matches_original_navigation(&source.navigation, budget)?
            || self.diagnostics.navigation.len() != source.navigation_diagnostics.len()
            || self.diagnostics.source.len() != diagnostics.len()
        {
            return Ok(false);
        }
        for (saved, original) in self
            .diagnostics
            .navigation
            .iter()
            .zip(&source.navigation_diagnostics)
        {
            if saved != &SourceInputDiagnostic::capture(original, budget)? {
                return Ok(false);
            }
        }
        // The established extractor walks documents in a different order
        // from the persisted map, and its duplicate-ID order is not stable.
        // Inventory order is stable; compare the remaining evidence by identity.
        let mut rebuilt = budget.vector(diagnostics.len())?;
        for diagnostic in diagnostics {
            rebuilt.push(SourceIssue::capture(diagnostic, budget)?);
        }
        let mut saved = budget.vector(self.diagnostics.source.len())?;
        saved.extend(self.diagnostics.source.iter());
        let mut rebuilt_refs = budget.vector(rebuilt.len())?;
        rebuilt_refs.extend(rebuilt.iter());
        saved.sort_by(|a, b| issue_key(a).cmp(&issue_key(b)));
        rebuilt_refs.sort_by(|a, b| issue_key(a).cmp(&issue_key(b)));
        Ok(saved == rebuilt_refs)
    }
}

fn issue_key(issue: &SourceIssue) -> (u8, &str, &str) {
    match issue {
        SourceIssue::InventoryPartial {} => (0, "", ""),
        SourceIssue::Inventory { .. } => (1, "", ""),
        SourceIssue::InvalidOptionalNavigation { canonical_path } => (2, canonical_path, ""),
        SourceIssue::UnsupportedFixedLayout { canonical_path } => (3, canonical_path, ""),
        SourceIssue::UnsupportedEncryption { canonical_path } => (4, canonical_path, ""),
        SourceIssue::DuplicateXhtmlId { canonical_path, id } => (5, canonical_path, id),
        SourceIssue::ProtectedResourceMissing { canonical_path } => (6, canonical_path, ""),
    }
}

fn original_diagnostics(
    inventory: &EpubInventory,
    document_diagnostics: &[SourceDiagnostic],
) -> Vec<SourceDiagnostic> {
    let mut diagnostics = Vec::new();
    if inventory.coverage == InventoryCoverage::Partial {
        diagnostics.push(SourceDiagnostic::InventoryPartial);
    }
    diagnostics.extend(
        inventory
            .diagnostics
            .iter()
            .cloned()
            .map(SourceDiagnostic::Inventory),
    );
    diagnostics.extend(inventory.resources.iter().filter_map(|resource| {
        if resource.media_type == "application/xhtml+xml"
            && resource.encryption == ResourceEncryption::Unsupported
        {
            resource
                .path
                .as_ref()
                .map(|canonical_path| SourceDiagnostic::UnsupportedEncryption {
                    canonical_path: canonical_path.clone(),
                })
        } else {
            None
        }
    }));
    diagnostics.extend_from_slice(document_diagnostics);
    diagnostics
}

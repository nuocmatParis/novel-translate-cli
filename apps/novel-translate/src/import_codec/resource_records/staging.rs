//! Stage or verify resources of the exact captured archive, including all uses.

use std::collections::BTreeMap;
use std::io::{Read, Seek};

use nt_application::object_store::{ImmutableObjectStore, ObjectStoreError};
use nt_application::volume_import::{
    ImageInformation, ObjectHash, ObjectReference, VolumeId, VolumeResource,
};
use nt_epub::{
    ArchiveError, EpubInventory, EpubZip, ReferenceIndex, ReferenceIndexError, ReferenceKind,
    ReferenceTarget, ResourceReference, SourceExtraction,
};
use nt_storage::{FileImportSourceStore, FileObjectStore};

use super::super::records::hash_bytes;
use super::{Budget, CodecError, Digest, ResourceInput, ResourceRecord, ResourceRole};

#[derive(Debug)]
pub enum ResourceFileError {
    Archive(ArchiveError),
    Reference(ReferenceIndexError),
    Storage(ObjectStoreError),
    Codec(CodecError),
    MissingEntry,
    EvidenceMismatch,
}

impl From<CodecError> for ResourceFileError {
    fn from(error: CodecError) -> Self {
        Self::Codec(error)
    }
}

pub struct ResourceFiles<'a> {
    pub objects: &'a FileObjectStore,
    pub source_images: &'a FileImportSourceStore,
    pub volume: &'a VolumeId,
    pub epub: Digest,
}

impl ResourceFiles<'_> {
    /// Fill one volume's complete ordered resource records before DB writes.
    pub fn stage(
        &self,
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        references: &ReferenceIndex,
        reserved_working_bytes: usize,
    ) -> Result<Vec<ResourceRecord>, ResourceFileError> {
        self.stage_with_budget(
            archive,
            inventory,
            references,
            &mut Budget::new(reserved_working_bytes)?,
        )
    }

    /// Check the original entry, configuration, relationships, sealed object
    /// and source image copy against the already-read snapshot and DB rows.
    pub fn verify(
        &self,
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        references: &ReferenceIndex,
        records: &[ResourceRecord],
        rows: &[VolumeResource],
        reserved_working_bytes: usize,
    ) -> Result<(), ResourceFileError> {
        self.run(
            archive,
            inventory,
            references,
            Action::Verify { records, rows },
            &mut Budget::new(reserved_working_bytes)?,
        )?;
        Ok(())
    }

    pub(in crate::import_codec) fn stage_with_budget(
        &self,
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        references: &ReferenceIndex,
        budget: &mut Budget,
    ) -> Result<Vec<ResourceRecord>, ResourceFileError> {
        self.run(archive, inventory, references, Action::Stage, budget)
    }

    fn run(
        &self,
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        references: &ReferenceIndex,
        action: Action<'_>,
        budget: &mut Budget,
    ) -> Result<Vec<ResourceRecord>, ResourceFileError> {
        archive
            .validate_windows_materialization()
            .map_err(ResourceFileError::Archive)?;
        let plan = plan_resources(inventory, references, budget)?;
        let mut result = Vec::new();
        let mut stored_rows = BTreeMap::new();
        if let Action::Verify { records, rows } = action {
            if records.len() != plan.len() || rows.len() != plan.len() {
                return Err(ResourceFileError::EvidenceMismatch);
            }
            charge_entries(budget, rows.len())?;
            for row in rows {
                if stored_rows
                    .insert(row.canonical_path.as_str(), row)
                    .is_some()
                {
                    return Err(ResourceFileError::EvidenceMismatch);
                }
            }
        } else {
            result = budget.vector(plan.len())?;
        }

        for (index, (path, plan)) in plan.into_iter().enumerate() {
            let entry = archive
                .entry_by_path(path)
                .map_err(ResourceFileError::Archive)?
                .filter(|entry| !entry.is_directory())
                .ok_or(ResourceFileError::MissingEntry)?;
            let input_bytes =
                usize::try_from(entry.declared_size()).map_err(|_| CodecError::ResourceLimit)?;
            let transient = entry_working_bytes(input_bytes, plan.media_type)?;
            let remaining = budget.remaining_bytes();
            if transient > remaining {
                return Err(CodecError::ResourceLimit.into());
            }
            let bytes = archive
                .read_entry(entry.id())
                .map_err(ResourceFileError::Archive)?;
            let (role, info) = plan.details(&bytes)?;
            let object = match action {
                Action::Stage => self
                    .objects
                    .put(&mut &bytes[..])
                    .map_err(ResourceFileError::Storage)?,
                Action::Verify { .. } => ObjectReference {
                    hash: ObjectHash::from_bytes(hash_bytes(&bytes)?.0),
                    byte_len: bytes.len() as u64,
                },
            };
            drop(bytes);

            let source_path = if role == ResourceRole::Image {
                Some(
                    FileImportSourceStore::source_relative_path(self.volume, self.epub.0, path)
                        .map_err(ResourceFileError::Storage)?,
                )
            } else {
                None
            };
            let input = ResourceInput {
                role,
                canonical_path: path,
                object,
                source_relative_path: source_path.as_deref(),
                declared_media_type: plan.media_type,
                image_information: &info,
            };
            match action {
                Action::Stage => {
                    if source_path.is_some() {
                        let mut original = self
                            .objects
                            .open_import_object(object)
                            .map_err(ResourceFileError::Storage)?;
                        self.source_images
                            .materialize(self.volume, self.epub.0, path, object, &mut original)
                            .map_err(ResourceFileError::Storage)?;
                    }
                    result.push(ResourceRecord::capture_with_budget(
                        input, &plan.uses, budget,
                    )?);
                }
                Action::Verify { records, .. } => {
                    let mut temporary = Budget::new(256 * 1024 * 1024 - budget.remaining_bytes())?;
                    let expected =
                        ResourceRecord::capture_with_budget(input, &plan.uses, &mut temporary)?;
                    let row = stored_rows
                        .get(path)
                        .ok_or(ResourceFileError::EvidenceMismatch)?;
                    if expected != records[index] || !expected.matches_storage_row(row) {
                        return Err(ResourceFileError::EvidenceMismatch);
                    }
                    self.objects
                        .open_import_object(object)
                        .map_err(ResourceFileError::Storage)?;
                    if source_path.is_some() {
                        self.source_images
                            .open_verified(self.volume, self.epub.0, path, object)
                            .map_err(ResourceFileError::Storage)?;
                    }
                }
            }
        }
        Ok(result)
    }
}

impl ResourceRecord {
    pub fn object(&self) -> ObjectReference {
        ObjectReference {
            hash: ObjectHash::from_bytes(self.object_sha256.0),
            byte_len: self.byte_len,
        }
    }

    pub fn matches_storage_row(&self, row: &VolumeResource) -> bool {
        self.role.storage_name() == row.role
            && self.canonical_path == row.canonical_path
            && self.object() == row.object
            && self.source_relative_path == row.source_relative_path
    }
}

#[derive(Clone, Copy)]
enum Action<'a> {
    Stage,
    Verify {
        records: &'a [ResourceRecord],
        rows: &'a [VolumeResource],
    },
}

#[derive(Default)]
struct Plan<'a> {
    media_type: Option<&'a str>,
    uses: Vec<&'a ResourceReference>,
}
impl Plan<'_> {
    fn details(&self, bytes: &[u8]) -> Result<(ResourceRole, ImageInformation), ResourceFileError> {
        let declared = self.media_type.unwrap_or_default();
        let explicit_image_reference = self.uses.iter().any(|reference| {
            matches!(
                reference.kind,
                ReferenceKind::XhtmlImageSrc
                    | ReferenceKind::SvgImageHref
                    | ReferenceKind::SrcsetCandidate
            )
        });
        let information = SourceExtraction::image_information(bytes, declared)
            .map_err(ResourceFileError::Reference)?;
        let role = if declared.starts_with("image/")
            || explicit_image_reference
            || information.format.is_some()
        {
            ResourceRole::Image
        } else if declared == "text/css"
            || self.uses.iter().any(|reference| {
                matches!(
                    reference.kind,
                    ReferenceKind::CssImport | ReferenceKind::StylesheetLink
                )
            })
        {
            ResourceRole::Stylesheet
        } else if declared.starts_with("font/")
            || matches!(
                declared,
                "application/font-woff"
                    | "application/vnd.ms-opentype"
                    | "application/x-font-ttf"
                    | "application/x-font-opentype"
            )
        {
            ResourceRole::Font
        } else if declared == "application/xhtml+xml" {
            ResourceRole::Xhtml
        } else {
            ResourceRole::Other
        };
        Ok((role, information))
    }
}

fn plan_resources<'a>(
    inventory: &'a EpubInventory,
    references: &'a ReferenceIndex,
    budget: &mut Budget,
) -> Result<BTreeMap<&'a str, Plan<'a>>, ResourceFileError> {
    charge_entries(budget, inventory.resources.len())?;
    charge_entries(budget, references.references.len())?;
    let mut paths = BTreeMap::new();
    for resource in inventory
        .resources
        .iter()
        .filter(|resource| resource.available)
    {
        if let Some(path) = resource.path.as_deref()
            && paths
                .insert(
                    path,
                    Plan {
                        media_type: Some(&resource.media_type),
                        uses: Vec::new(),
                    },
                )
                .is_some()
        {
            return Err(ResourceFileError::EvidenceMismatch);
        }
    }
    for reference in &references.references {
        if let ReferenceTarget::Internal { path, .. } = &reference.target {
            paths.entry(path).or_default().uses.push(reference);
        }
    }
    Ok(paths)
}

fn charge_entries(budget: &mut Budget, count: usize) -> Result<(), CodecError> {
    budget.charge(
        count.checked_mul(128).ok_or(CodecError::ResourceLimit)?,
        count,
    )
}

fn entry_working_bytes(bytes: usize, media_type: Option<&str>) -> Result<usize, CodecError> {
    let steps = if media_type == Some("image/svg+xml") {
        8
    } else {
        2
    };
    bytes
        .checked_mul(steps)
        .and_then(|value| value.checked_add(256 * 1024))
        .ok_or(CodecError::ResourceLimit)
}

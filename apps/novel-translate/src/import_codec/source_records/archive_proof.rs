//! Verify all stored source documents against the captured archive.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Read, Seek};

use nt_application::volume_import::{
    CapturedBlock, CapturedBlockPlacement, CapturedImagePlacement, CapturedSourceError,
    CapturedSourceRequest, CapturedUnitRun, chapter_map,
    source_range::{ProtectedSourceUnit, SourceBlockId},
};
use nt_epub::{
    ArchiveError, EpubInventory, EpubZip, ResourceEncryption, SourceDiagnostic, SourceExtraction,
    SourceExtractionError, SpineContentKind,
};

use super::{
    BlockLocator, Budget, Domain, ProtectedUnit, SourceAnchor, StoredSourceDocument,
    StoredSourceIndex, StoredUnitRun,
};
use crate::import_codec::{CodecError, Digest, ResourceRecord};

#[derive(Debug)]
pub enum ArchiveSourceError {
    Archive(ArchiveError),
    XmlProof(CapturedSourceError),
    Navigation(SourceExtractionError),
    Codec(CodecError),
    MissingRequiredSource,
    EvidenceMismatch,
}
impl From<CodecError> for ArchiveSourceError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

pub struct ReplayedSource {
    pub(crate) working_bytes: usize,
    pub navigation: Vec<chapter_map::NavigationTree>,
    pub navigation_diagnostics: Vec<chapter_map::SourceInputDiagnostic>,
    pub document_diagnostics: Vec<SourceDiagnostic>,
    pub images: BTreeMap<String, Vec<CapturedImagePlacement>>,
    pub placements: HashMap<SourceBlockId, CapturedBlockPlacement>,
}

impl StoredSourceIndex {
    /// Replay the complete document set against original XML or image-page
    /// bytes. The owner verifies the supplied resource records against their
    /// actual objects and relationships before exposing a saved volume.
    pub fn replay_capture(
        &self,
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        resources: &[ResourceRecord],
        epub_sha256: Digest,
        parser_version: &str,
        reserved_working_bytes: usize,
    ) -> Result<ReplayedSource, ArchiveSourceError> {
        let mut budget = super::Budget::new(reserved_working_bytes)?;
        let plan = document_plan(inventory, &mut budget)?;
        let mut objects = BTreeMap::new();
        budget.charge(
            resources
                .len()
                .checked_mul(96)
                .ok_or(CodecError::ResourceLimit)?,
            0,
        )?;
        for resource in resources {
            if objects
                .insert(resource.canonical_path.as_str(), resource.object_sha256)
                .is_some()
            {
                return Err(ArchiveSourceError::EvidenceMismatch);
            }
        }
        if self
            .anchors
            .windows(2)
            .any(|pair| anchor_order(&pair[0]) > anchor_order(&pair[1]))
        {
            return Err(ArchiveSourceError::EvidenceMismatch);
        }

        let mut saved_documents = self.documents.iter().peekable();
        let mut anchor_count = 0usize;
        let mut diagnostics = Vec::new();
        let mut optional_navigation = Vec::new();
        let mut images = BTreeMap::new();
        let mut block_placements = HashMap::new();
        for spec in plan {
            let expected = saved_documents
                .peek()
                .copied()
                .filter(|doc| doc.canonical_path == spec.path);
            if let Some(document) = expected {
                check_document_metadata(document, &spec)?;
            }
            let mut placements = reserve_placements(expected, &mut budget)?;
            let mut expected_blocks = expected.map(|doc| doc.blocks.iter()).into_iter().flatten();
            let parser_memory = budget.remaining_bytes() / 2;
            let request = CapturedSourceRequest {
                canonical_path: spec.path,
                epub_sha256: epub_sha256.0,
                parser_version,
                is_navigation_document: spec.navigation,
                working_bytes: parser_memory,
            };
            let mut compare = |block: &CapturedBlock| {
                let saved = expected_blocks
                    .next()
                    .ok_or(CapturedSourceError::EvidenceMismatch)?;
                if !saved
                    .matches_captured_block(block)
                    .map_err(|error| match error {
                        CodecError::ResourceLimit => CapturedSourceError::ResourceLimit,
                        _ => CapturedSourceError::EvidenceMismatch,
                    })?
                    || !protected_resources_agree(block, saved, &objects)
                {
                    return Err(CapturedSourceError::EvidenceMismatch);
                }
                retain_placement(block, &mut block_placements, &mut budget, parser_memory)?;
                append_placements(block, &mut placements)
            };
            let parsed = replay_one(archive, &spec, request, &mut compare);
            let (mut actual_anchors, observed) = match parsed {
                Err(ArchiveSourceError::XmlProof(
                    error @ (CapturedSourceError::InvalidXml { .. }
                    | CapturedSourceError::InvalidRoot),
                )) if spec.navigation && spec.positions.is_empty() && expected.is_none() => {
                    let (code, reason, line, column) = match error {
                        CapturedSourceError::InvalidXml { line, column } => (
                            "NavigationInvalidXml",
                            "optional navigation XHTML could not be parsed: Malformed",
                            line,
                            column,
                        ),
                        _ => (
                            "NavigationInvalidXhtmlRoot",
                            "optional navigation resource is not an XHTML document",
                            None,
                            None,
                        ),
                    };
                    budget.charge(
                        2 * (std::mem::size_of::<SourceDiagnostic>()
                            + std::mem::size_of::<chapter_map::SourceInputDiagnostic>()),
                        2,
                    )?;
                    diagnostics.push(SourceDiagnostic::InvalidOptionalNavigation {
                        canonical_path: budget.text(spec.path)?,
                    });
                    optional_navigation.push(chapter_map::SourceInputDiagnostic {
                        source: Some(chapter_map::NavigationSource::Epub3Nav),
                        resource_path: Some(budget.text(spec.path)?),
                        code: budget.text(code)?,
                        reason: budget.text(reason)?,
                        line,
                        column,
                    });
                    continue;
                }
                other => other?,
            };
            if expected.is_none() || expected_blocks.next().is_some() {
                return Err(ArchiveSourceError::EvidenceMismatch);
            }
            saved_documents.next();
            if !placements.is_empty() {
                budget.charge(128, 1)?;
                if images.insert(budget.text(spec.path)?, placements).is_some() {
                    return Err(ArchiveSourceError::EvidenceMismatch);
                }
            }
            actual_anchors
                .sort_by(|a, b| (&a.fragment, a.unit_offset).cmp(&(&b.fragment, b.unit_offset)));
            let start = self
                .anchors
                .partition_point(|a| a.canonical_path.as_str() < spec.path);
            let end = self
                .anchors
                .partition_point(|a| a.canonical_path.as_str() <= spec.path);
            if end - start != actual_anchors.len()
                || !self.anchors[start..end]
                    .iter()
                    .zip(&actual_anchors)
                    .all(|(s, a)| anchor_matches(s, a))
            {
                return Err(ArchiveSourceError::EvidenceMismatch);
            }
            anchor_count += actual_anchors.len();
            for diagnostic in observed {
                if let SourceDiagnostic::DuplicateXhtmlId { canonical_path, id } = &diagnostic {
                    budget.charge(
                        canonical_path
                            .len()
                            .checked_add(id.len())
                            .ok_or(CodecError::ResourceLimit)?,
                        1,
                    )?;
                }
                budget.charge(2 * std::mem::size_of::<SourceDiagnostic>(), 0)?;
                diagnostics.push(diagnostic);
            }
        }
        if saved_documents.next().is_some() || anchor_count != self.anchors.len() {
            return Err(ArchiveSourceError::EvidenceMismatch);
        }
        // Source navigation inputs may include NCX metadata absent in the
        // lightweight inventory. Reopen the actual original Nav/NCX bytes.
        let (navigation, mut navigation_diagnostics) = SourceExtraction::read_captured_navigation(
            archive,
            inventory,
            budget.remaining_bytes(),
        )
        .map_err(|error| match error {
            SourceExtractionError::WorkingMemoryLimit => {
                ArchiveSourceError::XmlProof(CapturedSourceError::ResourceLimit)
            }
            other => ArchiveSourceError::Navigation(other),
        })?;
        navigation_diagnostics.append(&mut optional_navigation);
        let working_bytes = crate::import_codec::memory::working(&navigation, budget.used_bytes())?;
        let working_bytes =
            crate::import_codec::memory::working(&navigation_diagnostics, working_bytes)?;
        Ok(ReplayedSource {
            working_bytes,
            navigation,
            navigation_diagnostics,
            document_diagnostics: diagnostics,
            images,
            placements: block_placements,
        })
    }
}

fn retain_placement(
    block: &CapturedBlock,
    output: &mut HashMap<SourceBlockId, CapturedBlockPlacement>,
    budget: &mut Budget,
    parser_memory: usize,
) -> Result<(), CapturedSourceError> {
    let mut bytes = 128usize
        .checked_add(std::mem::size_of::<CapturedBlockPlacement>())
        .and_then(|n| n.checked_add(block.placement.element_ordinals.len().checked_mul(4)?))
        .ok_or(CapturedSourceError::ResourceLimit)?;
    for element in &block.placement.context {
        bytes = bytes
            .checked_add(std::mem::size_of_val(element))
            .and_then(|n| n.checked_add(element.element_ordinals.len().checked_mul(4)?))
            .and_then(|n| n.checked_add(element.epub_type.as_ref().map_or(0, String::len)))
            .and_then(|n| n.checked_add(element.role.as_ref().map_or(0, String::len)))
            .ok_or(CapturedSourceError::ResourceLimit)?;
    }
    if bytes > budget.remaining_bytes().saturating_sub(parser_memory) {
        return Err(CapturedSourceError::ResourceLimit);
    }
    budget
        .charge(bytes, 1)
        .map_err(|_| CapturedSourceError::ResourceLimit)?;
    if output.insert(block.id, block.placement.clone()).is_some() {
        return Err(CapturedSourceError::EvidenceMismatch);
    }
    Ok(())
}

fn reserve_placements(
    document: Option<&StoredSourceDocument>,
    budget: &mut Budget,
) -> Result<Vec<CapturedImagePlacement>, CodecError> {
    let mut count = 0usize;
    for block in document
        .into_iter()
        .flat_map(|doc| &doc.blocks)
        .filter(|b| b.element_name != "image-page")
    {
        for run in &block.unit_runs {
            let StoredUnitRun::Protected(run) = run else {
                continue;
            };
            let ProtectedUnit::Image {
                raw_source: Some(raw),
                resource_path,
                alt,
                node_path,
                ..
            } = &run.unit
            else {
                continue;
            };
            let locator_bytes = match &block.locator {
                BlockLocator::UniqueXhtmlId { id } => id.len(),
                BlockLocator::NodePath {
                    element_ordinals, ..
                } => std::mem::size_of_val(element_ordinals.as_slice()),
            };
            for bytes in [
                locator_bytes,
                raw.len(),
                resource_path.as_ref().map_or(0, String::len),
                alt.as_ref().map_or(0, String::len),
                std::mem::size_of_val(node_path.as_slice()),
            ] {
                budget.charge(bytes, 0)?;
            }
            count = count.checked_add(1).ok_or(CodecError::ResourceLimit)?;
        }
    }
    budget.vector(count)
}

fn append_placements(
    block: &CapturedBlock,
    images: &mut Vec<CapturedImagePlacement>,
) -> Result<(), CapturedSourceError> {
    if block.element_name == "image-page" {
        return Ok(());
    }
    let mut offset = 0u32;
    for run in &block.runs {
        let width = match run {
            CapturedUnitRun::Text { unit_count, .. } => *unit_count,
            CapturedUnitRun::Protected { unit, .. } => {
                if let ProtectedSourceUnit::Image(image) = unit
                    && image.raw_source.is_some()
                {
                    if images.len() == images.capacity() {
                        return Err(CapturedSourceError::EvidenceMismatch);
                    }
                    images.push(CapturedImagePlacement {
                        block_id: block.id,
                        block_locator: block.locator.clone(),
                        unit_offset: offset,
                        image: image.clone(),
                    });
                }
                1
            }
        };
        offset = offset
            .checked_add(width)
            .ok_or(CapturedSourceError::ResourceLimit)?;
    }
    Ok(())
}

fn anchor_order(anchor: &SourceAnchor) -> (&str, &str, u32) {
    (&anchor.canonical_path, &anchor.fragment, anchor.unit_offset)
}

fn anchor_matches(saved: &SourceAnchor, original: &nt_epub::SourceAnchor) -> bool {
    saved.canonical_path == original.canonical_path
        && saved.fragment == original.fragment
        && saved.block_id.0 == *original.block_id.as_bytes()
        && saved.locator.block.matches(&original.locator.block)
        && saved.locator.element_ordinals == original.locator.element_ordinals
        && saved.unit_offset == original.unit_offset
        && saved.is_unique == original.is_unique
}

fn protected_resources_agree(
    block: &CapturedBlock,
    saved: &super::StoredSourceBlock,
    objects: &BTreeMap<&str, Digest>,
) -> bool {
    let paths = block
        .runs
        .iter()
        .filter_map(|run| {
            if let CapturedUnitRun::Protected {
                unit: ProtectedSourceUnit::Image(image),
                ..
            } = run
            {
                image.resource_path.as_deref()
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    paths.len() == saved.protected_resource_hashes.len()
        && paths
            .iter()
            .zip(&saved.protected_resource_hashes)
            .all(|(&path, hash)| {
                path == hash.canonical_path && objects.get(path) == Some(&hash.sha256)
            })
}

struct PlannedDocument<'a> {
    path: &'a str,
    positions: Vec<usize>,
    linear: Vec<usize>,
    domain: Domain,
    kind: PageKind,
    navigation: bool,
    resource_index: usize,
}

#[derive(Clone, Copy)]
enum PageKind {
    Xhtml,
    SvgImage,
    RasterImage,
}

fn document_plan<'a>(
    inventory: &'a EpubInventory,
    budget: &mut super::Budget,
) -> Result<Vec<PlannedDocument<'a>>, ArchiveSourceError> {
    if inventory.spine.is_empty() {
        return Err(ArchiveSourceError::MissingRequiredSource);
    }
    budget.charge(
        inventory
            .spine
            .len()
            .checked_mul(56)
            .ok_or(CodecError::ResourceLimit)?,
        inventory.spine.len(),
    )?;
    let mut positions = BTreeMap::<usize, (Vec<usize>, Vec<usize>, bool)>::new();
    for entry in &inventory.spine {
        let index = entry
            .resource_index
            .ok_or(ArchiveSourceError::MissingRequiredSource)?;
        let resource = inventory
            .resources
            .get(index)
            .ok_or(ArchiveSourceError::MissingRequiredSource)?;
        if !resource.available
            || resource.encryption == ResourceEncryption::Unsupported
            || !matches!(
                entry.kind,
                SpineContentKind::ReflowableDocument | SpineContentKind::ImagePage
            )
        {
            return Err(ArchiveSourceError::MissingRequiredSource);
        }
        let slot = positions.entry(index).or_default();
        slot.0.push(entry.reading_order);
        if entry.linear {
            slot.1.push(entry.reading_order);
            slot.2 |= entry.kind == SpineContentKind::ImagePage;
        }
    }
    let mut plan = budget.vector(inventory.resources.len())?;
    for (index, resource) in inventory.resources.iter().enumerate() {
        let Some(path) = resource.path.as_deref().filter(|_| resource.available) else {
            continue;
        };
        if resource.encryption == ResourceEncryption::Unsupported {
            continue;
        }
        let (positions, linear, is_image) = positions.remove(&index).unwrap_or_default();
        let kind = match resource.media_type.as_str() {
            "application/xhtml+xml" => PageKind::Xhtml,
            "image/svg+xml" if is_image => PageKind::SvgImage,
            _ if is_image => PageKind::RasterImage,
            _ => continue,
        };
        let navigation = resource.properties.iter().any(|value| value == "nav");
        let domain = if linear.is_empty() || navigation {
            Domain::Auxiliary
        } else {
            Domain::MainFlow
        };
        plan.push(PlannedDocument {
            path,
            positions,
            linear,
            domain,
            navigation,
            kind,
            resource_index: index,
        });
    }
    plan.sort_unstable_by_key(|doc| {
        (
            doc.domain != Domain::MainFlow,
            doc.linear.iter().min().copied().unwrap_or(usize::MAX),
            doc.resource_index,
        )
    });
    Ok(plan)
}

fn check_document_metadata(
    saved: &StoredSourceDocument,
    spec: &PlannedDocument<'_>,
) -> Result<(), ArchiveSourceError> {
    fn positions_match(saved: &[u64], original: &[usize]) -> bool {
        saved.len() == original.len() && saved.iter().zip(original).all(|(&s, &o)| s == o as u64)
    }
    if saved.domain != spec.domain
        || !positions_match(&saved.spine_positions, &spec.positions)
        || !positions_match(&saved.linear_spine_positions, &spec.linear)
    {
        return Err(ArchiveSourceError::EvidenceMismatch);
    }
    Ok(())
}

fn replay_one(
    archive: &mut EpubZip<impl Read + Seek>,
    spec: &PlannedDocument<'_>,
    request: CapturedSourceRequest<'_>,
    compare: impl FnMut(&CapturedBlock) -> Result<(), CapturedSourceError>,
) -> Result<(Vec<nt_epub::SourceAnchor>, Vec<SourceDiagnostic>), ArchiveSourceError> {
    let entry = archive
        .entry_by_path(spec.path)
        .map_err(ArchiveSourceError::Archive)?
        .filter(|item| !item.is_directory())
        .ok_or(ArchiveSourceError::MissingRequiredSource)?;
    let bytes = if matches!(spec.kind, PageKind::RasterImage) {
        // Its original bytes are independently checked with the resource.
        None
    } else {
        if entry.declared_size() > (request.working_bytes / 7).min(16 * 1024 * 1024) as u64 {
            return Err(CodecError::ResourceLimit.into());
        }
        Some(
            archive
                .read_entry(entry.id())
                .map_err(ArchiveSourceError::Archive)?,
        )
    };
    match spec.kind {
        PageKind::Xhtml => SourceExtraction::visit_captured_xhtml(
            bytes
                .as_deref()
                .ok_or(ArchiveSourceError::MissingRequiredSource)?,
            request,
            compare,
        )
        .map_err(ArchiveSourceError::XmlProof),
        PageKind::SvgImage | PageKind::RasterImage => SourceExtraction::visit_captured_image_page(
            bytes.as_deref(),
            request,
            *spec
                .linear
                .iter()
                .min()
                .ok_or(ArchiveSourceError::MissingRequiredSource)?,
            compare,
        )
        .map(|anchors| (anchors, Vec::new()))
        .map_err(ArchiveSourceError::XmlProof),
    }
}

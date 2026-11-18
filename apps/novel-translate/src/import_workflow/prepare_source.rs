use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};
use std::mem::size_of;

use nt_application::volume_import::{
    CapturedBlock, CapturedBlockPlacement, CapturedSourceError, CapturedSourceRequest,
    CapturedUnitRun, MAX_IMPORT_WORKING_BYTES,
    source_range::{BlockLocator, SourceBlockId, SourceSpan, SourceUnit},
};
use nt_epub::{EpubInventory, EpubZip, PARSER_VERSION, ResourceEncryption, SourceExtraction};

use super::ImportFlowError;
use super::eligibility::BlockingSource;

pub(super) struct SourceReservation {
    pub placements: HashMap<SourceBlockId, CapturedBlockPlacement>,
    pub source_bytes: usize,
    metadata_bytes: usize,
    archive_peak: usize,
}

impl SourceReservation {
    /// Read text with the compact parser before allowing the legacy eager
    /// extractor to allocate its separate grapheme and span vectors.
    pub fn measure(
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        epub_sha256: [u8; 32],
        retained_inputs: usize,
    ) -> Result<Self, ImportFlowError> {
        let mut result = Self {
            placements: HashMap::new(),
            source_bytes: 0,
            metadata_bytes: crate::import_codec::memory::working(inventory, retained_inputs)?,
            archive_peak: 0,
        };
        let spine_paths: HashSet<_> = inventory
            .spine
            .iter()
            .filter_map(|entry| entry.resource_index)
            .collect();
        for (index, resource) in inventory.resources.iter().enumerate() {
            let Some(path) = resource.path.as_deref().filter(|_| resource.available) else {
                continue;
            };
            if resource.encryption == ResourceEncryption::Unsupported {
                continue;
            }
            let entry = archive
                .entry_by_path(path)
                .map_err(ImportFlowError::Archive)?
                .ok_or_else(|| {
                    ImportFlowError::Blocked(BlockingSource::Resource {
                        canonical_path: path.to_owned(),
                    })
                })?;
            let expanded = usize::try_from(entry.declared_size())
                .map_err(|_| CapturedSourceError::ResourceLimit)?;
            result.archive_peak = result.archive_peak.max(expanded);
            result.remaining()?;
            if resource.media_type != "application/xhtml+xml" {
                continue;
            }
            let working_bytes = result.remaining()?;
            if expanded > working_bytes / 7 {
                return Err(CapturedSourceError::ResourceLimit.into());
            }
            let bytes = archive
                .read_entry(entry.id())
                .map_err(ImportFlowError::Archive)?;
            let request = CapturedSourceRequest {
                canonical_path: path,
                epub_sha256,
                parser_version: PARSER_VERSION,
                is_navigation_document: resource
                    .properties
                    .iter()
                    .any(|property| property == "nav"),
                working_bytes,
            };
            match SourceExtraction::visit_captured_xhtml(&bytes, request, |block| {
                result.measure_block(block, path.len())
            }) {
                Ok((anchors, diagnostics)) => {
                    let anchors_bytes = anchors.iter().try_fold(0usize, |total, anchor| {
                        let bytes = size_of::<nt_epub::SourceAnchor>()
                            + locator_bytes(&anchor.locator.block)
                            + anchor.locator.element_ordinals.len() * size_of::<u32>()
                            + anchor.fragment.len()
                            + anchor.canonical_path.len();
                        checked_add(total, 2 * bytes)
                    })?;
                    let diag_bytes = checked_product(diagnostics.len(), path.len() + 256)?;
                    result.source_bytes =
                        checked_add(result.source_bytes, checked_add(anchors_bytes, diag_bytes)?)?;
                    result.remaining()?;
                }
                Err(CapturedSourceError::InvalidXml { .. } | CapturedSourceError::InvalidRoot)
                    if resource.properties.iter().any(|property| property == "nav")
                        && !spine_paths.contains(&index) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(result)
    }

    fn measure_block(
        &mut self,
        block: &CapturedBlock,
        path_bytes: usize,
    ) -> Result<(), CapturedSourceError> {
        let locator = locator_bytes(&block.locator);
        let mut source = 1024 + path_bytes + locator;
        let units = block.unit_count as usize;
        // Eager vectors may grow geometrically. Most spans occupy the first
        // four slots, with an extra span per lexical projection boundary.
        let unit_slot = 2 * size_of::<SourceUnit>() + 4 * size_of::<SourceSpan>() + locator + 32;
        source = checked_add(source, checked_product(units, unit_slot)?)?;
        for run in &block.runs {
            let extra = match run {
                CapturedUnitRun::Text { text, segments, .. } => checked_add(
                    text.len(),
                    checked_product(segments.len(), 4 * size_of::<SourceSpan>() + locator)?,
                )?,
                CapturedUnitRun::Protected { unit, spans } => {
                    use nt_application::volume_import::source_range::ProtectedSourceUnit::*;
                    let text = match unit {
                        Ruby(ruby) => {
                            ruby.base.len()
                                + ruby.annotation.len()
                                + (ruby.base_spans.len() + ruby.annotation_spans.len())
                                    * (size_of::<SourceSpan>() + locator)
                        }
                        Image(image) => {
                            image.raw_source.as_ref().map_or(0, String::len)
                                + image.resource_path.as_ref().map_or(0, String::len)
                                + image.alt.as_ref().map_or(0, String::len)
                        }
                        MarkerPair(pair) => {
                            let endpoint_bytes = |endpoint: &nt_application::volume_import::source_range::MarkerEndpoint| {
                                endpoint.node_path.len() * size_of::<u32>()
                                    + endpoint.attributes.iter().map(|(name, value)| {
                                        2 * size_of::<String>() + name.len() + value.len()
                                    }).sum::<usize>()
                            };
                            pair.kind.len()
                                + endpoint_bytes(&pair.start)
                                + endpoint_bytes(&pair.end)
                        }
                        LineBreak { element_path } | SceneBreak { element_path } => {
                            element_path.len() * size_of::<u32>()
                        }
                    };
                    checked_add(
                        text,
                        checked_product(spans.len(), size_of::<SourceSpan>() + locator)?,
                    )?
                }
            };
            source = checked_add(source, extra)?;
        }
        for metadata in &block.inline_metadata {
            let links = metadata.ranges.iter().try_fold(0usize, |total, range| {
                checked_add(total, (range.end - range.start) as usize)
            })?;
            source = checked_add(source, checked_product(links, 2 * size_of::<usize>())?)?;
            source = checked_add(source, locator_bytes(&metadata.locator.block) + 512)?;
        }
        for attribute in &block.attribute_text {
            source = checked_add(
                source,
                attribute.value.len()
                    + attribute.resource_path.as_ref().map_or(0, String::len)
                    + locator_bytes(&attribute.locator.block)
                    + 256,
            )?;
        }
        self.source_bytes = checked_add(self.source_bytes, source)?;

        let placement = &block.placement;
        let working = placement.context.iter().try_fold(
            256 + placement.element_ordinals.len() * size_of::<u32>(),
            |total, element| {
                checked_add(
                    total,
                    2 * size_of::<nt_application::volume_import::CapturedScopeElement>()
                        + element.element_ordinals.len() * size_of::<u32>()
                        + element.epub_type.as_ref().map_or(0, String::len)
                        + element.role.as_ref().map_or(0, String::len),
                )
            },
        )?;
        self.metadata_bytes = checked_add(self.metadata_bytes, working)?;
        self.remaining()?;
        self.placements
            .try_reserve(1)
            .map_err(|_| CapturedSourceError::ResourceLimit)?;
        if self
            .placements
            .insert(block.id, placement.clone())
            .is_some()
        {
            return Err(CapturedSourceError::InvalidSource);
        }
        Ok(())
    }

    pub fn retained_bytes(&self) -> Result<usize, CapturedSourceError> {
        checked_add(self.source_bytes, self.metadata_bytes)
    }

    pub fn remaining(&self) -> Result<usize, CapturedSourceError> {
        let reserved = checked_add(self.retained_bytes()?, self.archive_peak)?;
        MAX_IMPORT_WORKING_BYTES
            .checked_sub(reserved)
            .ok_or(CapturedSourceError::ResourceLimit)
    }
}

fn locator_bytes(locator: &BlockLocator) -> usize {
    match locator {
        BlockLocator::UniqueXhtmlId(id) => id.len(),
        BlockLocator::NodePath {
            element_ordinals, ..
        } => element_ordinals.len() * size_of::<u32>(),
    }
}

fn checked_add(left: usize, right: usize) -> Result<usize, CapturedSourceError> {
    left.checked_add(right)
        .ok_or(CapturedSourceError::ResourceLimit)
}

fn checked_product(count: usize, bytes: usize) -> Result<usize, CapturedSourceError> {
    count
        .checked_mul(bytes)
        .ok_or(CapturedSourceError::ResourceLimit)
}

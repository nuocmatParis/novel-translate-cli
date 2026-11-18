//! Lossless source records. Decoding is structural validation only; the volume
//! reader must verify these records against the immutable original EPUB before
//! exposing a trusted snapshot. No SourceSpan is created from stored offsets.
use nt_application::volume_import::source_range as source;
use nt_epub::{CoverageDomain, SourceExtraction};
use serde::{Deserialize, Deserializer, Serialize, de};

use super::records::{
    BlockLocator, Budget, Digest, ElementLocator, Span, UnitRange, hash_bytes, safe_path,
};
use super::{CodecError, TextRun};

mod archive_proof;
mod original;
pub use archive_proof::{ArchiveSourceError, ReplayedSource};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributePair {
    pub name: String,
    pub value: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerEndpoint {
    pub node_path: Vec<u32>,
    pub attributes: Vec<AttributePair>,
}
impl MarkerEndpoint {
    fn capture(value: &source::MarkerEndpoint, budget: &mut Budget) -> Result<Self, CodecError> {
        let mut attributes = budget.vector(value.attributes.len())?;
        for (name, value) in &value.attributes {
            attributes.push(AttributePair {
                name: budget.text(name)?,
                value: budget.text(value)?,
            });
        }
        Ok(Self {
            node_path: budget.ordinals(&value.node_path)?,
            attributes,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum ProtectedUnit {
    Ruby {
        base: String,
        annotation: String,
        base_spans: Vec<Span>,
        annotation_spans: Vec<Span>,
        node_path: Vec<u32>,
    },
    Image {
        raw_source: Option<String>,
        resource_path: Option<String>,
        alt: Option<String>,
        occurrence: u64,
        node_path: Vec<u32>,
        external: bool,
    },
    MarkerPair {
        kind: String,
        start: MarkerEndpoint,
        end: MarkerEndpoint,
    },
    LineBreak {
        element_path: Vec<u32>,
    },
    SceneBreak {
        element_path: Vec<u32>,
    },
}
impl ProtectedUnit {
    fn capture(
        value: &source::ProtectedSourceUnit,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        use source::ProtectedSourceUnit as P;
        Ok(match value {
            P::Ruby(r) => Self::Ruby {
                base: budget.text(&r.base)?,
                annotation: budget.text(&r.annotation)?,
                base_spans: Span::list(&r.base_spans, budget)?,
                annotation_spans: Span::list(&r.annotation_spans, budget)?,
                node_path: budget.ordinals(&r.node_path)?,
            },
            P::Image(i) => Self::Image {
                raw_source: budget.optional_text(&i.raw_source)?,
                resource_path: budget.optional_text(&i.resource_path)?,
                alt: budget.optional_text(&i.alt)?,
                occurrence: u64::try_from(i.occurrence).map_err(|_| CodecError::ResourceLimit)?,
                node_path: budget.ordinals(&i.node_path)?,
                external: i.external,
            },
            P::MarkerPair(m) => Self::MarkerPair {
                kind: budget.text(&m.kind)?,
                start: MarkerEndpoint::capture(&m.start, budget)?,
                end: MarkerEndpoint::capture(&m.end, budget)?,
            },
            P::LineBreak { element_path } => Self::LineBreak {
                element_path: budget.ordinals(element_path)?,
            },
            P::SceneBreak { element_path } => Self::SceneBreak {
                element_path: budget.ordinals(element_path)?,
            },
        })
    }
    fn validate(&self, block: &BlockLocator) -> Result<(), CodecError> {
        match self {
            Self::Ruby {
                base,
                annotation,
                base_spans,
                annotation_spans,
                node_path,
            } => {
                if node_path.len() > 256
                    || base_spans
                        .iter()
                        .chain(annotation_spans)
                        .any(|s| &s.node.block != block || s.start_byte >= s.end_byte)
                {
                    return Err(CodecError::InvalidRecord);
                }
                for (text, spans) in [(base, base_spans), (annotation, annotation_spans)] {
                    let length = spans
                        .iter()
                        .try_fold(0usize, |sum, s| {
                            sum.checked_add((s.end_byte - s.start_byte) as usize)
                        })
                        .ok_or(CodecError::ResourceLimit)?;
                    if text.len() != length {
                        return Err(CodecError::InvalidRecord);
                    }
                }
            }
            Self::Image {
                resource_path,
                node_path,
                external,
                ..
            } => {
                if node_path.len() > 256
                    || resource_path.as_deref().is_some_and(|p| !safe_path(p))
                    || (*external && resource_path.is_some())
                {
                    return Err(CodecError::InvalidRecord);
                }
            }
            Self::MarkerPair { start, end, .. } => {
                if start.node_path.len() > 256 || end.node_path.len() > 256 {
                    return Err(CodecError::InvalidRecord);
                }
            }
            Self::LineBreak { element_path } | Self::SceneBreak { element_path } => {
                if element_path.len() > 256 {
                    return Err(CodecError::InvalidRecord);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProtectedTag {
    Protected,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectedRun {
    pub kind: ProtectedTag,
    pub unit: ProtectedUnit,
    pub spans: Vec<Span>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StoredUnitRun {
    Text(TextRun),
    Protected(ProtectedRun),
}

impl<'de> Deserialize<'de> for StoredUnitRun {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        enum Kind {
            TextRun,
            Protected,
        }

        fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
            deserializer: D,
        ) -> Result<Option<T>, D::Error> {
            T::deserialize(deserializer).map(Some)
        }

        // Deserialize the existing field shape directly. Untagged serde
        // dispatch would first retain every key/value in a generic tree,
        // then allocate the typed block on top of that tree.
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RunFields {
            kind: Kind,
            #[serde(default, deserialize_with = "present")]
            text: Option<String>,
            #[serde(default, deserialize_with = "present")]
            grapheme_lengths: Option<String>,
            #[serde(default, deserialize_with = "present")]
            segments: Option<Vec<[u32; 5]>>,
            #[serde(default, deserialize_with = "present")]
            unit: Option<ProtectedUnit>,
            #[serde(default, deserialize_with = "present")]
            spans: Option<Vec<Span>>,
        }

        match RunFields::deserialize(deserializer)? {
            RunFields {
                kind: Kind::TextRun,
                text: Some(text),
                grapheme_lengths: Some(lengths),
                segments: Some(segments),
                unit: None,
                spans: None,
            } => Ok(Self::Text(TextRun::from_wire_parts(
                text, lengths, segments,
            ))),
            RunFields {
                kind: Kind::Protected,
                text: None,
                grapheme_lengths: None,
                segments: None,
                unit: Some(unit),
                spans: Some(spans),
            } => Ok(Self::Protected(ProtectedRun {
                kind: ProtectedTag::Protected,
                unit,
                spans,
            })),
            _ => Err(de::Error::custom("Invalid source run fields.")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum InlineKind {
    Formatting { element: String },
    Link { href: Option<String> },
    NoteReference { href: Option<String> },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InlineMetadata {
    pub locator: ElementLocator,
    pub kind: InlineKind,
    pub ranges: Vec<UnitRange>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeText {
    pub locator: ElementLocator,
    pub attribute: String,
    pub value: String,
    pub resource_path: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectedResourceHash {
    pub canonical_path: String,
    pub sha256: Digest,
}

type BlockMetadata = (
    Vec<InlineMetadata>,
    Vec<AttributeText>,
    Vec<ProtectedResourceHash>,
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredSourceBlock {
    pub id: Digest,
    pub canonical_path: String,
    pub element_name: String,
    pub locator: BlockLocator,
    pub parser_version: String,
    pub content_hash: Digest,
    pub unit_runs: Vec<StoredUnitRun>,
    pub inline_metadata: Vec<InlineMetadata>,
    pub attribute_text: Vec<AttributeText>,
    pub protected_resource_hashes: Vec<ProtectedResourceHash>,
}
impl StoredSourceBlock {
    fn capture(value: &source::SourceBlock, budget: &mut Budget) -> Result<Self, CodecError> {
        // Plan run count without allocating a record per grapheme. Split only
        // on a protected unit or an actual grapheme boundary at the byte caps.
        fn chunks(
            units: &[source::SourceUnit],
            mut visit: impl FnMut(&[source::SourceUnit]) -> Result<(), CodecError>,
        ) -> Result<(), CodecError> {
            let (mut start, mut text, mut lengths) = (0, 0usize, 0usize);
            for (index, unit) in units.iter().enumerate() {
                match &unit.kind {
                    source::SourceUnitKind::Grapheme(value) => {
                        let n = value.len();
                        let varint = if n == 0 {
                            return Err(CodecError::InvalidRecord);
                        } else {
                            (usize::BITS - n.leading_zeros()).div_ceil(7) as usize
                        };
                        if n > 16 * 1024 * 1024 {
                            return Err(CodecError::ResourceLimit);
                        }
                        if text.checked_add(n).is_none_or(|n| n > 16 * 1024 * 1024)
                            || lengths
                                .checked_add(varint)
                                .is_none_or(|n| n > 16 * 1024 * 1024)
                        {
                            visit(&units[start..index])?;
                            start = index;
                            text = 0;
                            lengths = 0;
                        }
                        text += n;
                        lengths += varint;
                    }
                    source::SourceUnitKind::Protected(_) => {
                        if start < index {
                            visit(&units[start..index])?;
                        }
                        visit(&units[index..index + 1])?;
                        start = index + 1;
                        text = 0;
                        lengths = 0;
                    }
                }
            }
            if start < units.len() {
                visit(&units[start..])?;
            }
            Ok(())
        }
        let mut count = 0usize;
        chunks(&value.units, |_| {
            count = count.checked_add(1).ok_or(CodecError::ResourceLimit)?;
            Ok(())
        })?;
        let mut runs = budget.vector(count)?;
        chunks(&value.units, |chunk| {
            match &chunk[0].kind {
                source::SourceUnitKind::Grapheme(_) => {
                    let (_, _, segments, working) = TextRun::capture_plan(chunk, &value.locator)?;
                    budget.charge(working, segments)?;
                    runs.push(StoredUnitRun::Text(TextRun::from_grapheme_units(
                        chunk,
                        &value.locator,
                    )?));
                }
                source::SourceUnitKind::Protected(unit) => {
                    runs.push(StoredUnitRun::Protected(ProtectedRun {
                        kind: ProtectedTag::Protected,
                        unit: ProtectedUnit::capture(unit, budget)?,
                        spans: Span::list(&chunk[0].spans, budget)?,
                    }))
                }
            }
            Ok(())
        })?;
        let (inline_metadata, attribute_text, hashes) = Self::capture_metadata(value, budget)?;
        Ok(Self {
            id: Digest(*value.id.as_bytes()),
            canonical_path: budget.text(&value.canonical_path)?,
            element_name: budget.text(&value.element_name)?,
            locator: BlockLocator::capture(&value.locator, budget)?,
            parser_version: budget.text(&value.parser_version)?,
            content_hash: Digest(value.content_hash),
            unit_runs: runs,
            inline_metadata,
            attribute_text,
            protected_resource_hashes: hashes,
        })
    }

    fn capture_metadata(
        value: &source::SourceBlock,
        budget: &mut Budget,
    ) -> Result<BlockMetadata, CodecError> {
        let mut inline_metadata = budget.vector(value.inline_metadata.len())?;
        for m in &value.inline_metadata {
            let kind = match &m.kind {
                source::InlineMetadataKind::Formatting { element } => InlineKind::Formatting {
                    element: budget.text(element)?,
                },
                source::InlineMetadataKind::Link { href } => InlineKind::Link {
                    href: budget.optional_text(href)?,
                },
                source::InlineMetadataKind::NoteReference { href } => InlineKind::NoteReference {
                    href: budget.optional_text(href)?,
                },
            };
            let mut ranges = budget.vector(m.ranges.len())?;
            for r in &m.ranges {
                ranges.push(UnitRange {
                    start: r.start,
                    end: r.end,
                });
            }
            inline_metadata.push(InlineMetadata {
                locator: ElementLocator::capture(&m.locator, budget)?,
                kind,
                ranges,
            });
        }
        let mut attribute_text = budget.vector(value.attribute_text.len())?;
        for a in &value.attribute_text {
            attribute_text.push(AttributeText {
                locator: ElementLocator::capture(&a.locator, budget)?,
                attribute: budget.text(&a.attribute)?,
                value: budget.text(&a.value)?,
                resource_path: budget.optional_text(&a.resource_path)?,
            });
        }
        let mut hashes = budget.vector(value.protected_resource_hashes.len())?;
        for h in &value.protected_resource_hashes {
            hashes.push(ProtectedResourceHash {
                canonical_path: budget.text(&h.canonical_path)?,
                sha256: Digest(h.sha256),
            });
        }
        Ok((inline_metadata, attribute_text, hashes))
    }

    pub fn unit_count(&self) -> Result<u32, CodecError> {
        self.unit_runs.iter().try_fold(0u32, |count, run| {
            let n = match run {
                StoredUnitRun::Text(text) => text.validate()?.unit_count(),
                StoredUnitRun::Protected(_) => 1,
            };
            count.checked_add(n).ok_or(CodecError::ResourceLimit)
        })
    }

    fn validate(&self, epub: Digest, parser: &str) -> Result<(), CodecError> {
        if !safe_path(&self.canonical_path)
            || self.parser_version != parser
            || parser.is_empty()
            || parser.len() > 16 * 1024
            || self.element_name.is_empty()
        {
            return Err(CodecError::InvalidRecord);
        }
        let identity = source::BlockIdentityRecord {
            epub_sha256: epub.0,
            parser_version: parser.to_owned(),
            canonical_path: self.canonical_path.clone(),
            locator: self.locator.source_locator()?,
        };
        if hash_bytes(
            &identity
                .canonical_bytes()
                .map_err(|_| CodecError::InvalidRecord)?,
        )? != self.id
        {
            return Err(CodecError::InvalidRecord);
        }
        let count = self.unit_count()?;
        for run in &self.unit_runs {
            if let StoredUnitRun::Protected(p) = run {
                p.unit.validate(&self.locator)?;
                if p.spans
                    .iter()
                    .any(|s| s.node.block != self.locator || s.start_byte >= s.end_byte)
                {
                    return Err(CodecError::InvalidRecord);
                }
            }
        }
        for m in &self.inline_metadata {
            if m.locator.element_ordinals.len() > 256
                || m.ranges.iter().any(|r| r.start > r.end || r.end > count)
            {
                return Err(CodecError::InvalidRecord);
            }
        }
        for a in &self.attribute_text {
            if a.locator.element_ordinals.len() > 256
                || a.resource_path
                    .as_deref()
                    .is_some_and(|path| !safe_path(path))
            {
                return Err(CodecError::InvalidRecord);
            }
        }
        if self
            .protected_resource_hashes
            .iter()
            .any(|h| !safe_path(&h.canonical_path))
        {
            return Err(CodecError::InvalidRecord);
        }
        Ok(())
    }

    /// Compare to an already verified producer output without allocating units
    /// or manufacturing spans. This does not replace the streaming EPUB reader.
    pub fn matches_extracted(&self, value: &source::SourceBlock) -> Result<bool, CodecError> {
        if self.id.0 != *value.id.as_bytes()
            || self.canonical_path != value.canonical_path
            || self.element_name != value.element_name
            || !self.locator.matches(&value.locator)
            || self.parser_version != value.parser_version
            || self.content_hash.0 != value.content_hash
        {
            return Ok(false);
        }
        let mut expected = value.units.iter();
        for run in &self.unit_runs {
            match run {
                StoredUnitRun::Text(text) => {
                    for unit in text.validate()?.units() {
                        let Some(original) = expected.next() else {
                            return Ok(false);
                        };
                        if !matches!(&original.kind,source::SourceUnitKind::Grapheme(text) if text==unit.text)
                        {
                            return Ok(false);
                        }
                        let mut spans = unit.spans();
                        for original in &original.spans {
                            let Some(span) = spans.next() else {
                                return Ok(false);
                            };
                            if span.text_node_order != original.node().text_node_order
                                || span.start_byte != original.start_byte()
                                || span.end_byte != original.end_byte()
                                || !self.locator.matches(&original.node().block)
                            {
                                return Ok(false);
                            }
                        }
                        if spans.next().is_some() {
                            return Ok(false);
                        }
                    }
                }
                StoredUnitRun::Protected(protected) => {
                    let Some(original) = expected.next() else {
                        return Ok(false);
                    };
                    let source::SourceUnitKind::Protected(unit) = &original.kind else {
                        return Ok(false);
                    };
                    let mut budget = Budget::new(0)?;
                    if protected.unit != ProtectedUnit::capture(unit, &mut budget)?
                        || protected.spans.len() != original.spans.len()
                        || !protected
                            .spans
                            .iter()
                            .zip(&original.spans)
                            .all(|(s, o)| s.matches(o))
                    {
                        return Ok(false);
                    }
                }
            }
        }
        if expected.next().is_some() {
            return Ok(false);
        }
        // Metadata is structurally independent of unit compression. Compare its
        // exact closed records, including locators that name an outer ancestor.
        let mut budget = Budget::new(0)?;
        let (inline_metadata, attribute_text, hashes) = Self::capture_metadata(value, &mut budget)?;
        Ok(self.inline_metadata == inline_metadata
            && self.attribute_text == attribute_text
            && self.protected_resource_hashes == hashes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Domain {
    MainFlow,
    Auxiliary,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredSourceDocument {
    pub canonical_path: String,
    pub domain: Domain,
    pub spine_positions: Vec<u64>,
    pub linear_spine_positions: Vec<u64>,
    pub blocks: Vec<StoredSourceBlock>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAnchor {
    pub canonical_path: String,
    pub fragment: String,
    pub locator: ElementLocator,
    pub block_id: Digest,
    pub unit_offset: u32,
    pub is_unique: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredSourceIndex {
    pub documents: Vec<StoredSourceDocument>,
    pub anchors: Vec<SourceAnchor>,
}
impl StoredSourceIndex {
    /// Capture source fields using one caller-supplied reservation for all live
    /// external allocations. The final envelope shares this budget with peers.
    pub fn capture(
        value: &SourceExtraction,
        reserved_working_bytes: usize,
    ) -> Result<Self, CodecError> {
        Self::capture_with_budget(value, &mut Budget::new(reserved_working_bytes)?)
    }
    pub(super) fn capture_with_budget(
        value: &SourceExtraction,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        let mut documents = budget.vector(value.documents.len())?;
        for d in &value.documents {
            let mut blocks = budget.vector(d.blocks.len())?;
            for block in &d.blocks {
                blocks.push(StoredSourceBlock::capture(block, budget)?);
            }
            let mut positions = budget.vector(d.spine_positions.len())?;
            for &p in &d.spine_positions {
                positions.push(u64::try_from(p).map_err(|_| CodecError::ResourceLimit)?);
            }
            let mut linear = budget.vector(d.linear_spine_positions.len())?;
            for &p in &d.linear_spine_positions {
                linear.push(u64::try_from(p).map_err(|_| CodecError::ResourceLimit)?);
            }
            documents.push(StoredSourceDocument {
                canonical_path: budget.text(&d.canonical_path)?,
                domain: match d.domain {
                    CoverageDomain::MainFlow => Domain::MainFlow,
                    CoverageDomain::Auxiliary => Domain::Auxiliary,
                },
                spine_positions: positions,
                linear_spine_positions: linear,
                blocks,
            });
        }
        let mut anchors = budget.vector(value.anchors.len())?;
        for a in &value.anchors {
            anchors.push(SourceAnchor {
                canonical_path: budget.text(&a.canonical_path)?,
                fragment: budget.text(&a.fragment)?,
                locator: ElementLocator::capture(&a.locator, budget)?,
                block_id: Digest(*a.block_id.as_bytes()),
                unit_offset: a.unit_offset,
                is_unique: a.is_unique,
            });
        }
        let result = Self { documents, anchors };
        let blocks = result
            .documents
            .iter()
            .try_fold(0usize, |n, d| n.checked_add(d.blocks.len()))
            .ok_or(CodecError::ResourceLimit)?;
        budget.charge(
            blocks
                .checked_mul(96)
                .and_then(|n| {
                    result
                        .documents
                        .len()
                        .checked_mul(64)
                        .and_then(|m| n.checked_add(m))
                })
                .ok_or(CodecError::ResourceLimit)?,
            0,
        )?;
        result.validate_structure(Digest(value.epub_sha256), &value.parser_version)?;
        Ok(result)
    }
    pub fn validate_structure(&self, epub: Digest, parser: &str) -> Result<(), CodecError> {
        // These bounded sets are additional temporary allocations accounted by
        // the enclosing decode preflight and reserved encoder working budget.
        let mut paths = std::collections::HashSet::new();
        let mut blocks = std::collections::HashMap::new();
        for d in &self.documents {
            if !safe_path(&d.canonical_path)
                || !paths.insert(d.canonical_path.as_str())
                || d.spine_positions.windows(2).any(|w| w[0] >= w[1])
                || d.linear_spine_positions
                    .iter()
                    .any(|p| !d.spine_positions.contains(p))
            {
                return Err(CodecError::InvalidRecord);
            }
            for block in &d.blocks {
                block.validate(epub, parser)?;
                if block.canonical_path != d.canonical_path
                    || blocks
                        .insert(block.id, (d.canonical_path.as_str(), block.unit_count()?))
                        .is_some()
                {
                    return Err(CodecError::InvalidRecord);
                }
            }
        }
        for anchor in &self.anchors {
            let Some((path, count)) = blocks.get(&anchor.block_id) else {
                return Err(CodecError::InvalidRecord);
            };
            if *path != anchor.canonical_path
                || anchor.unit_offset > *count
                || anchor.locator.element_ordinals.len() > 256
            {
                return Err(CodecError::InvalidRecord);
            }
        }
        Ok(())
    }
}

// Retained allocation accounting.
crate::import_codec::memory::heap_struct!(AttributePair { name, value });
crate::import_codec::memory::heap_struct!(MarkerEndpoint {
    node_path,
    attributes
});
crate::import_codec::memory::heap_enum!(ProtectedUnit {Ruby {base, annotation, base_spans, annotation_spans, node_path}, Image {raw_source, resource_path, alt, occurrence, node_path, external}, MarkerPair {kind, start, end}, LineBreak {element_path}, SceneBreak {element_path}});
crate::import_codec::memory::heap_enum!(ProtectedTag { Protected });
crate::import_codec::memory::heap_struct!(ProtectedRun { kind, unit, spans });
crate::import_codec::memory::heap_enum!(StoredUnitRun {Text(value), Protected(value)});
crate::import_codec::memory::heap_enum!(InlineKind {Formatting {element}, Link {href}, NoteReference {href}});
crate::import_codec::memory::heap_struct!(InlineMetadata {
    locator,
    kind,
    ranges
});
crate::import_codec::memory::heap_struct!(AttributeText {
    locator,
    attribute,
    value,
    resource_path
});
crate::import_codec::memory::heap_struct!(ProtectedResourceHash {
    canonical_path,
    sha256
});
crate::import_codec::memory::heap_struct!(StoredSourceBlock {
    id,
    canonical_path,
    element_name,
    locator,
    parser_version,
    content_hash,
    unit_runs,
    inline_metadata,
    attribute_text,
    protected_resource_hashes
});
crate::import_codec::memory::heap_enum!(Domain {
    MainFlow,
    Auxiliary
});
crate::import_codec::memory::heap_struct!(StoredSourceDocument {
    canonical_path,
    domain,
    spine_positions,
    linear_spine_positions,
    blocks
});
crate::import_codec::memory::heap_struct!(SourceAnchor {
    canonical_path,
    fragment,
    locator,
    block_id,
    unit_offset,
    is_unique
});
crate::import_codec::memory::heap_struct!(StoredSourceIndex { documents, anchors });

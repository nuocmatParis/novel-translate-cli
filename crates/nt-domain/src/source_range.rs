//! Stable source identities and grapheme/protected-unit ranges.

use std::fmt;

/// SHA-256 identity of one source block in an EPUB snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceBlockId([u8; 32]);

impl SourceBlockId {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(64);
        for byte in self.0 {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }
}

impl fmt::Display for SourceBlockId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

/// Stable locator for one element that owns a source block.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BlockLocator {
    UniqueXhtmlId(String),
    NodePath {
        element_ordinals: Vec<u32>,
        order: u32,
    },
}

/// Versioned byte record used as the input to SHA-256 for a source block ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockIdentityRecord {
    pub epub_sha256: [u8; 32],
    pub parser_version: String,
    pub canonical_path: String,
    pub locator: BlockLocator,
}

impl BlockIdentityRecord {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, InvalidIdentityRecord> {
        let parser = self.parser_version.as_bytes();
        let path = self.canonical_path.as_bytes();
        if parser.is_empty() || path.is_empty() {
            return Err(InvalidIdentityRecord);
        }
        let mut output = Vec::with_capacity(64 + parser.len() + path.len());
        output.extend_from_slice(b"NT-BLOCK-ID\0");
        output.extend_from_slice(&1_u16.to_be_bytes());
        output.extend_from_slice(&self.epub_sha256);
        append_bytes(&mut output, parser)?;
        append_bytes(&mut output, path)?;
        match &self.locator {
            BlockLocator::UniqueXhtmlId(id) if !id.is_empty() => {
                output.push(0x01);
                append_bytes(&mut output, id.as_bytes())?;
            }
            BlockLocator::UniqueXhtmlId(_) => return Err(InvalidIdentityRecord),
            BlockLocator::NodePath {
                element_ordinals,
                order,
            } => {
                output.push(0x02);
                let steps =
                    u32::try_from(element_ordinals.len()).map_err(|_| InvalidIdentityRecord)?;
                output.extend_from_slice(&steps.to_be_bytes());
                for ordinal in element_ordinals {
                    output.extend_from_slice(&ordinal.to_be_bytes());
                }
                output.extend_from_slice(&order.to_be_bytes());
            }
        }
        Ok(output)
    }
}

fn append_bytes(output: &mut Vec<u8>, value: &[u8]) -> Result<(), InvalidIdentityRecord> {
    let length = u32::try_from(value.len()).map_err(|_| InvalidIdentityRecord)?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidIdentityRecord;

impl fmt::Display for InvalidIdentityRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("source identity record contains an empty or oversized field")
    }
}

impl std::error::Error for InvalidIdentityRecord {}

/// Locator for one decoded XML text node inside a block.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextNodeLocator {
    pub block: BlockLocator,
    pub text_node_order: u32,
}

/// Locator for a source element relative to the structural element in `block`.
///
/// An element outside the current block's subtree uses its actual structural
/// ancestor as the base; that ancestor need not emit a source block. The
/// document-element base uses an empty node path with order zero.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ElementLocator {
    pub block: BlockLocator,
    pub element_ordinals: Vec<u32>,
}

/// Half-open byte span in one decoded UTF-8 XML text node.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SourceSpan {
    node: TextNodeLocator,
    start_byte: u32,
    end_byte: u32,
}

impl SourceSpan {
    pub fn new(
        node: TextNodeLocator,
        start_byte: usize,
        end_byte: usize,
        text_node: &str,
    ) -> Result<Self, InvalidSourceSpan> {
        if start_byte >= end_byte
            || end_byte > text_node.len()
            || !text_node.is_char_boundary(start_byte)
            || !text_node.is_char_boundary(end_byte)
            || start_byte > u32::MAX as usize
            || end_byte > u32::MAX as usize
        {
            return Err(InvalidSourceSpan);
        }
        Ok(Self {
            node,
            start_byte: start_byte as u32,
            end_byte: end_byte as u32,
        })
    }

    pub fn node(&self) -> &TextNodeLocator {
        &self.node
    }

    pub fn start_byte(&self) -> u32 {
        self.start_byte
    }

    pub fn end_byte(&self) -> u32 {
        self.end_byte
    }

    /// Return the same validated byte span attached to another block segment.
    pub fn with_block_locator(&self, block: BlockLocator) -> Self {
        let mut span = self.clone();
        span.node.block = block;
        span
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSourceSpan;

impl fmt::Display for InvalidSourceSpan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("source span is empty, out of bounds, or not on UTF-8 boundaries")
    }
}

impl std::error::Error for InvalidSourceSpan {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RubySource {
    pub base: String,
    pub annotation: String,
    pub base_spans: Vec<SourceSpan>,
    pub annotation_spans: Vec<SourceSpan>,
    pub node_path: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageSource {
    pub raw_source: Option<String>,
    pub resource_path: Option<String>,
    pub alt: Option<String>,
    pub occurrence: usize,
    pub node_path: Vec<u32>,
    pub external: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkerEndpoint {
    pub node_path: Vec<u32>,
    pub attributes: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkerPairSource {
    pub kind: String,
    pub start: MarkerEndpoint,
    pub end: MarkerEndpoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtectedSourceUnit {
    Ruby(RubySource),
    Image(ImageSource),
    MarkerPair(MarkerPairSource),
    LineBreak { element_path: Vec<u32> },
    SceneBreak { element_path: Vec<u32> },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnitRange {
    pub start: u32,
    pub end: u32,
}

impl UnitRange {
    /// Create a half-open metadata range. Empty ranges are valid for empty links.
    pub fn new(start: usize, end: usize, unit_count: usize) -> Result<Self, InvalidUnitRange> {
        if start > end || end > unit_count || end > u32::MAX as usize {
            return Err(InvalidUnitRange);
        }
        Ok(Self {
            start: start as u32,
            end: end as u32,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidUnitRange;

impl fmt::Display for InvalidUnitRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("metadata range is not ordered or exceeds the source block")
    }
}

impl std::error::Error for InvalidUnitRange {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InlineMetadataKind {
    Formatting { element: String },
    Link { href: Option<String> },
    NoteReference { href: Option<String> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InlineSourceMetadata {
    pub locator: ElementLocator,
    pub kind: InlineMetadataKind,
    pub ranges: Vec<UnitRange>,
}

/// Text stored in an element attribute, separate from the visible text run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributeSourceText {
    pub locator: ElementLocator,
    pub attribute: String,
    pub value: String,
    pub resource_path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedResourceHash {
    pub canonical_path: String,
    pub sha256: [u8; 32],
}

/// One extended grapheme cluster or protected atomic source unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceUnitKind {
    Grapheme(String),
    Protected(ProtectedSourceUnit),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceUnit {
    pub kind: SourceUnitKind,
    pub spans: Vec<SourceSpan>,
}

impl SourceUnit {
    pub fn grapheme(text: String, spans: Vec<SourceSpan>) -> Self {
        Self {
            kind: SourceUnitKind::Grapheme(text),
            spans,
        }
    }

    pub fn protected(unit: ProtectedSourceUnit, spans: Vec<SourceSpan>) -> Self {
        Self {
            kind: SourceUnitKind::Protected(unit),
            spans,
        }
    }

    pub fn is_protected(&self) -> bool {
        matches!(self.kind, SourceUnitKind::Protected(_))
    }
}

/// One stable source block with ordered visible graphemes and protected units.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceBlock {
    pub id: SourceBlockId,
    pub canonical_path: String,
    /// Lowercase local XHTML element name, used as boundary evidence only.
    pub element_name: String,
    pub locator: BlockLocator,
    pub parser_version: String,
    pub content_hash: [u8; 32],
    pub units: Vec<SourceUnit>,
    pub inline_metadata: Vec<InlineSourceMetadata>,
    pub attribute_text: Vec<AttributeSourceText>,
    pub protected_resource_hashes: Vec<ProtectedResourceHash>,
}

impl SourceBlock {
    pub fn unit_count(&self) -> usize {
        self.units.len()
    }
}

/// A half-open range over grapheme/protected-unit offsets in one source block.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceRange {
    block_id: SourceBlockId,
    start: u32,
    end: u32,
}

impl SourceRange {
    pub fn new(
        block_id: SourceBlockId,
        start: usize,
        end: usize,
        unit_count: usize,
    ) -> Result<Self, InvalidSourceRange> {
        if start >= end || end > unit_count || end > u32::MAX as usize {
            return Err(InvalidSourceRange);
        }
        Ok(Self {
            block_id,
            start: start as u32,
            end: end as u32,
        })
    }

    pub fn len(self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub fn block_id(self) -> SourceBlockId {
        self.block_id
    }

    pub fn start(self) -> u32 {
        self.start
    }

    pub fn end(self) -> u32 {
        self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSourceRange;

impl fmt::Display for InvalidSourceRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("source range must be non-empty and within the block")
    }
}

impl std::error::Error for InvalidSourceRange {}

#[cfg(test)]
mod tests {
    use super::{
        BlockIdentityRecord, BlockLocator, SourceBlockId, SourceRange, SourceSpan, TextNodeLocator,
    };

    #[test]
    fn source_ranges_are_non_empty_half_open_and_bounded_by_units() {
        let id = SourceBlockId::from_bytes([7; 32]);
        assert_eq!(SourceRange::new(id, 1, 3, 4).unwrap().len(), 2);
        assert!(SourceRange::new(id, 2, 2, 4).is_err());
        assert!(SourceRange::new(id, 0, 5, 4).is_err());
    }

    #[test]
    fn source_spans_validate_utf8_byte_boundaries_in_decoded_text_nodes() {
        let text = "日e\u{301}";
        let node = TextNodeLocator {
            block: BlockLocator::NodePath {
                element_ordinals: vec![0, 2],
                order: 0,
            },
            text_node_order: 1,
        };
        assert_eq!(
            SourceSpan::new(node.clone(), 3, 6, text).unwrap().end_byte,
            6
        );
        assert!(SourceSpan::new(node.clone(), 1, 3, text).is_err());
        assert!(SourceSpan::new(node, 0, 0, text).is_err());
    }

    #[test]
    fn marker_pair_is_one_atomic_protected_source_unit() {
        use super::{MarkerEndpoint, MarkerPairSource, ProtectedSourceUnit, SourceUnit};

        let marker = SourceUnit::protected(
            ProtectedSourceUnit::MarkerPair(MarkerPairSource {
                kind: "emphasis".to_owned(),
                start: MarkerEndpoint {
                    node_path: vec![1],
                    attributes: vec![("class".to_owned(), "open".to_owned())],
                },
                end: MarkerEndpoint {
                    node_path: vec![3],
                    attributes: vec![("class".to_owned(), "close".to_owned())],
                },
            }),
            Vec::new(),
        );
        assert!(marker.is_protected());
        let block = SourceBlockId::from_bytes([8; 32]);
        assert_eq!(SourceRange::new(block, 1, 2, 3).unwrap().len(), 1);
        assert!(SourceRange::new(block, 1, 1, 3).is_err());
    }
    #[test]
    fn source_block_ids_have_fixed_lowercase_hex_display() {
        let id = SourceBlockId::from_bytes([0xab; 32]);
        assert_eq!(id.to_string(), "ab".repeat(32));
    }

    #[test]
    fn encodes_the_accepted_block_identity_vector_byte_for_byte() {
        let record = BlockIdentityRecord {
            epub_sha256: core::array::from_fn(|index| index as u8),
            parser_version: "nt-epub/1.2.0".to_owned(),
            canonical_path: "EPUB/text/ch01.xhtml".to_owned(),
            locator: BlockLocator::UniqueXhtmlId("p-1".to_owned()),
        };
        let hex = record
            .canonical_bytes()
            .unwrap()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(
            hex,
            "4e542d424c4f434b2d4944000001000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0000000d6e742d657075622f312e322e3000000014455055422f746578742f636830312e7868746d6c0100000003702d31"
        );
    }
}

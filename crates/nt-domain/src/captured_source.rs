//! Compact source evidence produced by replaying the actual captured XML.

use crate::source_range::{
    AttributeSourceText, BlockLocator, ImageSource, InlineSourceMetadata, ProtectedSourceUnit,
    SourceBlockId, SourceSpan,
};

pub struct CapturedSourceRequest<'a> {
    pub canonical_path: &'a str,
    pub epub_sha256: [u8; 32],
    pub parser_version: &'a str,
    pub is_navigation_document: bool,
    pub working_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapturedSourceError {
    InvalidXml {
        line: Option<u32>,
        column: Option<u32>,
    },
    UnsafeXml,
    UnsupportedEncoding,
    InvalidRoot,
    InvalidSource,
    EvidenceMismatch,
    ResourceLimit,
    UnsupportedParser,
}

/// One block, with decoded source text and original lexical node offsets.
/// Callers can consume it immediately instead of retaining the whole document.
pub struct CapturedBlock {
    pub id: SourceBlockId,
    pub element_name: String,
    pub locator: BlockLocator,
    pub placement: CapturedBlockPlacement,
    pub content_hash: [u8; 32],
    pub unit_count: u32,
    pub runs: Vec<CapturedUnitRun>,
    pub inline_metadata: Vec<InlineSourceMetadata>,
    pub attribute_text: Vec<AttributeSourceText>,
}

/// Absolute element positions and the surrounding authored structure.
#[derive(Clone, Default)]
pub struct CapturedBlockPlacement {
    pub element_ordinals: Vec<u32>,
    /// The block's own element, then relevant ancestors, in that order.
    pub context: Vec<CapturedScopeElement>,
}

#[derive(Clone)]
pub struct CapturedScopeElement {
    pub element_ordinals: Vec<u32>,
    pub epub_type: Option<String>,
    pub role: Option<String>,
    pub figure_or_table: bool,
}

pub enum CapturedUnitRun {
    Text {
        text: String,
        unit_count: u32,
        /// Run start/end, lexical node ordinal, and start/end within that node.
        segments: Vec<[u32; 5]>,
    },
    Protected {
        unit: ProtectedSourceUnit,
        spans: Vec<SourceSpan>,
    },
}

/// The part of a verified original block needed to align markup references.
pub struct CapturedImagePlacement {
    pub block_id: SourceBlockId,
    pub block_locator: BlockLocator,
    pub unit_offset: u32,
    pub image: ImageSource,
}

/// One authored TOC label, at its actual XML element and navigation position.
pub struct CapturedNavigationLabel {
    pub canonical_path: String,
    pub item_ordinals: Vec<u32>,
    pub element_ordinals: Vec<u32>,
    pub text: String,
    pub raw_href: Option<String>,
}

//! Domain contract for ordered chapter maps and complete source coverage.

use crate::source_range::{SourceBlock, SourceBlockId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoverageDomain {
    MainFlow,
    Auxiliary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDocument {
    pub canonical_path: String,
    pub domain: CoverageDomain,
    /// All spine occurrences are retained; repeated resource bytes remain one source identity.
    pub spine_positions: Vec<usize>,
    pub linear_spine_positions: Vec<usize>,
    pub blocks: Vec<SourceBlock>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceAnchor {
    pub canonical_path: String,
    pub fragment: String,
    pub block_id: SourceBlockId,
    pub unit_offset: u32,
    pub is_unique: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NavigationSource {
    Epub3Nav,
    Epub2Ncx,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationTreeState {
    Usable,
    Partial,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NavigationTarget {
    Resolved {
        canonical_path: String,
        fragment: Option<String>,
    },
    Invalid {
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationItem {
    pub label: String,
    /// Preserve the original href even when canonical resolution failed.
    pub raw_href: Option<String>,
    pub target: Option<NavigationTarget>,
    pub children: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationTree {
    pub source: NavigationSource,
    pub resource_path: String,
    pub state: NavigationTreeState,
    pub items: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceInputDiagnostic {
    pub source: Option<NavigationSource>,
    pub resource_path: Option<String>,
    pub code: String,
    pub reason: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmedBoundary {
    pub epub_sha256: [u8; 32],
    pub parser_version: String,
    pub canonical_path: String,
    pub block_id: SourceBlockId,
    /// Insertion offset in `[0, block.unit_count()]`.
    pub unit_offset: u32,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChapterMapInput {
    pub epub_sha256: [u8; 32],
    pub parser_version: String,
    pub documents: Vec<SourceDocument>,
    pub anchors: Vec<SourceAnchor>,
    pub navigation: Vec<NavigationTree>,
    pub confirmed_boundaries: Vec<ConfirmedBoundary>,
    pub diagnostics: Vec<SourceInputDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockUnitRange {
    pub canonical_path: String,
    pub block_id: SourceBlockId,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingStatus {
    Confirmed,
    Provisional,
    NeedsReview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryEvidenceKind {
    Confirmed,
    Navigation,
    Heading,
    FileEdge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryEvidence {
    pub kind: BoundaryEvidenceKind,
    pub source: Option<NavigationSource>,
    pub canonical_path: String,
    pub block_id: Option<SourceBlockId>,
    pub unit_offset: Option<u32>,
    pub label: Option<String>,
    pub raw_href: Option<String>,
    pub reason: String,
    pub depth: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chapter {
    pub chapter_id: u32,
    pub label: Option<String>,
    pub status: MappingStatus,
    pub ranges: Vec<BlockUnitRange>,
    pub start_evidence: Vec<BoundaryEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MainFlowContent {
    Chapter(Chapter),
    OutOfChapter {
        ranges: Vec<BlockUnitRange>,
        reason: String,
    },
    Unclassified {
        ranges: Vec<BlockUnitRange>,
        reason: String,
        evidence: Vec<BoundaryEvidence>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuxiliaryDocumentCoverage {
    pub canonical_path: String,
    pub ranges: Vec<BlockUnitRange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeTree {
    pub main_flow: Vec<MainFlowContent>,
    pub auxiliary: Vec<AuxiliaryDocumentCoverage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChapterMapDiagnostic {
    NavigationSourceUnavailable {
        source: Option<NavigationSource>,
        resource_path: Option<String>,
        reason: String,
    },
    NavigationTargetUnresolved {
        source: NavigationSource,
        resource_path: String,
        label: String,
        raw_href: Option<String>,
        reason: String,
    },
    NavigationTargetAmbiguous {
        source: NavigationSource,
        resource_path: String,
        label: String,
        raw_href: Option<String>,
        reason: String,
    },
    NavigationSourcesConflict {
        from_path: Option<String>,
        to_path: Option<String>,
        reason: String,
    },
    NavigationHeadingConflict {
        from_path: Option<String>,
        to_path: Option<String>,
        reason: String,
    },
    ConfirmedBoundaryStale {
        canonical_path: String,
        block_id: SourceBlockId,
        reason: String,
    },
    HeadingFallback {
        canonical_path: String,
        block_id: SourceBlockId,
        level: u8,
    },
    FileEdgeFallback {
        canonical_path: String,
        block_id: SourceBlockId,
    },
    InputDiagnostic(SourceInputDiagnostic),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChapterMap {
    pub epub_sha256: [u8; 32],
    pub parser_version: String,
    /// One volume root. Main-flow and auxiliary coverage are intentionally separate.
    pub volume: VolumeTree,
    /// Both source trees are retained independently, including alternatives.
    pub navigation: Vec<NavigationTree>,
    /// Every candidate boundary remains inspectable, whether selected or not.
    pub boundary_evidence: Vec<BoundaryEvidence>,
    pub diagnostics: Vec<ChapterMapDiagnostic>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChapterMapError {
    EmptyParserVersion,
    DuplicateSourceBlockId,
    BlockPathMismatch,
    InvalidBoundaryRange,
    MappingIdentityMismatch,
    CoverageGap,
    CoverageOverlap,
    CoverageOrder,
    WrongCoverageDomain,
    UnknownSourceBlock,
}

impl std::fmt::Display for ChapterMapError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyParserVersion => "parser version must not be empty",
            Self::DuplicateSourceBlockId => "source block ID appears more than once",
            Self::BlockPathMismatch => "source block path does not match its document",
            Self::InvalidBoundaryRange => "mapping range is empty or exceeds its source block",
            Self::MappingIdentityMismatch => "mapping snapshot/parser identity is stale",
            Self::CoverageGap => "mapping leaves source units uncovered",
            Self::CoverageOverlap => "mapping assigns a source unit more than once",
            Self::CoverageOrder => "mapping ranges do not follow source reading order",
            Self::WrongCoverageDomain => "mapping assigns a range to the wrong coverage domain",
            Self::UnknownSourceBlock => "mapping references an unknown source block",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ChapterMapError {}

/// Validate that a candidate is a complete, non-overlapping partition of its source.
pub fn validate_coverage(input: &ChapterMapInput, map: &ChapterMap) -> Result<(), ChapterMapError> {
    if input.parser_version.trim().is_empty() {
        return Err(ChapterMapError::EmptyParserVersion);
    }
    if map.epub_sha256 != input.epub_sha256 || map.parser_version != input.parser_version {
        return Err(ChapterMapError::MappingIdentityMismatch);
    }

    let mut blocks = std::collections::HashMap::new();
    for document in &input.documents {
        for block in &document.blocks {
            if block.canonical_path != document.canonical_path {
                return Err(ChapterMapError::BlockPathMismatch);
            }
            if blocks
                .insert(
                    block.id,
                    (
                        document.canonical_path.as_str(),
                        document.domain,
                        block.unit_count(),
                    ),
                )
                .is_some()
            {
                return Err(ChapterMapError::DuplicateSourceBlockId);
            }
        }
    }

    let mut ranges =
        std::collections::HashMap::<SourceBlockId, Vec<(String, u32, u32, CoverageDomain)>>::new();
    for content in &map.volume.main_flow {
        let source_ranges = match content {
            MainFlowContent::Chapter(chapter) => &chapter.ranges,
            MainFlowContent::OutOfChapter { ranges, .. }
            | MainFlowContent::Unclassified { ranges, .. } => ranges,
        };
        for range in source_ranges {
            ranges.entry(range.block_id).or_default().push((
                range.canonical_path.clone(),
                range.start,
                range.end,
                CoverageDomain::MainFlow,
            ));
        }
    }
    for document in &map.volume.auxiliary {
        for range in &document.ranges {
            if range.canonical_path != document.canonical_path {
                return Err(ChapterMapError::BlockPathMismatch);
            }
            ranges.entry(range.block_id).or_default().push((
                range.canonical_path.clone(),
                range.start,
                range.end,
                CoverageDomain::Auxiliary,
            ));
        }
    }

    let mut main_documents = input
        .documents
        .iter()
        .enumerate()
        .filter(|(_, document)| document.domain == CoverageDomain::MainFlow)
        .collect::<Vec<_>>();
    main_documents.sort_by_key(|(index, document)| {
        (
            document
                .linear_spine_positions
                .iter()
                .copied()
                .min()
                .unwrap_or(usize::MAX),
            document
                .spine_positions
                .iter()
                .copied()
                .min()
                .unwrap_or(usize::MAX),
            *index,
        )
    });
    let mut main_order = std::collections::HashMap::new();
    for (_, document) in main_documents {
        for block in &document.blocks {
            let next = main_order.len();
            main_order.insert(block.id, next);
        }
    }

    let mut previous_main_range: Option<(usize, u32, u32)> = None;
    for content in &map.volume.main_flow {
        let source_ranges = match content {
            MainFlowContent::Chapter(chapter) => &chapter.ranges,
            MainFlowContent::OutOfChapter { ranges, .. }
            | MainFlowContent::Unclassified { ranges, .. } => ranges,
        };
        for range in source_ranges {
            let (_, domain, _) = blocks
                .get(&range.block_id)
                .copied()
                .ok_or(ChapterMapError::UnknownSourceBlock)?;
            if domain != CoverageDomain::MainFlow {
                return Err(ChapterMapError::WrongCoverageDomain);
            }
            let order = *main_order
                .get(&range.block_id)
                .ok_or(ChapterMapError::UnknownSourceBlock)?;
            if let Some((previous_order, previous_start, previous_end)) = previous_main_range {
                if order < previous_order
                    || (order == previous_order && range.start < previous_start)
                {
                    return Err(ChapterMapError::CoverageOrder);
                }
                if order == previous_order && range.start < previous_end {
                    return Err(ChapterMapError::CoverageOverlap);
                }
            }
            previous_main_range = Some((order, range.start, range.end));
        }
    }

    for (block_id, assigned) in &ranges {
        let (block_path, domain, unit_count) = blocks
            .get(block_id)
            .copied()
            .ok_or(ChapterMapError::UnknownSourceBlock)?;
        if assigned
            .iter()
            .any(|(path, _, _, assigned_domain)| path != block_path || *assigned_domain != domain)
        {
            return Err(ChapterMapError::WrongCoverageDomain);
        }
        if assigned
            .iter()
            .any(|(_, start, end, _)| start >= end || *end as usize > unit_count)
        {
            return Err(ChapterMapError::InvalidBoundaryRange);
        }
    }

    for (block_id, (_, domain, unit_count)) in blocks {
        if unit_count == 0 {
            continue;
        }
        let Some(mut assigned) = ranges.remove(&block_id) else {
            return Err(ChapterMapError::CoverageGap);
        };
        assigned.sort_by_key(|(_, start, end, _)| (*start, *end));
        let mut cursor = 0_u32;
        for (_, start, end, _) in assigned {
            if start < cursor {
                return Err(ChapterMapError::CoverageOverlap);
            }
            if start > cursor {
                return Err(ChapterMapError::CoverageGap);
            }
            cursor = end;
        }
        if cursor as usize != unit_count {
            return Err(ChapterMapError::CoverageGap);
        }
        if domain == CoverageDomain::Auxiliary
            && map
                .volume
                .auxiliary
                .iter()
                .all(|item| item.ranges.iter().all(|range| range.block_id != block_id))
        {
            return Err(ChapterMapError::WrongCoverageDomain);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::source_range::{
        BlockLocator, SourceBlock, SourceBlockId, SourceUnit, SourceUnitKind,
    };

    use super::{
        BlockUnitRange, ChapterMap, ChapterMapError, ChapterMapInput, CoverageDomain,
        MainFlowContent, SourceDocument, VolumeTree, validate_coverage,
    };

    fn fixture(ranges: Vec<(u32, u32)>) -> (ChapterMapInput, ChapterMap) {
        let block_id = SourceBlockId::from_bytes([1; 32]);
        let block = SourceBlock {
            id: block_id,
            canonical_path: "OPS/text/chapter.xhtml".to_owned(),
            element_name: "p".to_owned(),
            locator: BlockLocator::UniqueXhtmlId("chapter".to_owned()),
            parser_version: "test/1".to_owned(),
            content_hash: [2; 32],
            units: ["a", "b", "c", "d", "e", "f", "g", "h"]
                .into_iter()
                .map(|text| SourceUnit {
                    kind: SourceUnitKind::Grapheme(text.to_owned()),
                    spans: Vec::new(),
                })
                .collect(),
            inline_metadata: Vec::new(),
            attribute_text: Vec::new(),
            protected_resource_hashes: Vec::new(),
        };
        let input = ChapterMapInput {
            epub_sha256: [3; 32],
            parser_version: "test/1".to_owned(),
            documents: vec![SourceDocument {
                canonical_path: block.canonical_path.clone(),
                domain: CoverageDomain::MainFlow,
                spine_positions: vec![0],
                linear_spine_positions: vec![0],
                blocks: vec![block],
            }],
            anchors: Vec::new(),
            navigation: Vec::new(),
            confirmed_boundaries: Vec::new(),
            diagnostics: Vec::new(),
        };
        let assigned = ranges
            .into_iter()
            .map(|(start, end)| BlockUnitRange {
                canonical_path: "OPS/text/chapter.xhtml".to_owned(),
                block_id,
                start,
                end,
            })
            .collect();
        let map = ChapterMap {
            epub_sha256: [3; 32],
            parser_version: "test/1".to_owned(),
            volume: VolumeTree {
                main_flow: vec![MainFlowContent::OutOfChapter {
                    ranges: assigned,
                    reason: "test fixture".to_owned(),
                }],
                auxiliary: Vec::new(),
            },
            navigation: Vec::new(),
            boundary_evidence: Vec::new(),
            diagnostics: Vec::new(),
        };
        (input, map)
    }

    #[test]
    fn rejects_reversed_ranges_in_one_source_block() {
        let (input, map) = fixture(vec![(4, 8), (0, 4)]);
        assert_eq!(
            validate_coverage(&input, &map),
            Err(ChapterMapError::CoverageOrder)
        );
    }

    #[test]
    fn rejects_source_blocks_in_reverse_reading_order() {
        let (mut input, mut map) = fixture(vec![(0, 8)]);
        let first = input.documents[0].blocks[0].clone();
        let mut second = first.clone();
        second.id = SourceBlockId::from_bytes([2; 32]);
        second.locator = BlockLocator::UniqueXhtmlId("second".to_owned());
        input.documents[0].blocks.push(second.clone());
        map.volume.main_flow = vec![MainFlowContent::OutOfChapter {
            ranges: vec![
                BlockUnitRange {
                    canonical_path: second.canonical_path.clone(),
                    block_id: second.id,
                    start: 0,
                    end: 8,
                },
                BlockUnitRange {
                    canonical_path: first.canonical_path.clone(),
                    block_id: first.id,
                    start: 0,
                    end: 8,
                },
            ],
            reason: "reverse source order".to_owned(),
        }];
        assert_eq!(
            validate_coverage(&input, &map),
            Err(ChapterMapError::CoverageOrder)
        );
    }
    #[test]
    fn rejects_a_gap_in_source_unit_coverage() {
        let (input, map) = fixture(vec![(0, 1), (2, 3)]);
        assert_eq!(
            validate_coverage(&input, &map),
            Err(ChapterMapError::CoverageGap)
        );
    }

    #[test]
    fn rejects_overlapping_source_unit_coverage() {
        let (input, map) = fixture(vec![(0, 2), (1, 3)]);
        assert_eq!(
            validate_coverage(&input, &map),
            Err(ChapterMapError::CoverageOverlap)
        );
    }
}

//! Closed chapter-map records. Source ranges refer to compact unit offsets.
use super::inventory_records::{names, validate_parents};
use super::records::{Budget, Digest, safe_path};
use super::{CodecError, StoredSourceIndex};
use nt_application::volume_import::chapter_map as map;
use serde::{Deserialize, Serialize};

names!(NavigationSource, map::NavigationSource, Epub3Nav, Epub2Ncx);
names!(
    NavigationState,
    map::NavigationTreeState,
    Usable,
    Partial,
    Invalid
);
names!(
    MappingStatus,
    map::MappingStatus,
    Confirmed,
    Provisional,
    NeedsReview
);
names!(
    EvidenceKind,
    map::BoundaryEvidenceKind,
    Confirmed,
    Navigation,
    Heading,
    FileEdge
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockRange {
    pub canonical_path: String,
    pub block_id: Digest,
    pub start: u32,
    pub end: u32,
}
impl BlockRange {
    fn restore(self) -> map::BlockUnitRange {
        map::BlockUnitRange {
            canonical_path: self.canonical_path,
            block_id: nt_application::volume_import::source_range::SourceBlockId::from_bytes(
                self.block_id.0,
            ),
            start: self.start,
            end: self.end,
        }
    }
    fn capture(r: &map::BlockUnitRange, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(Self {
            canonical_path: budget.text(&r.canonical_path)?,
            block_id: Digest(*r.block_id.as_bytes()),
            start: r.start,
            end: r.end,
        })
    }
    fn all(values: &[map::BlockUnitRange], budget: &mut Budget) -> Result<Vec<Self>, CodecError> {
        let mut out = budget.vector(values.len())?;
        for value in values {
            out.push(Self::capture(value, budget)?);
        }
        Ok(out)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryEvidence {
    pub kind: EvidenceKind,
    pub source: Option<NavigationSource>,
    pub canonical_path: String,
    pub block_id: Option<Digest>,
    pub unit_offset: Option<u32>,
    pub label: Option<String>,
    pub raw_href: Option<String>,
    pub reason: String,
    pub depth: Option<u64>,
}
impl BoundaryEvidence {
    fn restore(self) -> Result<map::BoundaryEvidence, CodecError> {
        Ok(map::BoundaryEvidence {
            kind: self.kind.into(),
            source: self.source.map(Into::into),
            canonical_path: self.canonical_path,
            block_id: self.block_id.map(|id| {
                nt_application::volume_import::source_range::SourceBlockId::from_bytes(id.0)
            }),
            unit_offset: self.unit_offset,
            label: self.label,
            raw_href: self.raw_href,
            reason: self.reason,
            depth: self
                .depth
                .map(|n| usize::try_from(n).map_err(|_| CodecError::ResourceLimit))
                .transpose()?,
        })
    }
    pub(super) fn capture(
        e: &map::BoundaryEvidence,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(Self {
            kind: e.kind.into(),
            source: e.source.map(Into::into),
            canonical_path: budget.text(&e.canonical_path)?,
            block_id: e.block_id.map(|id| Digest(*id.as_bytes())),
            unit_offset: e.unit_offset,
            label: budget.optional_text(&e.label)?,
            raw_href: budget.optional_text(&e.raw_href)?,
            reason: budget.text(&e.reason)?,
            depth: e
                .depth
                .map(|d| u64::try_from(d).map_err(|_| CodecError::ResourceLimit))
                .transpose()?,
        })
    }
    fn all(values: &[map::BoundaryEvidence], budget: &mut Budget) -> Result<Vec<Self>, CodecError> {
        let mut out = budget.vector(values.len())?;
        for value in values {
            out.push(Self::capture(value, budget)?);
        }
        Ok(out)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub chapter_id: u32,
    pub label: Option<String>,
    pub status: MappingStatus,
    pub ranges: Vec<BlockRange>,
    pub start_evidence: Vec<BoundaryEvidence>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum MainFlow {
    Chapter(Chapter),
    OutOfChapter {
        ranges: Vec<BlockRange>,
        reason: String,
    },
    Unclassified {
        ranges: Vec<BlockRange>,
        reason: String,
        evidence: Vec<BoundaryEvidence>,
    },
}
impl MainFlow {
    pub fn ranges(&self) -> &[BlockRange] {
        match self {
            Self::Chapter(ch) => &ch.ranges,
            Self::OutOfChapter { ranges, .. } | Self::Unclassified { ranges, .. } => ranges,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Auxiliary {
    pub canonical_path: String,
    pub ranges: Vec<BlockRange>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VolumeTree {
    pub main_flow: Vec<MainFlow>,
    pub auxiliary: Vec<Auxiliary>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum NavigationTarget {
    Resolved {
        canonical_path: String,
        fragment: Option<String>,
    },
    Invalid {
        reason: String,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationNode {
    pub parent_index: Option<u32>,
    pub sibling_index: u32,
    pub label: String,
    pub raw_href: Option<String>,
    pub target: Option<NavigationTarget>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Navigation {
    pub source: NavigationSource,
    pub resource_path: String,
    pub state: NavigationState,
    pub items: Vec<NavigationNode>,
}
impl Navigation {
    fn restore(self) -> Result<map::NavigationTree, CodecError> {
        validate_parents(self.items.iter().map(|n| (n.parent_index, n.sibling_index)))?;
        let mut children: Vec<Vec<map::NavigationItem>> =
            (0..=self.items.len()).map(|_| Vec::new()).collect();
        for (index, node) in self.items.into_iter().enumerate().rev() {
            let mut branch = std::mem::take(&mut children[index + 1]);
            branch.reverse();
            let target = node.target.map(|t| match t {
                NavigationTarget::Resolved {
                    canonical_path,
                    fragment,
                } => map::NavigationTarget::Resolved {
                    canonical_path,
                    fragment,
                },
                NavigationTarget::Invalid { reason } => map::NavigationTarget::Invalid { reason },
            });
            children[node.parent_index.map_or(0, |n| n as usize + 1)].push(map::NavigationItem {
                label: node.label,
                raw_href: node.raw_href,
                target,
                children: branch,
            });
        }
        let mut items = std::mem::take(&mut children[0]);
        items.reverse();
        Ok(map::NavigationTree {
            source: self.source.into(),
            resource_path: self.resource_path,
            state: self.state.into(),
            items,
        })
    }
    fn capture(tree: &map::NavigationTree, budget: &mut Budget) -> Result<Self, CodecError> {
        fn count(items: &[map::NavigationItem], depth: usize) -> Result<usize, CodecError> {
            if items.is_empty() {
                return Ok(0);
            }
            if depth >= 256 {
                return Err(CodecError::ResourceLimit);
            }
            let mut n = items.len();
            for item in items {
                n = n
                    .checked_add(count(&item.children, depth + 1)?)
                    .ok_or(CodecError::ResourceLimit)?;
            }
            Ok(n)
        }
        fn flatten(
            items: &[map::NavigationItem],
            parent: Option<u32>,
            out: &mut Vec<NavigationNode>,
            budget: &mut Budget,
        ) -> Result<(), CodecError> {
            for (sibling, node) in items.iter().enumerate() {
                let current = u32::try_from(out.len()).map_err(|_| CodecError::ResourceLimit)?;
                let target = node
                    .target
                    .as_ref()
                    .map(|t| {
                        Ok::<_, CodecError>(match t {
                            map::NavigationTarget::Resolved {
                                canonical_path,
                                fragment,
                            } => NavigationTarget::Resolved {
                                canonical_path: budget.text(canonical_path)?,
                                fragment: budget.optional_text(fragment)?,
                            },
                            map::NavigationTarget::Invalid { reason } => {
                                NavigationTarget::Invalid {
                                    reason: budget.text(reason)?,
                                }
                            }
                        })
                    })
                    .transpose()?;
                out.push(NavigationNode {
                    parent_index: parent,
                    sibling_index: u32::try_from(sibling).map_err(|_| CodecError::ResourceLimit)?,
                    label: budget.text(&node.label)?,
                    raw_href: budget.optional_text(&node.raw_href)?,
                    target,
                });
                flatten(&node.children, Some(current), out, budget)?;
            }
            Ok(())
        }
        let mut items = budget.vector(count(&tree.items, 0)?)?;
        flatten(&tree.items, None, &mut items, budget)?;
        Ok(Self {
            source: tree.source.into(),
            resource_path: budget.text(&tree.resource_path)?,
            state: tree.state.into(),
            items,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmedBoundary {
    pub epub_sha256: Digest,
    pub parser_version: String,
    pub canonical_path: String,
    pub block_id: Digest,
    pub unit_offset: u32,
    pub label: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInputDiagnostic {
    pub source: Option<NavigationSource>,
    pub resource_path: Option<String>,
    pub code: String,
    pub reason: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}
impl SourceInputDiagnostic {
    fn restore(self) -> map::SourceInputDiagnostic {
        map::SourceInputDiagnostic {
            source: self.source.map(Into::into),
            resource_path: self.resource_path,
            code: self.code,
            reason: self.reason,
            line: self.line,
            column: self.column,
        }
    }
    pub(super) fn capture(
        d: &map::SourceInputDiagnostic,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(Self {
            source: d.source.map(Into::into),
            resource_path: budget.optional_text(&d.resource_path)?,
            code: budget.text(&d.code)?,
            reason: budget.text(&d.reason)?,
            line: d.line,
            column: d.column,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum MapIssue {
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
        block_id: Digest,
        reason: String,
    },
    HeadingFallback {
        canonical_path: String,
        block_id: Digest,
        level: u8,
    },
    FileEdgeFallback {
        canonical_path: String,
        block_id: Digest,
    },
    InputDiagnostic {
        value: SourceInputDiagnostic,
    },
}
impl MapIssue {
    fn restore(self) -> map::ChapterMapDiagnostic {
        use map::ChapterMapDiagnostic as D;
        match self {
            Self::NavigationSourceUnavailable {
                source,
                resource_path,
                reason,
            } => D::NavigationSourceUnavailable {
                source: source.map(Into::into),
                resource_path,
                reason,
            },
            Self::NavigationTargetUnresolved {
                source,
                resource_path,
                label,
                raw_href,
                reason,
            } => D::NavigationTargetUnresolved {
                source: source.into(),
                resource_path,
                label,
                raw_href,
                reason,
            },
            Self::NavigationTargetAmbiguous {
                source,
                resource_path,
                label,
                raw_href,
                reason,
            } => D::NavigationTargetAmbiguous {
                source: source.into(),
                resource_path,
                label,
                raw_href,
                reason,
            },
            Self::NavigationSourcesConflict {
                from_path,
                to_path,
                reason,
            } => D::NavigationSourcesConflict {
                from_path,
                to_path,
                reason,
            },
            Self::NavigationHeadingConflict {
                from_path,
                to_path,
                reason,
            } => D::NavigationHeadingConflict {
                from_path,
                to_path,
                reason,
            },
            Self::ConfirmedBoundaryStale {
                canonical_path,
                block_id,
                reason,
            } => D::ConfirmedBoundaryStale {
                canonical_path,
                block_id: nt_application::volume_import::source_range::SourceBlockId::from_bytes(
                    block_id.0,
                ),
                reason,
            },
            Self::HeadingFallback {
                canonical_path,
                block_id,
                level,
            } => D::HeadingFallback {
                canonical_path,
                block_id: nt_application::volume_import::source_range::SourceBlockId::from_bytes(
                    block_id.0,
                ),
                level,
            },
            Self::FileEdgeFallback {
                canonical_path,
                block_id,
            } => D::FileEdgeFallback {
                canonical_path,
                block_id: nt_application::volume_import::source_range::SourceBlockId::from_bytes(
                    block_id.0,
                ),
            },
            Self::InputDiagnostic { value } => D::InputDiagnostic(value.restore()),
        }
    }
    pub(super) fn capture(
        d: &map::ChapterMapDiagnostic,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        use map::ChapterMapDiagnostic as D;
        Ok(match d {
            D::NavigationSourceUnavailable {
                source,
                resource_path,
                reason,
            } => Self::NavigationSourceUnavailable {
                source: source.map(Into::into),
                resource_path: budget.optional_text(resource_path)?,
                reason: budget.text(reason)?,
            },
            D::NavigationTargetUnresolved {
                source,
                resource_path,
                label,
                raw_href,
                reason,
            } => Self::NavigationTargetUnresolved {
                source: (*source).into(),
                resource_path: budget.text(resource_path)?,
                label: budget.text(label)?,
                raw_href: budget.optional_text(raw_href)?,
                reason: budget.text(reason)?,
            },
            D::NavigationTargetAmbiguous {
                source,
                resource_path,
                label,
                raw_href,
                reason,
            } => Self::NavigationTargetAmbiguous {
                source: (*source).into(),
                resource_path: budget.text(resource_path)?,
                label: budget.text(label)?,
                raw_href: budget.optional_text(raw_href)?,
                reason: budget.text(reason)?,
            },
            D::NavigationSourcesConflict {
                from_path,
                to_path,
                reason,
            } => Self::NavigationSourcesConflict {
                from_path: budget.optional_text(from_path)?,
                to_path: budget.optional_text(to_path)?,
                reason: budget.text(reason)?,
            },
            D::NavigationHeadingConflict {
                from_path,
                to_path,
                reason,
            } => Self::NavigationHeadingConflict {
                from_path: budget.optional_text(from_path)?,
                to_path: budget.optional_text(to_path)?,
                reason: budget.text(reason)?,
            },
            D::ConfirmedBoundaryStale {
                canonical_path,
                block_id,
                reason,
            } => Self::ConfirmedBoundaryStale {
                canonical_path: budget.text(canonical_path)?,
                block_id: Digest(*block_id.as_bytes()),
                reason: budget.text(reason)?,
            },
            D::HeadingFallback {
                canonical_path,
                block_id,
                level,
            } => Self::HeadingFallback {
                canonical_path: budget.text(canonical_path)?,
                block_id: Digest(*block_id.as_bytes()),
                level: *level,
            },
            D::FileEdgeFallback {
                canonical_path,
                block_id,
            } => Self::FileEdgeFallback {
                canonical_path: budget.text(canonical_path)?,
                block_id: Digest(*block_id.as_bytes()),
            },
            D::InputDiagnostic(value) => Self::InputDiagnostic {
                value: SourceInputDiagnostic::capture(value, budget)?,
            },
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredMapping {
    pub epub_sha256: Digest,
    pub parser_version: String,
    pub volume: VolumeTree,
    pub navigation: Vec<Navigation>,
    pub boundary_evidence: Vec<BoundaryEvidence>,
    pub confirmed_boundaries: Vec<ConfirmedBoundary>,
    pub diagnostics: Vec<MapIssue>,
}
impl StoredMapping {
    pub(super) fn matches_original_navigation(
        &self,
        trees: &[map::NavigationTree],
        budget: &mut Budget,
    ) -> Result<bool, CodecError> {
        if trees.len() != self.navigation.len() {
            return Ok(false);
        }
        for (saved, original) in self.navigation.iter().zip(trees) {
            if saved != &Navigation::capture(original, budget)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Consume the wire data without creating any source units. The caller
    /// still checks source identity/coverage before trusting the restored map.
    pub fn into_map(self) -> Result<(map::ChapterMap, Vec<map::ConfirmedBoundary>), CodecError> {
        fn evidence(
            values: Vec<BoundaryEvidence>,
        ) -> Result<Vec<map::BoundaryEvidence>, CodecError> {
            values.into_iter().map(BoundaryEvidence::restore).collect()
        }
        let mut main_flow = Vec::with_capacity(self.volume.main_flow.len());
        for flow in self.volume.main_flow {
            main_flow.push(match flow {
                MainFlow::Chapter(ch) => map::MainFlowContent::Chapter(map::Chapter {
                    chapter_id: ch.chapter_id,
                    label: ch.label,
                    status: ch.status.into(),
                    ranges: ch.ranges.into_iter().map(BlockRange::restore).collect(),
                    start_evidence: evidence(ch.start_evidence)?,
                }),
                MainFlow::OutOfChapter { ranges, reason } => map::MainFlowContent::OutOfChapter {
                    ranges: ranges.into_iter().map(BlockRange::restore).collect(),
                    reason,
                },
                MainFlow::Unclassified {
                    ranges,
                    reason,
                    evidence: alternatives,
                } => map::MainFlowContent::Unclassified {
                    ranges: ranges.into_iter().map(BlockRange::restore).collect(),
                    reason,
                    evidence: evidence(alternatives)?,
                },
            });
        }
        let auxiliary = self
            .volume
            .auxiliary
            .into_iter()
            .map(|d| map::AuxiliaryDocumentCoverage {
                canonical_path: d.canonical_path,
                ranges: d.ranges.into_iter().map(BlockRange::restore).collect(),
            })
            .collect();
        let mut confirmed = Vec::with_capacity(self.confirmed_boundaries.len());
        for c in self.confirmed_boundaries {
            confirmed.push(map::ConfirmedBoundary {
                epub_sha256: c.epub_sha256.0,
                parser_version: c.parser_version,
                canonical_path: c.canonical_path,
                block_id: nt_application::volume_import::source_range::SourceBlockId::from_bytes(
                    c.block_id.0,
                ),
                unit_offset: c.unit_offset,
                label: c.label,
            });
        }
        Ok((
            map::ChapterMap {
                epub_sha256: self.epub_sha256.0,
                parser_version: self.parser_version,
                volume: map::VolumeTree {
                    main_flow,
                    auxiliary,
                },
                navigation: self
                    .navigation
                    .into_iter()
                    .map(Navigation::restore)
                    .collect::<Result<_, _>>()?,
                boundary_evidence: evidence(self.boundary_evidence)?,
                diagnostics: self
                    .diagnostics
                    .into_iter()
                    .map(MapIssue::restore)
                    .collect(),
            },
            confirmed,
        ))
    }
    pub fn capture(
        value: &map::ChapterMap,
        confirmed: &[map::ConfirmedBoundary],
        reserved_working_bytes: usize,
    ) -> Result<Self, CodecError> {
        Self::capture_with_budget(value, confirmed, &mut Budget::new(reserved_working_bytes)?)
    }
    pub(super) fn capture_with_budget(
        value: &map::ChapterMap,
        confirmed: &[map::ConfirmedBoundary],
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        let mut main_flow = budget.vector(value.volume.main_flow.len())?;
        for flow in &value.volume.main_flow {
            main_flow.push(match flow {
                map::MainFlowContent::Chapter(ch) => MainFlow::Chapter(Chapter {
                    chapter_id: ch.chapter_id,
                    label: budget.optional_text(&ch.label)?,
                    status: ch.status.into(),
                    ranges: BlockRange::all(&ch.ranges, budget)?,
                    start_evidence: BoundaryEvidence::all(&ch.start_evidence, budget)?,
                }),
                map::MainFlowContent::OutOfChapter { ranges, reason } => MainFlow::OutOfChapter {
                    ranges: BlockRange::all(ranges, budget)?,
                    reason: budget.text(reason)?,
                },
                map::MainFlowContent::Unclassified {
                    ranges,
                    reason,
                    evidence,
                } => MainFlow::Unclassified {
                    ranges: BlockRange::all(ranges, budget)?,
                    reason: budget.text(reason)?,
                    evidence: BoundaryEvidence::all(evidence, budget)?,
                },
            });
        }
        let mut auxiliary = budget.vector(value.volume.auxiliary.len())?;
        for d in &value.volume.auxiliary {
            auxiliary.push(Auxiliary {
                canonical_path: budget.text(&d.canonical_path)?,
                ranges: BlockRange::all(&d.ranges, budget)?,
            });
        }
        let mut navigation = budget.vector(value.navigation.len())?;
        for tree in &value.navigation {
            navigation.push(Navigation::capture(tree, budget)?);
        }
        let mut confirmed_boundaries = budget.vector(confirmed.len())?;
        for c in confirmed {
            confirmed_boundaries.push(ConfirmedBoundary {
                epub_sha256: Digest(c.epub_sha256),
                parser_version: budget.text(&c.parser_version)?,
                canonical_path: budget.text(&c.canonical_path)?,
                block_id: Digest(*c.block_id.as_bytes()),
                unit_offset: c.unit_offset,
                label: budget.optional_text(&c.label)?,
            });
        }
        let mut diagnostics = budget.vector(value.diagnostics.len())?;
        for d in &value.diagnostics {
            diagnostics.push(MapIssue::capture(d, budget)?);
        }
        Ok(Self {
            epub_sha256: Digest(value.epub_sha256),
            parser_version: budget.text(&value.parser_version)?,
            volume: VolumeTree {
                main_flow,
                auxiliary,
            },
            navigation,
            boundary_evidence: BoundaryEvidence::all(&value.boundary_evidence, budget)?,
            confirmed_boundaries,
            diagnostics,
        })
    }

    /// Check coverage using compact unit counts. No synthetic grapheme or
    /// SourceSpan is allocated to call an API designed for extracted units.
    pub fn validate_coverage(
        &self,
        source: &StoredSourceIndex,
        epub: Digest,
        parser: &str,
    ) -> Result<(), CodecError> {
        use super::source_records::Domain;
        if self.epub_sha256 != epub || self.parser_version != parser {
            return Err(CodecError::InvalidRecord);
        }
        let mut blocks = std::collections::BTreeMap::new();
        let mut main_documents = source
            .documents
            .iter()
            .enumerate()
            .filter(|(_, d)| d.domain == Domain::MainFlow)
            .collect::<Vec<_>>();
        main_documents.sort_by_key(|(i, d)| {
            (
                d.linear_spine_positions
                    .iter()
                    .min()
                    .copied()
                    .unwrap_or(u64::MAX),
                d.spine_positions.iter().min().copied().unwrap_or(u64::MAX),
                *i,
            )
        });
        let mut order = std::collections::BTreeMap::new();
        for (_, d) in main_documents {
            for b in &d.blocks {
                order.insert(b.id, order.len());
            }
        }
        for d in &source.documents {
            for b in &d.blocks {
                if blocks
                    .insert(
                        b.id,
                        (
                            d.canonical_path.as_str(),
                            d.domain,
                            b.unit_count()?,
                            Vec::<(u32, u32)>::new(),
                        ),
                    )
                    .is_some()
                {
                    return Err(CodecError::InvalidRecord);
                }
            }
        }
        let mut previous: Option<(usize, u32)> = None;
        for r in self.volume.main_flow.iter().flat_map(MainFlow::ranges) {
            let Some(&current) = order.get(&r.block_id) else {
                return Err(CodecError::InvalidRecord);
            };
            if previous
                .is_some_and(|(last, end)| current < last || (current == last && r.start < end))
            {
                return Err(CodecError::InvalidRecord);
            }
            previous = Some((current, r.end));
            assign(&mut blocks, r, Domain::MainFlow)?;
        }
        for d in &self.volume.auxiliary {
            if !safe_path(&d.canonical_path) {
                return Err(CodecError::InvalidRecord);
            }
            for r in &d.ranges {
                if r.canonical_path != d.canonical_path {
                    return Err(CodecError::InvalidRecord);
                }
                assign(&mut blocks, r, Domain::Auxiliary)?;
            }
        }
        for (_, _, count, ranges) in blocks.values_mut() {
            ranges.sort_unstable();
            let mut cursor = 0;
            for &(start, end) in ranges.iter() {
                if start != cursor {
                    return Err(CodecError::InvalidRecord);
                }
                cursor = end;
            }
            if cursor != *count {
                return Err(CodecError::InvalidRecord);
            }
        }
        for n in &self.navigation {
            validate_parents(n.items.iter().map(|n| (n.parent_index, n.sibling_index)))?;
            if !safe_path(&n.resource_path) {
                return Err(CodecError::InvalidRecord);
            }
        }
        for c in &self.confirmed_boundaries {
            if c.epub_sha256 != epub
                || c.parser_version != parser
                || !blocks.get(&c.block_id).is_some_and(|(path, _, count, _)| {
                    *path == c.canonical_path && c.unit_offset <= *count
                })
            {
                return Err(CodecError::InvalidRecord);
            }
        }
        Ok(())
    }
}
type CoverageBlocks<'a> = std::collections::BTreeMap<
    Digest,
    (&'a str, super::source_records::Domain, u32, Vec<(u32, u32)>),
>;
fn assign(
    blocks: &mut CoverageBlocks<'_>,
    r: &BlockRange,
    domain: super::source_records::Domain,
) -> Result<(), CodecError> {
    let (path, actual, count, ranges) = blocks
        .get_mut(&r.block_id)
        .ok_or(CodecError::InvalidRecord)?;
    if *path != r.canonical_path || *actual != domain || r.start >= r.end || r.end > *count {
        return Err(CodecError::InvalidRecord);
    }
    ranges
        .try_reserve(1)
        .map_err(|_| CodecError::ResourceLimit)?;
    ranges.push((r.start, r.end));
    Ok(())
}

// Retained allocation accounting.
crate::import_codec::memory::heap_struct!(BlockRange {
    canonical_path,
    block_id,
    start,
    end
});
crate::import_codec::memory::heap_struct!(BoundaryEvidence {
    kind,
    source,
    canonical_path,
    block_id,
    unit_offset,
    label,
    raw_href,
    reason,
    depth
});
crate::import_codec::memory::heap_struct!(Chapter {
    chapter_id,
    label,
    status,
    ranges,
    start_evidence
});
crate::import_codec::memory::heap_enum!(MainFlow {Chapter(value), OutOfChapter {ranges, reason}, Unclassified {ranges, reason, evidence}});
crate::import_codec::memory::heap_struct!(Auxiliary {
    canonical_path,
    ranges
});
crate::import_codec::memory::heap_struct!(VolumeTree {
    main_flow,
    auxiliary
});
crate::import_codec::memory::heap_enum!(NavigationTarget {Resolved {canonical_path, fragment}, Invalid {reason}});
crate::import_codec::memory::heap_struct!(NavigationNode {
    parent_index,
    sibling_index,
    label,
    raw_href,
    target
});
crate::import_codec::memory::heap_struct!(Navigation {
    source,
    resource_path,
    state,
    items
});
crate::import_codec::memory::heap_struct!(ConfirmedBoundary {
    epub_sha256,
    parser_version,
    canonical_path,
    block_id,
    unit_offset,
    label
});
crate::import_codec::memory::heap_struct!(SourceInputDiagnostic {
    source,
    resource_path,
    code,
    reason,
    line,
    column
});
crate::import_codec::memory::heap_enum!(MapIssue {NavigationSourceUnavailable {source, resource_path, reason}, NavigationTargetUnresolved {source, resource_path, label, raw_href, reason}, NavigationTargetAmbiguous {source, resource_path, label, raw_href, reason}, NavigationSourcesConflict {from_path, to_path, reason}, NavigationHeadingConflict {from_path, to_path, reason}, ConfirmedBoundaryStale {canonical_path, block_id, reason}, HeadingFallback {canonical_path, block_id, level}, FileEdgeFallback {canonical_path, block_id}, InputDiagnostic {value}});
crate::import_codec::memory::heap_struct!(StoredMapping {
    epub_sha256,
    parser_version,
    volume,
    navigation,
    boundary_evidence,
    confirmed_boundaries,
    diagnostics
});

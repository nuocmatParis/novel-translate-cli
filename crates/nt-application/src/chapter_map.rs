//! Build provisional, ordered volume chapter maps from verified source ranges.

use std::collections::{BTreeMap, HashMap, HashSet};

use nt_domain::chapter_map::{
    AuxiliaryDocumentCoverage, BoundaryEvidence, BoundaryEvidenceKind, Chapter, ChapterMap,
    ChapterMapDiagnostic, ChapterMapError, ChapterMapInput, CoverageDomain, MainFlowContent,
    MappingStatus, NavigationItem, NavigationSource, NavigationTarget, NavigationTreeState,
    SourceDocument,
};
use nt_domain::source_range::{SourceBlock, SourceBlockId, SourceUnitKind};

#[derive(Clone, Debug)]
struct OrderedBlock<'a> {
    path: &'a str,
    block: &'a SourceBlock,
    start: usize,
    end: usize,
}

#[derive(Clone, Debug)]
struct Candidate {
    position: usize,
    evidence: BoundaryEvidence,
    confirmed: bool,
    supporting: Vec<BoundaryEvidence>,
}

#[derive(Clone, Debug)]
struct ConflictInterval {
    start: usize,
    end: usize,
    reason: String,
    evidence: Vec<BoundaryEvidence>,
}

/// Build a reviewable candidate map. Only a current, exact user boundary is final.
pub fn build_candidate_map(input: &ChapterMapInput) -> Result<ChapterMap, ChapterMapError> {
    if input.parser_version.trim().is_empty() {
        return Err(ChapterMapError::EmptyParserVersion);
    }
    let ordered = ordered_main_blocks(&input.documents)?;
    let total_units = ordered.last().map_or(0, |block| block.end);
    let mut diagnostics = input
        .diagnostics
        .iter()
        .cloned()
        .map(ChapterMapDiagnostic::InputDiagnostic)
        .collect::<Vec<_>>();
    let mut evidence = Vec::new();
    let mut selected = Vec::<Candidate>::new();

    for boundary in &input.confirmed_boundaries {
        match confirmed_position(boundary, input, &ordered) {
            Some(position) => {
                let item = BoundaryEvidence {
                    kind: BoundaryEvidenceKind::Confirmed,
                    source: None,
                    canonical_path: boundary.canonical_path.clone(),
                    block_id: Some(boundary.block_id),
                    unit_offset: Some(boundary.unit_offset),
                    label: boundary.label.clone(),
                    raw_href: None,
                    reason: "confirmed for this EPUB snapshot and parser version".to_owned(),
                    depth: None,
                };
                evidence.push(item.clone());
                selected.push(Candidate {
                    position,
                    evidence: item.clone(),
                    confirmed: true,
                    supporting: vec![item],
                });
            }
            None => diagnostics.push(ChapterMapDiagnostic::ConfirmedBoundaryStale {
                canonical_path: boundary.canonical_path.clone(),
                block_id: boundary.block_id,
                reason:
                    "snapshot, parser, path, source block, domain, or unit offset no longer matches"
                        .to_owned(),
            }),
        }
    }

    let (nav_candidates, mut nav_evidence, mut nav_diagnostics) =
        resolve_navigation(input, &ordered);
    evidence.append(&mut nav_evidence);
    diagnostics.append(&mut nav_diagnostics);
    let mut conflicts = navigation_conflicts(&input.navigation, &nav_candidates, total_units);
    for conflict in &conflicts {
        diagnostics.push(ChapterMapDiagnostic::NavigationSourcesConflict {
            from_path: path_at(&ordered, conflict.start).map(str::to_owned),
            to_path: path_at(&ordered, conflict.end.saturating_sub(1)).map(str::to_owned),
            reason: conflict.reason.clone(),
        });
    }

    let all_headings = heading_candidates(&ordered, &mut evidence);
    let heading_candidates = fallback_heading_candidates(&all_headings);
    let grouped_navigation = group_navigation_by_position(&nav_candidates);
    let active_source_count = nav_candidates
        .iter()
        .filter_map(|candidate| candidate.evidence.source)
        .collect::<HashSet<_>>()
        .len();
    for (position, candidates) in grouped_navigation {
        let nested_only = candidates
            .iter()
            .all(|candidate| candidate.evidence.depth.unwrap_or(0) > 0);
        let nested_heading_match = candidates.iter().any(|navigation| {
            heading_candidates.iter().any(|heading| {
                heading.position == position
                    && normalized_label(navigation.evidence.label.as_deref())
                        == normalized_label(heading.evidence.label.as_deref())
            })
        });
        if nested_only && !nested_heading_match {
            continue;
        }
        if candidates.iter().any(|candidate| {
            candidate
                .evidence
                .label
                .as_deref()
                .is_some_and(is_part_label)
        }) || is_position_conflicted(position, &conflicts)
        {
            continue;
        }
        let sources = candidates
            .iter()
            .filter_map(|candidate| candidate.evidence.source)
            .collect::<HashSet<_>>();
        let duplicate_within_source = sources.iter().any(|source| {
            candidates
                .iter()
                .filter(|candidate| candidate.evidence.source == Some(*source))
                .count()
                > 1
        });
        if duplicate_within_source {
            if let Some((start, end)) =
                local_conflict_bounds(position, &nav_candidates, total_units)
            {
                conflicts.push(ConflictInterval {
                    start,
                    end,
                    reason: "multiple navigation items resolve to the same source boundary"
                        .to_owned(),
                    evidence: candidates
                        .iter()
                        .map(|candidate| candidate.evidence.clone())
                        .collect(),
                });
                diagnostics.push(ChapterMapDiagnostic::NavigationSourcesConflict {
                    from_path: path_at(&ordered, start).map(str::to_owned),
                    to_path: path_at(&ordered, end.saturating_sub(1)).map(str::to_owned),
                    reason: "duplicate navigation targets cannot be aligned one-to-one".to_owned(),
                });
            }
            continue;
        }
        let corroborated = active_source_count > 1 && sources.len() > 1;
        let source_usable = sources.iter().all(|source| {
            input
                .navigation
                .iter()
                .any(|tree| tree.source == *source && tree.state == NavigationTreeState::Usable)
        });
        let mut chosen = candidates[0].clone();
        let labels = candidates
            .iter()
            .filter_map(|candidate| normalized_label(candidate.evidence.label.as_deref()))
            .collect::<HashSet<_>>();
        chosen.supporting = candidates
            .iter()
            .map(|candidate| candidate.evidence.clone())
            .collect();
        if nested_heading_match {
            chosen.supporting.extend(
                heading_candidates
                    .iter()
                    .filter(|heading| {
                        heading.position == position
                            && candidates.iter().any(|navigation| {
                                normalized_label(navigation.evidence.label.as_deref())
                                    == normalized_label(heading.evidence.label.as_deref())
                            })
                    })
                    .map(|heading| heading.evidence.clone()),
            );
        }
        if labels.len() > 1 {
            chosen.evidence.label = None;
            chosen.evidence.reason = "Nav/NCX target positions agree but labels differ; retain each source label without choosing one".to_owned();
            diagnostics.push(ChapterMapDiagnostic::NavigationSourcesConflict {
                from_path: Some(chosen.evidence.canonical_path.clone()),
                to_path: Some(chosen.evidence.canonical_path.clone()),
                reason: "Nav and NCX identify the same source position with different labels"
                    .to_owned(),
            });
        } else {
            chosen.evidence.reason = if nested_heading_match {
                "nested navigation target is corroborated by a same-position heading".to_owned()
            } else if corroborated && source_usable {
                "Nav/NCX targets agree at this source position".to_owned()
            } else {
                "navigation target is a provisional boundary candidate".to_owned()
            };
        }
        if let Some(part) = leading_part_before(&ordered, position) {
            chosen.position = part.start;
            chosen.supporting.push(BoundaryEvidence {
                kind: BoundaryEvidenceKind::Heading,
                source: None,
                canonical_path: part.path.to_owned(),
                block_id: Some(part.block.id),
                unit_offset: Some(0),
                label: chosen.evidence.label.clone(),
                raw_href: None,
                reason: "leading Part marker stays inside the following chapter range".to_owned(),
                depth: None,
            });
            chosen
                .evidence
                .reason
                .push_str("; leading Part marker is retained in this chapter");
        }
        selected.push(chosen);
    }

    let heading_conflicts = navigation_heading_conflicts(&nav_candidates, &all_headings);
    for conflict in &heading_conflicts {
        diagnostics.push(ChapterMapDiagnostic::NavigationHeadingConflict {
            from_path: path_at(&ordered, conflict.start).map(str::to_owned),
            to_path: path_at(&ordered, conflict.end.saturating_sub(1)).map(str::to_owned),
            reason: conflict.reason.clone(),
        });
    }
    conflicts.extend(heading_conflicts);
    conflicts = merge_conflicts(conflicts);
    let nav_has_unresolved = diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic,
            ChapterMapDiagnostic::NavigationTargetUnresolved { .. }
                | ChapterMapDiagnostic::NavigationTargetAmbiguous { .. }
        )
    });
    let nav_positions = selected
        .iter()
        .filter(|candidate| candidate.evidence.kind == BoundaryEvidenceKind::Navigation)
        .map(|candidate| candidate.position)
        .collect::<HashSet<_>>();
    let has_selected_nav = selected
        .iter()
        .any(|candidate| candidate.evidence.kind == BoundaryEvidenceKind::Navigation);
    for heading in &heading_candidates {
        if nav_positions.contains(&heading.position)
            || is_position_conflicted(heading.position, &conflicts)
        {
            continue;
        }
        let has_usable_nav = input
            .navigation
            .iter()
            .any(|tree| tree.state == NavigationTreeState::Usable);
        if has_selected_nav && !nav_has_unresolved && has_usable_nav {
            // A usable TOC is stronger than headings; retain subheadings as evidence only.
            continue;
        }
        let mut candidate = heading.clone();
        candidate.evidence.reason =
            "heading fallback; review before treating it as a chapter".to_owned();
        if let Some(part) = leading_part_before(&ordered, candidate.position) {
            candidate.position = part.start;
            candidate.evidence.canonical_path = part.path.to_owned();
            candidate.evidence.block_id = Some(part.block.id);
            candidate.evidence.unit_offset = Some(0);
            candidate.evidence.reason =
                "heading fallback includes the leading Part marker as chapter content".to_owned();
            candidate.supporting.push(BoundaryEvidence {
                kind: BoundaryEvidenceKind::Heading,
                source: None,
                canonical_path: part.path.to_owned(),
                block_id: Some(part.block.id),
                unit_offset: Some(0),
                label: Some(block_text(part.block)),
                raw_href: None,
                reason: "possible Part heading retained within the following chapter".to_owned(),
                depth: heading.evidence.depth,
            });
        }
        let level = candidate.evidence.depth.unwrap_or(0).saturating_add(1) as u8;
        diagnostics.push(ChapterMapDiagnostic::HeadingFallback {
            canonical_path: candidate.evidence.canonical_path.clone(),
            block_id: candidate
                .evidence
                .block_id
                .expect("heading evidence has a block"),
            level,
        });
        selected.push(candidate);
    }

    if !has_selected_nav {
        add_file_edge_fallbacks(&ordered, &mut evidence, &mut selected, &mut diagnostics);
    }

    let mut by_position = BTreeMap::<usize, Vec<Candidate>>::new();
    for candidate in selected {
        by_position
            .entry(candidate.position)
            .or_default()
            .push(candidate);
    }
    let mut boundaries = Vec::<(usize, Candidate)>::new();
    for (position, candidates) in by_position {
        let mut winner = candidates
            .iter()
            .find(|candidate| candidate.confirmed)
            .cloned()
            .unwrap_or_else(|| candidates[0].clone());
        let mut supporting = candidates
            .iter()
            .flat_map(|candidate| {
                let mut items = candidate.supporting.clone();
                if !items.contains(&candidate.evidence) {
                    items.push(candidate.evidence.clone());
                }
                items
            })
            .collect::<Vec<_>>();
        supporting.dedup();
        winner.supporting = supporting;
        boundaries.push((position, winner));
    }
    boundaries.retain(|(position, _)| *position < total_units);
    conflicts = merge_conflicts(conflicts);

    let main_flow = partition_main_flow(total_units, &ordered, &boundaries, &conflicts);
    let auxiliary = auxiliary_coverage(&input.documents)?;
    let map = ChapterMap {
        epub_sha256: input.epub_sha256,
        parser_version: input.parser_version.clone(),
        volume: nt_domain::chapter_map::VolumeTree {
            main_flow,
            auxiliary,
        },
        navigation: input.navigation.clone(),
        boundary_evidence: evidence,
        diagnostics,
    };
    nt_domain::chapter_map::validate_coverage(input, &map)?;
    Ok(map)
}

fn ordered_main_blocks<'a>(
    documents: &'a [SourceDocument],
) -> Result<Vec<OrderedBlock<'a>>, ChapterMapError> {
    let mut docs = documents
        .iter()
        .enumerate()
        .filter(|(_, document)| document.domain == CoverageDomain::MainFlow)
        .collect::<Vec<_>>();
    docs.sort_by_key(|(index, document)| {
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
    let mut result = Vec::new();
    let mut cursor = 0_usize;
    let mut seen = HashSet::new();
    for (_, document) in docs {
        for block in &document.blocks {
            if block.canonical_path != document.canonical_path {
                return Err(ChapterMapError::BlockPathMismatch);
            }
            if !seen.insert(block.id) {
                return Err(ChapterMapError::DuplicateSourceBlockId);
            }
            if block.unit_count() > u32::MAX as usize {
                return Err(ChapterMapError::InvalidBoundaryRange);
            }
            let end = cursor
                .checked_add(block.unit_count())
                .ok_or(ChapterMapError::InvalidBoundaryRange)?;
            result.push(OrderedBlock {
                path: &document.canonical_path,
                block,
                start: cursor,
                end,
            });
            cursor = end;
        }
    }
    for document in documents
        .iter()
        .filter(|document| document.domain == CoverageDomain::Auxiliary)
    {
        for block in &document.blocks {
            if block.canonical_path != document.canonical_path {
                return Err(ChapterMapError::BlockPathMismatch);
            }
            if !seen.insert(block.id) {
                return Err(ChapterMapError::DuplicateSourceBlockId);
            }
        }
    }
    Ok(result)
}

fn confirmed_position(
    boundary: &nt_domain::chapter_map::ConfirmedBoundary,
    input: &ChapterMapInput,
    ordered: &[OrderedBlock<'_>],
) -> Option<usize> {
    if boundary.epub_sha256 != input.epub_sha256 || boundary.parser_version != input.parser_version
    {
        return None;
    }
    let block = ordered
        .iter()
        .find(|item| item.block.id == boundary.block_id && item.path == boundary.canonical_path)?;
    (boundary.unit_offset as usize <= block.block.unit_count())
        .then_some(block.start + boundary.unit_offset as usize)
}

fn resolve_navigation(
    input: &ChapterMapInput,
    ordered: &[OrderedBlock<'_>],
) -> (
    Vec<Candidate>,
    Vec<BoundaryEvidence>,
    Vec<ChapterMapDiagnostic>,
) {
    let mut candidates = Vec::new();
    let mut evidence = Vec::new();
    let mut diagnostics = Vec::new();
    for tree in &input.navigation {
        if tree.state == NavigationTreeState::Invalid {
            diagnostics.push(ChapterMapDiagnostic::NavigationSourceUnavailable {
                source: Some(tree.source),
                resource_path: Some(tree.resource_path.clone()),
                reason: "the inventory marked this navigation tree invalid".to_owned(),
            });
            continue;
        }
        for item in &tree.items {
            resolve_item(
                tree.source,
                &tree.resource_path,
                item,
                0,
                input,
                ordered,
                &mut candidates,
                &mut evidence,
                &mut diagnostics,
            );
        }
    }
    (candidates, evidence, diagnostics)
}

#[allow(clippy::too_many_arguments)]
fn resolve_item(
    source: NavigationSource,
    source_path: &str,
    item: &NavigationItem,
    depth: usize,
    input: &ChapterMapInput,
    ordered: &[OrderedBlock<'_>],
    candidates: &mut Vec<Candidate>,
    evidence: &mut Vec<BoundaryEvidence>,
    diagnostics: &mut Vec<ChapterMapDiagnostic>,
) {
    if let Some(target) = &item.target {
        match resolve_target(target, input, ordered) {
            Ok((position, path, block_id, offset)) => {
                let is_part = is_part_label(&item.label);
                let boundary = BoundaryEvidence {
                    kind: BoundaryEvidenceKind::Navigation,
                    source: Some(source),
                    canonical_path: path,
                    block_id: Some(block_id),
                    unit_offset: Some(offset),
                    label: Some(item.label.clone()),
                    raw_href: item.raw_href.clone(),
                    reason: if is_part {
                        "Part/content marker retained in the chapter".to_owned()
                    } else {
                        "resolved navigation target".to_owned()
                    },
                    depth: Some(depth),
                };
                if !is_part {
                    candidates.push(Candidate {
                        position,
                        evidence: boundary.clone(),
                        confirmed: false,
                        supporting: vec![boundary.clone()],
                    });
                }
                evidence.push(boundary);
            }
            Err((ambiguous, reason)) => {
                let diagnostic = if ambiguous {
                    ChapterMapDiagnostic::NavigationTargetAmbiguous {
                        source,
                        resource_path: source_path.to_owned(),
                        label: item.label.clone(),
                        raw_href: item.raw_href.clone(),
                        reason: reason.clone(),
                    }
                } else {
                    ChapterMapDiagnostic::NavigationTargetUnresolved {
                        source,
                        resource_path: source_path.to_owned(),
                        label: item.label.clone(),
                        raw_href: item.raw_href.clone(),
                        reason: reason.clone(),
                    }
                };
                diagnostics.push(diagnostic);
                evidence.push(BoundaryEvidence {
                    kind: BoundaryEvidenceKind::Navigation,
                    source: Some(source),
                    canonical_path: source_path.to_owned(),
                    block_id: None,
                    unit_offset: None,
                    label: Some(item.label.clone()),
                    raw_href: item.raw_href.clone(),
                    reason,
                    depth: Some(depth),
                });
            }
        }
    } else if item.raw_href.is_some() {
        let reason = "raw navigation href has no resolved inventory target".to_owned();
        diagnostics.push(ChapterMapDiagnostic::NavigationTargetUnresolved {
            source,
            resource_path: source_path.to_owned(),
            label: item.label.clone(),
            raw_href: item.raw_href.clone(),
            reason: reason.clone(),
        });
        evidence.push(BoundaryEvidence {
            kind: BoundaryEvidenceKind::Navigation,
            source: Some(source),
            canonical_path: source_path.to_owned(),
            block_id: None,
            unit_offset: None,
            label: Some(item.label.clone()),
            raw_href: item.raw_href.clone(),
            reason,
            depth: Some(depth),
        });
    }
    for child in &item.children {
        resolve_item(
            source,
            source_path,
            child,
            depth + 1,
            input,
            ordered,
            candidates,
            evidence,
            diagnostics,
        );
    }
}

fn resolve_target(
    target: &NavigationTarget,
    input: &ChapterMapInput,
    ordered: &[OrderedBlock<'_>],
) -> Result<(usize, String, SourceBlockId, u32), (bool, String)> {
    let (path, fragment) = match target {
        NavigationTarget::Resolved {
            canonical_path,
            fragment,
        } => (canonical_path, fragment.as_deref()),
        NavigationTarget::Invalid { reason } => return Err((false, reason.clone())),
    };
    if let Some(fragment) = fragment.filter(|fragment| !fragment.is_empty()) {
        let matches = input
            .anchors
            .iter()
            .filter(|anchor| anchor.canonical_path == *path && anchor.fragment == fragment)
            .collect::<Vec<_>>();
        if matches.len() != 1 || !matches[0].is_unique {
            return Err(if matches.is_empty() {
                (
                    false,
                    "target fragment has no extracted source anchor".to_owned(),
                )
            } else {
                (
                    true,
                    "target fragment is not unique in the source document".to_owned(),
                )
            });
        }
        let anchor = matches[0];
        let block = ordered
            .iter()
            .find(|item| item.block.id == anchor.block_id && item.path == path)
            .ok_or_else(|| {
                (
                    false,
                    "target anchor is outside the main-flow source domain".to_owned(),
                )
            })?;
        if anchor.unit_offset as usize > block.block.unit_count() {
            return Err((false, "anchor offset exceeds its block".to_owned()));
        }
        return Ok((
            block.start + anchor.unit_offset as usize,
            path.clone(),
            block.block.id,
            anchor.unit_offset,
        ));
    }
    let block = ordered
        .iter()
        .find(|item| item.path == path && item.block.unit_count() > 0)
        .ok_or_else(|| {
            (
                false,
                "target document has no main-flow source units".to_owned(),
            )
        })?;
    Ok((block.start, path.clone(), block.block.id, 0))
}

fn navigation_conflicts(
    trees: &[nt_domain::chapter_map::NavigationTree],
    candidates: &[Candidate],
    total: usize,
) -> Vec<ConflictInterval> {
    let nav = candidates
        .iter()
        .filter(|item| item.evidence.source == Some(NavigationSource::Epub3Nav))
        .collect::<Vec<_>>();
    let ncx = candidates
        .iter()
        .filter(|item| item.evidence.source == Some(NavigationSource::Epub2Ncx))
        .collect::<Vec<_>>();
    if nav.is_empty() || ncx.is_empty() {
        return Vec::new();
    }
    let nav_positions = nav.iter().map(|item| item.position).collect::<HashSet<_>>();
    let ncx_positions = ncx.iter().map(|item| item.position).collect::<HashSet<_>>();
    let mut shared = nav_positions
        .intersection(&ncx_positions)
        .copied()
        .collect::<Vec<_>>();
    shared.sort_unstable();
    let mut unmatched = nav_positions
        .symmetric_difference(&ncx_positions)
        .copied()
        .collect::<Vec<_>>();
    unmatched.extend(duplicate_positions(&nav));
    unmatched.extend(duplicate_positions(&ncx));
    unmatched.sort_unstable();
    unmatched.dedup();
    let mut conflicts = Vec::new();
    for position in unmatched {
        let start = shared
            .iter()
            .copied()
            .filter(|value| *value < position)
            .max()
            .unwrap_or(0);
        let end = shared
            .iter()
            .copied()
            .filter(|value| *value > position)
            .min()
            .unwrap_or(total);
        if end > start {
            conflicts.push(ConflictInterval { start, end,
                reason: "EPUB 3 Nav and EPUB 2 NCX cannot be aligned one-to-one; keep both alternatives for review".to_owned(),
                evidence: candidates.iter().filter(|item| item.position >= start && item.position <= end)
                    .map(|item| item.evidence.clone()).collect() });
        }
    }
    for item in &nav {
        let Some(label) = normalized_label(item.evidence.label.as_deref()) else {
            continue;
        };
        let left = nav
            .iter()
            .filter(|candidate| {
                normalized_label(candidate.evidence.label.as_deref()).as_deref()
                    == Some(label.as_str())
            })
            .collect::<Vec<_>>();
        let right = ncx
            .iter()
            .filter(|candidate| {
                normalized_label(candidate.evidence.label.as_deref()).as_deref()
                    == Some(label.as_str())
            })
            .collect::<Vec<_>>();
        if left.len() == 1 && right.len() == 1 && left[0].position != right[0].position {
            let start = left[0].position.min(right[0].position);
            let end = left[0].position.max(right[0].position);
            conflicts.push(ConflictInterval {
                start,
                end,
                reason: "the same navigation label resolves to different source positions"
                    .to_owned(),
                evidence: vec![left[0].evidence.clone(), right[0].evidence.clone()],
            });
        }
        if left.len() > 1 || right.len() > 1 {
            let positions = left
                .iter()
                .chain(right.iter())
                .map(|candidate| candidate.position)
                .collect::<Vec<_>>();
            let bounds = positions
                .iter()
                .min()
                .zip(positions.iter().max())
                .filter(|(start, end)| end > start);
            if let Some((start, end)) = bounds {
                conflicts.push(ConflictInterval {
                    start: *start,
                    end: *end,
                    reason: "repeated navigation labels cannot identify a one-to-one boundary"
                        .to_owned(),
                    evidence: left
                        .iter()
                        .chain(right.iter())
                        .map(|candidate| candidate.evidence.clone())
                        .collect(),
                });
            }
        }
    }
    let both_available = trees.iter().any(|tree| {
        tree.source == NavigationSource::Epub3Nav && tree.state != NavigationTreeState::Invalid
    }) && trees.iter().any(|tree| {
        tree.source == NavigationSource::Epub2Ncx && tree.state != NavigationTreeState::Invalid
    });
    if both_available && nav_positions == ncx_positions {
        // Equal source positions corroborate each other; labels remain separate provenance.
    }
    merge_conflicts(conflicts)
}

fn duplicate_positions(candidates: &[&Candidate]) -> Vec<usize> {
    let mut counts = HashMap::<usize, usize>::new();
    for candidate in candidates {
        *counts.entry(candidate.position).or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(position, count)| (count > 1).then_some(position))
        .collect()
}

fn group_navigation_by_position(candidates: &[Candidate]) -> BTreeMap<usize, Vec<Candidate>> {
    let mut groups = BTreeMap::new();
    for candidate in candidates {
        groups
            .entry(candidate.position)
            .or_insert_with(Vec::new)
            .push(candidate.clone());
    }
    groups
}

fn local_conflict_bounds(
    position: usize,
    candidates: &[Candidate],
    total: usize,
) -> Option<(usize, usize)> {
    let start = candidates
        .iter()
        .map(|item| item.position)
        .filter(|item| *item < position)
        .max()
        .unwrap_or(0);
    let end = candidates
        .iter()
        .map(|item| item.position)
        .filter(|item| *item > position)
        .min()
        .unwrap_or(total);
    (end > start).then_some((start, end))
}

fn is_position_conflicted(position: usize, conflicts: &[ConflictInterval]) -> bool {
    conflicts
        .iter()
        .any(|item| position >= item.start && position < item.end)
}

fn heading_candidates(
    ordered: &[OrderedBlock<'_>],
    evidence: &mut Vec<BoundaryEvidence>,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for block in ordered {
        let Some(level) = heading_level_name(&block.block.element_name) else {
            continue;
        };
        let label = block_text(block.block);
        let is_part = is_part_label(&label);
        let item = BoundaryEvidence {
            kind: BoundaryEvidenceKind::Heading,
            source: None,
            canonical_path: block.path.to_owned(),
            block_id: Some(block.block.id),
            unit_offset: Some(0),
            label: Some(label),
            raw_href: None,
            reason: if is_part {
                "possible Part heading retained as content; not promoted to chapter".to_owned()
            } else {
                format!("XHTML heading h{level}; fallback evidence")
            },
            depth: Some(level.saturating_sub(1) as usize),
        };
        evidence.push(item.clone());
        if !is_part && block.start < block.end {
            candidates.push(Candidate {
                position: block.start,
                evidence: item.clone(),
                confirmed: false,
                supporting: vec![item],
            });
        }
    }

    candidates
}

fn fallback_heading_candidates(candidates: &[Candidate]) -> Vec<Candidate> {
    let mut frequency = HashMap::<u8, usize>::new();
    for candidate in candidates {
        let level = candidate.evidence.depth.unwrap_or(0) as u8 + 1;
        *frequency.entry(level).or_default() += 1;
    }
    let preferred_level = frequency
        .iter()
        .max_by_key(|(level, count)| (**count, std::cmp::Reverse(**level)))
        .map(|(level, _)| *level);
    candidates
        .iter()
        .filter(|candidate| {
            preferred_level == Some(candidate.evidence.depth.unwrap_or(0) as u8 + 1)
        })
        .cloned()
        .collect()
}

fn leading_part_before<'a, 'b>(
    ordered: &'a [OrderedBlock<'b>],
    position: usize,
) -> Option<&'a OrderedBlock<'b>> {
    let prefix = ordered
        .iter()
        .filter(|block| block.start < position && block.end <= position)
        .collect::<Vec<_>>();
    if prefix.is_empty()
        || prefix.iter().any(|block| {
            heading_level_name(&block.block.element_name).is_none()
                || !is_part_label(&block_text(block.block))
        })
    {
        return None;
    }
    prefix.first().copied()
}
fn navigation_heading_conflicts(
    nav: &[Candidate],
    headings: &[Candidate],
) -> Vec<ConflictInterval> {
    let mut conflicts = Vec::new();
    let mut compared_labels = HashSet::new();
    for navigation in nav {
        let Some(label) = normalized_label(navigation.evidence.label.as_deref()) else {
            continue;
        };
        if !compared_labels.insert(label.clone()) {
            continue;
        }
        let matching_nav = nav
            .iter()
            .filter(|candidate| {
                normalized_label(candidate.evidence.label.as_deref()).as_deref()
                    == Some(label.as_str())
            })
            .collect::<Vec<_>>();
        let matching_headings = headings
            .iter()
            .filter(|candidate| {
                normalized_label(candidate.evidence.label.as_deref()).as_deref()
                    == Some(label.as_str())
            })
            .collect::<Vec<_>>();
        let positions = matching_nav
            .iter()
            .map(|candidate| candidate.position)
            .collect::<HashSet<_>>();
        let sources = matching_nav
            .iter()
            .filter_map(|candidate| candidate.evidence.source)
            .collect::<HashSet<_>>();
        if positions.len() == 1
            && sources.len() == matching_nav.len()
            && matching_headings.len() == 1
            && matching_nav[0].position != matching_headings[0].position
        {
            let start = matching_nav[0].position.min(matching_headings[0].position);
            let end = matching_nav[0].position.max(matching_headings[0].position);
            let mut supporting = matching_nav
                .iter()
                .map(|candidate| candidate.evidence.clone())
                .collect::<Vec<_>>();
            supporting.push(matching_headings[0].evidence.clone());
            conflicts.push(ConflictInterval {
                start,
                end,
                reason: "navigation target and same-title XHTML heading point to different source positions".to_owned(),
                evidence: supporting,
            });
        }
    }
    merge_conflicts(conflicts)
}

fn add_file_edge_fallbacks(
    ordered: &[OrderedBlock<'_>],
    evidence: &mut Vec<BoundaryEvidence>,
    selected: &mut Vec<Candidate>,
    diagnostics: &mut Vec<ChapterMapDiagnostic>,
) {
    let mut seen = HashSet::new();
    for block in ordered.iter().filter(|block| block.start < block.end) {
        if !seen.insert(block.path) {
            continue;
        }
        let item = BoundaryEvidence {
            kind: BoundaryEvidenceKind::FileEdge,
            source: None,
            canonical_path: block.path.to_owned(),
            block_id: Some(block.block.id),
            unit_offset: Some(0),
            label: None,
            raw_href: None,
            reason: "file edge is the last provisional fallback, not a confirmed boundary"
                .to_owned(),
            depth: None,
        };
        evidence.push(item.clone());
        diagnostics.push(ChapterMapDiagnostic::FileEdgeFallback {
            canonical_path: block.path.to_owned(),
            block_id: block.block.id,
        });
        selected.push(Candidate {
            position: block.start,
            evidence: item.clone(),
            confirmed: false,
            supporting: vec![item],
        });
    }
}

fn auxiliary_coverage(
    documents: &[SourceDocument],
) -> Result<Vec<AuxiliaryDocumentCoverage>, ChapterMapError> {
    let mut result = Vec::new();
    for document in documents
        .iter()
        .filter(|document| document.domain == CoverageDomain::Auxiliary)
    {
        let mut ranges = Vec::new();
        for block in &document.blocks {
            if block.canonical_path != document.canonical_path {
                return Err(ChapterMapError::BlockPathMismatch);
            }
            let count = block.unit_count();
            if count > u32::MAX as usize {
                return Err(ChapterMapError::InvalidBoundaryRange);
            }
            if count > 0 {
                ranges.push(nt_domain::chapter_map::BlockUnitRange {
                    canonical_path: document.canonical_path.clone(),
                    block_id: block.id,
                    start: 0,
                    end: count as u32,
                });
            }
        }
        result.push(AuxiliaryDocumentCoverage {
            canonical_path: document.canonical_path.clone(),
            ranges,
        });
    }
    Ok(result)
}

fn partition_main_flow(
    total: usize,
    ordered: &[OrderedBlock<'_>],
    boundaries: &[(usize, Candidate)],
    conflicts: &[ConflictInterval],
) -> Vec<MainFlowContent> {
    if total == 0 {
        return Vec::new();
    }
    let mut cuts = vec![0, total];
    cuts.extend(boundaries.iter().map(|(position, _)| *position));
    for conflict in conflicts {
        cuts.push(conflict.start.min(total));
        cuts.push(conflict.end.min(total));
    }
    cuts.sort_unstable();
    cuts.dedup();
    let mut result = Vec::new();
    let mut active: Option<(u32, Candidate)> = None;
    let mut next_id = 0_u32;
    let mut index = 0;
    for pair in cuts.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        while index < boundaries.len() && boundaries[index].0 <= start {
            active = Some((next_id, boundaries[index].1.clone()));
            next_id = next_id.saturating_add(1);
            index += 1;
        }
        if start >= end {
            continue;
        }
        let ranges = ranges_for_span(start, end, ordered);
        if ranges.is_empty() {
            continue;
        }
        if let Some((id, candidate)) = active.as_ref().filter(|(_, candidate)| candidate.confirmed)
        {
            append_chapter_segment(&mut result, *id, candidate, ranges);
        } else if let Some(conflict) = conflicts
            .iter()
            .find(|item| start >= item.start && end <= item.end)
        {
            append_unclassified(&mut result, ranges, conflict);
        } else if let Some((id, candidate)) = &active {
            append_chapter_segment(&mut result, *id, candidate, ranges);
        } else {
            append_out_of_chapter(&mut result, ranges);
        }
    }
    result
}

fn append_chapter_segment(
    result: &mut Vec<MainFlowContent>,
    id: u32,
    candidate: &Candidate,
    ranges: Vec<nt_domain::chapter_map::BlockUnitRange>,
) {
    let status = if candidate.confirmed {
        MappingStatus::Confirmed
    } else {
        MappingStatus::Provisional
    };
    match result.last_mut() {
        Some(MainFlowContent::Chapter(chapter))
            if chapter.chapter_id == id && chapter.status == status =>
        {
            extend_ranges(&mut chapter.ranges, ranges);
            return;
        }
        _ => {}
    }
    result.push(MainFlowContent::Chapter(Chapter {
        chapter_id: id,
        label: candidate.evidence.label.clone(),
        status,
        ranges,
        start_evidence: if candidate.supporting.is_empty() {
            vec![candidate.evidence.clone()]
        } else {
            candidate.supporting.clone()
        },
    }));
}

fn append_out_of_chapter(
    result: &mut Vec<MainFlowContent>,
    ranges: Vec<nt_domain::chapter_map::BlockUnitRange>,
) {
    if let Some(MainFlowContent::OutOfChapter { ranges: old, .. }) = result.last_mut() {
        extend_ranges(old, ranges);
    } else {
        result.push(MainFlowContent::OutOfChapter {
            ranges,
            reason: "content precedes the first mapped boundary".to_owned(),
        });
    }
}

fn append_unclassified(
    result: &mut Vec<MainFlowContent>,
    ranges: Vec<nt_domain::chapter_map::BlockUnitRange>,
    conflict: &ConflictInterval,
) {
    match result.last_mut() {
        Some(MainFlowContent::Unclassified {
            ranges: old,
            reason,
            evidence,
        }) if *reason == conflict.reason => {
            extend_ranges(old, ranges);
            for item in &conflict.evidence {
                if !evidence.contains(item) {
                    evidence.push(item.clone());
                }
            }
            return;
        }
        _ => {}
    }
    result.push(MainFlowContent::Unclassified {
        ranges,
        reason: conflict.reason.clone(),
        evidence: conflict.evidence.clone(),
    });
}

fn extend_ranges(
    old: &mut Vec<nt_domain::chapter_map::BlockUnitRange>,
    additions: Vec<nt_domain::chapter_map::BlockUnitRange>,
) {
    for range in additions {
        match old.last_mut() {
            Some(last)
                if last.block_id == range.block_id
                    && last.canonical_path == range.canonical_path
                    && last.end == range.start =>
            {
                last.end = range.end;
                continue;
            }
            _ => {}
        }
        old.push(range);
    }
}

fn ranges_for_span(
    start: usize,
    end: usize,
    ordered: &[OrderedBlock<'_>],
) -> Vec<nt_domain::chapter_map::BlockUnitRange> {
    ordered
        .iter()
        .filter_map(|block| {
            let left = start.max(block.start);
            let right = end.min(block.end);
            (left < right).then(|| nt_domain::chapter_map::BlockUnitRange {
                canonical_path: block.path.to_owned(),
                block_id: block.block.id,
                start: (left - block.start) as u32,
                end: (right - block.start) as u32,
            })
        })
        .collect()
}

fn merge_conflicts(mut conflicts: Vec<ConflictInterval>) -> Vec<ConflictInterval> {
    conflicts.retain(|item| item.start < item.end);
    conflicts.sort_by_key(|item| (item.start, item.end));
    let mut result = Vec::<ConflictInterval>::new();
    for conflict in conflicts {
        match result.last_mut() {
            Some(last) if conflict.start <= last.end => {
                last.end = last.end.max(conflict.end);
                if last.reason != conflict.reason {
                    last.reason.push_str("; ");
                    last.reason.push_str(&conflict.reason);
                }
                for item in conflict.evidence {
                    if !last.evidence.contains(&item) {
                        last.evidence.push(item);
                    }
                }
                continue;
            }
            _ => {}
        }
        result.push(conflict);
    }
    result
}

fn path_at<'a>(ordered: &[OrderedBlock<'a>], position: usize) -> Option<&'a str> {
    ordered
        .iter()
        .find(|block| position >= block.start && position < block.end)
        .map(|block| block.path)
}

fn block_text(block: &SourceBlock) -> String {
    block
        .units
        .iter()
        .filter_map(|unit| match &unit.kind {
            SourceUnitKind::Grapheme(text) => Some(text.as_str()),
            SourceUnitKind::Protected(_) => None,
        })
        .collect::<String>()
        .trim()
        .to_owned()
}

fn heading_level_name(name: &str) -> Option<u8> {
    let bytes = name.as_bytes();
    if bytes.len() == 2 && bytes[0] == b'h' && (b'1'..=b'6').contains(&bytes[1]) {
        Some(bytes[1] - b'0')
    } else {
        None
    }
}

fn is_part_label(label: &str) -> bool {
    let value = label.trim().to_lowercase();
    value.starts_with("part ")
        || value.starts_with("part:")
        || value.starts_with("phần ")
        || (value.starts_with('第') && value.ends_with('部'))
}

fn normalized_label(label: Option<&str>) -> Option<String> {
    let value = label?
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::build_candidate_map;
    use nt_domain::chapter_map::{
        ChapterMapInput, ConfirmedBoundary, CoverageDomain, MainFlowContent, NavigationItem,
        NavigationSource, NavigationTarget, NavigationTree, NavigationTreeState, SourceAnchor,
        SourceDocument,
    };
    use nt_domain::source_range::{
        BlockLocator, ImageSource, ProtectedSourceUnit, SourceBlock, SourceBlockId, SourceUnit,
    };

    fn block(path: &str, byte: u8, element: &str, units: Vec<SourceUnit>) -> SourceBlock {
        SourceBlock {
            id: SourceBlockId::from_bytes([byte; 32]),
            canonical_path: path.to_owned(),
            element_name: element.to_owned(),
            locator: BlockLocator::UniqueXhtmlId(format!("id-{byte}")),
            parser_version: "nt-epub/1.2.0".to_owned(),
            content_hash: [byte; 32],
            units,
            inline_metadata: Vec::new(),
            attribute_text: Vec::new(),
            protected_resource_hashes: Vec::new(),
        }
    }
    fn text(value: &str) -> Vec<SourceUnit> {
        value
            .chars()
            .map(|c| SourceUnit::grapheme(c.to_string(), Vec::new()))
            .collect()
    }
    fn doc(path: &str, position: usize, blocks: Vec<SourceBlock>) -> SourceDocument {
        SourceDocument {
            canonical_path: path.to_owned(),
            domain: CoverageDomain::MainFlow,
            spine_positions: vec![position],
            linear_spine_positions: vec![position],
            blocks,
        }
    }
    fn item(label: &str, path: &str, fragment: &str) -> NavigationItem {
        NavigationItem {
            label: label.to_owned(),
            raw_href: Some(format!("{path}#{fragment}")),
            target: Some(NavigationTarget::Resolved {
                canonical_path: path.to_owned(),
                fragment: Some(fragment.to_owned()),
            }),
            children: Vec::new(),
        }
    }
    fn tree(source: NavigationSource, items: Vec<NavigationItem>) -> NavigationTree {
        NavigationTree {
            source,
            resource_path: match source {
                NavigationSource::Epub3Nav => "EPUB/nav.xhtml",
                NavigationSource::Epub2Ncx => "EPUB/toc.ncx",
            }
            .to_owned(),
            state: NavigationTreeState::Usable,
            items,
        }
    }
    fn input(documents: Vec<SourceDocument>) -> ChapterMapInput {
        ChapterMapInput {
            epub_sha256: [9; 32],
            parser_version: "nt-epub/1.2.0".to_owned(),
            documents,
            anchors: Vec::new(),
            navigation: Vec::new(),
            confirmed_boundaries: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
    fn anchor(path: &str, fragment: &str, block: &SourceBlock, offset: u32) -> SourceAnchor {
        SourceAnchor {
            canonical_path: path.to_owned(),
            fragment: fragment.to_owned(),
            block_id: block.id,
            unit_offset: offset,
            is_unique: true,
        }
    }

    #[test]
    fn maps_a_chapter_across_files_and_multiple_chapters_in_one_file_with_inline_image_coverage() {
        let a_heading = block("EPUB/a.xhtml", 1, "h1", text("One"));
        let image = SourceUnit::protected(
            ProtectedSourceUnit::Image(ImageSource {
                raw_source: Some("../img.png".to_owned()),
                resource_path: Some("EPUB/img.png".to_owned()),
                alt: Some("cover".to_owned()),
                occurrence: 0,
                node_path: vec![0],
                external: false,
            }),
            Vec::new(),
        );
        let a_body = block("EPUB/a.xhtml", 2, "p", vec![text("A")[0].clone(), image]);
        let b_continue = block("EPUB/b.xhtml", 3, "p", text("BC"));
        let b_two = block("EPUB/b.xhtml", 4, "h1", text("Two"));
        let b_three = block("EPUB/b.xhtml", 5, "h1", text("Three"));
        let b_tail = block("EPUB/b.xhtml", 6, "p", text("Z"));
        let mut input = input(vec![
            doc("EPUB/a.xhtml", 0, vec![a_heading.clone(), a_body.clone()]),
            doc(
                "EPUB/b.xhtml",
                1,
                vec![
                    b_continue.clone(),
                    b_two.clone(),
                    b_three.clone(),
                    b_tail.clone(),
                ],
            ),
        ]);
        input.anchors = vec![
            anchor("EPUB/a.xhtml", "one", &a_heading, 0),
            anchor("EPUB/b.xhtml", "two", &b_two, 0),
            anchor("EPUB/b.xhtml", "three", &b_three, 0),
        ];
        input.navigation = vec![tree(
            NavigationSource::Epub3Nav,
            vec![
                item("One", "EPUB/a.xhtml", "one"),
                item("Two", "EPUB/b.xhtml", "two"),
                item("Three", "EPUB/b.xhtml", "three"),
            ],
        )];
        let map = build_candidate_map(&input).unwrap();
        let chapters = map
            .volume
            .main_flow
            .iter()
            .filter_map(|item| match item {
                MainFlowContent::Chapter(chapter) => Some(chapter),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].label.as_deref(), Some("One"));
        assert!(
            chapters[0]
                .ranges
                .iter()
                .any(|range| range.block_id == a_body.id && range.end == 2)
        );
        assert!(
            chapters[0]
                .ranges
                .iter()
                .any(|range| range.block_id == b_continue.id)
        );
        assert_eq!(chapters[1].label.as_deref(), Some("Two"));
        assert_eq!(chapters[2].label.as_deref(), Some("Three"));
    }

    #[test]
    fn nav_ncx_conflict_preserves_both_sources_and_marks_the_region_unclassified() {
        let first = block("EPUB/a.xhtml", 10, "h1", text("Start"));
        let second = block("EPUB/a.xhtml", 11, "h1", text("Chapter A"));
        let third = block("EPUB/a.xhtml", 12, "h1", text("Chapter B"));
        let mut input = input(vec![doc(
            "EPUB/a.xhtml",
            0,
            vec![first.clone(), second.clone(), third.clone()],
        )]);
        input.anchors = vec![
            anchor("EPUB/a.xhtml", "start", &first, 0),
            anchor("EPUB/a.xhtml", "a", &second, 0),
            anchor("EPUB/a.xhtml", "b", &third, 0),
        ];
        input.navigation = vec![
            tree(
                NavigationSource::Epub3Nav,
                vec![
                    item("Start", "EPUB/a.xhtml", "start"),
                    item("Chapter", "EPUB/a.xhtml", "a"),
                ],
            ),
            tree(
                NavigationSource::Epub2Ncx,
                vec![
                    item("Start", "EPUB/a.xhtml", "start"),
                    item("Chapter", "EPUB/a.xhtml", "b"),
                ],
            ),
        ];
        let map = build_candidate_map(&input).unwrap();
        assert_eq!(map.navigation.len(), 2);
        assert!(map.diagnostics.iter().any(|item| matches!(
            item,
            nt_domain::chapter_map::ChapterMapDiagnostic::NavigationSourcesConflict { .. }
        )));
        assert!(
            map.volume
                .main_flow
                .iter()
                .any(|item| matches!(item, MainFlowContent::Unclassified { .. }))
        );
        assert!(
            map.boundary_evidence
                .iter()
                .any(|item| item.source == Some(NavigationSource::Epub3Nav))
        );
        assert!(
            map.boundary_evidence
                .iter()
                .any(|item| item.source == Some(NavigationSource::Epub2Ncx))
        );
    }

    #[test]
    fn broken_target_falls_back_to_heading_and_stale_confirmation_is_reported() {
        let part = block("EPUB/a.xhtml", 20, "h1", text("Part I"));
        let chapter = block("EPUB/a.xhtml", 21, "h1", text("Chapter One"));
        let mut input = input(vec![doc(
            "EPUB/a.xhtml",
            0,
            vec![part.clone(), chapter.clone()],
        )]);
        input.navigation = vec![NavigationTree {
            source: NavigationSource::Epub3Nav,
            resource_path: "EPUB/nav.xhtml".to_owned(),
            state: NavigationTreeState::Partial,
            items: vec![NavigationItem {
                label: "Broken".to_owned(),
                raw_href: Some("../outside.xhtml#x".to_owned()),
                target: Some(NavigationTarget::Invalid {
                    reason: "path escapes package root".to_owned(),
                }),
                children: Vec::new(),
            }],
        }];
        input.confirmed_boundaries.push(ConfirmedBoundary {
            epub_sha256: [0; 32],
            parser_version: input.parser_version.clone(),
            canonical_path: "EPUB/a.xhtml".to_owned(),
            block_id: chapter.id,
            unit_offset: 0,
            label: Some("Stale".to_owned()),
        });
        let map = build_candidate_map(&input).unwrap();
        assert!(map.diagnostics.iter().any(|item| matches!(
            item,
            nt_domain::chapter_map::ChapterMapDiagnostic::ConfirmedBoundaryStale { .. }
        )));
        assert!(map.diagnostics.iter().any(|item| matches!(item, nt_domain::chapter_map::ChapterMapDiagnostic::NavigationTargetUnresolved { raw_href: Some(href), .. } if href == "../outside.xhtml#x")));
        assert!(map.volume.main_flow.iter().any(|item| matches!(item, MainFlowContent::Chapter(chapter) if chapter.label.as_deref() == Some("Chapter One") && chapter.status == nt_domain::chapter_map::MappingStatus::Provisional)));
        assert!(!map.volume.main_flow.iter().any(|item| matches!(item, MainFlowContent::Chapter(chapter) if chapter.label.as_deref() == Some("Part I"))));
    }

    #[test]
    fn confirmed_boundary_wins_inside_navigation_source_conflict() {
        let source = block("EPUB/a.xhtml", 35, "p", text("abcdefgh"));
        let mut input = input(vec![doc("EPUB/a.xhtml", 0, vec![source.clone()])]);
        input.anchors = vec![
            anchor("EPUB/a.xhtml", "nav-start", &source, 0),
            anchor("EPUB/a.xhtml", "ncx-start", &source, 4),
        ];
        input.navigation = vec![
            tree(
                NavigationSource::Epub3Nav,
                vec![item("Chapter", "EPUB/a.xhtml", "nav-start")],
            ),
            tree(
                NavigationSource::Epub2Ncx,
                vec![item("Chapter", "EPUB/a.xhtml", "ncx-start")],
            ),
        ];
        input.confirmed_boundaries.push(ConfirmedBoundary {
            epub_sha256: input.epub_sha256,
            parser_version: input.parser_version.clone(),
            canonical_path: "EPUB/a.xhtml".to_owned(),
            block_id: source.id,
            unit_offset: 0,
            label: Some("Confirmed".to_owned()),
        });

        let map = build_candidate_map(&input).unwrap();
        assert!(map.volume.main_flow.iter().any(|item| matches!(
            item,
            MainFlowContent::Chapter(chapter)
                if chapter.status == nt_domain::chapter_map::MappingStatus::Confirmed
                    && chapter.label.as_deref() == Some("Confirmed")
                    && chapter.ranges.iter().any(|range| range.start == 0 && range.end == 8)
        )));
        assert!(map.diagnostics.iter().any(|item| matches!(
            item,
            nt_domain::chapter_map::ChapterMapDiagnostic::NavigationSourcesConflict { .. }
        )));
    }

    #[test]
    fn confirmed_boundary_wins_inside_navigation_heading_conflict() {
        let heading = block("EPUB/a.xhtml", 36, "h1", text("Chapter One"));
        let later = block("EPUB/a.xhtml", 37, "p", text("continued"));
        let mut input = input(vec![doc(
            "EPUB/a.xhtml",
            0,
            vec![heading.clone(), later.clone()],
        )]);
        input.anchors = vec![anchor("EPUB/a.xhtml", "later", &later, 0)];
        input.navigation = vec![tree(
            NavigationSource::Epub3Nav,
            vec![item("Chapter One", "EPUB/a.xhtml", "later")],
        )];
        input.confirmed_boundaries.push(ConfirmedBoundary {
            epub_sha256: input.epub_sha256,
            parser_version: input.parser_version.clone(),
            canonical_path: "EPUB/a.xhtml".to_owned(),
            block_id: heading.id,
            unit_offset: 0,
            label: Some("Confirmed heading".to_owned()),
        });

        let map = build_candidate_map(&input).unwrap();
        assert!(map.volume.main_flow.iter().any(|item| matches!(
            item,
            MainFlowContent::Chapter(chapter)
                if chapter.status == nt_domain::chapter_map::MappingStatus::Confirmed
                    && chapter.label.as_deref() == Some("Confirmed heading")
        )));
        assert!(map.diagnostics.iter().any(|item| matches!(
            item,
            nt_domain::chapter_map::ChapterMapDiagnostic::NavigationHeadingConflict { .. }
        )));
    }
    #[test]
    fn navigation_heading_conflict_keeps_both_positions_for_review() {
        let heading = block("EPUB/a.xhtml", 25, "h1", text("Chapter One"));
        let later_target = block("EPUB/a.xhtml", 26, "p", text("continued"));
        let mut input = input(vec![doc(
            "EPUB/a.xhtml",
            0,
            vec![heading.clone(), later_target.clone()],
        )]);
        input.anchors = vec![anchor("EPUB/a.xhtml", "later", &later_target, 0)];
        input.navigation = vec![tree(
            NavigationSource::Epub3Nav,
            vec![item("Chapter One", "EPUB/a.xhtml", "later")],
        )];

        let map = build_candidate_map(&input).unwrap();
        assert!(map.diagnostics.iter().any(|item| matches!(
            item,
            nt_domain::chapter_map::ChapterMapDiagnostic::NavigationHeadingConflict { .. }
        )));
        assert!(map.volume.main_flow.iter().any(
            |item| matches!(item, MainFlowContent::Unclassified { evidence, .. }
            if evidence.iter().any(|evidence| evidence.block_id == Some(heading.id))
                && evidence.iter().any(|evidence| evidence.block_id == Some(later_target.id)))
        ));
    }
    #[test]
    fn aligned_targets_with_different_labels_keep_both_alternatives_without_choosing_a_label() {
        let heading = block("EPUB/a.xhtml", 27, "h1", text("Heading"));
        let mut input = input(vec![doc("EPUB/a.xhtml", 0, vec![heading.clone()])]);
        input.anchors = vec![anchor("EPUB/a.xhtml", "start", &heading, 0)];
        input.navigation = vec![
            tree(
                NavigationSource::Epub3Nav,
                vec![item("Chapter One", "EPUB/a.xhtml", "start")],
            ),
            tree(
                NavigationSource::Epub2Ncx,
                vec![item("Main Story", "EPUB/a.xhtml", "start")],
            ),
        ];

        let map = build_candidate_map(&input).unwrap();
        let chapter = map
            .volume
            .main_flow
            .iter()
            .find_map(|item| match item {
                MainFlowContent::Chapter(chapter) => Some(chapter),
                _ => None,
            })
            .unwrap();
        assert_eq!(chapter.label, None);
        assert!(
            chapter
                .start_evidence
                .iter()
                .any(|item| item.source == Some(NavigationSource::Epub3Nav))
        );
        assert!(
            chapter
                .start_evidence
                .iter()
                .any(|item| item.source == Some(NavigationSource::Epub2Ncx))
        );
        assert!(map.diagnostics.iter().any(|item| matches!(item,
            nt_domain::chapter_map::ChapterMapDiagnostic::NavigationSourcesConflict { reason, .. }
                if reason.contains("different labels"))));
    }

    #[test]
    fn nested_navigation_item_needs_same_position_heading_before_it_becomes_a_chapter() {
        let part = block("EPUB/a.xhtml", 28, "h1", text("Part I"));
        let chapter = block("EPUB/a.xhtml", 29, "h2", text("Chapter One"));
        let mut input = input(vec![doc(
            "EPUB/a.xhtml",
            0,
            vec![part.clone(), chapter.clone()],
        )]);
        input.anchors = vec![anchor("EPUB/a.xhtml", "chapter", &chapter, 0)];
        input.navigation = vec![tree(
            NavigationSource::Epub3Nav,
            vec![NavigationItem {
                label: "Part I".to_owned(),
                raw_href: None,
                target: None,
                children: vec![item("Chapter One", "EPUB/a.xhtml", "chapter")],
            }],
        )];

        let map = build_candidate_map(&input).unwrap();
        let chapter_map = map
            .volume
            .main_flow
            .iter()
            .find_map(|item| match item {
                MainFlowContent::Chapter(chapter) => Some(chapter),
                _ => None,
            })
            .unwrap();
        assert_eq!(chapter_map.label.as_deref(), Some("Chapter One"));
        assert!(
            chapter_map
                .ranges
                .iter()
                .any(|range| range.block_id == part.id)
        );
        assert!(
            chapter_map
                .start_evidence
                .iter()
                .any(|item| item.source == Some(NavigationSource::Epub3Nav))
        );
        assert!(
            chapter_map
                .start_evidence
                .iter()
                .any(|item| item.kind == nt_domain::chapter_map::BoundaryEvidenceKind::Heading)
        );
    }

    #[test]
    fn missing_navigation_uses_heading_then_file_edge_as_reviewable_fallback() {
        let heading = block("EPUB/a.xhtml", 33, "h1", text("Chapter One"));
        let body = block("EPUB/a.xhtml", 34, "p", text("Text"));
        let input = input(vec![doc("EPUB/a.xhtml", 0, vec![heading, body])]);
        let map = build_candidate_map(&input).unwrap();
        assert!(map.volume.main_flow.iter().any(|item| matches!(item,
            MainFlowContent::Chapter(chapter) if chapter.label.as_deref() == Some("Chapter One")
                && chapter.status == nt_domain::chapter_map::MappingStatus::Provisional)));
        assert!(map.diagnostics.iter().any(|item| matches!(
            item,
            nt_domain::chapter_map::ChapterMapDiagnostic::HeadingFallback { .. }
        )));
        assert!(map.diagnostics.iter().any(|item| matches!(
            item,
            nt_domain::chapter_map::ChapterMapDiagnostic::FileEdgeFallback { .. }
        )));
    }
    #[test]
    fn valid_confirmed_split_wins_and_auxiliary_sources_remain_separate() {
        let first = block("EPUB/a.xhtml", 30, "h1", text("Chapter One"));
        let second = block("EPUB/a.xhtml", 31, "p", text("AB"));
        let aux = block("EPUB/note.xhtml", 32, "p", text("Note"));
        let mut aux_doc = doc("EPUB/note.xhtml", 9, vec![aux.clone()]);
        aux_doc.domain = CoverageDomain::Auxiliary;
        aux_doc.linear_spine_positions.clear();
        let mut input = input(vec![
            doc("EPUB/a.xhtml", 0, vec![first, second.clone()]),
            aux_doc,
        ]);
        input.confirmed_boundaries.push(ConfirmedBoundary {
            epub_sha256: input.epub_sha256,
            parser_version: input.parser_version.clone(),
            canonical_path: "EPUB/a.xhtml".to_owned(),
            block_id: second.id,
            unit_offset: 1,
            label: Some("Confirmed split".to_owned()),
        });
        let map = build_candidate_map(&input).unwrap();
        assert!(map.volume.main_flow.iter().any(|item| matches!(item, MainFlowContent::Chapter(chapter) if chapter.label.as_deref() == Some("Confirmed split") && chapter.status == nt_domain::chapter_map::MappingStatus::Confirmed)));
        assert_eq!(map.volume.auxiliary.len(), 1);
        assert_eq!(map.volume.auxiliary[0].ranges[0].block_id, aux.id);
    }
}

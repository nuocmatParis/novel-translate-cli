//! Preview edits to an ordered chapter map without changing its source.

use std::{collections::HashMap, fmt, mem};

use crate::source_range::SourceBlockId;

use super::{
    BlockUnitRange, BoundaryEvidence, BoundaryEvidenceKind, Chapter, ChapterMap, ChapterMapError,
    MainFlowContent, MappingStatus,
};

/// A position between complete visible or protected units.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChapterBoundary {
    pub canonical_path: String,
    pub block_id: SourceBlockId,
    pub offset: u32,
}

impl ChapterBoundary {
    fn at_start(range: &BlockUnitRange) -> Self {
        Self {
            canonical_path: range.canonical_path.clone(),
            block_id: range.block_id,
            offset: range.start,
        }
    }

    fn lies_in(&self, range: &BlockUnitRange) -> bool {
        self.block_id == range.block_id
            && self.canonical_path == range.canonical_path
            && (range.start..=range.end).contains(&self.offset)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChapterEdit {
    Rename {
        chapter: u32,
        label: String,
    },
    Merge {
        first: u32,
        second: u32,
    },
    Split {
        chapter: u32,
        boundary: ChapterBoundary,
        label: Option<String>,
    },
    MoveBoundary {
        first: u32,
        second: u32,
        boundary: ChapterBoundary,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChapterEditImpact {
    /// IDs in reading order, including IDs added or retired by the edit.
    pub chapters: Vec<u32>,
    pub previous_boundary: Option<ChapterBoundary>,
    pub new_boundary: Option<ChapterBoundary>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ChapterEditPreview {
    pub mapping: ChapterMap,
    pub impact: ChapterEditImpact,
}

/// Exact before and after source ranges and start evidence for one affected
/// chapter. A removed or added chapter has only one side.
#[derive(Debug, Eq, PartialEq)]
pub struct ChapterChange<'map> {
    pub chapter_id: u32,
    pub before: Option<&'map Chapter>,
    pub after: Option<&'map Chapter>,
}

/// Project the changed chapters from two immutable mappings of the same
/// source. This retains references to the persisted ranges and evidence.
pub fn changed_chapters<'map>(
    before: &'map ChapterMap,
    after: &'map ChapterMap,
) -> Result<Vec<ChapterChange<'map>>, ChapterMapError> {
    if before.epub_sha256 != after.epub_sha256 || before.parser_version != after.parser_version {
        return Err(ChapterMapError::MappingIdentityMismatch);
    }
    let mut remaining: HashMap<_, _> = chapters(before)
        .map(|chapter| (chapter.chapter_id, chapter))
        .collect();
    let mut changes = Vec::new();
    for chapter in chapters(after) {
        let before = remaining.remove(&chapter.chapter_id);
        if before != Some(chapter) {
            changes.push(ChapterChange {
                chapter_id: chapter.chapter_id,
                before,
                after: Some(chapter),
            });
        }
    }
    changes.extend(
        chapters(before)
            .filter(|chapter| remaining.contains_key(&chapter.chapter_id))
            .map(|chapter| ChapterChange {
                chapter_id: chapter.chapter_id,
                before: Some(chapter),
                after: None,
            }),
    );
    Ok(changes)
}

fn chapters(mapping: &ChapterMap) -> impl Iterator<Item = &Chapter> {
    mapping
        .volume
        .main_flow
        .iter()
        .filter_map(|item| match item {
            MainFlowContent::Chapter(chapter) => Some(chapter),
            _ => None,
        })
}

#[derive(Debug, Eq, PartialEq)]
pub enum ChapterEditError {
    UnknownChapter(u32),
    ChaptersNotAdjacent { first: u32, second: u32 },
    EmptyChapter(u32),
    InvalidBoundary(ChapterBoundary),
    EmptyLabel,
    Unchanged,
    ChapterIdExhausted,
}

impl fmt::Display for ChapterEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownChapter(id) => write!(formatter, "chapter {id} was not found"),
            Self::ChaptersNotAdjacent { first, second } => {
                write!(
                    formatter,
                    "chapters {first} and {second} are not adjacent in that order"
                )
            }
            Self::EmptyChapter(id) => write!(formatter, "chapter {id} has no source ranges"),
            Self::InvalidBoundary(boundary) => write!(
                formatter,
                "no interior boundary at unit {} in {} ({})",
                boundary.offset, boundary.canonical_path, boundary.block_id
            ),
            Self::EmptyLabel => formatter.write_str("chapter label must not be empty"),
            Self::Unchanged => formatter.write_str("chapter edit does not change the mapping"),
            Self::ChapterIdExhausted => formatter.write_str("no chapter ID is available"),
        }
    }
}

impl std::error::Error for ChapterEditError {}

/// Apply one edit to a verified mapping for preview. The caller owns the
/// draft identity and must validate the preview against its pinned source
/// before it can publish a new version.
pub fn preview_chapter_edit(
    mut mapping: ChapterMap,
    edit: &ChapterEdit,
) -> Result<ChapterEditPreview, ChapterEditError> {
    let flow = &mut mapping.volume.main_flow;
    let impact = match edit {
        ChapterEdit::Rename { chapter, label } => {
            if label.trim().is_empty() {
                return Err(ChapterEditError::EmptyLabel);
            }
            let position = chapter_position(flow, *chapter)?;
            let chapter = chapter_mut(&mut flow[position], *chapter)?;
            if chapter.label.as_ref() == Some(label) {
                return Err(ChapterEditError::Unchanged);
            }
            chapter.label = Some(label.clone());
            ChapterEditImpact {
                chapters: vec![chapter.chapter_id],
                previous_boundary: None,
                new_boundary: None,
            }
        }
        ChapterEdit::Merge { first, second } => {
            let position = adjacent_chapters(flow, *first, *second)?;
            let MainFlowContent::Chapter(second) = flow.remove(position + 1) else {
                return Err(ChapterEditError::UnknownChapter(*second));
            };
            let first = chapter_mut(&mut flow[position], *first)?;
            let previous_boundary = start_of(&second)?;
            join_ranges(&mut first.ranges, second.ranges);
            retire_confirmed(&mut mapping.boundary_evidence, &second.start_evidence);
            ChapterEditImpact {
                chapters: vec![first.chapter_id, second.chapter_id],
                previous_boundary: Some(previous_boundary),
                new_boundary: None,
            }
        }
        ChapterEdit::Split {
            chapter,
            boundary,
            label,
        } => {
            if label.as_ref().is_some_and(|label| label.trim().is_empty()) {
                return Err(ChapterEditError::EmptyLabel);
            }
            let position = chapter_position(flow, *chapter)?;
            let new_id = next_chapter_id(flow)?;
            let first = chapter_mut(&mut flow[position], *chapter)?;
            let (left, right) = partition(mem::take(&mut first.ranges), boundary)?;
            first.ranges = left;
            let new_boundary = ChapterBoundary::at_start(&right[0]);
            let evidence = confirmed_evidence(&new_boundary, label);
            let second = Chapter {
                chapter_id: new_id,
                label: label.clone(),
                status: MappingStatus::Confirmed,
                ranges: right,
                start_evidence: vec![evidence.clone()],
            };
            flow.insert(position + 1, MainFlowContent::Chapter(second));
            mapping.boundary_evidence.push(evidence);
            ChapterEditImpact {
                chapters: vec![*chapter, new_id],
                previous_boundary: None,
                new_boundary: Some(new_boundary),
            }
        }
        ChapterEdit::MoveBoundary {
            first,
            second,
            boundary,
        } => {
            let position = adjacent_chapters(flow, *first, *second)?;
            let (before, after) = flow.split_at_mut(position + 1);
            let first = chapter_mut(&mut before[position], *first)?;
            let second = chapter_mut(&mut after[0], *second)?;
            let previous_boundary = start_of(second)?;
            let mut combined = mem::take(&mut first.ranges);
            join_ranges(&mut combined, mem::take(&mut second.ranges));
            let (left, right) = partition(combined, boundary)?;
            let new_boundary = ChapterBoundary::at_start(&right[0]);
            if new_boundary == previous_boundary {
                return Err(ChapterEditError::Unchanged);
            }
            first.ranges = left;
            second.ranges = right;
            second.status = MappingStatus::Confirmed;
            let evidence = confirmed_evidence(&new_boundary, &second.label);
            retire_confirmed(&mut mapping.boundary_evidence, &second.start_evidence);
            second.start_evidence = vec![evidence.clone()];
            mapping.boundary_evidence.push(evidence);
            ChapterEditImpact {
                chapters: vec![first.chapter_id, second.chapter_id],
                previous_boundary: Some(previous_boundary),
                new_boundary: Some(new_boundary),
            }
        }
    };
    Ok(ChapterEditPreview { mapping, impact })
}

fn chapter_position(flow: &[MainFlowContent], id: u32) -> Result<usize, ChapterEditError> {
    flow.iter()
        .position(
            |item| matches!(item, MainFlowContent::Chapter(chapter) if chapter.chapter_id == id),
        )
        .ok_or(ChapterEditError::UnknownChapter(id))
}

fn chapter_mut(item: &mut MainFlowContent, id: u32) -> Result<&mut Chapter, ChapterEditError> {
    match item {
        MainFlowContent::Chapter(chapter) if chapter.chapter_id == id => Ok(chapter),
        _ => Err(ChapterEditError::UnknownChapter(id)),
    }
}

fn adjacent_chapters(
    flow: &[MainFlowContent],
    first: u32,
    second: u32,
) -> Result<usize, ChapterEditError> {
    let position = chapter_position(flow, first)?;
    if chapter_position(flow, second)? == position + 1 {
        Ok(position)
    } else {
        Err(ChapterEditError::ChaptersNotAdjacent { first, second })
    }
}

fn next_chapter_id(flow: &[MainFlowContent]) -> Result<u32, ChapterEditError> {
    flow.iter()
        .filter_map(|item| match item {
            MainFlowContent::Chapter(chapter) => Some(chapter.chapter_id),
            _ => None,
        })
        .max()
        .and_then(|id| id.checked_add(1))
        .ok_or(ChapterEditError::ChapterIdExhausted)
}

fn start_of(chapter: &Chapter) -> Result<ChapterBoundary, ChapterEditError> {
    chapter
        .ranges
        .first()
        .map(ChapterBoundary::at_start)
        .ok_or(ChapterEditError::EmptyChapter(chapter.chapter_id))
}

fn join_ranges(left: &mut Vec<BlockUnitRange>, mut right: Vec<BlockUnitRange>) {
    if let (Some(last), Some(first)) = (left.last_mut(), right.first())
        && last.block_id == first.block_id
        && last.canonical_path == first.canonical_path
        && last.end == first.start
    {
        last.end = first.end;
        right.remove(0);
    }
    left.append(&mut right);
}

fn partition(
    mut ranges: Vec<BlockUnitRange>,
    boundary: &ChapterBoundary,
) -> Result<(Vec<BlockUnitRange>, Vec<BlockUnitRange>), ChapterEditError> {
    let Some(position) = ranges.iter().position(|range| boundary.lies_in(range)) else {
        return Err(ChapterEditError::InvalidBoundary(boundary.clone()));
    };
    let mut right = ranges.split_off(position);
    let right_start = &mut right[0];
    if boundary.offset > right_start.start {
        let mut left_end = right_start.clone();
        left_end.end = boundary.offset;
        ranges.push(left_end);
        right_start.start = boundary.offset;
    }
    if right_start.start == right_start.end {
        right.remove(0);
    }
    if ranges.is_empty() || right.is_empty() {
        return Err(ChapterEditError::InvalidBoundary(boundary.clone()));
    }
    Ok((ranges, right))
}

fn confirmed_evidence(boundary: &ChapterBoundary, label: &Option<String>) -> BoundaryEvidence {
    BoundaryEvidence {
        kind: BoundaryEvidenceKind::Confirmed,
        source: None,
        canonical_path: boundary.canonical_path.clone(),
        block_id: Some(boundary.block_id),
        unit_offset: Some(boundary.offset),
        label: label.clone(),
        raw_href: None,
        reason: "confirmed by a chapter mapping edit".to_owned(),
        depth: None,
    }
}

fn retire_confirmed(evidence: &mut Vec<BoundaryEvidence>, previous: &[BoundaryEvidence]) {
    evidence
        .retain(|item| item.kind != BoundaryEvidenceKind::Confirmed || !previous.contains(item));
}

#[cfg(test)]
mod tests;

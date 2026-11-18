//! Replay one source block without materializing one object per grapheme.

use nt_domain::volume_import::{
    CapturedBlock, CapturedBlockPlacement, CapturedSourceError, CapturedUnitRun,
};

use super::super::{
    AttributeSourceText, BlockIdentityRecord, Digest, ElementLocator, InlineMetadataKind,
    InlineSourceMetadata, MetadataBuilder, ProtectedSourceUnit, Sha256, SourceBlockId, Token,
    UnicodeSegmentation, UnitRange, hash_bytes, hash_endpoint, hash_optional_bytes, rebase_tokens,
};
use super::{BlockSpace, visit_vec};

type Error = CapturedSourceError;
pub(super) type Anchor = (String, ElementLocator, u32);

pub(super) fn replay_block(
    mut tokens: Vec<Token>,
    metadata: &[MetadataBuilder],
    identity: BlockIdentityRecord,
    element_name: &str,
    placement: CapturedBlockPlacement,
    mut space: BlockSpace,
) -> Result<(CapturedBlock, Vec<Anchor>), Error> {
    rebase_tokens(&mut tokens, &identity.locator);
    let id = SourceBlockId::from_bytes(
        Sha256::digest(
            identity
                .canonical_bytes()
                .map_err(|_| Error::InvalidSource)?,
        )
        .into(),
    );
    let mut state = RunState {
        hash: Sha256::new(),
        unit_count: 0,
        runs: Vec::new(),
        ranges: space.allocate(metadata.len())?,
        anchors: Vec::new(),
        attribute_text: Vec::new(),
        space,
    };
    state.hash.update(b"NT-SOURCE-CONTENT\0");
    state.hash.update(1_u16.to_be_bytes());
    state.ranges.resize_with(metadata.len(), Vec::new);

    let mut remaining = tokens.into_iter();
    while !remaining.as_slice().is_empty() {
        let text_group = remaining
            .as_slice()
            .iter()
            .take_while(|token| !matches!(token, Token::Atomic { .. }))
            .count();
        if text_group > 0 {
            let capacity = text_capacity(&remaining.as_slice()[..text_group])?;
            state.text(remaining.by_ref().take(text_group), capacity)?;
        }
        if let Some(token) = remaining.next() {
            state.protected(token)?;
        }
    }

    let mut inline_metadata = state.space.allocate(
        state
            .ranges
            .iter()
            .filter(|ranges| !ranges.is_empty())
            .count(),
    )?;
    for (builder, ranges) in metadata.iter().zip(state.ranges) {
        if ranges.is_empty() {
            continue;
        }
        let mut locator = builder.locator.clone();
        if builder.rebase_to_segment {
            locator.block = identity.locator.clone();
        }
        inline_metadata.push(InlineSourceMetadata {
            locator,
            kind: builder.kind.clone(),
            ranges,
        });
    }
    hash_metadata(&mut state.hash, &inline_metadata, &state.attribute_text);
    Ok((
        CapturedBlock {
            id,
            element_name: element_name.to_ascii_lowercase(),
            locator: identity.locator,
            placement,
            content_hash: state.hash.finalize().into(),
            unit_count: state.unit_count,
            runs: state.runs,
            inline_metadata,
            attribute_text: state.attribute_text,
        },
        state.anchors,
    ))
}

struct TextCapacity {
    bytes: usize,
    segments: usize,
    anchors: usize,
}

fn text_capacity(tokens: &[Token]) -> Result<TextCapacity, Error> {
    let mut result = TextCapacity {
        bytes: 0,
        segments: 0,
        anchors: 0,
    };
    for token in tokens {
        match token {
            Token::Text { text, .. } => {
                result.bytes = result
                    .bytes
                    .checked_add(text.len())
                    .ok_or(Error::ResourceLimit)?;
                result.segments += 1;
            }
            Token::Anchor { .. } => result.anchors += 1,
            Token::AttributeText(_) => {}
            _ => return Err(Error::InvalidSource),
        }
    }
    if result.bytes > crate::xml::DEFAULT_XML_MAX_BYTES {
        return Err(Error::ResourceLimit);
    }
    Ok(result)
}

struct RunState {
    hash: Sha256,
    unit_count: u32,
    runs: Vec<CapturedUnitRun>,
    ranges: Vec<Vec<UnitRange>>,
    anchors: Vec<Anchor>,
    attribute_text: Vec<AttributeSourceText>,
    space: BlockSpace,
}

impl RunState {
    fn text(
        &mut self,
        tokens: impl Iterator<Item = Token>,
        capacity: TextCapacity,
    ) -> Result<(), Error> {
        let mut text = self.space.string(capacity.bytes)?;
        let mut segments = self.space.allocate(capacity.segments)?;
        let mut segment_metadata = self.space.allocate(capacity.segments)?;
        let mut pending = self.space.allocate(capacity.anchors)?;
        for token in tokens {
            match token {
                Token::Text {
                    text: value,
                    locator,
                    metadata,
                } => {
                    let start = text.len() as u32;
                    text.push_str(&value);
                    segments.push([
                        start,
                        text.len() as u32,
                        locator.text_node_order,
                        0,
                        value.len() as u32,
                    ]);
                    segment_metadata.push(metadata);
                }
                Token::Anchor { id, locator } => pending.push((text.len(), id, locator)),
                Token::AttributeText(value) => {
                    visit_vec(&mut self.space, &mut self.attribute_text, value)?;
                }
                _ => return Err(Error::InvalidSource),
            }
        }
        let mut pending = pending.into_iter().peekable();
        let mut segment = 0;
        let run_start = self.unit_count;
        let mut segment_start = run_start;
        for (start, grapheme) in text.grapheme_indices(true) {
            let end = start + grapheme.len();
            let unit = self.unit_count;
            let next = unit.checked_add(1).ok_or(Error::ResourceLimit)?;
            self.hash.update([0x01]);
            hash_bytes(&mut self.hash, grapheme.as_bytes());

            // Consecutive lexical pieces may end in the same grapheme. Each
            // metadata element receives the whole grapheme it contributes to.
            while let Some(projection) = segments.get(segment) {
                let piece_end = projection[1] as usize;
                if piece_end > end {
                    break;
                }
                self.mark(&segment_metadata[segment], segment_start, next)?;
                segment_start = if piece_end == end { next } else { unit };
                segment += 1;
            }
            while pending.peek().is_some_and(|anchor| anchor.0 <= end) {
                let (position, fragment, locator) = pending.next().ok_or(Error::InvalidSource)?;
                let offset = if position == end { next } else { unit };
                visit_vec(
                    &mut self.space,
                    &mut self.anchors,
                    (fragment, locator, offset),
                )?;
            }
            self.unit_count = next;
        }
        if segment != segments.len() {
            return Err(Error::InvalidSource);
        }
        for (_, fragment, locator) in pending {
            visit_vec(
                &mut self.space,
                &mut self.anchors,
                (fragment, locator, self.unit_count),
            )?;
        }
        if !text.is_empty() {
            visit_vec(
                &mut self.space,
                &mut self.runs,
                CapturedUnitRun::Text {
                    text,
                    segments,
                    unit_count: self.unit_count - run_start,
                },
            )?;
        }
        Ok(())
    }

    fn protected(&mut self, token: Token) -> Result<(), Error> {
        let Token::Atomic {
            unit,
            spans,
            metadata,
        } = token
        else {
            return Err(Error::InvalidSource);
        };
        let end = self.unit_count.checked_add(1).ok_or(Error::ResourceLimit)?;
        self.mark(&metadata, self.unit_count, end)?;
        self.unit_count = end;
        hash_protected(&mut self.hash, &unit);
        visit_vec(
            &mut self.space,
            &mut self.runs,
            CapturedUnitRun::Protected { unit, spans },
        )
    }

    fn mark(&mut self, metadata: &[usize], start: u32, end: u32) -> Result<(), Error> {
        for &index in metadata {
            let ranges = self.ranges.get_mut(index).ok_or(Error::InvalidSource)?;
            if let Some(last) = ranges.last_mut()
                && start <= last.end
            {
                last.end = last.end.max(end);
                continue;
            }
            visit_vec(&mut self.space, ranges, UnitRange { start, end })?;
        }
        Ok(())
    }
}

// The parser's content hash includes ordered unit records followed by
// formatting and attribute records. Reuse the same length/value encoding.
fn hash_protected(hash: &mut Sha256, unit: &ProtectedSourceUnit) {
    match unit {
        ProtectedSourceUnit::Ruby(ruby) => {
            hash.update([0x02]);
            hash_bytes(hash, ruby.base.as_bytes());
            hash_bytes(hash, ruby.annotation.as_bytes());
        }
        ProtectedSourceUnit::Image(image) => {
            hash.update([0x03]);
            hash_optional_bytes(hash, image.resource_path.as_deref().map(str::as_bytes));
            hash_optional_bytes(hash, image.raw_source.as_deref().map(str::as_bytes));
            hash_optional_bytes(hash, image.alt.as_deref().map(str::as_bytes));
        }
        ProtectedSourceUnit::MarkerPair(pair) => {
            hash.update([0x04]);
            hash_bytes(hash, pair.kind.as_bytes());
            hash_endpoint(hash, &pair.start.node_path, &pair.start.attributes);
            hash_endpoint(hash, &pair.end.node_path, &pair.end.attributes);
        }
        ProtectedSourceUnit::LineBreak { .. } => hash.update([0x05]),
        ProtectedSourceUnit::SceneBreak { .. } => hash.update([0x06]),
    }
}

fn hash_metadata(
    hash: &mut Sha256,
    metadata: &[InlineSourceMetadata],
    attributes: &[AttributeSourceText],
) {
    for item in metadata {
        hash.update([0x10]);
        match &item.kind {
            InlineMetadataKind::Formatting { element } => {
                hash.update([0x01]);
                hash_bytes(hash, element.as_bytes());
            }
            InlineMetadataKind::Link { href } => {
                hash.update([0x02]);
                hash_optional_bytes(hash, href.as_deref().map(str::as_bytes));
            }
            InlineMetadataKind::NoteReference { href } => {
                hash.update([0x03]);
                hash_optional_bytes(hash, href.as_deref().map(str::as_bytes));
            }
        }
        for range in &item.ranges {
            hash.update(range.start.to_be_bytes());
            hash.update(range.end.to_be_bytes());
        }
    }
    for attribute in attributes {
        hash.update([0x20]);
        hash_bytes(hash, attribute.attribute.as_bytes());
        hash_bytes(hash, attribute.value.as_bytes());
        hash_optional_bytes(hash, attribute.resource_path.as_deref().map(str::as_bytes));
    }
}

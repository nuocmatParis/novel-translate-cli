use nt_application::volume_import::source_range::{BlockLocator, SourceUnit, SourceUnitKind};
use nt_epub::SourceExtraction;
use serde::{Deserialize, Serialize};

use super::CodecError;

const MAX_RUN_BYTES: usize = 16 * 1024 * 1024;
const MAX_WORKING_BYTES: usize = 256 * 1024 * 1024;
const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum Tag {
    TextRun,
}

/// Offset evidence from a captured source, with compact ordered text runs.
/// Call validate before use; a validated run still needs original-node checks
/// before it is part of a trusted stored source snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextRun {
    kind: Tag,
    text: String,
    grapheme_lengths: String,
    segments: Vec<[u32; 5]>,
}

crate::import_codec::memory::heap_enum!(Tag { TextRun });
crate::import_codec::memory::heap_struct!(TextRun {
    kind,
    text,
    grapheme_lengths,
    segments
});

fn reserve<T>(values: &mut Vec<T>, count: usize) -> Result<(), CodecError> {
    values
        .try_reserve_exact(count)
        .map_err(|_| CodecError::ResourceLimit)
}
fn string_with_capacity(capacity: usize) -> Result<String, CodecError> {
    let mut result = String::new();
    result
        .try_reserve_exact(capacity)
        .map_err(|_| CodecError::ResourceLimit)?;
    Ok(result)
}
fn add(total: &mut usize, bytes: usize) -> Result<(), CodecError> {
    *total = total
        .checked_add(bytes)
        .filter(|size| *size <= MAX_WORKING_BYTES)
        .ok_or(CodecError::ResourceLimit)?;
    Ok(())
}
fn grapheme(unit: &SourceUnit) -> Result<&str, CodecError> {
    match &unit.kind {
        SourceUnitKind::Grapheme(text) if !text.is_empty() => Ok(text),
        _ => Err(CodecError::InvalidTextRun),
    }
}

/// Emit compact segments in two bounded passes. Only the first contributor of
/// the next grapheme can coalesce with the last contributor of the previous one.
fn projections(
    units: &[SourceUnit],
    locator: &BlockLocator,
    mut emit: impl FnMut([u32; 5]) -> Result<(), CodecError>,
) -> Result<(), CodecError> {
    let mut offset = 0_u32;
    let mut pending: Option<[u32; 5]> = None;
    for unit in units {
        let text = grapheme(unit)?;
        if unit.spans.is_empty() {
            return Err(CodecError::InvalidTextRun);
        }
        let unit_start = offset;
        for (ordinal, span) in unit.spans.iter().enumerate() {
            if &span.node().block != locator {
                return Err(CodecError::InvalidTextRun);
            }
            let length = span
                .end_byte()
                .checked_sub(span.start_byte())
                .filter(|length| *length > 0)
                .ok_or(CodecError::InvalidTextRun)?;
            let end = offset
                .checked_add(length)
                .ok_or(CodecError::ResourceLimit)?;
            let local_start =
                usize::try_from(offset - unit_start).map_err(|_| CodecError::ResourceLimit)?;
            let local_end =
                usize::try_from(end - unit_start).map_err(|_| CodecError::ResourceLimit)?;
            if local_end > text.len()
                || !text.is_char_boundary(local_start)
                || !text.is_char_boundary(local_end)
            {
                return Err(CodecError::InvalidTextRun);
            }
            let next = [
                offset,
                end,
                span.node().text_node_order,
                span.start_byte(),
                span.end_byte(),
            ];
            if ordinal == 0
                && let Some(previous) = pending.as_mut()
                && previous[1] == next[0]
                && previous[2] == next[2]
                && previous[4] == next[3]
            {
                previous[1] = next[1];
                previous[4] = next[4];
            } else {
                if let Some(previous) = pending.replace(next) {
                    emit(previous)?;
                }
            }
            offset = end;
        }
        if usize::try_from(offset - unit_start).map_err(|_| CodecError::ResourceLimit)?
            != text.len()
        {
            return Err(CodecError::InvalidTextRun);
        }
    }
    if let Some(previous) = pending {
        emit(previous)?;
    }
    Ok(())
}

impl TextRun {
    pub(super) fn from_wire_parts(
        text: String,
        grapheme_lengths: String,
        segments: Vec<[u32; 5]>,
    ) -> Self {
        Self {
            kind: Tag::TextRun,
            text,
            grapheme_lengths,
            segments,
        }
    }

    pub(super) fn original_unit_count(
        &self,
        text: &str,
        segments: &[[u32; 5]],
    ) -> Result<Option<u32>, CodecError> {
        if self.text != text || self.segments != segments {
            return Ok(None);
        }
        self.validate()
            .map(|validated| Some(validated.unit_count()))
    }

    /// Decode one canonical JSON run segment. The enclosing payload reader
    /// must verify its binary envelope and original source independently.
    pub fn read_canonical_json(
        source: &mut (impl std::io::Read + std::io::Seek),
        byte_len: u64,
    ) -> Result<Self, CodecError> {
        let run: Self = super::canonical_json::decode(
            source,
            byte_len,
            super::canonical_json::Limits::default(),
        )?;
        run.validate()?;
        Ok(run)
    }
    /// Encode one contiguous slice of grapheme units. The caller splits at
    /// protected units and at the explicit 16 MiB text/length-byte run bounds.
    pub fn from_grapheme_units(
        units: &[SourceUnit],
        locator: &BlockLocator,
    ) -> Result<Self, CodecError> {
        let (text_bytes, encoded, segment_count, _) = Self::capture_plan(units, locator)?;
        let mut text = string_with_capacity(text_bytes)?;
        let mut lengths = Base64Writer {
            output: string_with_capacity(encoded)?,
            bits: 0,
            count: 0,
        };
        for unit in units {
            let value = grapheme(unit)?;
            text.push_str(value);
            let mut length = value.len();
            loop {
                let low = (length & 0x7f) as u8;
                length >>= 7;
                lengths.push(low | if length == 0 { 0 } else { 0x80 });
                if length == 0 {
                    break;
                }
            }
        }
        let mut segments = Vec::new();
        reserve(&mut segments, segment_count)?;
        projections(units, locator, |segment| {
            segments.push(segment);
            Ok(())
        })?;
        let result = Self {
            kind: Tag::TextRun,
            text,
            grapheme_lengths: lengths.finish(),
            segments,
        };
        result.validate()?;
        Ok(result)
    }

    pub(super) fn capture_plan(
        units: &[SourceUnit],
        locator: &BlockLocator,
    ) -> Result<(usize, usize, usize, usize), CodecError> {
        if units.is_empty() {
            return Err(CodecError::InvalidTextRun);
        }
        let (mut text_bytes, mut length_bytes, mut segment_count) = (0_usize, 0_usize, 0_usize);
        for unit in units {
            let text = grapheme(unit)?;
            text_bytes = text_bytes
                .checked_add(text.len())
                .filter(|n| *n <= MAX_RUN_BYTES)
                .ok_or(CodecError::ResourceLimit)?;
            let mut length = text.len();
            loop {
                length_bytes = length_bytes
                    .checked_add(1)
                    .filter(|n| *n <= MAX_RUN_BYTES)
                    .ok_or(CodecError::ResourceLimit)?;
                length >>= 7;
                if length == 0 {
                    break;
                }
            }
        }
        projections(units, locator, |_| {
            segment_count = segment_count
                .checked_add(1)
                .ok_or(CodecError::ResourceLimit)?;
            Ok(())
        })?;
        let encoded = length_bytes
            .checked_add(2)
            .and_then(|n| (n / 3).checked_mul(4))
            .ok_or(CodecError::ResourceLimit)?;
        let mut working = 0;
        // Include bounded output buffers before their first allocation.
        for bytes in [
            text_bytes,
            encoded,
            segment_count
                .checked_mul(std::mem::size_of::<[u32; 5]>())
                .ok_or(CodecError::ResourceLimit)?,
        ] {
            add(&mut working, bytes)?;
        }
        Ok((text_bytes, encoded, segment_count, working))
    }

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn grapheme_lengths(&self) -> &str {
        &self.grapheme_lengths
    }
    pub fn segments(&self) -> &[[u32; 5]] {
        &self.segments
    }

    pub fn validate(&self) -> Result<ValidatedTextRun<'_>, CodecError> {
        if self.text.is_empty() {
            return Err(CodecError::InvalidTextRun);
        }
        let encoded = self.grapheme_lengths.as_bytes();
        if self.text.len() > MAX_RUN_BYTES || encoded.len() > MAX_RUN_BYTES.div_ceil(3) * 4 {
            return Err(CodecError::ResourceLimit);
        }
        if encoded.is_empty() || !encoded.len().is_multiple_of(4) {
            return Err(CodecError::InvalidTextRun);
        }
        let padding = usize::from(encoded.ends_with(b"=")) + usize::from(encoded.ends_with(b"=="));
        let decoded = (encoded.len() / 4)
            .checked_mul(3)
            .and_then(|n| n.checked_sub(padding))
            .ok_or(CodecError::InvalidTextRun)?;
        if decoded > MAX_RUN_BYTES {
            return Err(CodecError::ResourceLimit);
        }
        let mut working = 0;
        for bytes in [
            self.text.capacity(),
            self.grapheme_lengths.capacity(),
            self.segments
                .capacity()
                .checked_mul(std::mem::size_of::<[u32; 5]>())
                .ok_or(CodecError::ResourceLimit)?,
        ] {
            add(&mut working, bytes)?;
        }
        if !SourceExtraction::text_run_lengths_match(&self.text, Lengths::new(encoded)) {
            return Err(CodecError::InvalidTextRun);
        }
        let mut covered = 0;
        let mut previous: Option<[u32; 5]> = None;
        let mut lengths = Lengths::new(encoded);
        let mut boundary = lengths.next().expect("validated nonempty run")? as u32;
        for &[start, end, node, node_start, node_end] in &self.segments {
            if start != covered
                || start >= end
                || end as usize > self.text.len()
                || node_start >= node_end
                || end - start != node_end - node_start
                || !self.text.is_char_boundary(start as usize)
                || !self.text.is_char_boundary(end as usize)
            {
                return Err(CodecError::InvalidTextRun);
            }
            while boundary < start {
                boundary += lengths.next().ok_or(CodecError::InvalidTextRun)?? as u32;
            }
            let segment = [start, end, node, node_start, node_end];
            if let Some(prior) = previous
                && start == boundary
                && prior[2] == segment[2]
                && prior[4] == node_start
            {
                return Err(CodecError::InvalidTextRun);
            }
            previous = Some(segment);
            covered = end;
        }
        if covered as usize != self.text.len() {
            return Err(CodecError::InvalidTextRun);
        }
        let units =
            u32::try_from(Lengths::new(encoded).count()).map_err(|_| CodecError::ResourceLimit)?;
        Ok(ValidatedTextRun { run: self, units })
    }
}

struct Base64Writer {
    output: String,
    bits: u32,
    count: u8,
}
impl Base64Writer {
    fn push(&mut self, value: u8) {
        self.bits = (self.bits << 8) | u32::from(value);
        self.count += 1;
        if self.count == 3 {
            for shift in [18, 12, 6, 0] {
                self.output
                    .push(BASE64[((self.bits >> shift) & 63) as usize] as char);
            }
            self.bits = 0;
            self.count = 0;
        }
    }
    fn finish(mut self) -> String {
        if self.count != 0 {
            self.bits <<= (3 - self.count) * 8;
            self.output
                .push(BASE64[((self.bits >> 18) & 63) as usize] as char);
            self.output
                .push(BASE64[((self.bits >> 12) & 63) as usize] as char);
            self.output.push(if self.count == 2 {
                BASE64[((self.bits >> 6) & 63) as usize] as char
            } else {
                '='
            });
            self.output.push('=');
        }
        self.output
    }
}
struct Bytes<'a> {
    text: &'a [u8],
    offset: usize,
    bytes: [u8; 3],
    start: usize,
    end: usize,
    failed: bool,
}
fn sextet(value: u8) -> Option<u8> {
    match value {
        b'A'..=b'Z' => Some(value - b'A'),
        b'a'..=b'z' => Some(value - b'a' + 26),
        b'0'..=b'9' => Some(value - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
impl Iterator for Bytes<'_> {
    type Item = Result<u8, CodecError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        if self.start == self.end {
            if self.offset == self.text.len() {
                return None;
            }
            let Some(quartet) = self.text.get(self.offset..self.offset + 4) else {
                self.failed = true;
                return Some(Err(CodecError::InvalidTextRun));
            };
            let valid = (|| {
                let a = sextet(quartet[0])?;
                let b = sextet(quartet[1])?;
                let pad2 = quartet[2] == b'=';
                let pad3 = quartet[3] == b'=';
                if (pad2 && !pad3) || ((pad2 || pad3) && self.offset + 4 != self.text.len()) {
                    return None;
                }
                let c = if pad2 { 0 } else { sextet(quartet[2])? };
                let d = if pad3 { 0 } else { sextet(quartet[3])? };
                if (pad2 && b & 15 != 0) || (pad3 && !pad2 && c & 3 != 0) {
                    return None;
                }
                self.bytes = [(a << 2) | (b >> 4), (b << 4) | (c >> 2), (c << 6) | d];
                self.start = 0;
                self.end = 3 - usize::from(pad2) - usize::from(pad3);
                self.offset += 4;
                Some(())
            })();
            if valid.is_none() {
                self.failed = true;
                return Some(Err(CodecError::InvalidTextRun));
            }
        }
        let value = self.bytes[self.start];
        self.start += 1;
        Some(Ok(value))
    }
}
struct Lengths<'a> {
    bytes: Bytes<'a>,
    failed: bool,
}
impl<'a> Lengths<'a> {
    fn new(text: &'a [u8]) -> Self {
        Self {
            bytes: Bytes {
                text,
                offset: 0,
                bytes: [0; 3],
                start: 0,
                end: 0,
                failed: false,
            },
            failed: false,
        }
    }
}
impl Iterator for Lengths<'_> {
    type Item = Result<usize, CodecError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let first = self.bytes.next()?;
        let decoded = (|| {
            let mut value = 0_u32;
            let mut next = first;
            for index in 0..5 {
                let byte = next?;
                if index == 4 && byte > 15 {
                    return Err(CodecError::InvalidTextRun);
                }
                value |= u32::from(byte & 127) << (index * 7);
                if byte & 128 == 0 {
                    if value == 0 || (index > 0 && byte == 0) {
                        return Err(CodecError::InvalidTextRun);
                    }
                    return usize::try_from(value).map_err(|_| CodecError::ResourceLimit);
                }
                next = self.bytes.next().ok_or(CodecError::InvalidTextRun)?;
            }
            Err(CodecError::InvalidTextRun)
        })();
        if decoded.is_err() {
            self.failed = true;
        }
        Some(decoded)
    }
}

/// Validated compact offsets; original XML evidence must still be checked by
/// the enclosing stored-snapshot reader before presenting trusted source data.
pub struct ValidatedTextRun<'a> {
    run: &'a TextRun,
    units: u32,
}
impl<'a> ValidatedTextRun<'a> {
    pub fn unit_count(&self) -> u32 {
        self.units
    }
    pub fn units(&self) -> TextUnits<'a> {
        TextUnits {
            run: self.run,
            lengths: Lengths::new(self.run.grapheme_lengths.as_bytes()),
            offset: 0,
            segment: 0,
        }
    }
}
pub struct TextUnits<'a> {
    run: &'a TextRun,
    lengths: Lengths<'a>,
    offset: u32,
    segment: usize,
}
pub struct TextUnit<'a> {
    pub text: &'a str,
    segments: &'a [[u32; 5]],
    start: u32,
    end: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectedSpan {
    pub text_node_order: u32,
    pub start_byte: u32,
    pub end_byte: u32,
}
impl TextUnit<'_> {
    pub fn spans(&self) -> impl Iterator<Item = ProjectedSpan> + '_ {
        self.segments
            .iter()
            .take_while(|s| s[0] < self.end)
            .filter_map(|s| {
                let left = s[0].max(self.start);
                let right = s[1].min(self.end);
                (left < right).then(|| ProjectedSpan {
                    text_node_order: s[2],
                    start_byte: s[3] + (left - s[0]),
                    end_byte: s[3] + (right - s[0]),
                })
            })
    }
}
impl<'a> Iterator for TextUnits<'a> {
    type Item = TextUnit<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let length = self
            .lengths
            .next()?
            .expect("the complete immutable run was validated") as u32;
        let start = self.offset;
        let end = start + length;
        while self.run.segments[self.segment][1] <= start {
            self.segment += 1;
        }
        self.offset = end;
        Some(TextUnit {
            text: &self.run.text[start as usize..end as usize],
            segments: &self.run.segments[self.segment..],
            start,
            end,
        })
    }
}

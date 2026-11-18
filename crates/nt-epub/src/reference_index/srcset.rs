//! HTML srcset candidate parsing. Commas inside a URL are not separators.

use super::{Budget, ReferenceIndexError};

pub(super) struct Candidate {
    pub value: String,
    pub ordinal: u32,
    pub range: (usize, usize),
}

pub(super) fn scan(
    value: &str,
    budget: &mut Budget,
) -> Result<Vec<Candidate>, ReferenceIndexError> {
    let bytes = value.as_bytes();
    let mut cursor = 0;
    let mut output = Vec::new();
    while cursor < bytes.len() {
        while cursor < bytes.len() && (space(bytes[cursor]) || bytes[cursor] == b',') {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        let begin = cursor;
        while cursor < bytes.len() && !space(bytes[cursor]) {
            cursor += 1;
        }
        let mut end = cursor;
        while end > begin && bytes[end - 1] == b',' {
            end -= 1;
        }
        if begin == end {
            return Err(ReferenceIndexError::InvalidSrcset);
        }
        if end == cursor {
            let mut descriptors = Vec::new();
            let mut token_start = None;
            let mut parens = 0_usize;
            while cursor < bytes.len() {
                let byte = bytes[cursor];
                if byte == b'(' {
                    parens = parens
                        .checked_add(1)
                        .ok_or(ReferenceIndexError::InvalidSrcset)?;
                }
                if byte == b')' && parens > 0 {
                    parens -= 1;
                }
                if (space(byte) || (byte == b',' && parens == 0)) && token_start.is_some() {
                    if descriptors.len() == 3 {
                        return Err(ReferenceIndexError::InvalidSrcset);
                    }
                    descriptors.push(&value[token_start.take().unwrap_or(cursor)..cursor]);
                }
                if byte == b',' && parens == 0 {
                    cursor += 1;
                    break;
                }
                if !space(byte) && token_start.is_none() {
                    token_start = Some(cursor);
                }
                cursor += 1;
            }
            if let Some(start) = token_start {
                if descriptors.len() == 3 {
                    return Err(ReferenceIndexError::InvalidSrcset);
                }
                descriptors.push(&value[start..cursor]);
            }
            if parens != 0 || !valid_descriptors(&descriptors) {
                return Err(ReferenceIndexError::InvalidSrcset);
            }
        }
        budget.reference(end - begin)?;
        output.push(Candidate {
            value: value[begin..end].to_owned(),
            ordinal: u32::try_from(output.len())
                .map_err(|_| ReferenceIndexError::ReferenceCountLimit)?,
            range: (begin, end),
        });
    }
    Ok(output)
}

fn valid_descriptors(values: &[&str]) -> bool {
    let (mut width, mut height, mut density) = (false, false, false);
    for value in values {
        let Some((number, unit)) = value.split_at_checked(value.len().saturating_sub(1)) else {
            return false;
        };
        match unit {
            "w" if !width && !density && positive_integer(number) => width = true,
            "h" if !height && !density && positive_integer(number) => height = true,
            "x" if !density
                && !width
                && !height
                && valid_float(number)
                && number
                    .parse::<f64>()
                    .is_ok_and(|n| n.is_finite() && n >= 0.0) =>
            {
                density = true
            }
            _ => return false,
        }
    }
    !height || width
}

fn space(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | 0x0c | b'\r' | b' ')
}

fn valid_float(value: &str) -> bool {
    // HTML floating-point syntax has no leading '+', NaN, or infinity.
    let bytes = value.as_bytes();
    let mut p = usize::from(bytes.first() == Some(&b'-'));
    let start = p;
    while bytes.get(p).is_some_and(u8::is_ascii_digit) {
        p += 1;
    }
    let before = p - start;
    if bytes.get(p) == Some(&b'.') {
        p += 1;
        let start = p;
        while bytes.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
        }
        if p == start {
            return false;
        }
    } else if before == 0 {
        return false;
    }
    if bytes.get(p).is_some_and(|b| matches!(b, b'e' | b'E')) {
        p += 1;
        if bytes.get(p).is_some_and(|b| matches!(b, b'+' | b'-')) {
            p += 1;
        }
        let start = p;
        while bytes.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
        }
        if p == start {
            return false;
        }
    }
    p == bytes.len()
}

fn positive_integer(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|b| b.is_ascii_digit())
        && value.bytes().any(|b| b != b'0')
}

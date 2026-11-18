//! Bounded CSS input decoding and lexical reference discovery.

use super::{Budget, ReferenceIndexError, ReferenceKind};

pub(super) struct CssReference {
    pub kind: ReferenceKind,
    pub value: String,
    pub raw: String,
    pub position: usize,
    pub range: (usize, usize),
}

pub(super) fn decode(bytes: &[u8], max: usize) -> Result<String, ReferenceIndexError> {
    if bytes.len() > max {
        return Err(ReferenceIndexError::CssByteLimit);
    }
    let (payload, encoding, bom) = if let Some(rest) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        (rest, 0, true)
    } else if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        (rest, 1, true)
    } else if let Some(rest) = bytes.strip_prefix(&[0xfe, 0xff]) {
        (rest, 2, true)
    } else {
        (bytes, 0, false)
    };
    let text = if encoding == 0 {
        std::str::from_utf8(payload)
            .map_err(|_| ReferenceIndexError::UnsupportedEncoding)?
            .to_owned()
    } else {
        if payload.len() % 2 != 0 {
            return Err(ReferenceIndexError::UnsupportedEncoding);
        }
        let units = payload.as_chunks::<2>().0.iter().map(|pair| {
            if encoding == 1 {
                u16::from_le_bytes([pair[0], pair[1]])
            } else {
                u16::from_be_bytes([pair[0], pair[1]])
            }
        });
        let mut result = String::new();
        for value in char::decode_utf16(units) {
            let value = value.map_err(|_| ReferenceIndexError::UnsupportedEncoding)?;
            if result
                .len()
                .checked_add(value.len_utf8())
                .is_none_or(|n| n > max)
            {
                return Err(ReferenceIndexError::CssByteLimit);
            }
            result.push(value);
        }
        result
    };
    if text.len() > max {
        return Err(ReferenceIndexError::CssByteLimit);
    }
    // CSS Syntax gives a BOM precedence over an encoding declaration. EPUB
    // permits UTF-8 and UTF-16; a UTF-16 input without a BOM is not accepted.
    if !bom
        && let Some(rest) = text.strip_prefix("@charset \"")
        && let Some((label, _)) = rest.split_once("\";")
        && !label.eq_ignore_ascii_case("utf-8")
    {
        return Err(ReferenceIndexError::UnsupportedEncoding);
    }
    Ok(text)
}

pub(super) fn scan(
    text: &str,
    budget: &mut Budget,
) -> Result<Vec<CssReference>, ReferenceIndexError> {
    let mut scanner = Scanner {
        text,
        cursor: 0,
        budget,
        depth: 0,
    };
    let mut refs = Vec::new();
    let result = (|| {
        while scanner.cursor < text.len() {
            scanner.budget.token()?;
            let start = scanner.cursor;
            let byte = scanner.byte();
            if byte.is_ascii_whitespace() {
                scanner.cursor += 1;
            } else if scanner.starts(b"/*") {
                scanner.comment()?;
            } else if matches!(byte, b'\'' | b'"') {
                scanner.string(false)?;
            } else if byte == b'#' {
                scanner.cursor += 1;
                scanner.name()?;
            } else if scanner.starts_number() {
                scanner.number();
                // A dimension token's unit is not a function token (e.g. 1url).
                scanner.name()?;
            } else if byte == b'@' {
                scanner.cursor += 1;
                let name = scanner.name()?;
                if name.eq_ignore_ascii_case("import") {
                    scanner.space_and_comments()?;
                    let import = if matches!(scanner.peek(), Some(b'\'' | b'"')) {
                        let (value, begin, end) = scanner.string(true)?;
                        Some((value, begin, end))
                    } else {
                        let save = scanner.cursor;
                        let name = scanner.name()?;
                        if name.eq_ignore_ascii_case("url") && scanner.peek() == Some(b'(') {
                            scanner.url()?
                        } else {
                            scanner.cursor = save;
                            None
                        }
                    };
                    if let Some((value, begin, end)) = import {
                        scanner.push(
                            &mut refs,
                            ReferenceKind::CssImport,
                            value,
                            start,
                            begin,
                            end,
                        )?;
                    }
                }
            } else if is_name_start(byte) || byte == b'\\' {
                let name = scanner.name()?;
                if scanner.peek() == Some(b'(') {
                    if name.eq_ignore_ascii_case("url") {
                        if let Some((value, begin, end)) = scanner.url()? {
                            scanner.push(
                                &mut refs,
                                ReferenceKind::CssUrl,
                                value,
                                start,
                                begin,
                                end,
                            )?;
                        }
                    } else {
                        scanner.open_paren()?;
                    }
                }
            } else if byte == b'(' {
                scanner.open_paren()?;
            } else if byte == b')' {
                if scanner.depth == 0 {
                    return Err(ReferenceIndexError::InvalidCss);
                }
                scanner.depth -= 1;
                scanner.cursor += 1;
            } else {
                scanner.advance_char();
            }
        }
        if scanner.depth != 0 {
            return Err(ReferenceIndexError::InvalidCss);
        }
        Ok(refs)
    })();
    result.map_err(|reason| {
        let (line, column) = super::markup::css_position(text, scanner.cursor, 1, 1);
        super::at("", Some(line), Some(column), reason)
    })
}

struct Scanner<'a, 'b> {
    text: &'a str,
    cursor: usize,
    budget: &'b mut Budget,
    depth: usize,
}

impl Scanner<'_, '_> {
    fn starts_number(&self) -> bool {
        let bytes = &self.text.as_bytes()[self.cursor..];
        let bytes = if bytes.first().is_some_and(|b| matches!(b, b'+' | b'-')) {
            &bytes[1..]
        } else {
            bytes
        };
        bytes.first().is_some_and(u8::is_ascii_digit)
            || (bytes.first() == Some(&b'.') && bytes.get(1).is_some_and(u8::is_ascii_digit))
    }
    fn number(&mut self) {
        if matches!(self.peek(), Some(b'+' | b'-')) {
            self.cursor += 1;
        }
        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
            self.cursor += 1;
        }
        if self.peek() == Some(b'.')
            && self
                .text
                .as_bytes()
                .get(self.cursor + 1)
                .is_some_and(u8::is_ascii_digit)
        {
            self.cursor += 1;
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.cursor += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            let start = self.cursor;
            self.cursor += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.cursor += 1;
            }
            if self.peek().is_some_and(|b| b.is_ascii_digit()) {
                while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                    self.cursor += 1;
                }
            } else {
                self.cursor = start;
            }
        }
    }
    fn byte(&self) -> u8 {
        self.text.as_bytes()[self.cursor]
    }
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.cursor).copied()
    }
    fn starts(&self, value: &[u8]) -> bool {
        self.text.as_bytes()[self.cursor..].starts_with(value)
    }
    fn advance_char(&mut self) {
        if let Some(value) = self.text[self.cursor..].chars().next() {
            self.cursor += value.len_utf8();
        }
    }
    fn open_paren(&mut self) -> Result<(), ReferenceIndexError> {
        self.depth = self
            .depth
            .checked_add(1)
            .ok_or(ReferenceIndexError::CssFunctionDepthLimit)?;
        if self.depth > self.budget.limits.max_css_function_depth {
            return Err(ReferenceIndexError::CssFunctionDepthLimit);
        }
        self.cursor += 1;
        Ok(())
    }
    fn comment(&mut self) -> Result<(), ReferenceIndexError> {
        self.cursor += 2;
        let Some(end) = self.text[self.cursor..].find("*/") else {
            return Err(ReferenceIndexError::InvalidCss);
        };
        self.cursor += end + 2;
        Ok(())
    }
    fn space_and_comments(&mut self) -> Result<(), ReferenceIndexError> {
        loop {
            while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
                self.cursor += 1;
            }
            if self.starts(b"/*") {
                self.budget.token()?;
                self.comment()?;
            } else {
                return Ok(());
            }
        }
    }
    fn escape(&mut self, allow_continuation: bool) -> Result<Option<char>, ReferenceIndexError> {
        self.cursor += 1;
        let Some(first) = self.peek() else {
            return Err(ReferenceIndexError::InvalidCss);
        };
        if matches!(first, b'\n' | b'\r' | 0x0c) {
            if !allow_continuation {
                return Err(ReferenceIndexError::InvalidCss);
            }
            self.cursor += 1;
            if first == b'\r' && self.peek() == Some(b'\n') {
                self.cursor += 1;
            }
            return Ok(None);
        }
        if first.is_ascii_hexdigit() {
            let mut value = 0_u32;
            for _ in 0..6 {
                let Some(byte) = self.peek().filter(u8::is_ascii_hexdigit) else {
                    break;
                };
                value = value * 16 + (byte as char).to_digit(16).unwrap_or(0);
                self.cursor += 1;
            }
            if self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
                let first = self.byte();
                self.cursor += 1;
                if first == b'\r' && self.peek() == Some(b'\n') {
                    self.cursor += 1;
                }
            }
            return Ok(Some(
                char::from_u32(value)
                    .filter(|c| *c != '\0')
                    .unwrap_or('\u{fffd}'),
            ));
        }
        let value = self.text[self.cursor..]
            .chars()
            .next()
            .ok_or(ReferenceIndexError::InvalidCss)?;
        self.cursor += value.len_utf8();
        Ok(Some(value))
    }
    fn name(&mut self) -> Result<String, ReferenceIndexError> {
        let mut output = String::new();
        while let Some(byte) = self.peek() {
            let value = if byte == b'\\' {
                self.escape(false)?
            } else if is_name(byte) {
                let value = self.text[self.cursor..]
                    .chars()
                    .next()
                    .ok_or(ReferenceIndexError::InvalidCss)?;
                self.cursor += value.len_utf8();
                Some(value)
            } else {
                break;
            };
            if output.len() < 64
                && let Some(value) = value
            {
                output.push(value);
            }
        }
        Ok(output)
    }
    fn string(&mut self, capture: bool) -> Result<(String, usize, usize), ReferenceIndexError> {
        let quote = self.byte();
        self.cursor += 1;
        let begin = self.cursor;
        let mut output = String::new();
        while let Some(byte) = self.peek() {
            if byte == quote {
                let end = self.cursor;
                self.cursor += 1;
                return Ok((output, begin, end));
            }
            if matches!(byte, b'\n' | b'\r' | 0x0c) {
                return Err(ReferenceIndexError::InvalidCss);
            }
            let value = if byte == b'\\' {
                self.escape(true)?
            } else {
                let value = self.text[self.cursor..]
                    .chars()
                    .next()
                    .ok_or(ReferenceIndexError::InvalidCss)?;
                self.cursor += value.len_utf8();
                Some(value)
            };
            if capture && let Some(value) = value {
                self.push_char(&mut output, value)?;
            }
        }
        Err(ReferenceIndexError::InvalidCss)
    }
    fn push_char(&self, output: &mut String, value: char) -> Result<(), ReferenceIndexError> {
        if output
            .len()
            .checked_add(value.len_utf8())
            .is_none_or(|n| n > self.budget.limits.max_reference_bytes)
        {
            return Err(ReferenceIndexError::ReferenceByteLimit);
        }
        output.push(value);
        Ok(())
    }
    fn url(&mut self) -> Result<Option<(String, usize, usize)>, ReferenceIndexError> {
        self.open_paren()?;
        self.space_and_comments()?;
        if matches!(self.peek(), Some(b'\'' | b'"')) {
            let result = self.string(true)?;
            self.space_and_comments()?;
            if self.peek() != Some(b')') {
                return Err(ReferenceIndexError::InvalidCss);
            }
            self.cursor += 1;
            self.depth -= 1;
            return Ok(Some(result));
        }
        let begin = self.cursor;
        let mut value = String::new();
        let end;
        loop {
            let Some(byte) = self.peek() else {
                return Err(ReferenceIndexError::InvalidCss);
            };
            if byte == b')' {
                end = self.cursor;
                self.cursor += 1;
                self.depth -= 1;
                break;
            }
            if byte.is_ascii_whitespace() {
                end = self.cursor;
                self.space_and_comments()?;
                if self.peek() != Some(b')') {
                    return Err(ReferenceIndexError::InvalidCss);
                }
                self.cursor += 1;
                self.depth -= 1;
                break;
            }
            if matches!(byte, b'\'' | b'"' | b'(' | 0..=8 | 11 | 14..=31 | 127) {
                return Err(ReferenceIndexError::InvalidCss);
            }
            let character = if byte == b'\\' {
                self.escape(false)?
            } else {
                let value = self.text[self.cursor..]
                    .chars()
                    .next()
                    .ok_or(ReferenceIndexError::InvalidCss)?;
                self.cursor += value.len_utf8();
                Some(value)
            };
            if let Some(character) = character {
                self.push_char(&mut value, character)?;
            }
        }
        Ok(Some((value, begin, end)))
    }
    fn push(
        &mut self,
        output: &mut Vec<CssReference>,
        kind: ReferenceKind,
        value: String,
        position: usize,
        begin: usize,
        end: usize,
    ) -> Result<(), ReferenceIndexError> {
        self.budget.reference(
            end.checked_sub(begin)
                .ok_or(ReferenceIndexError::InvalidCss)?,
        )?;
        output.push(CssReference {
            kind,
            value,
            raw: self.text[begin..end].to_owned(),
            position,
            range: (begin, end),
        });
        Ok(())
    }
}

fn is_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'-') || byte >= 0x80
}
fn is_name(byte: u8) -> bool {
    is_name_start(byte) || byte.is_ascii_digit()
}

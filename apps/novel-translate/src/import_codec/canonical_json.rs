//! Bounded lexical pass before typed serde allocation, followed by streaming
//! canonical-byte comparison. The enclosing volume codec owns framing and
//! source/ownership/hash validation; this module only validates JSON records.
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};

use serde::{Serialize, de::DeserializeOwned};

use super::CodecError;

const BUFFER: usize = 64 * 1024;
const MAX_ENCODED: u64 = 512 * 1024 * 1024;
const MAX_WORKING: usize = 256 * 1024 * 1024;
const MAX_RECORDS: usize = 32_000_000;
const MAX_DEPTH: usize = 128;
const MAX_STRING: usize = 32 * 1024 * 1024;
const MAX_KEY: usize = 128;
const MAX_FIELDS: usize = 64;
const OBJECT_BYTES: usize = 1024;
const ADJACENT_TAG_BYTES: usize = 256;

#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub bytes: u64,
    pub working: usize,
    pub records: usize,
    pub depth: usize,
    pub string: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            bytes: MAX_ENCODED,
            working: MAX_WORKING,
            records: MAX_RECORDS,
            depth: MAX_DEPTH,
            string: MAX_STRING,
        }
    }
}
impl Limits {
    fn bounded(self) -> Self {
        Self {
            bytes: self.bytes.min(MAX_ENCODED),
            working: self.working.min(MAX_WORKING),
            records: self.records.min(MAX_RECORDS),
            depth: self.depth.min(MAX_DEPTH),
            string: self.string.min(MAX_STRING),
        }
    }
}
struct Scanner<R> {
    input: BufReader<R>,
    limits: Limits,
    working: usize,
    records: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Field {
    Kind,
    Data,
    Other,
}

impl<R: Read> Scanner<R> {
    fn peek(&mut self) -> Result<Option<u8>, CodecError> {
        self.input
            .fill_buf()
            .map(|bytes| bytes.first().copied())
            .map_err(|_| CodecError::Io)
    }
    fn byte(&mut self) -> Result<u8, CodecError> {
        let byte = self.peek()?.ok_or(CodecError::InvalidJson)?;
        self.input.consume(1);
        Ok(byte)
    }
    fn space(&mut self) -> Result<(), CodecError> {
        while self
            .peek()?
            .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.input.consume(1);
        }
        Ok(())
    }
    fn expect(&mut self, byte: u8) -> Result<(), CodecError> {
        if self.byte()? == byte {
            Ok(())
        } else {
            Err(CodecError::InvalidJson)
        }
    }
    fn charge(&mut self, bytes: usize) -> Result<(), CodecError> {
        self.working = self
            .working
            .checked_add(bytes)
            .filter(|n| *n <= self.limits.working)
            .ok_or(CodecError::ResourceLimit)?;
        Ok(())
    }
    fn key(&mut self) -> Result<Field, CodecError> {
        self.expect(b'"')?;
        let maximum = MAX_KEY.min(self.limits.string);
        let mut value = [0; MAX_KEY];
        let mut count = 0;
        loop {
            match self.byte()? {
                b'"' => break,
                // Closed DTO field names are ASCII and use no escapes.
                0..=31 | b'\\' | 128..=255 => return Err(CodecError::InvalidJson),
                byte if count < maximum => {
                    value[count] = byte;
                    count += 1;
                }
                _ => return Err(CodecError::ResourceLimit),
            }
        }
        // The typed readers consume keys directly. Their scratch space can be
        // reused across records instead of accumulating in the budget.
        Ok(match &value[..count] {
            b"kind" => Field::Kind,
            b"data" => Field::Data,
            _ => Field::Other,
        })
    }
    fn string(&mut self) -> Result<(), CodecError> {
        self.expect(b'"')?;
        let maximum = self.limits.string;
        let mut raw = 0_usize;
        loop {
            match self.byte()? {
                b'"' => break,
                0..=31 => return Err(CodecError::InvalidJson),
                b'\\' => {
                    raw = raw
                        .checked_add(1)
                        .filter(|n| *n <= maximum)
                        .ok_or(CodecError::ResourceLimit)?;
                    match self.byte()? {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                        b'u' => {
                            for _ in 0..4 {
                                if !self.byte()?.is_ascii_hexdigit() {
                                    return Err(CodecError::InvalidJson);
                                }
                            }
                            raw = raw.checked_add(4).ok_or(CodecError::ResourceLimit)?;
                        }
                        _ => return Err(CodecError::InvalidJson),
                    }
                    raw = raw
                        .checked_add(1)
                        .filter(|n| *n <= maximum)
                        .ok_or(CodecError::ResourceLimit)?;
                }
                _ => {
                    raw = raw
                        .checked_add(1)
                        .filter(|n| *n <= maximum)
                        .ok_or(CodecError::ResourceLimit)?;
                }
            }
        }
        // Includes retained typed strings, growth/scratch and the current
        // decoded token conservatively. The scanner itself retains no values.
        self.charge_string(raw)
    }
    fn charge_string(&mut self, raw: usize) -> Result<(), CodecError> {
        self.charge(
            raw.checked_mul(4)
                .and_then(|n| n.checked_add(64))
                .ok_or(CodecError::ResourceLimit)?,
        )
    }
    fn number(&mut self) -> Result<(), CodecError> {
        let first = self.byte()?;
        if first == b'0' {
            return Ok(());
        }
        let mut value = u64::from(first - b'0');
        while let Some(byte @ b'0'..=b'9') = self.peek()? {
            self.input.consume(1);
            value = value
                .checked_mul(10)
                .and_then(|n| n.checked_add(u64::from(byte - b'0')))
                .ok_or(CodecError::InvalidJson)?;
        }
        Ok(())
    }
    fn object(&mut self, depth: usize) -> Result<(), CodecError> {
        self.expect(b'{')?;
        self.space()?;
        if self.peek()? == Some(b'}') {
            self.input.consume(1);
            return self.charge(OBJECT_BYTES);
        }
        let mut fields = 0;
        let mut tag_first = false;
        let mut tagged_object = false;
        loop {
            fields += 1;
            if fields > MAX_FIELDS {
                return Err(CodecError::ResourceLimit);
            }
            self.space()?;
            let field = self.key()?;
            self.space()?;
            self.expect(b':')?;
            self.space()?;
            let first = self.peek()?;
            if fields == 1 {
                tag_first = field == Field::Kind && first == Some(b'"');
            }
            if field == Field::Data {
                // All adjacent enums in the wire format have their tag
                // first. Otherwise serde would buffer an arbitrary value.
                if fields != 2 || !tag_first {
                    return Err(CodecError::NonCanonicalJson);
                }
                tagged_object = first == Some(b'{');
            }
            self.value(depth + 1)?;
            self.space()?;
            match self.byte()? {
                b'}' => break,
                b',' => {}
                _ => return Err(CodecError::InvalidJson),
            }
        }
        let layout = if fields == 2 && tagged_object {
            // The data object has already reserved the native payload and
            // collection growth. The enclosing enum stores that payload
            // inline, with this additional allowance for the tag and union.
            ADJACENT_TAG_BYTES
        } else {
            OBJECT_BYTES
        };
        self.charge(layout)
    }
    fn value(&mut self, depth: usize) -> Result<(), CodecError> {
        if depth > self.limits.depth {
            return Err(CodecError::ResourceLimit);
        }
        self.records = self
            .records
            .checked_add(1)
            .filter(|n| *n <= self.limits.records)
            .ok_or(CodecError::ResourceLimit)?;
        self.charge(32)?;
        self.space()?;
        match self.peek()?.ok_or(CodecError::InvalidJson)? {
            b'{' => self.object(depth)?,
            b'[' => {
                self.charge(64)?;
                self.input.consume(1);
                self.space()?;
                if self.peek()? == Some(b']') {
                    self.input.consume(1);
                    return Ok(());
                }
                loop {
                    self.value(depth + 1)?;
                    self.space()?;
                    match self.byte()? {
                        b']' => break,
                        b',' => {}
                        _ => return Err(CodecError::InvalidJson),
                    }
                }
            }
            b'"' => self.string()?,
            b'0'..=b'9' => self.number()?,
            b't' => {
                for byte in b"true" {
                    self.expect(*byte)?;
                }
            }
            b'f' => {
                for byte in b"false" {
                    self.expect(*byte)?;
                }
            }
            b'n' => {
                for byte in b"null" {
                    self.expect(*byte)?;
                }
            }
            _ => return Err(CodecError::InvalidJson),
        }
        Ok(())
    }
}

/// Measure the budget for this exact closed JSON segment without allocating
/// typed records. The writer and reader use the same limits.
pub(super) fn preflight(
    source: &mut impl Read,
    length: u64,
    limits: Limits,
) -> Result<usize, CodecError> {
    let limits = limits.bounded();
    validate_segment_limits(length, limits)?;
    let mut scan = Scanner {
        input: BufReader::with_capacity(BUFFER, source.take(length)),
        limits,
        working: BUFFER,
        records: 0,
    };
    scan.value(0)?;
    scan.space()?;
    if scan.peek()?.is_some() || scan.input.get_ref().limit() != 0 {
        return Err(CodecError::InvalidJson);
    }
    Ok(scan.working)
}

fn validate_segment_limits(length: u64, limits: Limits) -> Result<(), CodecError> {
    if length == 0 {
        return Err(CodecError::InvalidJson);
    }
    if length > limits.bytes || limits.working < BUFFER {
        return Err(CodecError::ResourceLimit);
    }
    Ok(())
}

struct Compare<R> {
    input: BufReader<R>,
    remaining: u64,
    mismatch: bool,
}
impl<R: Read> Write for Compare<R> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() as u64 > self.remaining {
            self.mismatch = true;
            return Err(io::Error::other("Noncanonical stored JSON."));
        }
        let mut expected = [0_u8; 4096];
        for part in bytes.chunks(expected.len()) {
            self.input.read_exact(&mut expected[..part.len()])?;
            if &expected[..part.len()] != part {
                self.mismatch = true;
                return Err(io::Error::other("Noncanonical stored JSON."));
            }
        }
        self.remaining -= bytes.len() as u64;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Decode exactly one byte-bounded JSON segment at the current seek position.
/// Declared framing, outer trailing bytes and original source are caller gates.
/// The caller must hold an immutable verified stream across all three passes.
/// Call only for closed typed DTOs; Value/maps and unbounded custom visitors are
/// not an allowed private storage schema.
pub(super) fn decode<T: DeserializeOwned + Serialize>(
    source: &mut (impl Read + Seek),
    length: u64,
    limits: Limits,
) -> Result<T, CodecError> {
    let limits = limits.bounded();
    validate_segment_limits(length, limits)?;
    let start = source.stream_position().map_err(|_| CodecError::Io)?;
    start.checked_add(length).ok_or(CodecError::ResourceLimit)?;
    preflight(source, length, limits)?;
    source
        .seek(SeekFrom::Start(start))
        .map_err(|_| CodecError::Io)?;
    let value = {
        let buffered = BufReader::with_capacity(BUFFER, (&mut *source).take(length));
        let mut reader = serde_json::Deserializer::from_reader(buffered);
        let value = T::deserialize(&mut reader).map_err(|error| {
            if error.is_io() {
                CodecError::Io
            } else {
                CodecError::InvalidJson
            }
        })?;
        reader.end().map_err(|error| {
            if error.is_io() {
                CodecError::Io
            } else {
                CodecError::InvalidJson
            }
        })?;
        value
    };
    source
        .seek(SeekFrom::Start(start))
        .map_err(|_| CodecError::Io)?;
    let mut compare = Compare {
        input: BufReader::with_capacity(BUFFER, (&mut *source).take(length)),
        remaining: length,
        mismatch: false,
    };
    if serde_json::to_writer(&mut compare, &value).is_err() {
        return Err(if compare.mismatch {
            CodecError::NonCanonicalJson
        } else {
            CodecError::Io
        });
    }
    if compare.remaining != 0 {
        return Err(CodecError::NonCanonicalJson);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::{cell::Cell, io::Cursor, rc::Rc};

    #[test]
    fn repeated_tagged_payloads_fit_without_buffering_keys_or_counting_two_payloads() {
        #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
        #[serde(tag = "kind", content = "data", deny_unknown_fields)]
        enum Entry {
            Accepted { position: u64 },
        }
        let expected = vec![Entry::Accepted { position: 42 }; 16];
        let encoded = serde_json::to_vec(&expected).unwrap();
        let limit = Limits {
            working: BUFFER + 26_000,
            ..Limits::default()
        };
        assert_eq!(
            decode::<Vec<Entry>>(&mut Cursor::new(&encoded), encoded.len() as u64, limit),
            Ok(expected),
        );
        let reordered = br#"{"data":{"position":42},"kind":"Accepted"}"#;
        assert_eq!(
            decode::<Entry>(&mut Cursor::new(reordered), reordered.len() as u64, limit),
            Err(CodecError::NonCanonicalJson),
        );
        let too_many = vec![Entry::Accepted { position: 42 }; 32];
        let encoded = serde_json::to_vec(&too_many).unwrap();
        assert!(matches!(
            preflight(&mut Cursor::new(&encoded), encoded.len() as u64, limit),
            Err(CodecError::ResourceLimit)
        ));
    }

    #[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Record {
        label: String,
        count: u64,
        flags: Vec<bool>,
    }

    fn record(bytes: &[u8]) -> Result<Record, CodecError> {
        decode(
            &mut Cursor::new(bytes),
            bytes.len() as u64,
            Limits::default(),
        )
    }

    struct Counted {
        inner: Cursor<Vec<u8>>,
        reads: Rc<Cell<usize>>,
        seeks: Rc<Cell<usize>>,
        largest_request: Rc<Cell<usize>>,
    }
    impl Counted {
        fn new(bytes: Vec<u8>) -> Self {
            Self {
                inner: Cursor::new(bytes),
                reads: Rc::new(Cell::new(0)),
                seeks: Rc::new(Cell::new(0)),
                largest_request: Rc::new(Cell::new(0)),
            }
        }
    }
    impl Read for Counted {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.reads.set(self.reads.get() + 1);
            self.largest_request
                .set(self.largest_request.get().max(buf.len()));
            self.inner.read(buf)
        }
    }
    impl Seek for Counted {
        fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
            self.seeks.set(self.seeks.get() + 1);
            self.inner.seek(pos)
        }
    }

    #[test]
    fn canonical_unicode_record_reads_only_the_declared_segment() {
        let expected = Record {
            label: "a/é👩🏽‍💻\n".to_owned(),
            count: u64::MAX,
            flags: vec![true, false],
        };
        let json = serde_json::to_vec(&expected).unwrap();
        let mut bytes = b"PREFIX".to_vec();
        bytes.extend_from_slice(&json);
        bytes.extend_from_slice(b"unrelated trailer");
        let mut source = Cursor::new(bytes);
        source.set_position(6);
        assert_eq!(
            decode::<Record>(&mut source, json.len() as u64, Limits::default()),
            Ok(expected)
        );
        assert_eq!(source.position(), 6 + json.len() as u64);
        assert_eq!(
            &source.get_ref()[source.position() as usize..],
            b"unrelated trailer"
        );
    }

    #[test]
    fn typed_reader_rejects_duplicate_unknown_and_wrong_width_fields() {
        for bytes in [
            br#"{"label":"x","label":"y","count":1,"flags":[]}"#.as_slice(),
            br#"{"label":"x","co\u0075nt":1,"count":2,"flags":[]}"#,
            br#"{"label":"x","count":1,"flags":[],"extra":"private"}"#,
            br#"{"label":"x","count":"1","flags":[]}"#,
            br#"{"label":"x","count":1,"flags":[1]}"#,
            br#"{"label":"x","count":18446744073709551616,"flags":[]}"#,
            br#"{"label":"x","count":-1,"flags":[]}"#,
            br#"{"label":"x","count":1.0,"flags":[]}"#,
            br#"{"label":"x","count":1e0,"flags":[]}"#,
            br#"{"label":"x","count":01,"flags":[]}"#,
            br#"{"label":"\uD800","count":1,"flags":[]}"#,
        ] {
            assert_eq!(record(bytes), Err(CodecError::InvalidJson));
        }
        let mut invalid_utf8 = br#"{"label":"x","count":1,"flags":[]}"#.to_vec();
        invalid_utf8[10] = 0xff;
        assert_eq!(record(&invalid_utf8), Err(CodecError::InvalidJson));
    }

    #[test]
    fn equivalent_json_has_one_stored_byte_representation() {
        for bytes in [
            br#"{"count":1,"label":"x","flags":[]}"#.as_slice(),
            br#"{ "label":"x","count":1,"flags":[]}"#,
            br#"{"label":"\u0078","count":1,"flags":[]}"#,
            br#"{"label":"\/","count":1,"flags":[]}"#,
            br#"{"label":"x","count":1,"flags":[]} "#,
            b"\n{\"label\":\"x\",\"count\":1,\"flags\":[]}",
        ] {
            assert_eq!(record(bytes), Err(CodecError::NonCanonicalJson));
        }
    }

    #[test]
    fn malformed_or_wrong_length_segment_never_accepts_a_prefix() {
        let bytes = br#"{"label":"x","count":1,"flags":[]}"#;
        for length in [0, 1, bytes.len() as u64 - 1, bytes.len() as u64 + 1] {
            assert_eq!(
                decode::<Record>(&mut Cursor::new(bytes), length, Limits::default()),
                Err(CodecError::InvalidJson)
            );
        }
        for invalid in [
            b"[1,]".as_slice(),
            b"{\"label\":}",
            b"truefalse",
            b"[] []",
            b"\"a\n\"",
            b"\"\\q\"",
            b"\"\\u12x4\"",
            b"[18446744073709551616]",
        ] {
            assert_eq!(
                decode::<Vec<u64>>(
                    &mut Cursor::new(invalid),
                    invalid.len() as u64,
                    Limits::default()
                ),
                Err(CodecError::InvalidJson)
            );
        }
    }

    #[test]
    fn declared_byte_and_initial_working_limits_reject_before_any_io() {
        for (length, limits) in [
            (MAX_ENCODED + 1, Limits::default()),
            (
                2,
                Limits {
                    bytes: 1,
                    ..Limits::default()
                },
            ),
            (
                2,
                Limits {
                    working: BUFFER - 1,
                    ..Limits::default()
                },
            ),
        ] {
            let mut source = Counted::new(b"[]".to_vec());
            assert_eq!(
                decode::<Vec<u64>>(&mut source, length, limits),
                Err(CodecError::ResourceLimit)
            );
            assert_eq!(source.reads.get(), 0);
            assert_eq!(source.seeks.get(), 0);
        }
    }

    #[test]
    fn record_depth_and_working_limits_accept_exact_boundary() {
        let bytes = b"[7]";
        for limits in [
            Limits {
                bytes: 3,
                records: 2,
                depth: 1,
                working: BUFFER + 128,
                ..Limits::default()
            },
            Limits::default(),
        ] {
            assert_eq!(
                decode::<Vec<u64>>(&mut Cursor::new(bytes), 3, limits),
                Ok(vec![7])
            );
        }
        for limits in [
            Limits {
                bytes: 2,
                ..Limits::default()
            },
            Limits {
                records: 1,
                ..Limits::default()
            },
            Limits {
                depth: 0,
                ..Limits::default()
            },
            Limits {
                working: BUFFER + 127,
                ..Limits::default()
            },
        ] {
            assert_eq!(
                decode::<Vec<u64>>(&mut Cursor::new(bytes), 3, limits),
                Err(CodecError::ResourceLimit)
            );
        }
        let nested = b"[[7]]";
        assert_eq!(
            decode::<Vec<Vec<u64>>>(
                &mut Cursor::new(nested),
                5,
                Limits {
                    depth: 2,
                    ..Limits::default()
                }
            ),
            Ok(vec![vec![7]])
        );
        assert_eq!(
            decode::<Vec<Vec<u64>>>(
                &mut Cursor::new(nested),
                5,
                Limits {
                    depth: 1,
                    ..Limits::default()
                }
            ),
            Err(CodecError::ResourceLimit)
        );
    }

    #[test]
    fn raw_string_escape_and_field_count_limits_are_bounded_before_serde() {
        for (bytes, boundary, accepted) in [
            (b"[\"ab\"]".as_slice(), 2, Ok(vec!["ab".to_owned()])),
            (
                b"[\"\\u0061\"]".as_slice(),
                6,
                Err(CodecError::NonCanonicalJson),
            ),
        ] {
            assert_eq!(
                decode::<Vec<String>>(
                    &mut Cursor::new(bytes),
                    bytes.len() as u64,
                    Limits {
                        string: boundary,
                        ..Limits::default()
                    }
                ),
                accepted
            );
            assert_eq!(
                decode::<Vec<String>>(
                    &mut Cursor::new(bytes),
                    bytes.len() as u64,
                    Limits {
                        string: boundary - 1,
                        ..Limits::default()
                    }
                ),
                Err(CodecError::ResourceLimit)
            );
        }
        let too_many_fields = format!(
            "{{{}}}",
            (0..65)
                .map(|n| format!("\"x{n}\":0"))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert_eq!(
            record(too_many_fields.as_bytes()),
            Err(CodecError::ResourceLimit)
        );
        let long_key = format!("{{\"{}\":0}}", "k".repeat(MAX_KEY + 1));
        assert_eq!(record(long_key.as_bytes()), Err(CodecError::ResourceLimit));
    }

    #[test]
    fn canonical_comparison_streams_large_strings_in_bounded_reads() {
        let expected = Record {
            label: "é".repeat(1_000_000),
            count: 8,
            flags: vec![],
        };
        let json = serde_json::to_vec(&expected).unwrap();
        let length = json.len() as u64;
        let mut source = Counted::new(json);
        assert_eq!(
            decode::<Record>(&mut source, length, Limits::default()),
            Ok(expected)
        );
        assert!(source.reads.get() > 30);
        assert!(source.largest_request.get() <= BUFFER);
        assert_eq!(source.inner.position(), length);
    }

    #[test]
    fn io_errors_are_distinct_and_do_not_echo_input() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("secret path"))
            }
        }
        impl Seek for Broken {
            fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
                Ok(0)
            }
        }
        assert_eq!(
            decode::<Record>(&mut Broken, 2, Limits::default()),
            Err(CodecError::Io)
        );
        assert!(!CodecError::Io.to_string().contains("secret"));
    }
}

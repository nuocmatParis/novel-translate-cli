use crate::xml::{XmlFailure, XmlFailureKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum XmlEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
}

pub(crate) fn decode_xml(bytes: &[u8], max_bytes: usize) -> Result<String, XmlFailure> {
    if bytes.len() > max_bytes {
        return Err(XmlFailure::new(XmlFailureKind::ByteLimit, None));
    }

    let (encoding, payload) = detect_encoding(bytes);
    let text = match encoding {
        XmlEncoding::Utf8 => std::str::from_utf8(payload)
            .map(str::to_owned)
            .map_err(|_| XmlFailure::new(XmlFailureKind::UnsupportedEncoding, None))?,
        XmlEncoding::Utf16Le => decode_utf16(payload, true)?,
        XmlEncoding::Utf16Be => decode_utf16(payload, false)?,
    };

    if text.len() > max_bytes {
        return Err(XmlFailure::new(XmlFailureKind::ByteLimit, None));
    }
    if !declaration_matches(&text, encoding) {
        return Err(XmlFailure::new(XmlFailureKind::UnsupportedEncoding, None));
    }
    Ok(text)
}

fn detect_encoding(bytes: &[u8]) -> (XmlEncoding, &[u8]) {
    if let Some(payload) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        return (XmlEncoding::Utf8, payload);
    }
    if let Some(payload) = bytes.strip_prefix(&[0xff, 0xfe]) {
        return (XmlEncoding::Utf16Le, payload);
    }
    if let Some(payload) = bytes.strip_prefix(&[0xfe, 0xff]) {
        return (XmlEncoding::Utf16Be, payload);
    }
    if bytes.starts_with(&[0x3c, 0x00, 0x3f, 0x00]) {
        return (XmlEncoding::Utf16Le, bytes);
    }
    if bytes.starts_with(&[0x00, 0x3c, 0x00, 0x3f]) {
        return (XmlEncoding::Utf16Be, bytes);
    }
    (XmlEncoding::Utf8, bytes)
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> Result<String, XmlFailure> {
    if !bytes.len().is_multiple_of(2) {
        return Err(XmlFailure::new(XmlFailureKind::UnsupportedEncoding, None));
    }
    let (pairs, _) = bytes.as_chunks::<2>();
    let units = pairs.iter().map(|pair| {
        if little_endian {
            u16::from_le_bytes(*pair)
        } else {
            u16::from_be_bytes(*pair)
        }
    });
    let units = units.collect::<Vec<_>>();
    String::from_utf16(&units)
        .map_err(|_| XmlFailure::new(XmlFailureKind::UnsupportedEncoding, None))
}

fn declaration_matches(text: &str, detected: XmlEncoding) -> bool {
    let Some(declaration) = text.strip_prefix("<?xml") else {
        return true;
    };
    let Some((declaration, _)) = declaration.split_once("?>") else {
        return true;
    };

    let Some(value) = declaration_encoding(declaration) else {
        return true;
    };
    let value = value.to_ascii_lowercase();
    matches!(
        (detected, value.as_str()),
        (XmlEncoding::Utf8, "utf-8" | "utf8")
            | (XmlEncoding::Utf16Le, "utf-16" | "utf-16le")
            | (XmlEncoding::Utf16Be, "utf-16" | "utf-16be")
    )
}

fn declaration_encoding(declaration: &str) -> Option<&str> {
    let bytes = declaration.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let start = cursor;
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric()
                || bytes[cursor] == b'_'
                || bytes[cursor] == b'-')
        {
            cursor += 1;
        }
        if start == cursor {
            cursor += 1;
            continue;
        }
        let name = &declaration[start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor == bytes.len() || bytes[cursor] != b'=' {
            continue;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor == bytes.len() || !matches!(bytes[cursor], b'\'' | b'"') {
            return None;
        }
        let quote = bytes[cursor];
        cursor += 1;
        let value_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != quote {
            cursor += 1;
        }
        if cursor == bytes.len() {
            return None;
        }
        let value = &declaration[value_start..cursor];
        cursor += 1;
        if name == "encoding" {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::decode_xml;
    use crate::xml::XmlFailureKind;

    fn utf16(text: &str, little_endian: bool, with_bom: bool) -> Vec<u8> {
        let mut result = Vec::new();
        if with_bom {
            result.extend_from_slice(if little_endian {
                &[0xff, 0xfe]
            } else {
                &[0xfe, 0xff]
            });
        }
        for unit in text.encode_utf16() {
            let pair = if little_endian {
                unit.to_le_bytes()
            } else {
                unit.to_be_bytes()
            };
            result.extend_from_slice(&pair);
        }
        result
    }

    #[test]
    fn accepts_utf8_and_both_utf16_byte_orders() {
        let text = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root>日本語</root>";
        assert_eq!(decode_xml(text.as_bytes(), 1024).unwrap(), text);
        for (little, label) in [(true, "UTF-16LE"), (false, "UTF-16BE")] {
            let text = format!("<?xml version=\"1.0\" encoding=\"{label}\"?><root>日本語</root>");
            assert_eq!(decode_xml(&utf16(&text, little, true), 1024).unwrap(), text);
        }
    }

    #[test]
    fn enforces_raw_and_decoded_byte_caps_and_encoding_declaration() {
        assert_eq!(decode_xml(b"<x/>", 5).unwrap(), "<x/>");
        assert_eq!(decode_xml(b"<x/>", 4).unwrap(), "<x/>");
        assert_eq!(
            decode_xml(b"<x/>", 3).unwrap_err().kind,
            XmlFailureKind::ByteLimit
        );
        assert_eq!(
            decode_xml(b"<xy/>", 4).unwrap_err().kind,
            XmlFailureKind::ByteLimit
        );
        let utf16_bytes = utf16("<x/>", true, true);
        assert_eq!(decode_xml(&utf16_bytes, 1024).unwrap(), "<x/>");
        assert_eq!(
            decode_xml(&utf16_bytes, 7).unwrap_err().kind,
            XmlFailureKind::ByteLimit
        );
        let mismatch = utf16("<?xml version=\"1.0\" encoding=\"UTF-8\"?><x/>", true, true);
        assert_eq!(
            decode_xml(&mismatch, 1024).unwrap_err().kind,
            XmlFailureKind::UnsupportedEncoding
        );
    }
}

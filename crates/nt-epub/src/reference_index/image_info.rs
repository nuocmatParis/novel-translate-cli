//! Bounded source-image metadata only. Full image validity/edit/export checks
//! belong to the image workflow; recognizing a header does not satisfy them.
use super::{ReferenceIndexError, xml_error};
use crate::{SourceExtraction, xml::with_xml_document};
use nt_domain::volume_import::{DetectedImageFormat as Format, ImageInformation};

const MAX_BYTES: usize = 128 * 1024 * 1024;
const MAX_JPEG_HEADER: usize = 1024 * 1024;

impl SourceExtraction {
    pub fn image_information(
        bytes: &[u8],
        declared_media_type: &str,
    ) -> Result<ImageInformation, ReferenceIndexError> {
        if bytes.len() > MAX_BYTES {
            return Err(ReferenceIndexError::ArchiveEntryLimit);
        }
        let (format, dimensions) = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            let dimensions = if bytes.get(8..16) == Some(b"\0\0\0\rIHDR") {
                bytes.get(16..24).and_then(|data| {
                    positive(
                        u32::from_be_bytes(data[..4].try_into().ok()?),
                        u32::from_be_bytes(data[4..].try_into().ok()?),
                    )
                })
            } else {
                None
            };
            (Format::Png, dimensions)
        } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            (
                Format::Gif,
                bytes.get(6..10).and_then(|data| {
                    positive(
                        u32::from(u16::from_le_bytes(data[..2].try_into().ok()?)),
                        u32::from(u16::from_le_bytes(data[2..].try_into().ok()?)),
                    )
                }),
            )
        } else if bytes.starts_with(b"\xff\xd8") {
            (Format::Jpeg, jpeg_dimensions(bytes))
        } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
            (Format::Webp, webp_dimensions(bytes))
        } else if declared_media_type.eq_ignore_ascii_case("image/svg+xml") {
            return with_xml_document(
                bytes,
                crate::xml::DEFAULT_XML_MAX_BYTES,
                crate::xml::DEFAULT_XML_MAX_NODES,
                |doc| {
                    let root = doc.root_element();
                    if root.tag_name().namespace() != Some("http://www.w3.org/2000/svg")
                        || root.tag_name().name() != "svg"
                    {
                        return Err(ReferenceIndexError::SourceMismatch);
                    }
                    let attribute = |name| {
                        root.attributes()
                            .find(|a| a.namespace().is_none() && a.name() == name)
                            .map(|a| a.value().to_owned())
                    };
                    let width = attribute("width");
                    let height = attribute("height");
                    let view_box = attribute("viewBox");
                    let dimensions = width
                        .as_deref()
                        .and_then(pixels)
                        .zip(height.as_deref().and_then(pixels));
                    Ok(ImageInformation {
                        format: Some(Format::Svg),
                        pixel_dimensions: dimensions,
                        svg_width: width,
                        svg_height: height,
                        svg_view_box: view_box,
                    })
                },
            )
            .map_err(|failure| xml_error("", failure))?;
        } else {
            return Ok(ImageInformation::default());
        };
        Ok(ImageInformation {
            format: Some(format),
            pixel_dimensions: dimensions,
            ..Default::default()
        })
    }
}
fn positive(width: u32, height: u32) -> Option<(u32, u32)> {
    (width > 0 && height > 0).then_some((width, height))
}
fn pixels(raw: &str) -> Option<u32> {
    let raw = raw.trim_matches(|ch: char| ch.is_ascii_whitespace());
    let value = raw.strip_suffix("px").unwrap_or(raw);
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse::<u32>().ok().filter(|n| *n > 0)
}
fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let bytes = &bytes[..bytes.len().min(MAX_JPEG_HEADER)];
    let limit = bytes.len();
    let mut at = 2usize;
    while at < limit {
        if bytes[at] != 0xff {
            return None;
        }
        while at < limit && bytes[at] == 0xff {
            at += 1;
        }
        let marker = *bytes.get(at)?;
        at += 1;
        if matches!(marker, 0xda | 0xd9 | 0x00) {
            return None;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let length =
            u16::from_be_bytes(bytes.get(at..at.checked_add(2)?)?.try_into().ok()?) as usize;
        if length < 2 {
            return None;
        }
        let end = at.checked_add(length)?;
        if end > limit {
            return None;
        }
        if (0xc0..=0xcf).contains(&marker) && !matches!(marker, 0xc4 | 0xc8 | 0xcc) {
            if length < 8 {
                return None;
            }
            let data = bytes.get(at + 3..at + 7)?;
            return positive(
                u32::from(u16::from_be_bytes(data[2..4].try_into().ok()?)),
                u32::from(u16::from_be_bytes(data[..2].try_into().ok()?)),
            );
        }
        at = end;
    }
    None
}
fn webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let declared = u32::from_le_bytes(bytes.get(4..8)?.try_into().ok()?) as u64;
    if declared.checked_add(8)? != bytes.len() as u64 {
        return None;
    }
    let size = u32::from_le_bytes(bytes.get(16..20)?.try_into().ok()?) as usize;
    let end = 20usize.checked_add(size)?;
    if end > bytes.len() {
        return None;
    }
    match bytes.get(12..16)? {
        b"VP8X" if size == 10 => {
            fn u24(data: &[u8]) -> u32 {
                u32::from(data[0]) | (u32::from(data[1]) << 8) | (u32::from(data[2]) << 16)
            }
            Some((u24(bytes.get(24..27)?) + 1, u24(bytes.get(27..30)?) + 1))
        }
        b"VP8L" if size >= 5 && bytes.get(20) == Some(&0x2f) => {
            let bits = u32::from_le_bytes(bytes.get(21..25)?.try_into().ok()?);
            if bits >> 29 != 0 {
                return None;
            }
            Some(((bits & 0x3fff) + 1, ((bits >> 14) & 0x3fff) + 1))
        }
        b"VP8 " if size >= 10 && bytes.get(23..26) == Some(b"\x9d\x01\x2a") => positive(
            u32::from(u16::from_le_bytes(bytes.get(26..28)?.try_into().ok()?) & 0x3fff),
            u32::from(u16::from_le_bytes(bytes.get(28..30)?.try_into().ok()?) & 0x3fff),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raster_headers_report_only_actual_positive_dimensions() {
        let png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\x01\x90\0\0\x01\x2c";
        let info = SourceExtraction::image_information(png, "wrong/type").unwrap();
        assert_eq!(info.format, Some(Format::Png));
        assert_eq!(info.pixel_dimensions, Some((400, 300)));
        let gif = b"GIF89a\x40\x01\xf0\0";
        assert_eq!(
            SourceExtraction::image_information(gif, "image/gif")
                .unwrap()
                .pixel_dimensions,
            Some((320, 240))
        );
        let jpeg = b"\xff\xd8\xff\xe0\0\x04ab\xff\xc2\0\x08\x08\x01\x2c\x01\x90\x01";
        assert_eq!(
            SourceExtraction::image_information(jpeg, "image/jpeg")
                .unwrap()
                .pixel_dimensions,
            Some((400, 300))
        );
        for bad in [b"".as_slice(), b"not an image", b"\xff\xd8\xff\xe0\xff\xff"] {
            assert_eq!(
                SourceExtraction::image_information(bad, "image/png")
                    .unwrap()
                    .pixel_dimensions,
                None
            );
        }
        assert_eq!(
            SourceExtraction::image_information(&png[..20], "image/png")
                .unwrap()
                .pixel_dimensions,
            None
        );
    }
    #[test]
    fn webp_variants_keep_literal_encoded_canvas_sizes_and_reject_bad_lengths() {
        let mut x = b"RIFF\x16\0\0\0WEBPVP8X\x0a\0\0\0\0\0\0\0\x3f\x01\0\xef\0\0".to_vec();
        assert_eq!(
            SourceExtraction::image_information(&x, "image/webp")
                .unwrap()
                .pixel_dimensions,
            Some((320, 240))
        );
        x[16] = 11;
        assert_eq!(
            SourceExtraction::image_information(&x, "image/webp")
                .unwrap()
                .pixel_dimensions,
            None
        );
        let mut lossless = b"RIFF\x12\0\0\0WEBPVP8L\x05\0\0\0\x2f\0\0\0\0\0".to_vec();
        let bits = (319u32) | ((239u32) << 14);
        lossless[21..25].copy_from_slice(&bits.to_le_bytes());
        assert_eq!(
            SourceExtraction::image_information(&lossless, "image/webp")
                .unwrap()
                .pixel_dimensions,
            Some((320, 240))
        );
    }
    #[test]
    fn svg_keeps_raw_attributes_without_guessing_relative_or_fractional_pixels() {
        let svg=br#"<svg xmlns="http://www.w3.org/2000/svg" width=" 320px " height="240" viewBox="0 0 320 240"/>"#;
        let info = SourceExtraction::image_information(svg, "image/svg+xml").unwrap();
        assert_eq!(info.pixel_dimensions, Some((320, 240)));
        assert_eq!(info.svg_width.as_deref(), Some(" 320px "));
        assert_eq!(info.svg_view_box.as_deref(), Some("0 0 320 240"));
        for value in ["50%", "1em", "1.5px", "0", "4294967296"] {
            let svg =
                format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{value}" height="20"/>"#);
            let info =
                SourceExtraction::image_information(svg.as_bytes(), "image/svg+xml").unwrap();
            assert_eq!(info.pixel_dimensions, None);
            assert_eq!(info.svg_width.as_deref(), Some(value));
        }
        let evil=br#"<!DOCTYPE svg [<!ENTITY ext SYSTEM "https://private.invalid/secret">]><svg xmlns="http://www.w3.org/2000/svg"/>"#;
        assert!(SourceExtraction::image_information(evil, "image/svg+xml").is_err());
    }
}

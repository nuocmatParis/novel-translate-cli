//! Tiny stored ZIP fixtures, created offline without a production dependency.
//! The files use conventional OCF/package/Nav/NCX records; no fixture is read
//! from a developer's ignored documents or a real novel.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
pub fn stored_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    fn word(out: &mut Vec<u8>, n: u16) {
        out.extend_from_slice(&n.to_le_bytes());
    }
    fn dword(out: &mut Vec<u8>, n: u32) {
        out.extend_from_slice(&n.to_le_bytes());
    }
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for &(name, bytes) in entries {
        let offset = u32::try_from(out.len()).unwrap();
        let length = u32::try_from(bytes.len()).unwrap();
        let name_len = u16::try_from(name.len()).unwrap();
        let crc = crc32(bytes);
        dword(&mut out, 0x0403_4b50);
        word(&mut out, 20);
        word(&mut out, 0x0800);
        word(&mut out, 0);
        word(&mut out, 0);
        word(&mut out, 0);
        dword(&mut out, crc);
        dword(&mut out, length);
        dword(&mut out, length);
        word(&mut out, name_len);
        word(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(bytes);
        dword(&mut directory, 0x0201_4b50);
        word(&mut directory, 20);
        word(&mut directory, 20);
        word(&mut directory, 0x0800);
        word(&mut directory, 0);
        word(&mut directory, 0);
        word(&mut directory, 0);
        dword(&mut directory, crc);
        dword(&mut directory, length);
        dword(&mut directory, length);
        word(&mut directory, name_len);
        for _ in 0..4 {
            word(&mut directory, 0);
        }
        dword(&mut directory, 0);
        dword(&mut directory, offset);
        directory.extend_from_slice(name.as_bytes());
    }
    let offset = u32::try_from(out.len()).unwrap();
    let length = u32::try_from(directory.len()).unwrap();
    out.extend_from_slice(&directory);
    dword(&mut out, 0x0605_4b50);
    word(&mut out, 0);
    word(&mut out, 0);
    word(&mut out, u16::try_from(entries.len()).unwrap());
    word(&mut out, u16::try_from(entries.len()).unwrap());
    dword(&mut out, length);
    dword(&mut out, offset);
    word(&mut out, 0);
    out
}

pub const CHAPTER: &str = r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Chapter</title></head><body><h1 id="title">第一章</h1><p id="p">A<em>e</em><![CDATA[́]]>👩🏽‍💻<ruby>漢<rt>かん</rt></ruby><img src="../Images/a.png" alt="説明"/><br/>Z<a epub:type="noteref" href="notes.xhtml#note">1</a></p></body></html>"#;

pub fn book(version: u8, chapter: &str) -> Vec<u8> {
    let nav_item = if version == 3 {
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>"#
    } else {
        r#"<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>"#
    };
    let spine = if version == 3 { "" } else { r#" toc="ncx""# };
    let modified = if version == 3 {
        r#"<meta property="dcterms:modified">2026-01-01T00:00:00Z</meta>"#
    } else {
        ""
    };
    let opf = format!(
        r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="{version}.0" unique-identifier="uid"><metadata><dc:identifier id="uid">urn:uuid:832fc946-6732-407f-a929-2a1c267d51ea</dc:identifier><dc:title>Codec test</dc:title><dc:language>ja</dc:language>{modified}</metadata><manifest><item id="ch" href="Text/ch.xhtml" media-type="application/xhtml+xml"/><item id="notes" href="Text/notes.xhtml" media-type="application/xhtml+xml"/><item id="a" href="Images/a.png" media-type="image/png"/>{nav_item}</manifest><spine{spine}><itemref idref="ch"/><itemref idref="notes" linear="no"/></spine></package>"#
    );
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Contents</title></head><body><nav epub:type="toc"><h1>Contents</h1><ol><li><a href="Text/ch.xhtml#title">第一章</a></li></ol></nav></body></html>"#;
    let ncx = r#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1"><head><meta name="dtb:uid" content="urn:uuid:832fc946-6732-407f-a929-2a1c267d51ea"/><meta name="dtb:depth" content="1"/><meta name="dtb:totalPageCount" content="0"/><meta name="dtb:maxPageNumber" content="0"/></head><docTitle><text>Codec test</text></docTitle><navMap><navPoint id="n1" playOrder="1"><navLabel><text>第一章</text></navLabel><content src="Text/ch.xhtml#title"/></navPoint></navMap></ncx>"#;
    let notes = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Notes</title></head><body><aside id="note" epub:type="footnote"><p>注釈</p></aside></body></html>"#;
    let png:Vec<u8>="89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000b49444154789c636000020000050001a5f645400000000049454e44ae426082".as_bytes().as_chunks::<2>().0.iter().map(|pair|u8::from_str_radix(std::str::from_utf8(pair).unwrap(),16).unwrap()).collect();
    let mut entries=vec![
        ("mimetype",b"application/epub+zip".as_slice()),
        ("META-INF/container.xml",br#"<?xml version="1.0"?><container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#),
        ("OPS/book.opf",opf.as_bytes()),("OPS/Text/ch.xhtml",chapter.as_bytes()),("OPS/Text/notes.xhtml",notes.as_bytes()),("OPS/Images/a.png",png.as_slice()),
    ];
    if version == 3 {
        entries.push(("OPS/nav.xhtml", nav.as_bytes()));
    } else {
        entries.push(("OPS/toc.ncx", ncx.as_bytes()));
    }
    stored_zip(&entries)
}

use std::io::{Cursor, Write};

use nt_epub::{
    ArchiveError, ArchiveLimits, DiagnosticCode, EpubZip, InventoryCoverage, NavigationSource,
    ResourceEncryption, SpineContentKind, inspect,
};
use zip::{
    CompressionMethod,
    write::{SimpleFileOptions, ZipWriter},
};

const CONTAINER: &str = r#"<?xml version="1.0"?><container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;

fn archive(files: &[(&str, &str)]) -> EpubZip<Cursor<Vec<u8>>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (path, contents) in files {
        writer
            .start_file(*path, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(contents.as_bytes()).unwrap();
    }
    EpubZip::new(Cursor::new(writer.finish().unwrap().into_inner())).unwrap()
}

fn image_epub_bytes(image: &[u8]) -> Vec<u8> {
    let package = package(
        r#"<item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="image" href="pic.png" media-type="image/png"/>"#,
        r#"<itemref idref="chapter"/>"#,
        METADATA,
    );
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="start">Chapter text.</p><img src="pic.png"/></body></html>"#;
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#start">Chapter</a></li></ol></nav></body></html>"#;
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));

    writer.start_file("mimetype", options).unwrap();
    writer.write_all(b"application/epub+zip").unwrap();
    writer
        .start_file("META-INF/container.xml", options)
        .unwrap();
    writer.write_all(CONTAINER.as_bytes()).unwrap();
    writer.start_file("OPS/book.opf", options).unwrap();
    writer.write_all(package.as_bytes()).unwrap();
    writer.start_file("OPS/chapter.xhtml", options).unwrap();
    writer.write_all(chapter.as_bytes()).unwrap();
    writer.start_file("OPS/nav.xhtml", options).unwrap();
    writer.write_all(nav.as_bytes()).unwrap();
    writer.start_file("OPS/pic.png", options).unwrap();
    writer.write_all(image).unwrap();
    writer.finish().unwrap().into_inner()
}

fn png_1x1_with_text(padding: usize) -> Vec<u8> {
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], payload: &[u8]) {
        out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(payload);

        let mut crc = u32::MAX;
        for byte in kind.iter().chain(payload) {
            crc ^= *byte as u32;
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb8_8320_u32 & 0_u32.wrapping_sub(crc & 1));
            }
        }
        out.extend_from_slice(&(!crc).to_be_bytes());
    }

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 0, 0, 0, 0]);
    chunk(
        &mut png,
        b"IDAT",
        &[0x78, 0x01, 0x01, 0x02, 0x00, 0xfd, 0xff, 0, 0, 0, 2, 0, 1],
    );
    if padding > 0 {
        let mut text = b"note\0".to_vec();
        text.extend(std::iter::repeat_n(b'x', padding));
        chunk(&mut png, b"tEXt", &text);
    }
    chunk(&mut png, b"IEND", &[]);
    png
}

fn corrupt_local_entry_payload(bytes: &mut [u8], entry_name: &str) {
    for position in 0..bytes.len().saturating_sub(30) {
        if bytes[position..position + 4] != [0x50, 0x4b, 0x03, 0x04] {
            continue;
        }
        let name_len =
            u16::from_le_bytes(bytes[position + 26..position + 28].try_into().unwrap()) as usize;
        let extra_len =
            u16::from_le_bytes(bytes[position + 28..position + 30].try_into().unwrap()) as usize;
        let name_start = position + 30;
        if bytes.get(name_start..name_start + name_len) == Some(entry_name.as_bytes()) {
            bytes[name_start + name_len + extra_len] ^= 1;
            return;
        }
    }
    panic!("entry not found: {entry_name}");
}

fn assert_image_relation(result: &nt_epub::EpubInventory) {
    let image = result
        .spine
        .first()
        .and_then(|item| item.image_uses.first())
        .unwrap();
    assert_eq!(image.resource_path.as_deref(), Some("OPS/pic.png"));
    assert_eq!(image.manifest_resource_index, Some(2));
}

fn package(manifest: &str, spine: &str, metadata: &str) -> String {
    format!(
        r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="3.0" unique-identifier="bookid"><metadata><dc:identifier id="bookid">id-1</dc:identifier>{metadata}</metadata><manifest>{manifest}</manifest><spine>{spine}</spine></package>"#
    )
}

const METADATA: &str =
    "<dc:title>Book</dc:title><dc:creator>Author</dc:creator><dc:language>ja</dc:language>";

#[test]
fn epub2_inventory_preserves_ncx_hierarchy_spine_order_and_optional_metadata_warning() {
    let package = r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0" unique-identifier="bookid"><metadata><dc:identifier id="bookid">id-1</dc:identifier><dc:title>Book</dc:title><dc:language>ja</dc:language></metadata><manifest><item id="chapter" href="text/ch.xhtml" media-type="application/xhtml+xml"/><item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/></manifest><spine toc="ncx"><itemref idref="chapter"/><itemref idref="chapter" linear="no"/></spine></package>"#;
    let ncx = r#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap><navPoint><navLabel><text>Chapter One</text></navLabel><content src="text/ch.xhtml#start"/><navPoint><navLabel><text>Part</text></navLabel><content src="text/ch.xhtml#part"/></navPoint></navPoint></navMap></ncx>"#;
    let xhtml = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="start">Text</p><p id="part">Part</p></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", package),
        ("OPS/toc.ncx", ncx),
        ("OPS/text/ch.xhtml", xhtml),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Complete);
    assert_eq!(result.package_path.as_deref(), Some("OPS/book.opf"));
    assert_eq!(result.package_version.as_deref(), Some("2.0"));
    assert_eq!(result.spine.len(), 2);
    assert!(result.spine[0].linear);
    assert!(!result.spine[1].linear);
    assert_eq!(result.spine[1].reading_order, 1);
    assert_eq!(result.navigation[0].source, NavigationSource::Epub2Ncx);
    assert_eq!(result.navigation[0].items[0].label, "Chapter One");
    assert_eq!(result.navigation[0].items[0].children[0].label, "Part");
    assert_eq!(
        result.navigation[0].items[0]
            .target
            .as_ref()
            .unwrap()
            .resource_path
            .as_deref(),
        Some("OPS/text/ch.xhtml")
    );
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::MetadataMissing
            && diagnostic.severity == nt_epub::DiagnosticSeverity::Warning
    }));
}

#[test]
fn epub3_hybrid_keeps_image_positions_and_font_obfuscation_distinct() {
    let package = package(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/><item id="chapter" href="text/ch.xhtml" media-type="application/xhtml+xml"/><item id="image" href="images/page.jpg" media-type="image/jpeg"/><item id="font" href="fonts/book.otf" media-type="font/otf"/>"#,
        r#"toc="ncx"><itemref idref="chapter"/><itemref idref="image"/><itemref idref="chapter" linear="no"/>"#,
        METADATA,
    );
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="text/ch.xhtml#start">Chapter</a><ol><li><a href="text/ch.xhtml#part">Part</a></li></ol></li></ol></nav></body></html>"#;
    let ncx = r#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap><navPoint><navLabel><text>Chapter</text></navLabel><content src="text/ch.xhtml#start"/><navPoint><navLabel><text>Part</text></navLabel><content src="text/ch.xhtml#part"/></navPoint></navPoint></navMap></ncx>"#;
    let encryption = r#"<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container" xmlns:enc="http://www.w3.org/2001/04/xmlenc#"><enc:EncryptedData><enc:EncryptionMethod Algorithm="http://www.idpf.org/2008/embedding"/><enc:CipherData><enc:CipherReference URI="OPS/fonts/book.otf"/></enc:CipherData></enc:EncryptedData></encryption>"#;
    let xhtml = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="start">Text</p><p id="part">Part</p><img src="../images/page.jpg"/></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("META-INF/encryption.xml", encryption),
        ("OPS/book.opf", &package),
        ("OPS/nav.xhtml", nav),
        ("OPS/toc.ncx", ncx),
        ("OPS/text/ch.xhtml", xhtml),
        ("OPS/images/page.jpg", "image bytes"),
        ("OPS/fonts/book.otf", "font bytes"),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Complete);
    assert_eq!(result.navigation.len(), 2);
    assert_eq!(result.primary_navigation, Some(0));
    assert_eq!(result.navigation[0].source, NavigationSource::Epub3Nav);
    assert_eq!(result.spine[1].kind, SpineContentKind::ImagePage);
    assert_eq!(
        result.spine[0].image_uses[0].resource_path.as_deref(),
        Some("OPS/images/page.jpg")
    );
    assert_eq!(
        result.spine[0].image_uses[0].manifest_resource_index,
        Some(3)
    );
    assert_eq!(
        result.resources[4].encryption,
        ResourceEncryption::IdpfFontObfuscation
    );
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::ScriptedContentUnsupported })
    );
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::FontObfuscationDetected
            && diagnostic.severity == nt_epub::DiagnosticSeverity::Info
            && diagnostic.resource_path.as_deref() == Some("OPS/fonts/book.otf")
    }));
}

#[test]
fn navigation_missing_unlisted_and_ambiguous_targets_have_distinct_diagnostics() {
    let package = package(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="same-a" href="ambiguous.xhtml" media-type="application/xhtml+xml"/><item id="same-b" href="ambiguous.xhtml" media-type="application/xhtml+xml"/>"#,
        r#"><itemref idref="chapter"/>"#,
        METADATA,
    );
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="missing.xhtml#gone">Missing</a></li><li><a href="unlisted.xhtml">Unlisted</a></li><li><a href="ambiguous.xhtml">Ambiguous</a></li></ol></nav></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/nav.xhtml", nav),
        (
            "OPS/chapter.xhtml",
            "<html xmlns='http://www.w3.org/1999/xhtml'><body>text</body></html>",
        ),
        ("OPS/unlisted.xhtml", "unlisted"),
        ("OPS/ambiguous.xhtml", "ambiguous"),
    ]);

    let result = inspect(&mut source);
    let diagnostic_for = |code, target: &str| {
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == code && diagnostic.related_resource.as_deref() == Some(target)
        })
    };

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert!(diagnostic_for(
        DiagnosticCode::NavigationTargetMissing,
        "OPS/missing.xhtml"
    ));
    assert!(!diagnostic_for(
        DiagnosticCode::NavigationTargetUnlisted,
        "OPS/missing.xhtml"
    ));
    assert!(diagnostic_for(
        DiagnosticCode::NavigationTargetUnlisted,
        "OPS/unlisted.xhtml"
    ));
    assert!(diagnostic_for(
        DiagnosticCode::NavigationTargetAmbiguous,
        "OPS/ambiguous.xhtml"
    ));
}

#[test]
fn navigation_fragments_are_checked_against_xhtml_ids() {
    let package = package(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>"#,
        r#"><itemref idref="chapter"/>"#,
        METADATA,
    );
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#present">Present</a></li><li><a href="chapter.xhtml#absent">Absent</a></li></ol></nav></body></html>"#;
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="present">Text</p></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/nav.xhtml", nav),
        ("OPS/chapter.xhtml", chapter),
    ]);

    let result = inspect(&mut source);
    let missing_fragments = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::NavigationTargetMissing)
        .collect::<Vec<_>>();

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert_eq!(missing_fragments.len(), 1);
    assert_eq!(
        missing_fragments[0].related_resource.as_deref(),
        Some("OPS/chapter.xhtml")
    );
    assert_eq!(
        result.navigation[0].items[0]
            .target
            .as_ref()
            .unwrap()
            .fragment
            .as_deref(),
        Some("present")
    );
    assert_eq!(
        result.navigation[0].items[1]
            .target
            .as_ref()
            .unwrap()
            .fragment
            .as_deref(),
        Some("absent")
    );
}

#[test]
fn ignores_a_nested_toc_when_the_navigation_document_root_is_invalid() {
    let package = package(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>"#,
        r#"><itemref idref="chapter"/>"#,
        METADATA,
    );
    let nav = r#"<wrapper><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml">Chapter</a></li></ol></nav></body></html></wrapper>"#;
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body>Text</body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/nav.xhtml", nav),
        ("OPS/chapter.xhtml", chapter),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert!(result.navigation.is_empty());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::NavigationStructureUnsupported
            && diagnostic.resource_path.as_deref() == Some("OPS/nav.xhtml")
    }));
}

#[test]
fn font_obfuscation_algorithm_on_xhtml_is_treated_as_unsupported_encryption() {
    let package = package(
        r#"<item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>"#,
        r#"><itemref idref="chapter"/>"#,
        METADATA,
    );
    let encryption = r#"<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container" xmlns:enc="http://www.w3.org/2001/04/xmlenc#"><enc:EncryptedData><enc:EncryptionMethod Algorithm="http://www.idpf.org/2008/embedding"/><enc:CipherData><enc:CipherReference URI="OPS/chapter.xhtml"/></enc:CipherData></enc:EncryptedData></encryption>"#;
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p>Readable bytes were replaced.</p></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("META-INF/encryption.xml", encryption),
        ("OPS/book.opf", &package),
        ("OPS/chapter.xhtml", chapter),
    ]);

    let result = inspect(&mut source);

    assert_eq!(
        result.resources[0].encryption,
        ResourceEncryption::Unsupported
    );
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::UnsupportedEncryption
            && diagnostic.severity == nt_epub::DiagnosticSeverity::Incomplete
            && diagnostic.related_resource.as_deref() == Some("OPS/chapter.xhtml")
    }));
}

#[test]
fn required_missing_locked_and_script_dependent_content_stays_partial() {
    let missing_package_container = r#"<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OPS/missing.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
    let mut source = archive(&[("META-INF/container.xml", missing_package_container)]);
    let missing_package = inspect(&mut source);
    assert_eq!(missing_package.coverage, InventoryCoverage::Partial);
    assert!(missing_package.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PackageMissing
            && diagnostic.resource_path.as_deref() == Some("OPS/missing.opf")
    }));

    let package = package(
        r#"<item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml" properties="scripted remote-resources"/><item id="image" href="scan.png" media-type="image/png"/>"#,
        r#"><itemref idref="chapter"/><itemref idref="image"/>"#,
        METADATA,
    );
    let encryption = r#"<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container" xmlns:enc="http://www.w3.org/2001/04/xmlenc#"><enc:EncryptedData><enc:EncryptionMethod Algorithm="urn:publisher:locked-resource"/><enc:CipherData><enc:CipherReference URI="OPS/chapter.xhtml"/></enc:CipherData></enc:EncryptedData></encryption>"#;
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><script src="https://blocked.invalid/app.js"/></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("META-INF/encryption.xml", encryption),
        ("OPS/book.opf", &package),
        ("OPS/chapter.xhtml", chapter),
        ("OPS/scan.png", "scan bytes"),
    ]);
    let result = inspect(&mut source);
    let rendered_diagnostics = format!("{:?}", result.diagnostics);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::UnsupportedEncryption
            && diagnostic.severity == nt_epub::DiagnosticSeverity::Incomplete
            && diagnostic.related_resource.as_deref() == Some("OPS/chapter.xhtml")
    }));
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::ScriptedContentUnsupported })
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::RemoteResourcesUnsupported })
    );
    assert!(!rendered_diagnostics.contains("blocked.invalid"));
}

#[test]
fn image_only_spine_is_preserved_and_signals_the_separate_image_workflow() {
    let package = package(
        r#"<item id="scan" href="scan.png" media-type="image/png"/><item id="appendix" href="appendix.xhtml" media-type="application/xhtml+xml"/>"#,
        r#"><itemref idref="scan"/><itemref idref="appendix" linear="no"/>"#,
        METADATA,
    );
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/scan.png", "scan bytes"),
        (
            "OPS/appendix.xhtml",
            "<html xmlns='http://www.w3.org/1999/xhtml'><body>Appendix text</body></html>",
        ),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert_eq!(result.spine[0].kind, SpineContentKind::ImagePage);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::ImageOnlyContentNeedsImageWorkflow
    }));
}

#[test]
fn reports_empty_and_unusable_spines_as_incomplete() {
    for spine in ["", r#"<itemref idref="missing"/>"#] {
        let package = package(
            r#"<item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>"#,
            spine,
            METADATA,
        );
        let mut source = archive(&[
            ("META-INF/container.xml", CONTAINER),
            ("OPS/book.opf", &package),
            (
                "OPS/chapter.xhtml",
                "<html xmlns='http://www.w3.org/1999/xhtml'><body>Text</body></html>",
            ),
        ]);

        let result = inspect(&mut source);

        assert_eq!(result.coverage, InventoryCoverage::Partial);
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::SpineEmpty
                && diagnostic.severity == nt_epub::DiagnosticSeverity::Incomplete
        }));
    }
}

#[test]
fn checks_both_rendition_layout_overrides_and_keeps_fixed_image_pages() {
    let metadata = format!("{METADATA}<meta property='rendition:layout'>pre-paginated</meta>");
    let package = package(
        r#"<item id="fixed-image" href="fixed.xhtml" media-type="application/xhtml+xml"/><item id="fixed-text" href="text.xhtml" media-type="application/xhtml+xml"/><item id="reflow-override" href="reflow.xhtml" media-type="application/xhtml+xml"/><item id="page-image" href="page.png" media-type="image/png"/><item id="svg-page" href="page.svg" media-type="image/svg+xml"/>"#,
        r#"<itemref idref="fixed-image"/><itemref idref="fixed-text"/><itemref idref="reflow-override" properties="rendition:layout-reflowable"/><itemref idref="svg-page"/>"#,
        &metadata,
    );
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        (
            "OPS/fixed.xhtml",
            "<html xmlns='http://www.w3.org/1999/xhtml'><body><img src='page.png'/></body></html>",
        ),
        (
            "OPS/text.xhtml",
            "<html xmlns='http://www.w3.org/1999/xhtml'><body><p>fixed text</p></body></html>",
        ),
        (
            "OPS/reflow.xhtml",
            "<html xmlns='http://www.w3.org/1999/xhtml'><body><p>reflow text</p></body></html>",
        ),
        ("OPS/page.png", "image bytes"),
        (
            "OPS/page.svg",
            "<svg xmlns='http://www.w3.org/2000/svg'><image href='page.png'/></svg>",
        ),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.spine[0].kind, SpineContentKind::ImagePage);
    assert_eq!(result.spine[0].image_uses.len(), 1);
    assert_eq!(result.spine[1].kind, SpineContentKind::Unsupported);
    assert_eq!(result.spine[2].kind, SpineContentKind::ReflowableDocument);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::FixedLayoutTextUnsupported
            && diagnostic.related_resource.as_deref() == Some("fixed-text")
    }));
    assert!(!result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::FixedLayoutTextUnsupported
            && diagnostic.related_resource.as_deref() == Some("reflow-override")
    }));
    assert_eq!(result.spine[3].kind, SpineContentKind::ImagePage);
    assert_eq!(
        result.spine[3].image_uses[0].resource_path.as_deref(),
        Some("OPS/page.png")
    );
}

#[test]
fn per_item_fixed_layout_override_preserves_an_xhtml_image_page() {
    let metadata = format!("{METADATA}<meta property='rendition:layout'>reflowable</meta>");
    let package = package(
        r#"<item id="fixed" href="page.xhtml" media-type="application/xhtml+xml"/><item id="image" href="page.png" media-type="image/png"/>"#,
        r#"<itemref idref="fixed" properties="rendition:layout-pre-paginated"/>"#,
        &metadata,
    );
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        (
            "OPS/page.xhtml",
            "<html xmlns='http://www.w3.org/1999/xhtml'><body><img src='page.png'/></body></html>",
        ),
        ("OPS/page.png", "image bytes"),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.spine[0].kind, SpineContentKind::ImagePage);
    assert_eq!(
        result.spine[0].image_uses[0].resource_path.as_deref(),
        Some("OPS/page.png")
    );
}

#[test]
fn malformed_entity_reference_runs_return_located_partial_diagnostics() {
    let package = package(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>"#,
        r#"<itemref idref="chapter"/>"#,
        METADATA,
    );
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#start">Chapter</a></li></ol></nav></body></html>"#;
    let chapter = format!(
        "<html xmlns='http://www.w3.org/1999/xhtml'><body><p id='start'>{}</p></body></html>",
        "&".repeat(256)
    );
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/nav.xhtml", nav),
        ("OPS/chapter.xhtml", &chapter),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == nt_epub::DiagnosticSeverity::Incomplete
            && diagnostic.resource_path.as_deref() == Some("OPS/chapter.xhtml")
            && diagnostic.line.is_some()
            && diagnostic.column.is_some()
    }));
}

#[test]
fn fixed_layout_svg_text_is_partial_and_image_only_pages_remain_supported() {
    let metadata = format!("{METADATA}<meta property='rendition:layout'>reflowable</meta>");
    let package = package(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="fixed-svg" href="fixed.svg" media-type="image/svg+xml"/><item id="image-svg" href="image.svg" media-type="image/svg+xml"/><item id="image-xhtml" href="image.xhtml" media-type="application/xhtml+xml"/><item id="metadata-svg" href="metadata.svg" media-type="image/svg+xml"/><item id="page" href="page.png" media-type="image/png"/>"#,
        r#"<itemref idref="chapter"/><itemref idref="fixed-svg" properties="rendition:layout-pre-paginated"/><itemref idref="image-svg" properties="rendition:layout-pre-paginated"/><itemref idref="image-xhtml" properties="rendition:layout-pre-paginated"/><itemref idref="metadata-svg" properties="rendition:layout-pre-paginated"/>"#,
        &metadata,
    );
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#start">Chapter</a></li></ol></nav></body></html>"#;
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="start">Reflowable text</p></body></html>"#;
    let fixed_svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="40">Fixed story text</text></svg>"#;
    let image_svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="page.png"/></svg>"#;
    let image_xhtml =
        r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><img src="page.png"/></body></html>"#;
    let metadata_svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><title>Cover title</title><desc>Cover description</desc><image href="page.png"/></svg>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/nav.xhtml", nav),
        ("OPS/chapter.xhtml", chapter),
        ("OPS/fixed.svg", fixed_svg),
        ("OPS/image.svg", image_svg),
        ("OPS/image.xhtml", image_xhtml),
        ("OPS/metadata.svg", metadata_svg),
        ("OPS/page.png", "image bytes"),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert_eq!(result.spine[0].kind, SpineContentKind::ReflowableDocument);
    assert_eq!(result.spine[1].kind, SpineContentKind::Unsupported);
    assert_eq!(result.spine[2].kind, SpineContentKind::ImagePage);
    assert_eq!(result.spine[3].kind, SpineContentKind::ImagePage);
    assert_eq!(result.spine[4].kind, SpineContentKind::ImagePage);
    assert!(
        result.spine[2..=4]
            .iter()
            .all(|item| item.image_uses.len() == 1)
    );
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::FixedLayoutTextUnsupported
            && diagnostic.related_resource.as_deref() == Some("fixed-svg")
            && diagnostic.resource_path.as_deref() == Some("OPS/fixed.svg")
    }));
    assert!(!result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::FixedLayoutTextUnsupported
            && diagnostic.related_resource.as_deref() == Some("metadata-svg")
    }));
}

#[test]
fn fixed_layout_svg_foreign_object_text_is_partial_for_package_and_item_layouts() {
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#start">Chapter</a></li></ol></nav></body></html>"#;
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="start">Reflowable text</p></body></html>"#;
    let story_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="800"><image href="page.svg" width="600" height="800"/><foreignObject x="10" y="10" width="500" height="700"><div xmlns="http://www.w3.org/1999/xhtml"><p>Fixed story text</p></div></foreignObject></svg>"#;
    let page_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="800"><path d="M0 0h600v800H0z"/></svg>"#;

    for package_fixed in [false, true] {
        let default_layout = if package_fixed {
            "pre-paginated"
        } else {
            "reflowable"
        };
        let chapter_override = if package_fixed {
            " properties='rendition:layout-reflowable'"
        } else {
            ""
        };
        let story_override = if package_fixed {
            ""
        } else {
            " properties='rendition:layout-pre-paginated'"
        };
        let metadata =
            format!("{METADATA}<meta property='rendition:layout'>{default_layout}</meta>");
        let manifest = r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="story-svg" href="story.svg" media-type="image/svg+xml"/><item id="page-svg" href="page.svg" media-type="image/svg+xml"/>"#;
        let spine = format!(
            "<itemref idref='chapter'{chapter_override}/><itemref idref='story-svg'{story_override}/>"
        );
        let package = package(manifest, &spine, &metadata);
        let mut source = archive(&[
            ("META-INF/container.xml", CONTAINER),
            ("OPS/book.opf", &package),
            ("OPS/nav.xhtml", nav),
            ("OPS/chapter.xhtml", chapter),
            ("OPS/story.svg", story_svg),
            ("OPS/page.svg", page_svg),
        ]);

        let result = inspect(&mut source);

        assert_eq!(result.coverage, InventoryCoverage::Partial);
        assert_eq!(result.spine[0].kind, SpineContentKind::ReflowableDocument);
        assert_eq!(result.spine[1].kind, SpineContentKind::Unsupported);
        assert_eq!(result.spine[1].image_uses.len(), 1);
        assert_eq!(
            result.spine[1].image_uses[0].resource_path.as_deref(),
            Some("OPS/page.svg")
        );
        assert!(
            result.spine[1].image_uses[0]
                .manifest_resource_index
                .is_some()
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::FixedLayoutTextUnsupported
                && diagnostic.related_resource.as_deref() == Some("story-svg")
                && diagnostic.resource_path.as_deref() == Some("OPS/story.svg")
        }));
    }
}

#[test]
fn fixed_layout_image_only_svg_metadata_inside_foreign_object_is_complete_for_package_and_item_layouts()
 {
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#start">Chapter</a></li><li><a href="image-only.svg#panel">Image</a></li></ol></nav></body></html>"#;
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="start">Reflowable text</p></body></html>"#;
    let image_only_svg = r#"<svg id="panel" xmlns="http://www.w3.org/2000/svg" width="600" height="800"><foreignObject width="600" height="800"><div xmlns="http://www.w3.org/1999/xhtml"><svg xmlns="http://www.w3.org/2000/svg" width="600" height="800"><title>Cover metadata</title><desc>Description metadata</desc><image href="page.svg" width="600" height="800"/></svg></div></foreignObject></svg>"#;
    let page_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="800"><path d="M0 0h600v800H0z"/></svg>"#;

    for package_fixed in [false, true] {
        let default_layout = if package_fixed {
            "pre-paginated"
        } else {
            "reflowable"
        };
        let chapter_override = if package_fixed {
            " properties='rendition:layout-reflowable'"
        } else {
            ""
        };
        let image_override = if package_fixed {
            ""
        } else {
            " properties='rendition:layout-pre-paginated'"
        };
        let metadata =
            format!("{METADATA}<meta property='rendition:layout'>{default_layout}</meta>");
        let manifest = r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="image-only" href="image-only.svg" media-type="image/svg+xml"/><item id="page-svg" href="page.svg" media-type="image/svg+xml"/>"#;
        let spine = format!(
            "<itemref idref='chapter'{chapter_override}/><itemref idref='image-only'{image_override}/>"
        );
        let package = package(manifest, &spine, &metadata);
        let mut source = archive(&[
            ("META-INF/container.xml", CONTAINER),
            ("OPS/book.opf", &package),
            ("OPS/nav.xhtml", nav),
            ("OPS/chapter.xhtml", chapter),
            ("OPS/image-only.svg", image_only_svg),
            ("OPS/page.svg", page_svg),
        ]);

        let result = inspect(&mut source);

        assert_eq!(result.coverage, InventoryCoverage::Complete);
        assert_eq!(result.spine[0].kind, SpineContentKind::ReflowableDocument);
        assert_eq!(result.spine[1].kind, SpineContentKind::ImagePage);
        assert_eq!(result.spine[1].image_uses.len(), 1);
        assert_eq!(
            result.spine[1].image_uses[0].resource_path.as_deref(),
            Some("OPS/page.svg")
        );
        assert!(
            result.spine[1].image_uses[0]
                .manifest_resource_index
                .is_some()
        );
        assert_eq!(result.navigation[0].items.len(), 2);
        let image_target = result.navigation[0].items[1].target.as_ref().unwrap();
        assert_eq!(
            image_target.resource_path.as_deref(),
            Some("OPS/image-only.svg")
        );
        assert_eq!(image_target.fragment.as_deref(), Some("panel"));
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != DiagnosticCode::FixedLayoutTextUnsupported)
        );
    }
}

#[test]
fn duplicate_navigation_fragments_are_ambiguous_but_same_node_aliases_are_unique() {
    let package = package(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>"#,
        r#"<itemref idref="chapter"/>"#,
        METADATA,
    );
    let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#dup">Dup</a></li><li><a href="chapter.xhtml#legacy">Legacy</a></li><li><a href="chapter.xhtml#cross">Cross</a></li><li><a href="chapter.xhtml#same">Same</a></li><li><a href="chapter.xhtml#missing">Missing</a></li></ol></nav></body></html>"#;
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="dup">a</p><p id="dup">b</p><a name="legacy">a</a><a name="legacy">b</a><p id="cross">id</p><a name="cross">name</a><a id="same" name="same">same</a></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/nav.xhtml", nav),
        ("OPS/chapter.xhtml", chapter),
    ]);

    let result = inspect(&mut source);
    let ambiguous = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::NavigationTargetAmbiguous)
        .collect::<Vec<_>>();

    assert_eq!(ambiguous.len(), 3);
    assert!(
        ambiguous.iter().all(|diagnostic| {
            diagnostic.related_resource.as_deref() == Some("OPS/chapter.xhtml")
        })
    );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::NavigationTargetMissing)
            .count(),
        1
    );
}

#[test]
fn event_handler_attributes_mark_markup_scripted_without_execution() {
    let package = package(
        r#"<item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>"#,
        r#"<itemref idref="chapter"/>"#,
        METADATA,
    );
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body onload="blocked()" onclick="blocked()"><img src="missing.png" onerror="blocked()"/></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/chapter.xhtml", chapter),
    ]);

    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::ScriptedContentUnsupported
            && diagnostic.related_resource.as_deref() == Some("chapter")
    }));
}

#[test]
fn scans_non_spine_xhtml_image_relations_and_missing_targets() {
    let package = package(
        r#"<item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="notes" href="notes.xhtml" media-type="application/xhtml+xml"/><item id="image" href="notes.png" media-type="image/png"/>"#,
        r#"<itemref idref="chapter"/>"#,
        METADATA,
    );
    let notes = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><img src="notes.png"/><img src="absent.png"/></body></html>"#;
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        (
            "OPS/chapter.xhtml",
            "<html xmlns='http://www.w3.org/1999/xhtml'><body>Chapter</body></html>",
        ),
        ("OPS/notes.xhtml", notes),
        ("OPS/notes.png", "image bytes"),
    ]);

    let result = inspect(&mut source);
    let notes_resource = result
        .resources
        .iter()
        .find(|resource| resource.id == "notes")
        .unwrap();

    assert_eq!(notes_resource.image_uses.len(), 2);
    assert_eq!(
        notes_resource.image_uses[0].resource_path.as_deref(),
        Some("OPS/notes.png")
    );
    assert_eq!(
        notes_resource.image_uses[0].manifest_resource_index,
        Some(2)
    );
    assert_eq!(notes_resource.image_uses[0].occurrence, 0);
    assert!(notes_resource.image_uses[0].line > 0);
    assert!(notes_resource.image_uses[0].column > 0);
    assert_eq!(
        notes_resource.image_uses[1].resource_path.as_deref(),
        Some("OPS/absent.png")
    );
    assert_eq!(notes_resource.image_uses[1].occurrence, 1);
    assert!(notes_resource.image_uses[1].line > 0);
    assert!(notes_resource.image_uses[1].column > 0);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::ReferencedImageMissing
            && diagnostic.resource_path.as_deref() == Some("OPS/notes.xhtml")
            && diagnostic.related_resource.as_deref() == Some("OPS/absent.png")
    }));
}

fn deep_navigation_html(depth: usize) -> String {
    let mut html = String::from(
        "<html xmlns='http://www.w3.org/1999/xhtml' xmlns:epub='http://www.idpf.org/2007/ops'><body><nav epub:type='toc'><ol>",
    );
    for level in 0..depth {
        html.push_str("<li><a>item</a>");
        if level + 1 < depth {
            html.push_str("<ol>");
        }
    }
    for level in (0..depth).rev() {
        html.push_str("</li>");
        if level > 0 {
            html.push_str("</ol>");
        }
    }
    html.push_str("</ol></nav></body></html>");
    html
}

fn deep_navigation_ncx(depth: usize) -> String {
    let mut xml = String::from("<ncx xmlns='http://www.daisy.org/z3986/2005/ncx/'><navMap>");
    for _ in 0..depth {
        xml.push_str(
            "<navPoint><navLabel><text>item</text></navLabel><content src='chapter.xhtml'/>",
        );
    }
    for _ in 0..depth {
        xml.push_str("</navPoint>");
    }
    xml.push_str("</navMap></ncx>");
    xml
}

#[test]
fn deep_navigation_child_probe() {
    if std::env::var_os("NT_EPUB_DEEP_NAVIGATION_CHILD").is_none() {
        return;
    }
    let package = package(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>"#,
        r#"<itemref idref="nav"/>"#,
        METADATA,
    );
    let nav = deep_navigation_html(4096);
    let ncx = deep_navigation_ncx(4096);
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
        ("OPS/nav.xhtml", &nav),
        ("OPS/toc.ncx", &ncx),
    ]);

    let result = inspect(&mut source);
    let depth_diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == DiagnosticCode::NavigationDepthLimitExceeded
                && diagnostic.severity == nt_epub::DiagnosticSeverity::Incomplete
        })
        .collect::<Vec<_>>();

    assert_eq!(depth_diagnostics.len(), 2);
    assert!(
        depth_diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.resource_path.as_deref() == Some("OPS/nav.xhtml") })
    );
    assert!(
        depth_diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.resource_path.as_deref() == Some("OPS/toc.ncx") })
    );
}
#[test]
fn very_deep_navigation_returns_a_diagnostic_without_aborting_the_process() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("deep_navigation_child_probe")
        .arg("--nocapture")
        .env("NT_EPUB_DEEP_NAVIGATION_CHILD", "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "child status: {}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn redacts_untrusted_manifest_ids_and_keeps_safe_ids_in_diagnostics() {
    let package = package(
        r#"<item id="safe-id" href="https://safe.example.invalid/book" media-type="application/xhtml+xml"/><item id="https://user:secret@example.invalid/book?token=private" href="https://private.example.invalid/book" media-type="application/xhtml+xml"/>"#,
        "",
        METADATA,
    );
    let mut source = archive(&[
        ("META-INF/container.xml", CONTAINER),
        ("OPS/book.opf", &package),
    ]);

    let result = inspect(&mut source);
    let external = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::ExternalResourceNotRead)
        .collect::<Vec<_>>();
    let rendered = format!("{:?}", result.diagnostics);

    assert!(
        external
            .iter()
            .any(|diagnostic| { diagnostic.related_resource.as_deref() == Some("safe-id") })
    );
    assert!(
        external
            .iter()
            .any(|diagnostic| { diagnostic.related_resource.is_none() })
    );
    assert!(!rendered.contains("private.example.invalid"));
    assert!(!rendered.contains("user:secret"));
    assert!(!rendered.contains("token=private"));
}

#[test]
fn valid_required_image_is_read_before_inventory_is_complete() {
    let bytes = image_epub_bytes(&png_1x1_with_text(0));
    let mut source = EpubZip::new(Cursor::new(bytes)).unwrap();

    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Complete);
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ArchiveEntryUnreadable)
    );
    assert_image_relation(&result);
}

#[test]
fn required_image_with_corrupt_zip_payload_cannot_be_complete() {
    let mut bytes = image_epub_bytes(&png_1x1_with_text(0));
    corrupt_local_entry_payload(&mut bytes, "OPS/pic.png");

    let mut direct = EpubZip::new(Cursor::new(bytes.clone())).unwrap();
    let image_id = direct.entry_by_path("OPS/pic.png").unwrap().unwrap().id();
    assert!(matches!(
        direct.read_entry(image_id),
        Err(ArchiveError::EntryRead { entry, .. }) if entry == "OPS/pic.png"
    ));

    let mut source = EpubZip::new(Cursor::new(bytes)).unwrap();
    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::ArchiveEntryUnreadable)
        .unwrap();
    assert_eq!(
        diagnostic.resource_path.as_deref(),
        Some("OPS/chapter.xhtml")
    );
    assert_eq!(diagnostic.related_resource.as_deref(), Some("OPS/pic.png"));
    assert!(diagnostic.line.is_some());
    assert!(diagnostic.column.is_some());
    assert_image_relation(&result);
}

#[test]
fn required_image_blocked_by_t04_entry_cap_cannot_be_complete() {
    let image = png_1x1_with_text(8192);
    assert_eq!(image.len(), 8279);
    let bytes = image_epub_bytes(&image);
    let limits = ArchiveLimits {
        max_entry_bytes: 4096,
        ..ArchiveLimits::default()
    };

    let mut direct = EpubZip::with_limits(Cursor::new(bytes.clone()), limits.clone()).unwrap();
    let image_id = direct.entry_by_path("OPS/pic.png").unwrap().unwrap().id();
    assert!(matches!(
        direct.read_entry(image_id),
        Err(ArchiveError::EntryDeclaredTooLarge {
            entry,
            declared,
            limit
        }) if entry == "OPS/pic.png" && declared == image.len() as u64 && limit == 4096
    ));

    let mut source = EpubZip::with_limits(Cursor::new(bytes), limits).unwrap();
    let result = inspect(&mut source);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::ArchiveEntryUnreadable)
        .unwrap();
    assert_eq!(
        diagnostic.resource_path.as_deref(),
        Some("OPS/chapter.xhtml")
    );
    assert_eq!(diagnostic.related_resource.as_deref(), Some("OPS/pic.png"));
    assert!(diagnostic.line.is_some());
    assert!(diagnostic.column.is_some());
    assert_image_relation(&result);
}

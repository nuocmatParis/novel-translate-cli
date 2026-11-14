use std::io::{Cursor, Write};

use nt_epub::{DiagnosticCode, EpubZip, InventoryCoverage, inspect};
use zip::write::{SimpleFileOptions, ZipWriter};

fn utf16_xml(text: &str, little_endian: bool, declaration: &str) -> Vec<u8> {
    let text = text.replace("encoding=\"UTF-16\"", declaration);
    let mut bytes = Vec::from(if little_endian {
        [0xff, 0xfe]
    } else {
        [0xfe, 0xff]
    });
    for unit in text.encode_utf16() {
        let pair = if little_endian {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        };
        bytes.extend_from_slice(&pair);
    }
    bytes
}

fn make_archive(files: &[(&str, &[u8])]) -> EpubZip<Cursor<Vec<u8>>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (path, contents) in files {
        writer
            .start_file(*path, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(contents).unwrap();
    }
    EpubZip::new(Cursor::new(writer.finish().unwrap().into_inner())).unwrap()
}

#[test]
fn parses_utf16_container_and_package_in_both_byte_orders() {
    let container = r#"<?xml version="1.0" encoding="UTF-16"?><container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
    let package = r#"<?xml version="1.0" encoding="UTF-16"?><package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="3.0" unique-identifier="id"><metadata><dc:identifier id="id">book-id</dc:identifier><dc:title>日本語の本</dc:title><dc:creator>作者</dc:creator><dc:language>ja</dc:language></metadata><manifest><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="chapter"/></spine></package>"#;
    let xhtml =
        "<html xmlns='http://www.w3.org/1999/xhtml'><body><p>本文</p></body></html>".as_bytes();
    let container_utf16 = utf16_xml(container, true, "encoding=\"UTF-16LE\"");
    let package_utf16 = utf16_xml(package, false, "encoding=\"UTF-16BE\"");
    let mut archive = make_archive(&[
        ("META-INF/container.xml", &container_utf16),
        ("OPS/book.opf", &package_utf16),
        ("OPS/chapter.xhtml", xhtml),
    ]);

    let result = inspect(&mut archive);

    assert_eq!(result.package_version.as_deref(), Some("3.0"));
    assert_eq!(result.metadata.values[1].value, "日本語の本");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.code != DiagnosticCode::XmlEncodingUnsupported })
    );
    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::NavigationMissing })
    );
}

#[test]
fn external_entities_are_rejected_without_copying_the_uri_into_diagnostics() {
    let container = b"<container xmlns='urn:oasis:names:tc:opendocument:xmlns:container'><rootfiles><rootfile full-path='OPS/book.opf' media-type='application/oebps-package+xml'/></rootfiles></container>";
    let package = b"<package xmlns='http://www.idpf.org/2007/opf' xmlns:dc='http://purl.org/dc/elements/1.1/' version='3.0' unique-identifier='id'><metadata><dc:identifier id='id'>id</dc:identifier><dc:title>Book</dc:title><dc:creator>Author</dc:creator><dc:language>ja</dc:language></metadata><manifest><item id='chapter' href='chapter.xhtml' media-type='application/xhtml+xml'/></manifest><spine><itemref idref='chapter'/></spine></package>";
    let xhtml = b"<!DOCTYPE html [<!ENTITY private SYSTEM 'file:///private/sentinel'>]><html xmlns='http://www.w3.org/1999/xhtml'><body>&private;</body></html>";
    let mut archive = make_archive(&[
        ("META-INF/container.xml", container),
        ("OPS/book.opf", package),
        ("OPS/chapter.xhtml", xhtml),
    ]);

    let result = inspect(&mut archive);
    let diagnostics = format!("{:?}", result.diagnostics);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::XmlExternalEntityRejected
            && diagnostic.resource_path.as_deref() == Some("OPS/chapter.xhtml")
            && diagnostic.line.is_some()
            && diagnostic.column.is_some()
    }));
    assert!(!diagnostics.contains("private/sentinel"));
}

#[test]
fn rejects_rootfiles_nested_under_an_invalid_container_root() {
    let container = b"<not-container xmlns='urn:oasis:names:tc:opendocument:xmlns:container'><rootfiles><rootfile full-path='OPS/book.opf' media-type='application/oebps-package+xml'/></rootfiles></not-container>";
    let mut archive = make_archive(&[("META-INF/container.xml", container)]);

    let result = inspect(&mut archive);

    assert_eq!(result.coverage, InventoryCoverage::Partial);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::ContainerInvalidXml
            && diagnostic.resource_path.as_deref() == Some("META-INF/container.xml")
    }));
}

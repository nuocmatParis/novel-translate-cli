//! Actual image targets are independent of XML attribute order. The complete
//! EPUB3 counterexample was separately checked with EPUBCheck5.3.0.
#[path = "../../../apps/novel-translate/tests/support/epub.rs"]
#[allow(dead_code)]
mod fixture;
use nt_application::volume_import::source_range::{ProtectedSourceUnit, SourceUnitKind};
use nt_epub::{
    EpubZip, InventoryCoverage, PARSER_VERSION, ReferenceKind, ReferenceTarget, extract_source,
    index_references, inspect,
};
use sha2::{Digest, Sha256};
use std::io::Cursor;
const RED:&[u8]=br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><title>Red</title><rect width="10" height="10" fill="red"/></svg>"#;
const BLUE:&[u8]=br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><title>Blue</title><rect width="10" height="10" fill="blue"/></svg>"#;
fn bytes(attributes: &str, html: bool) -> Vec<u8> {
    let opf=br#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="3.0" unique-identifier="uid"><metadata><dc:identifier id="uid">urn:uuid:832fc946-6732-407f-a929-2a1c267d51ea</dc:identifier><dc:title>SVG reference</dc:title><dc:language>ja</dc:language><meta property="dcterms:modified">2026-01-01T00:00:00Z</meta></metadata><manifest><item id="ch" href="ch.xhtml" media-type="application/xhtml+xml" properties="svg"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="a" href="Images/a.svg" media-type="image/svg+xml"/><item id="b" href="Images/b.svg" media-type="image/svg+xml"/></manifest><spine><itemref idref="ch"/></spine></package>"#;
    let image = if html {
        format!("<img {attributes} alt=\"Illustration\"/>")
    } else {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"><title>Illustration</title><image {attributes} width=\"10\" height=\"10\"/></svg>"
        )
    };
    let chapter = format!(
        "<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" xmlns:f=\"urn:foreign\"><head><title>Story</title></head><body><h1 id=\"story\">Story</h1>{image}</body></html>"
    );
    let nav=br#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Contents</title></head><body><nav epub:type="toc"><h1>Contents</h1><ol><li><a href="ch.xhtml#story">Story</a></li></ol></nav></body></html>"#;
    fixture::stored_zip(&[("mimetype",b"application/epub+zip"),("META-INF/container.xml",br#"<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#),("OPS/book.opf",opf),("OPS/ch.xhtml",chapter.as_bytes()),("OPS/nav.xhtml",nav),("OPS/Images/a.svg",RED),("OPS/Images/b.svg",BLUE)])
}
fn check(attributes: &str, html: bool) {
    let bytes = bytes(attributes, html);
    let hash = Sha256::digest(&bytes).into();
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert_eq!(inventory.coverage, InventoryCoverage::Complete);
    let use_ = &inventory
        .resources
        .iter()
        .find(|r| r.path.as_deref() == Some("OPS/ch.xhtml"))
        .unwrap()
        .image_uses[0];
    assert_eq!(use_.resource_path.as_deref(), Some("OPS/Images/a.svg"));
    let source = extract_source(&mut archive, &inventory, hash, PARSER_VERSION).unwrap();
    assert!(source.complete);
    let blocks = &source
        .documents
        .iter()
        .find(|d| d.canonical_path == "OPS/ch.xhtml")
        .unwrap()
        .blocks;
    let image = blocks
        .iter()
        .flat_map(|b| &b.units)
        .find_map(|u| match &u.kind {
            SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) => Some(image),
            _ => None,
        })
        .unwrap();
    assert_eq!(image.resource_path.as_deref(), Some("OPS/Images/a.svg"));
    assert_eq!(image.raw_source.as_deref(), Some("Images/a.svg"));
    let expected: [u8; 32] = Sha256::digest(RED).into();
    let other: [u8; 32] = Sha256::digest(BLUE).into();
    assert_ne!(expected, other);
    let hashes = blocks
        .iter()
        .flat_map(|b| &b.protected_resource_hashes)
        .collect::<Vec<_>>();
    assert_eq!(hashes.len(), 1);
    assert_eq!(hashes[0].sha256, expected);
    let index = index_references(&mut archive, &inventory, &source, Default::default()).unwrap();
    let kind = if html {
        ReferenceKind::XhtmlImageSrc
    } else {
        ReferenceKind::SvgImageHref
    };
    let reference = index.references.iter().find(|r| r.kind == kind).unwrap();
    assert!(
        matches!(&reference.target,ReferenceTarget::Internal{path,..} if path=="OPS/Images/a.svg")
    );
    assert_eq!(reference.source_occurrences.len(), 1);
}
#[test]
fn plain_svg_href_wins_in_both_attribute_orders_with_distinct_actual_images() {
    check(r#"xlink:href="Images/b.svg" href="Images/a.svg""#, false);
    check(r#"href="Images/a.svg" xlink:href="Images/b.svg""#, false);
    check(r#"xlink:href="Images/a.svg""#, false);
}
#[test]
fn foreign_url_local_names_do_not_shadow_plain_url_attributes() {
    check(r#"f:href="Images/b.svg" href="Images/a.svg""#, false);
    check(r#"f:src="Images/b.svg" src="Images/a.svg""#, true);
}
#[test]
fn empty_plain_svg_href_is_not_replaced_by_xlink_fallback() {
    let bytes = bytes(r#"xlink:href="Images/b.svg" href="""#, false);
    let hash = Sha256::digest(&bytes).into();
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert!(
        inventory
            .resources
            .iter()
            .find(|r| r.path.as_deref() == Some("OPS/ch.xhtml"))
            .unwrap()
            .image_uses[0]
            .resource_path
            .is_none()
    );
    let source = extract_source(&mut archive, &inventory, hash, PARSER_VERSION).unwrap();
    assert!(!source.complete);
    for block in &source
        .documents
        .iter()
        .find(|d| d.canonical_path == "OPS/ch.xhtml")
        .unwrap()
        .blocks
    {
        for unit in &block.units {
            if let SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) = &unit.kind {
                assert_eq!(image.raw_source.as_deref(), Some(""));
                assert!(image.resource_path.is_none());
            }
        }
    }
}

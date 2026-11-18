//! Real, offline EPUB ZIPs with literal relationship and location oracles.

use std::io::{Cursor, Write};

use nt_epub::{
    ArchiveLimits, EpubInventory, EpubZip, PARSER_VERSION, ReferenceCoverage, ReferenceIndex,
    ReferenceIndexError, ReferenceIndexLimits, ReferenceKind, ReferenceTarget, ResourceReference,
    SourceExtraction, extract_source, index_references, inspect,
};
use sha2::{Digest, Sha256};
use zip::write::{SimpleFileOptions, ZipWriter};

const CHAPTER: &str = "<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head><link rel=\"stylesheet\" href=\"../Styles/main.css\"/></head>\n<body><img src=\"../Images/a.png\" srcset=\"../Images/a.png 1x, ../Images/b.png 2x\"/><p>Story</p></body>\n</html>";
const CSS: &str =
    "/* url(fake) */\n@import \"nested/more.css\";\n.x { background: url(\"../Images/a.png\"); }";
const NAV: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="Text/ch.xhtml">Chapter</a></li></ol></nav></body></html>"#;
const PLAIN: &str =
    r#"<html xmlns="http://www.w3.org/1999/xhtml"><head/><body><p>Story</p></body></html>"#;

struct Book {
    chapter: Vec<u8>,
    nav: Vec<u8>,
    css: Option<Vec<u8>>,
    manifest: String,
    metadata: String,
    spine: String,
    tail: String,
    version: &'static str,
    extra: Vec<(String, Vec<u8>)>,
}

impl Book {
    fn new(chapter: &str) -> Self {
        Self {
            chapter: chapter.as_bytes().to_vec(),
            nav: NAV.as_bytes().to_vec(),
            css: None,
            manifest: String::new(),
            metadata: String::new(),
            spine: "<itemref idref=\"ch\"/>".into(),
            tail: String::new(),
            version: "3.0",
            extra: Vec::new(),
        }
    }

    fn stylesheet(mut self, css: &str) -> Self {
        self.css = Some(css.as_bytes().to_vec());
        self
    }

    fn file(mut self, path: &str, content: &[u8]) -> Self {
        self.extra.push((path.to_owned(), content.to_vec()));
        self
    }

    fn bytes(&self) -> Vec<u8> {
        let css_item = if self.css.is_some() {
            r#"<item id="css" href="Styles/main.css" media-type="text/css"/>"#
        } else {
            ""
        };
        let package = format!(
            r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="{}" unique-identifier="uid"><metadata><dc:identifier id="uid">reference-test</dc:identifier><dc:title>Reference test</dc:title><dc:language>ja</dc:language>{}</metadata><manifest><item id="ch" href="Text/ch.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="a" href="Images/a.png" media-type="image/png"/><item id="b" href="Images/b.png" media-type="image/png"/>{}{}</manifest><spine>{}</spine>{}</package>"#,
            self.version, self.metadata, css_item, self.manifest, self.spine, self.tail
        );
        // A complete 1x1 PNG, not an externally downloaded asset.
        let png: Vec<u8> = "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000b49444154789c636000020000050001a5f645400000000049454e44ae426082"
            .as_bytes().as_chunks::<2>().0.iter().map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()).collect();
        let mut entries = vec![
            ("mimetype".into(), b"application/epub+zip".to_vec()),
            ("META-INF/container.xml".into(), br#"<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.to_vec()),
            ("OPS/book.opf".into(), package.into_bytes()),
            ("OPS/Text/ch.xhtml".into(), self.chapter.clone()),
            ("OPS/nav.xhtml".into(), self.nav.clone()),
            ("OPS/Images/a.png".into(), png.clone()),
            ("OPS/Images/b.png".into(), png),
        ];
        if let Some(css) = &self.css {
            entries.push(("OPS/Styles/main.css".into(), css.clone()));
        }
        entries.extend(self.extra.clone());
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (path, value) in entries {
            writer.start_file(path, options).unwrap();
            writer.write_all(&value).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
}

type Prepared = (EpubZip<Cursor<Vec<u8>>>, EpubInventory, SourceExtraction);

fn prepare(bytes: Vec<u8>, limits: ArchiveLimits) -> Prepared {
    let digest = Sha256::digest(&bytes).into();
    let mut archive = EpubZip::with_limits(Cursor::new(bytes), limits).unwrap();
    let inventory = inspect(&mut archive);
    let extracted = extract_source(&mut archive, &inventory, digest, PARSER_VERSION).unwrap();
    (archive, inventory, extracted)
}

fn index(book: &Book, limits: ReferenceIndexLimits) -> Result<ReferenceIndex, ReferenceIndexError> {
    let (mut archive, inventory, extracted) = prepare(book.bytes(), ArchiveLimits::default());
    index_references(&mut archive, &inventory, &extracted, limits)
}

fn all(book: &Book) -> ReferenceIndex {
    index(book, ReferenceIndexLimits::default()).unwrap()
}

#[test]
fn captured_inventory_and_navigation_preserve_the_legacy_evidence() {
    let bytes = Book::new(CHAPTER).bytes();
    let (mut legacy_archive, legacy_inventory, legacy_source) =
        prepare(bytes.clone(), ArchiveLimits::default());
    let mut captured = EpubZip::new(Cursor::new(bytes)).unwrap();
    let allowance = 4 * 1024 * 1024;
    let inventory = EpubInventory::inspect_captured(&mut captured, allowance).unwrap();
    assert_eq!(inventory, legacy_inventory);
    let (trees, diagnostics) =
        SourceExtraction::read_captured_navigation(&mut captured, &inventory, allowance).unwrap();
    assert_eq!(trees, legacy_source.navigation);
    assert_eq!(diagnostics, legacy_source.navigation_diagnostics);
    let (legacy_trees, _) = SourceExtraction::read_captured_navigation(
        &mut legacy_archive,
        &legacy_inventory,
        allowance,
    )
    .unwrap();
    assert_eq!(trees, legacy_trees);
}

#[test]
fn capture_with_insufficient_working_space_stops_before_reading_the_entry() {
    use nt_application::volume_import::CapturedSourceError;
    use nt_epub::SourceExtractionError;

    let bytes = Book::new(CHAPTER).bytes();
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    assert!(matches!(
        EpubInventory::inspect_captured(&mut archive, 1024),
        Err(CapturedSourceError::ResourceLimit)
    ));
    assert_eq!(archive.expanded_bytes_consumed(), 0);

    let inventory = inspect(&mut archive);
    let consumed = archive.expanded_bytes_consumed();
    assert!(matches!(
        SourceExtraction::read_captured_navigation(&mut archive, &inventory, 1024),
        Err(SourceExtractionError::WorkingMemoryLimit)
    ));
    assert_eq!(archive.expanded_bytes_consumed(), consumed);
    assert!(
        SourceExtraction::read_captured_navigation(&mut archive, &inventory, 4 * 1024 * 1024)
            .is_ok()
    );
}

fn uses<'a>(
    index: &'a ReferenceIndex,
    kind: ReferenceKind,
    source: &str,
) -> Vec<&'a ResourceReference> {
    index
        .references
        .iter()
        .filter(|item| item.kind == kind && item.source_path == source)
        .collect()
}

fn internal(item: &ResourceReference, path: &str) {
    assert_eq!(
        item.target,
        ReferenceTarget::Internal {
            path: path.to_owned(),
            fragment: None
        }
    );
}

fn reason(error: &ReferenceIndexError) -> &ReferenceIndexError {
    match error {
        ReferenceIndexError::At { reason: error, .. } => reason(error),
        error => error,
    }
}

fn utf16(value: &str, little: bool) -> Vec<u8> {
    let mut bytes = if little {
        vec![0xff, 0xfe]
    } else {
        vec![0xfe, 0xff]
    };
    for unit in value.encode_utf16() {
        bytes.extend_from_slice(&if little {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        });
    }
    bytes
}

#[test]
fn literal_locations_targets_and_repeated_spine_positions_are_preserved() {
    let mut book = Book::new(CHAPTER).stylesheet(CSS).file(
        "OPS/Styles/nested/more.css",
        b".x { background: url(\"../../Images/b.png\") }",
    );
    book.spine.push_str("<itemref idref=\"ch\" linear=\"no\"/>");
    let result = all(&book);
    assert_eq!(result.coverage, ReferenceCoverage::Complete);
    let link = uses(&result, ReferenceKind::StylesheetLink, "OPS/Text/ch.xhtml")[0];
    internal(link, "OPS/Styles/main.css");
    assert_eq!(link.location.element_ordinals, [0, 0]);
    assert_eq!((link.location.line, link.location.column), (2, 7));
    let images = uses(&result, ReferenceKind::XhtmlImageSrc, "OPS/Text/ch.xhtml");
    assert_eq!(
        images.len(),
        1,
        "inventory and source extraction refer to this same img"
    );
    internal(images[0], "OPS/Images/a.png");
    assert_eq!(images[0].location.element_ordinals, [1, 0]);
    assert_eq!((images[0].location.line, images[0].location.column), (3, 7));
    assert_eq!(images[0].inventory_occurrence, Some(0));
    assert_eq!(images[0].source_occurrences.len(), 1);
    assert_eq!(images[0].spine_positions, [0, 1]);
    let candidates = uses(&result, ReferenceKind::SrcsetCandidate, "OPS/Text/ch.xhtml");
    assert_eq!(candidates.len(), 2);
    internal(candidates[0], "OPS/Images/a.png");
    internal(candidates[1], "OPS/Images/b.png");
    assert_eq!(candidates[0].location.decoded_byte_range, Some((0, 15)));
    assert_eq!(candidates[1].location.decoded_byte_range, Some((20, 35)));
    assert_eq!(candidates[1].location.candidate_ordinal, 1);
    assert_eq!(
        candidates[0].location.attribute_local_name.as_deref(),
        Some("srcset")
    );
    assert_eq!(candidates[0].spine_positions, [0, 1]);
    let imports = uses(&result, ReferenceKind::CssImport, "OPS/Styles/main.css");
    assert_eq!(imports.len(), 1);
    internal(imports[0], "OPS/Styles/nested/more.css");
    assert_eq!(
        (imports[0].location.line, imports[0].location.column),
        (2, 1)
    );
    let css = uses(&result, ReferenceKind::CssUrl, "OPS/Styles/main.css");
    assert_eq!(css.len(), 1, "comment does not create a fake URL");
    internal(css[0], "OPS/Images/a.png");
    assert_eq!((css[0].location.line, css[0].location.column), (3, 18));
    assert!(css[0].location.element_ordinals.is_empty());
    internal(
        uses(&result, ReferenceKind::CssUrl, "OPS/Styles/nested/more.css")[0],
        "OPS/Images/b.png",
    );
}

#[test]
fn same_target_keeps_distinct_elements_and_inline_css_occurrences() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><style>.a { background: url('../Images/a.png') }</style></head><body><p id="p">Story<img src="../Images/a.png"/><img src="../Images/a.png" style="background:url('../Images/a.png')"/></p></body></html>"#;
    let result = all(&Book::new(chapter));
    let images = uses(&result, ReferenceKind::XhtmlImageSrc, "OPS/Text/ch.xhtml");
    assert_eq!(images.len(), 2);
    assert_eq!(images[0].location.element_ordinals, [1, 0, 0]);
    assert_eq!(images[1].location.element_ordinals, [1, 0, 1]);
    assert_eq!(images[0].inventory_occurrence, Some(0));
    assert_eq!(images[1].inventory_occurrence, Some(1));
    assert_eq!(
        images[0].source_occurrences[0].0,
        images[1].source_occurrences[0].0
    );
    assert_ne!(
        images[0].source_occurrences[0].1,
        images[1].source_occurrences[0].1
    );
    let css = uses(&result, ReferenceKind::CssUrl, "OPS/Text/ch.xhtml");
    assert_eq!(css.len(), 2);
    assert_eq!(css[1].location.element_ordinals, [1, 0, 1]);
    assert_eq!(
        css[1].location.attribute_local_name.as_deref(),
        Some("style")
    );
    assert_eq!(css[1].location.decoded_byte_range, Some((16, 31)));
    for item in images.into_iter().chain(css) {
        internal(item, "OPS/Images/a.png");
    }
}

#[test]
fn utf16_xml_entities_namespace_and_svg_href_precedence_agree_with_cp04() {
    let chapter = "<?xml version=\"1.0\" encoding=\"UTF-16\"?>\n<!DOCTYPE html [<!ENTITY img \"../Images/a.png\">]>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:f=\"urn:foreign\"><head/><body><p>語<img src=\"&img;\"/><f:img src=\"missing.png\"/><svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"><image href=\"../Images/a.png\" xlink:href=\"missing.png\"/><image xlink:href=\"../Images/b.png\"/></svg></p></body></html>";
    for little in [true, false] {
        let mut book = Book::new(PLAIN);
        book.chapter = utf16(chapter, little);
        let result = all(&book);
        let images = uses(&result, ReferenceKind::XhtmlImageSrc, "OPS/Text/ch.xhtml");
        assert_eq!(images.len(), 1);
        internal(images[0], "OPS/Images/a.png");
        assert_eq!(images[0].raw_reference, "../Images/a.png");
        assert_eq!(images[0].location.element_ordinals, [1, 0, 0]);
        assert_eq!(images[0].location.line, 3);
        let svg = uses(&result, ReferenceKind::SvgImageHref, "OPS/Text/ch.xhtml");
        assert_eq!(svg.len(), 2);
        internal(svg[0], "OPS/Images/a.png");
        internal(svg[1], "OPS/Images/b.png");
        assert_eq!(svg[0].location.element_ordinals, [1, 0, 2, 0]);
        assert_eq!(svg[0].location.attribute_namespace, None);
        assert_eq!(
            svg[1].location.attribute_namespace.as_deref(),
            Some("http://www.w3.org/1999/xlink")
        );
        assert!(svg.iter().all(|item| item.source_occurrences.len() == 1));
        assert!(
            !result
                .references
                .iter()
                .any(|item| item.raw_reference == "missing.png")
        );
    }
}

#[test]
fn css_bom_escapes_strings_functions_and_cycles_are_bounded_and_indexed_once() {
    let css = "@charset \"unsupported-label\";\n@\\69mport \"nested/more.css\";\n.x{background: u\\72l(../Images/\\61 .png); content: \"url(fake)\"; other: image-set(url('../Images/b.png') 1x); unused: #url(fake); dimension: 1url(fake)}";
    for little in [true, false] {
        let mut book = Book::new(PLAIN).stylesheet("").file(
            "OPS/Styles/nested/more.css",
            b"@import '../main.css'; .y { background: url('../../Images/b.png') }",
        );
        book.css = Some(utf16(css, little));
        let result = all(&book);
        let refs = uses(&result, ReferenceKind::CssUrl, "OPS/Styles/main.css");
        assert_eq!(refs.len(), 2);
        internal(refs[0], "OPS/Images/a.png");
        internal(refs[1], "OPS/Images/b.png");
        assert_eq!(refs[0].raw_reference, "../Images/\\61 .png");
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|d| d.code == ReferenceIndexError::ImportCycle)
                .count(),
            1
        );
        assert_eq!(result.coverage, ReferenceCoverage::NeedsReview);
        assert_eq!(
            uses(&result, ReferenceKind::CssUrl, "OPS/Styles/nested/more.css").len(),
            1
        );
    }
}

#[test]
fn srcset_data_commas_descriptors_and_private_evidence_are_preserved() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head/><body><p>Story<picture><source srcset="data:image/png;base64,AAAA 1x, ../Images/b.png 2x"/><img src="../Images/a.png"/></picture><a href="https://example.invalid/private?secret=do-not-log">link</a></p></body></html>"#;
    let result = all(&Book::new(chapter));
    let refs = uses(&result, ReferenceKind::SrcsetCandidate, "OPS/Text/ch.xhtml");
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].raw_reference, "data:image/png;base64,AAAA");
    assert_eq!(
        refs[0].target,
        ReferenceTarget::External {
            scheme: "data".into()
        }
    );
    internal(refs[1], "OPS/Images/b.png");
    assert!(!format!("{result:?}").contains("do-not-log"));
    assert!(!format!("{:?}", result.diagnostics).contains("example.invalid"));
}

#[test]
fn epub2_and_epub3_cover_roles_fallback_and_package_links_are_distinct() {
    let mut book = Book::new(PLAIN);
    book.version = "2.0";
    book.metadata =
        r#"<meta name="cover" content="a"/><link href="Images/b.png" rel="record"/>"#.into();
    book.manifest = r#"<item id="legacy" href="legacy.bin" media-type="application/octet-stream" fallback="ch" media-overlay="overlay"/><item id="overlay" href="overlay.smil" media-type="application/smil+xml"/>"#.into();
    book.tail = r#"<guide><reference type="cover" href="Images/a.png"/></guide>"#.into();
    book = book
        .file("OPS/legacy.bin", b"legacy")
        .file("OPS/overlay.smil", b"<smil/>");
    let result = all(&book);
    let covers = uses(&result, ReferenceKind::Cover, "OPS/book.opf");
    assert_eq!(
        covers.len(),
        2,
        "metadata role and guide use are distinct source declarations"
    );
    for cover in covers {
        internal(cover, "OPS/Images/a.png");
    }
    internal(
        uses(&result, ReferenceKind::Fallback, "OPS/book.opf")[0],
        "OPS/Text/ch.xhtml",
    );
    internal(
        uses(&result, ReferenceKind::MediaOverlay, "OPS/book.opf")[0],
        "OPS/overlay.smil",
    );
    internal(
        uses(&result, ReferenceKind::PackageLink, "OPS/book.opf")[0],
        "OPS/Images/b.png",
    );
    let mut epub3 = Book::new(PLAIN).file("OPS/Images/cover.png", b"cover");
    epub3.manifest = r#"<item id="cover" href="Images/cover.png" media-type="image/png" properties="cover-image"/>"#.into();
    let result = all(&epub3);
    let covers = uses(&result, ReferenceKind::Cover, "OPS/book.opf");
    assert_eq!(covers.len(), 1);
    internal(covers[0], "OPS/Images/cover.png");
    assert_eq!(
        covers[0].location.attribute_local_name.as_deref(),
        Some("properties")
    );
}

#[test]
fn missing_ambiguous_external_and_unsafe_references_have_distinct_safe_outcomes() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head/><body><p>Story<a href="absent.xhtml">missing</a><a href="https://example.invalid/sensitive">external</a><a href="../../../escape">unsafe</a><a href="../Images/a.png">ambiguous</a><a href="..%2fImages/b.png">encoded separator</a></p></body></html>"#;
    let mut book = Book::new(chapter);
    book.manifest = r#"<item id="alias" href="Images/./a.png" media-type="image/png"/>"#.into();
    let result = all(&book);
    let links = uses(&result, ReferenceKind::XhtmlLink, "OPS/Text/ch.xhtml");
    assert_eq!(links.len(), 5);
    assert_eq!(
        links[0].target,
        ReferenceTarget::Missing {
            path: "OPS/Text/absent.xhtml".into(),
            fragment: None
        }
    );
    assert_eq!(
        links[1].target,
        ReferenceTarget::External {
            scheme: "https".into()
        }
    );
    assert_eq!(
        links[2].target,
        ReferenceTarget::Invalid {
            reason: ReferenceIndexError::UnsafeUri
        }
    );
    assert_eq!(links[3].target, ReferenceTarget::Ambiguous);
    assert_eq!(
        links[4].target,
        ReferenceTarget::Invalid {
            reason: ReferenceIndexError::UnsafeUri
        }
    );
    assert_eq!(result.coverage, ReferenceCoverage::Incomplete);
    for code in [
        ReferenceIndexError::Missing,
        ReferenceIndexError::External,
        ReferenceIndexError::UnsafeUri,
        ReferenceIndexError::Ambiguous,
    ] {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == code && d.location.is_some())
        );
    }
}

#[test]
fn fragment_existence_and_ambiguity_have_deterministic_diagnostics() {
    let chapter = r##"<html xmlns="http://www.w3.org/1999/xhtml"><head/><body><p id="ok">Story</p><p id="dup">A</p><p id="dup">B</p><a name="legacy">Anchor</a><a href="#ok">ok</a><a href="#gone">missing</a><a href="#dup">ambiguous</a><a href="#legacy">named</a></body></html>"##;
    let book = Book::new(chapter);
    let first = all(&book);
    assert_eq!(first, all(&book));
    let links = uses(&first, ReferenceKind::XhtmlLink, "OPS/Text/ch.xhtml");
    assert!(
        matches!(&links[0].target, ReferenceTarget::Internal { fragment: Some(f), .. } if f == "ok")
    );
    assert!(
        matches!(&links[1].target, ReferenceTarget::Missing { fragment: Some(f), .. } if f == "gone")
    );
    assert_eq!(links[2].target, ReferenceTarget::Ambiguous);
    assert!(
        matches!(&links[3].target, ReferenceTarget::Internal { fragment: Some(f), .. } if f == "legacy")
    );
}

#[test]
fn html_base_uses_the_same_effective_image_as_inventory_and_extraction() {
    let chapter = "<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head><base href=\"../Images/\"/></head>\n<body><p>Story<img src=\"a.png\" alt=\"cover\"/></p></body></html>";
    let book = Book::new(chapter).file("OPS/Text/a.png", b"wrong document-relative bytes");
    let (mut archive, inventory, extracted) = prepare(book.bytes(), ArchiveLimits::default());
    let before = extracted.clone();
    let result = index_references(
        &mut archive,
        &inventory,
        &extracted,
        ReferenceIndexLimits::default(),
    )
    .unwrap();
    assert_eq!(extracted, before);
    internal(
        uses(&result, ReferenceKind::XhtmlImageSrc, "OPS/Text/ch.xhtml")[0],
        "OPS/Images/a.png",
    );
    assert_eq!(result.coverage, ReferenceCoverage::Complete);
    let image = uses(&result, ReferenceKind::XhtmlImageSrc, "OPS/Text/ch.xhtml")[0];
    assert_eq!(image.location.element_ordinals, [1, 0, 0]);
    assert_eq!(image.location.line, 3);
    assert_eq!(image.inventory_occurrence, Some(0));
    assert_eq!(image.source_occurrences.len(), 1);
    let block = extracted
        .documents
        .iter()
        .find(|d| d.canonical_path == "OPS/Text/ch.xhtml")
        .unwrap()
        .blocks
        .iter()
        .find(|b| !b.protected_resource_hashes.is_empty())
        .unwrap();
    assert_eq!(
        block
            .attribute_text
            .iter()
            .find(|a| a.attribute == "alt")
            .unwrap()
            .resource_path
            .as_deref(),
        Some("OPS/Images/a.png")
    );
    let id = archive
        .entry_by_path("OPS/Images/a.png")
        .unwrap()
        .unwrap()
        .id();
    let actual = archive.read_entry(id).unwrap();
    assert_eq!(
        block.protected_resource_hashes[0].sha256,
        <[u8; 32]>::from(Sha256::digest(actual))
    );
}

#[test]
fn xml_base_does_not_change_html_url_attribute_profile() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head/><body><p xml:base="../Images/">Story<img src="a.png"/></p></body></html>"#;
    let mut book = Book::new(chapter).file("OPS/Text/a.png", b"document-relative");
    book.manifest = r#"<item id="text-image" href="Text/a.png" media-type="image/png"/>"#.into();
    let result = all(&book);
    assert_eq!(result.coverage, ReferenceCoverage::Complete);
    internal(
        uses(&result, ReferenceKind::XhtmlImageSrc, "OPS/Text/ch.xhtml")[0],
        "OPS/Text/a.png",
    );
}

#[test]
fn shared_base_context_preserves_srcset_svg_inline_css_and_stylesheet_profiles() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:s="http://www.w3.org/2000/svg"><head><base href="../Images/"/><link rel="stylesheet" href="../Styles/main.css"/><style>.x{background:url(a.png)}</style></head><body><p>Story<img src="a.png" srcset="a.png 1x, b.png 2x"/></p><s:svg><s:g xml:base="../Svg/"><s:image href="a.png" style="fill:url(a.png)"/></s:g></s:svg></body></html>"#;
    let mut book = Book::new(chapter)
        .stylesheet(".x{background:url(../Images/b.png)}")
        .file("OPS/Svg/a.png", b"svg inherited bytes");
    book.manifest = r#"<item id="svg-image" href="Svg/a.png" media-type="image/png"/>"#.into();
    let result = all(&book);
    assert_eq!(
        result.coverage,
        ReferenceCoverage::Complete,
        "{:?}",
        result.diagnostics
    );
    internal(
        uses(&result, ReferenceKind::XhtmlImageSrc, "OPS/Text/ch.xhtml")[0],
        "OPS/Images/a.png",
    );
    internal(
        uses(&result, ReferenceKind::SvgImageHref, "OPS/Text/ch.xhtml")[0],
        "OPS/Svg/a.png",
    );
    let srcset = uses(&result, ReferenceKind::SrcsetCandidate, "OPS/Text/ch.xhtml");
    assert_eq!(srcset.len(), 2);
    internal(srcset[0], "OPS/Images/a.png");
    internal(srcset[1], "OPS/Images/b.png");
    let inline = uses(&result, ReferenceKind::CssUrl, "OPS/Text/ch.xhtml");
    assert_eq!(inline.len(), 2);
    for reference in inline {
        internal(reference, "OPS/Images/a.png");
    }
    internal(
        uses(&result, ReferenceKind::CssUrl, "OPS/Styles/main.css")[0],
        "OPS/Images/b.png",
    );
}

#[test]
fn first_eligible_html_base_can_appear_after_the_reference() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:f="urn:foreign"><head><f:base href="../Wrong/"/><base f:href="../Wrong/"/><base/></head><body><p>Story<img src="a.png"/></p><base href="../Images/"/><base href="../Wrong/"/></body></html>"#;
    let result = all(&Book::new(chapter));
    assert_eq!(result.coverage, ReferenceCoverage::Complete);
    internal(
        uses(&result, ReferenceKind::XhtmlImageSrc, "OPS/Text/ch.xhtml")[0],
        "OPS/Images/a.png",
    );
}

#[test]
fn optional_broken_nav_keeps_a_diagnosed_fallback_and_required_nav_still_fails() {
    let mut book = Book::new(PLAIN);
    book.nav = b"<broken".to_vec();
    let (mut archive, inventory, extracted) = prepare(book.bytes(), ArchiveLimits::default());
    assert!(extracted.diagnostics.iter().any(|d| matches!(
        d,
        nt_epub::SourceDiagnostic::InvalidOptionalNavigation { .. }
    )));
    let result = index_references(
        &mut archive,
        &inventory,
        &extracted,
        ReferenceIndexLimits::default(),
    )
    .unwrap();
    assert_eq!(result.coverage, ReferenceCoverage::Incomplete);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.source_path == "OPS/nav.xhtml"
                && d.code == ReferenceIndexError::OptionalNavigationSkipped)
    );
    assert!(!uses(&result, ReferenceKind::Manifest, "OPS/book.opf").is_empty());

    book.spine.push_str("<itemref idref=\"nav\"/>");
    let bytes = book.bytes();
    let hash = Sha256::digest(&bytes).into();
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert!(extract_source(&mut archive, &inventory, hash, PARSER_VERSION).is_err());
}

#[test]
fn inventory_source_identity_and_image_mismatches_are_rejected() {
    let book = Book::new(CHAPTER).stylesheet("");
    for case in 0..4 {
        let (mut archive, mut inventory, mut extracted) =
            prepare(book.bytes(), ArchiveLimits::default());
        match case {
            0 => extracted.epub_sha256[0] ^= 1,
            1 => extracted.documents[0].spine_positions.push(99),
            2 => inventory.resources[0].image_uses[0].line += 1,
            _ => {
                inventory.resources[0].image_uses[0].resource_path = Some("OPS/Images/b.png".into())
            }
        }
        let error = index_references(
            &mut archive,
            &inventory,
            &extracted,
            ReferenceIndexLimits::default(),
        )
        .unwrap_err();
        assert_eq!(reason(&error), &ReferenceIndexError::SourceMismatch);
    }
}

#[test]
fn all_configurable_limit_classes_return_typed_errors() {
    let book = Book::new(CHAPTER).stylesheet(CSS).file(
        "OPS/Styles/nested/more.css",
        b".x{background:url('../../Images/b.png')}",
    );
    let cases = [
        (
            ReferenceIndexLimits {
                max_working_bytes: 1,
                ..Default::default()
            },
            ReferenceIndexError::WorkingMemoryLimit,
        ),
        (
            ReferenceIndexLimits {
                max_files: 0,
                ..Default::default()
            },
            ReferenceIndexError::FileCountLimit,
        ),
        (
            ReferenceIndexLimits {
                max_references: 0,
                ..Default::default()
            },
            ReferenceIndexError::ReferenceCountLimit,
        ),
        (
            ReferenceIndexLimits {
                max_reference_bytes: 0,
                ..Default::default()
            },
            ReferenceIndexError::ReferenceByteLimit,
        ),
        (
            ReferenceIndexLimits {
                max_raw_reference_bytes: 0,
                ..Default::default()
            },
            ReferenceIndexError::RawEvidenceLimit,
        ),
        (
            ReferenceIndexLimits {
                max_xml_bytes: 1,
                ..Default::default()
            },
            ReferenceIndexError::XmlByteLimit,
        ),
        (
            ReferenceIndexLimits {
                max_xml_nodes: 1,
                ..Default::default()
            },
            ReferenceIndexError::XmlNodeLimit,
        ),
        (
            ReferenceIndexLimits {
                max_xml_depth: 0,
                ..Default::default()
            },
            ReferenceIndexError::XmlDepthLimit,
        ),
        (
            ReferenceIndexLimits {
                max_css_bytes: 1,
                ..Default::default()
            },
            ReferenceIndexError::CssByteLimit,
        ),
        (
            ReferenceIndexLimits {
                max_css_tokens: 0,
                ..Default::default()
            },
            ReferenceIndexError::CssTokenLimit,
        ),
        (
            ReferenceIndexLimits {
                max_css_function_depth: 0,
                ..Default::default()
            },
            ReferenceIndexError::CssFunctionDepthLimit,
        ),
        (
            ReferenceIndexLimits {
                max_css_import_depth: 0,
                ..Default::default()
            },
            ReferenceIndexError::CssImportDepthLimit,
        ),
    ];
    for (limits, expected) in cases {
        let error = index(&book, limits).unwrap_err();
        assert_eq!(reason(&error), &expected, "{error:?}");
        assert!(matches!(error, ReferenceIndexError::At { .. }));
    }
}

#[test]
fn limits_accept_exact_reference_count_and_raw_bytes_then_reject_one_less() {
    let book = Book::new(CHAPTER).stylesheet(".x{background:url('../Images/a.png')}");
    let result = all(&book);
    let count = result.references.len();
    let raw_bytes = result
        .references
        .iter()
        .map(|r| r.raw_reference.len())
        .sum();
    assert_eq!(
        index(
            &book,
            ReferenceIndexLimits {
                max_references: count,
                max_raw_reference_bytes: raw_bytes,
                ..Default::default()
            }
        )
        .unwrap(),
        result
    );
    assert_eq!(
        reason(
            &index(
                &book,
                ReferenceIndexLimits {
                    max_references: count - 1,
                    ..Default::default()
                }
            )
            .unwrap_err()
        ),
        &ReferenceIndexError::ReferenceCountLimit
    );
    assert_eq!(
        reason(
            &index(
                &book,
                ReferenceIndexLimits {
                    max_raw_reference_bytes: raw_bytes - 1,
                    ..Default::default()
                }
            )
            .unwrap_err()
        ),
        &ReferenceIndexError::RawEvidenceLimit
    );
}

#[test]
fn archive_rereads_consume_the_original_cumulative_budget() {
    let book = Book::new(PLAIN);
    let (archive, _, _) = prepare(book.bytes(), ArchiveLimits::default());
    let consumed = archive.expanded_bytes_consumed();
    let package_len = archive
        .entry_by_path("OPS/book.opf")
        .unwrap()
        .unwrap()
        .declared_size();
    for extra in [package_len - 1, package_len] {
        let (mut archive, inventory, extracted) = prepare(
            book.bytes(),
            ArchiveLimits {
                max_total_bytes: consumed + extra,
                ..Default::default()
            },
        );
        let error = index_references(
            &mut archive,
            &inventory,
            &extracted,
            ReferenceIndexLimits::default(),
        )
        .unwrap_err();
        assert_eq!(reason(&error), &ReferenceIndexError::ArchiveCumulativeLimit);
        // A declared entry that exceeds the remainder fails before any read.
        // At the exact boundary the package reread is charged once and the
        // following resource is rejected without resetting the session.
        let expected = if extra < package_len {
            consumed
        } else {
            consumed + package_len
        };
        assert_eq!(archive.expanded_bytes_consumed(), expected);
    }
}

#[test]
fn malformed_css_srcset_and_css_encoding_errors_are_located() {
    for css in [
        ".x{background:url(\"unterminated)}",
        ".x{background:url(bad\\\nurl)}",
        ".x{a:f(g(1)}",
    ] {
        let error = index(&Book::new(PLAIN).stylesheet(css), Default::default()).unwrap_err();
        assert_eq!(reason(&error), &ReferenceIndexError::InvalidCss);
        assert!(
            matches!(error, ReferenceIndexError::At { source_path, line: Some(_), column: Some(_), .. } if source_path == "OPS/Styles/main.css")
        );
    }
    for descriptors in ["1x 2x", "100w 2x", "100h", "+1x", "NaNx", "1.x", "0w"] {
        let chapter = format!(
            r#"<html xmlns="http://www.w3.org/1999/xhtml"><head/><body><p>Story<img src="../Images/a.png" srcset="../Images/a.png {descriptors}"/></p></body></html>"#
        );
        let error = index(&Book::new(&chapter), Default::default()).unwrap_err();
        assert_eq!(reason(&error), &ReferenceIndexError::InvalidSrcset);
        assert!(matches!(
            error,
            ReferenceIndexError::At {
                line: Some(1),
                column: Some(_),
                ..
            }
        ));
    }
    for bytes in [
        b"@charset \"shift_jis\"; .x {}".to_vec(),
        vec![0xff, 0xfe, 0x41],
        vec![0xc0, 0x80],
    ] {
        let mut book = Book::new(PLAIN);
        book.css = Some(bytes);
        assert_eq!(
            reason(&index(&book, Default::default()).unwrap_err()),
            &ReferenceIndexError::UnsupportedEncoding
        );
    }
}

#[test]
fn unsafe_xml_auxiliary_resources_reuse_the_existing_security_parser() {
    let cases: &[(&[u8], ReferenceIndexError)] = &[
        (br#"<!DOCTYPE svg [<!ENTITY bad SYSTEM "https://example.invalid/do-not-fetch">]><svg xmlns="http://www.w3.org/2000/svg"><text>&bad;</text></svg>"#, ReferenceIndexError::ExternalEntity),
        (br#"<svg xmlns="http://www.w3.org/2000/svg"><text>&unknown;</text></svg>"#, ReferenceIndexError::UnknownEntity),
        (br#"<?xml version="1.0" encoding="shift_jis"?><svg xmlns="http://www.w3.org/2000/svg"/>"#, ReferenceIndexError::UnsupportedEncoding),
        (br#"<svg xmlns="http://www.w3.org/2000/svg"><g></svg>"#, ReferenceIndexError::XmlMalformed),
    ];
    for (bytes, expected) in cases {
        let mut book = Book::new(PLAIN).file("OPS/bad.svg", bytes);
        book.manifest = r#"<item id="bad" href="bad.svg" media-type="image/svg+xml"/>"#.into();
        let error = index(&book, Default::default()).unwrap_err();
        assert_eq!(reason(&error), expected);
        assert!(!error.to_string().contains("do-not-fetch"));
    }
}

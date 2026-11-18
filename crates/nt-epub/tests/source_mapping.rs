//! Fixed source and mapping oracles over synthetic EPUB 2 and EPUB 3 packages.

use std::fs::{self, OpenOptions};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use nt_domain::chapter_map::{
    AuxiliaryDocumentCoverage, BlockUnitRange, BoundaryEvidence, BoundaryEvidenceKind, Chapter,
    ChapterMapDiagnostic, MainFlowContent, MappingStatus, NavigationSource, VolumeTree,
};
use nt_domain::source_range::{
    BlockIdentityRecord, BlockLocator, InlineMetadataKind, ProtectedSourceUnit, SourceUnitKind,
    UnitRange,
};
use nt_epub::{
    ArchiveError, ArchiveLimits, CoverageDomain, DiagnosticCode, EpubZip, PARSER_VERSION,
    SourceExtractionError, SpineContentKind, extract_source, inspect,
};
use sha2::{Digest, Sha256};
use zip::write::{SimpleFileOptions, ZipWriter};

const CONTAINER: &str = r#"<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OPS/book.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
const EPUB3_NAV: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head></head><body><nav epub:type="toc"><h2 id="contents-title">Contents heading.</h2><ol><li><a href="text/chapter.xhtml#container">Chapter One</a></li></ol></nav><p id="notice">Navigation notice.</p></body></html>"#;
const RUBY_FRAGMENT_NAV: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="text/chapter.xhtml#target">Ruby base</a></li></ol></nav></body></html>"#;
const EPUB3_PACKAGE: &str = r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="3.0" unique-identifier="uid"><metadata><dc:identifier id="uid">source-mapping-3</dc:identifier><dc:title>Source Mapping</dc:title><dc:language>ja</dc:language><dc:creator>Fixture</dc:creator></metadata><manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="text/chapter.xhtml" media-type="application/xhtml+xml"/><item id="page" href="images/page.png" media-type="image/png"/><item id="inline" href="images/inline.png" media-type="image/png"/><item id="end" href="text/end.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="chapter"/><itemref idref="page"/><itemref idref="end"/></spine></package>"#;
const EPUB3_CHAPTER: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body><div id="target">A<p id="middle">B</p>C</div><section id="container"><h1 id="heading">Chapter One</h1><p id="body">Body</p></section><p id="joined">ab<span>cd</span></p><p id="cross">e<em>&#x301;</em></p><p id="protected"><ruby><rb>漢</rb><rt>かん</rt><rp>(</rp><rp>)</rp></ruby><img src="../images/inline.png" alt="inline image"/><br/><hr/></p><p id="duplicate">one</p><p id="duplicate">two</p></body></html>"#;
const EPUB3_END: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body><p id="end">End.</p></body></html>"#;
const RUBY_FRAGMENT_CHAPTER: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="paragraph"><ruby><rb id="target">漢</rb><rt>かん</rt></ruby></p></body></html>"#;
const EPUB2_PACKAGE: &str = r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0" unique-identifier="uid"><metadata><dc-metadata><dc:identifier id="uid">source-mapping-2</dc:identifier><dc:title>Source Mapping</dc:title><dc:language>ja</dc:language><dc:creator>Fixture</dc:creator></dc-metadata></metadata><manifest><item id="toc" href="toc.ncx" media-type="application/x-dtbncx+xml"/><item id="one" href="text/one.xhtml" media-type="application/xhtml+xml"/><item id="two" href="text/two.xhtml" media-type="application/xhtml+xml"/><item id="appendix" href="appendix.xhtml" media-type="application/xhtml+xml"/></manifest><spine toc="toc"><itemref idref="one"/><itemref idref="two"/><itemref idref="appendix" linear="no"/></spine></package>"#;
const EPUB2_NCX: &str = r#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap><navPoint id="n1" playOrder="1"><navLabel><text>One</text></navLabel><content src="text/one.xhtml#first"/></navPoint><navPoint id="n2" playOrder="2"><navLabel><text>Two</text></navLabel><content src="text/two.xhtml#second"/></navPoint></navMap></ncx>"#;
const EPUB2_ONE: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body><h1 id="first">One</h1><p id="one-body">First text.</p></body></html>"#;
const EPUB2_TWO: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body><h1 id="second">Two</h1><p id="two-body">Second text.</p></body></html>"#;
const EPUB2_APPENDIX: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body><p id="appendix">Appendix text.</p></body></html>"#;
const FIXED_IMAGE_PACKAGE: &str = r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="3.0" unique-identifier="uid"><metadata><dc:identifier id="uid">fixed-image-mapping</dc:identifier><dc:title>Fixed image mapping</dc:title><dc:language>ja</dc:language><dc:creator>Fixture</dc:creator><meta property="rendition:layout">reflowable</meta></metadata><manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="xhtml-page" href="xhtml-page.xhtml" media-type="application/xhtml+xml"/><item id="svg-page" href="svg-page.svg" media-type="image/svg+xml"/><item id="pixel" href="images/pixel.png" media-type="image/png"/></manifest><spine><itemref idref="chapter"/><itemref idref="xhtml-page" properties="rendition:layout-pre-paginated"/><itemref idref="svg-page" properties="rendition:layout-pre-paginated"/></spine></package>"#;
const FIXED_IMAGE_NAV: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#opening">Chapter</a></li><li><a href="xhtml-page.xhtml#container">XHTML page</a></li><li><a href="svg-page.svg#panel">SVG page</a></li></ol></nav></body></html>"#;
const FIXED_IMAGE_CHAPTER: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="opening">Opening.</p></body></html>"#;
const XHTML_IMAGE_PAGE: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body id="container"><svg xmlns="http://www.w3.org/2000/svg"><image id="inline-pixel" href="images/pixel.png" aria-label="inline art"/></svg></body></html>"#;
const SVG_IMAGE_PAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" id="panel"><image id="embedded-pixel" href="images/pixel.png"/></svg>"#;

fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (path, bytes) in entries {
        writer.start_file(*path, stored).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn epub3(chapter: &str) -> Vec<u8> {
    epub3_with_nav(chapter, EPUB3_NAV)
}

fn epub3_with_nav(chapter: &str, nav: &str) -> Vec<u8> {
    zip(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OPS/book.opf", EPUB3_PACKAGE.as_bytes()),
        ("OPS/nav.xhtml", nav.as_bytes()),
        ("OPS/text/chapter.xhtml", chapter.as_bytes()),
        ("OPS/images/page.png", b"page-image"),
        ("OPS/images/inline.png", b"inline-image"),
        ("OPS/text/end.xhtml", EPUB3_END.as_bytes()),
    ])
}

fn epub2() -> Vec<u8> {
    zip(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OPS/book.opf", EPUB2_PACKAGE.as_bytes()),
        ("OPS/toc.ncx", EPUB2_NCX.as_bytes()),
        ("OPS/text/one.xhtml", EPUB2_ONE.as_bytes()),
        ("OPS/text/two.xhtml", EPUB2_TWO.as_bytes()),
        ("OPS/appendix.xhtml", EPUB2_APPENDIX.as_bytes()),
    ])
}

fn fixed_layout_image_pages() -> Vec<u8> {
    zip(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OPS/book.opf", FIXED_IMAGE_PACKAGE.as_bytes()),
        ("OPS/nav.xhtml", FIXED_IMAGE_NAV.as_bytes()),
        ("OPS/chapter.xhtml", FIXED_IMAGE_CHAPTER.as_bytes()),
        ("OPS/xhtml-page.xhtml", XHTML_IMAGE_PAGE.as_bytes()),
        ("OPS/svg-page.svg", SVG_IMAGE_PAGE.as_bytes()),
        ("OPS/images/pixel.png", b"fixed-page-pixel"),
    ])
}

fn extract(bytes: &[u8]) -> nt_epub::SourceExtraction {
    let mut archive = EpubZip::new(Cursor::new(bytes.to_vec())).unwrap();
    let inventory = inspect(&mut archive);
    extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION).unwrap()
}

fn document<'a>(
    extraction: &'a nt_epub::SourceExtraction,
    path: &str,
) -> &'a nt_epub::SourceDocument {
    extraction
        .documents
        .iter()
        .find(|document| document.canonical_path == path)
        .unwrap()
}

fn block<'a>(
    document: &'a nt_epub::SourceDocument,
    locator: &BlockLocator,
) -> &'a nt_domain::source_range::SourceBlock {
    document
        .blocks
        .iter()
        .find(|block| &block.locator == locator)
        .unwrap()
}

fn grapheme_text(block: &nt_domain::source_range::SourceBlock) -> String {
    block
        .units
        .iter()
        .map(|unit| match &unit.kind {
            SourceUnitKind::Grapheme(value) => value.as_str(),
            SourceUnitKind::Protected(_) => "",
        })
        .collect()
}

fn source_range(
    document: &nt_epub::SourceDocument,
    source_block: &nt_domain::source_range::SourceBlock,
    expected_units: usize,
) -> BlockUnitRange {
    assert_eq!(source_block.units.len(), expected_units);
    BlockUnitRange {
        canonical_path: document.canonical_path.clone(),
        block_id: source_block.id,
        start: 0,
        end: u32::try_from(expected_units).unwrap(),
    }
}

fn source_range_by_text(
    document: &nt_epub::SourceDocument,
    expected_text: &str,
    expected_units: usize,
) -> BlockUnitRange {
    let matches = document
        .blocks
        .iter()
        .filter(|candidate| grapheme_text(candidate) == expected_text)
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "{expected_text:?} must identify one source block"
    );
    source_range(document, matches[0], expected_units)
}

fn navigation_start_evidence(
    extraction: &nt_epub::SourceExtraction,
    source: NavigationSource,
    canonical_path: &str,
    fragment: &str,
    heading_locator: &BlockLocator,
    label: &str,
    raw_href: &str,
) -> Vec<BoundaryEvidence> {
    let anchor = extraction
        .anchors
        .iter()
        .find(|anchor| anchor.canonical_path == canonical_path && anchor.fragment == fragment)
        .unwrap();
    let heading_block = block(document(extraction, canonical_path), heading_locator);
    let navigation = BoundaryEvidence {
        kind: BoundaryEvidenceKind::Navigation,
        source: Some(source),
        canonical_path: canonical_path.to_owned(),
        block_id: Some(anchor.block_id),
        unit_offset: Some(anchor.unit_offset),
        label: Some(label.to_owned()),
        raw_href: Some(raw_href.to_owned()),
        reason: "resolved navigation target".to_owned(),
        depth: Some(0),
    };
    let heading = BoundaryEvidence {
        kind: BoundaryEvidenceKind::Heading,
        source: None,
        canonical_path: canonical_path.to_owned(),
        block_id: Some(heading_block.id),
        unit_offset: Some(0),
        label: Some(label.to_owned()),
        raw_href: None,
        reason: "XHTML heading h1; fallback evidence".to_owned(),
        depth: Some(0),
    };
    let mut corroborated_navigation = navigation.clone();
    corroborated_navigation.reason =
        "nested navigation target is corroborated by a same-position heading".to_owned();
    vec![navigation, heading, corroborated_navigation]
}

#[test]
fn epub3_source_order_spans_and_protected_units_match_the_written_oracle() {
    let extraction = extract(&epub3(EPUB3_CHAPTER));
    let chapter = document(&extraction, "OPS/text/chapter.xhtml");

    let target = block(chapter, &BlockLocator::UniqueXhtmlId("target".to_owned()));
    let middle = block(chapter, &BlockLocator::UniqueXhtmlId("middle".to_owned()));
    assert_eq!(grapheme_text(target), "A");
    assert_eq!(
        target.id.to_string(),
        "21f704cc24bf345a206293a7ac7ef954f51059d310169a1e9b6d1740179f4d04"
    );
    assert_eq!(grapheme_text(middle), "B");
    assert_eq!(
        middle.id.to_string(),
        "3eac6ea6d1afcec2c472b5a7c2c0ae89f575e3d6d09fa92059522c8d58d79c66"
    );
    let tail = chapter
        .blocks
        .iter()
        .find(|block| {
            matches!(
                &block.locator,
                BlockLocator::NodePath { element_ordinals, .. } if element_ordinals == &[1, 0]
            ) && grapheme_text(block) == "C"
        })
        .unwrap();
    assert_eq!(grapheme_text(tail), "C");
    assert!(
        chapter
            .blocks
            .iter()
            .position(|block| block.id == target.id)
            .unwrap()
            < chapter
                .blocks
                .iter()
                .position(|block| block.id == middle.id)
                .unwrap()
    );
    assert!(
        chapter
            .blocks
            .iter()
            .position(|block| block.id == middle.id)
            .unwrap()
            < chapter
                .blocks
                .iter()
                .position(|block| block.id == tail.id)
                .unwrap()
    );
    assert_eq!(target.units[0].spans[0].node().text_node_order, 0);
    assert_eq!(middle.units[0].spans[0].node().text_node_order, 0);
    assert_eq!(tail.units[0].spans[0].node().text_node_order, 2);
    assert_eq!(tail.units[0].spans[0].start_byte(), 0);
    assert_eq!(tail.units[0].spans[0].end_byte(), 1);

    let joined = block(chapter, &BlockLocator::UniqueXhtmlId("joined".to_owned()));
    assert_eq!(grapheme_text(joined), "abcd");
    assert_eq!(joined.units[0].spans[0].node().text_node_order, 0);
    assert_eq!(joined.units[2].spans[0].node().text_node_order, 1);
    assert_eq!(joined.units[3].spans[0].start_byte(), 1);

    let cross = block(chapter, &BlockLocator::UniqueXhtmlId("cross".to_owned()));
    assert_eq!(cross.units.len(), 1);
    assert_eq!(
        cross.units[0].kind,
        SourceUnitKind::Grapheme("e\u{301}".to_owned())
    );
    assert_eq!(cross.units[0].spans.len(), 2);
    assert_eq!(cross.units[0].spans[0].node().text_node_order, 0);
    assert_eq!(cross.units[0].spans[1].node().text_node_order, 1);

    let protected = block(
        chapter,
        &BlockLocator::UniqueXhtmlId("protected".to_owned()),
    );
    assert!(matches!(
        protected.units[0].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(_))
    ));
    assert!(matches!(
        protected.units[1].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::Image(_))
    ));
    assert!(matches!(
        protected.units[2].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::LineBreak { .. })
    ));
    assert!(matches!(
        protected.units[3].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::SceneBreak { .. })
    ));
    assert_eq!(protected.attribute_text[0].value, "inline image");
    assert!(chapter.blocks.iter().all(|block| {
        !block
            .locator
            .eq(&BlockLocator::UniqueXhtmlId("duplicate".to_owned()))
    }));
    assert!(
        !extraction
            .anchors
            .iter()
            .any(|anchor| anchor.fragment == "duplicate")
    );

    let container = extraction
        .anchors
        .iter()
        .find(|anchor| anchor.fragment == "container")
        .unwrap();
    let container_block = chapter
        .blocks
        .iter()
        .find(|block| block.id == container.block_id)
        .unwrap();
    assert!(container.is_unique);
    assert_eq!(container.unit_offset, 0);
    assert_eq!(
        container_block.id.to_string(),
        "0a49ae720ecddb8f96898aaebe5150f9004705dc6145a23c3d1fd08e1dda46f0"
    );
    assert!(container_block.units.is_empty());
    assert_eq!(
        container_block.locator,
        BlockLocator::UniqueXhtmlId("container".to_owned())
    );
}

#[test]
fn epub3_navigation_text_and_linear_image_pages_keep_distinct_domains() {
    let bytes = epub3(EPUB3_CHAPTER);
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    let page = inventory
        .spine
        .iter()
        .find(|item| item.idref == "page")
        .unwrap();
    assert!(page.linear);
    assert_eq!(page.kind, SpineContentKind::ImagePage);
    let extraction = extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION).unwrap();

    let main_paths = extraction
        .documents
        .iter()
        .filter(|document| document.domain == CoverageDomain::MainFlow)
        .map(|document| document.canonical_path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        main_paths,
        vec![
            "OPS/text/chapter.xhtml",
            "OPS/images/page.png",
            "OPS/text/end.xhtml"
        ]
    );

    let image_page = document(&extraction, "OPS/images/page.png");
    assert_eq!(image_page.domain, CoverageDomain::MainFlow);
    assert_eq!(image_page.linear_spine_positions, vec![1]);
    assert_eq!(image_page.blocks.len(), 1);
    assert_eq!(
        image_page.blocks[0].locator,
        BlockLocator::NodePath {
            element_ordinals: vec![],
            order: 0,
        }
    );
    assert!(matches!(
        &image_page.blocks[0].units[0].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) if image.occurrence == 1
    ));
    assert_eq!(image_page.blocks[0].protected_resource_hashes.len(), 1);
    let expected_page_hash: [u8; 32] = Sha256::digest(b"page-image").into();
    assert_eq!(
        image_page.blocks[0].protected_resource_hashes[0].sha256,
        expected_page_hash
    );

    let navigation_document = document(&extraction, "OPS/nav.xhtml");
    assert_eq!(navigation_document.domain, CoverageDomain::Auxiliary);
    assert_eq!(
        grapheme_text(block(
            navigation_document,
            &BlockLocator::UniqueXhtmlId("contents-title".to_owned())
        )),
        "Contents heading."
    );
    assert_eq!(
        grapheme_text(block(
            navigation_document,
            &BlockLocator::UniqueXhtmlId("notice".to_owned())
        )),
        "Navigation notice."
    );
    assert_eq!(navigation_document.blocks.len(), 2);
    assert!(
        navigation_document
            .blocks
            .iter()
            .all(|block| { !grapheme_text(block).contains("Chapter One") })
    );
    assert_eq!(extraction.navigation.len(), 1);
    assert_eq!(extraction.navigation[0].source, NavigationSource::Epub3Nav);
    assert_eq!(extraction.navigation[0].items[0].label, "Chapter One");
    assert!(extraction.complete, "{:?}", extraction.diagnostics);

    let map_input = extraction.chapter_map_input(Vec::new());
    let chapter_map = nt_application::chapter_map::build_candidate_map(&map_input).unwrap();
    let chapter_source = document(&extraction, "OPS/text/chapter.xhtml");
    let heading_locator = BlockLocator::UniqueXhtmlId("heading".to_owned());
    let expected_chapter = Chapter {
        chapter_id: 0,
        label: Some("Chapter One".to_owned()),
        status: MappingStatus::Provisional,
        ranges: vec![
            source_range(chapter_source, block(chapter_source, &heading_locator), 11),
            source_range(
                chapter_source,
                block(
                    chapter_source,
                    &BlockLocator::UniqueXhtmlId("body".to_owned()),
                ),
                4,
            ),
            source_range(
                chapter_source,
                block(
                    chapter_source,
                    &BlockLocator::UniqueXhtmlId("joined".to_owned()),
                ),
                4,
            ),
            source_range(
                chapter_source,
                block(
                    chapter_source,
                    &BlockLocator::UniqueXhtmlId("cross".to_owned()),
                ),
                1,
            ),
            source_range(
                chapter_source,
                block(
                    chapter_source,
                    &BlockLocator::UniqueXhtmlId("protected".to_owned()),
                ),
                4,
            ),
            source_range_by_text(chapter_source, "one", 3),
            source_range_by_text(chapter_source, "two", 3),
            source_range(
                document(&extraction, "OPS/images/page.png"),
                &document(&extraction, "OPS/images/page.png").blocks[0],
                1,
            ),
            source_range(
                document(&extraction, "OPS/text/end.xhtml"),
                block(
                    document(&extraction, "OPS/text/end.xhtml"),
                    &BlockLocator::UniqueXhtmlId("end".to_owned()),
                ),
                4,
            ),
        ],
        start_evidence: navigation_start_evidence(
            &extraction,
            NavigationSource::Epub3Nav,
            "OPS/text/chapter.xhtml",
            "container",
            &heading_locator,
            "Chapter One",
            "text/chapter.xhtml#container",
        ),
    };
    let expected_out_of_chapter = vec![
        source_range(
            chapter_source,
            block(
                chapter_source,
                &BlockLocator::UniqueXhtmlId("target".to_owned()),
            ),
            1,
        ),
        source_range(
            chapter_source,
            block(
                chapter_source,
                &BlockLocator::UniqueXhtmlId("middle".to_owned()),
            ),
            1,
        ),
        source_range_by_text(chapter_source, "C", 1),
    ];
    let navigation_document = document(&extraction, "OPS/nav.xhtml");
    let expected_volume = VolumeTree {
        main_flow: vec![
            MainFlowContent::OutOfChapter {
                ranges: expected_out_of_chapter,
                reason: "content precedes the first mapped boundary".to_owned(),
            },
            MainFlowContent::Chapter(expected_chapter),
        ],
        auxiliary: vec![AuxiliaryDocumentCoverage {
            canonical_path: "OPS/nav.xhtml".to_owned(),
            ranges: vec![
                source_range(
                    navigation_document,
                    block(
                        navigation_document,
                        &BlockLocator::UniqueXhtmlId("contents-title".to_owned()),
                    ),
                    17,
                ),
                source_range(
                    navigation_document,
                    block(
                        navigation_document,
                        &BlockLocator::UniqueXhtmlId("notice".to_owned()),
                    ),
                    18,
                ),
            ],
        }],
    };
    assert_eq!(chapter_map.epub_sha256, extraction.epub_sha256);
    assert_eq!(chapter_map.parser_version, extraction.parser_version);
    assert_eq!(chapter_map.volume, expected_volume);
    assert_eq!(chapter_map.navigation, map_input.navigation);
    assert!(chapter_map.diagnostics.is_empty());
    assert_eq!(
        map_input
            .documents
            .iter()
            .find(|document| document.canonical_path == "OPS/nav.xhtml")
            .unwrap()
            .domain,
        nt_domain::chapter_map::CoverageDomain::Auxiliary
    );
}

#[test]
fn epub3_navigation_target_inside_ruby_base_maps_to_the_protected_unit() {
    let extraction = extract(&epub3_with_nav(RUBY_FRAGMENT_CHAPTER, RUBY_FRAGMENT_NAV));
    assert!(extraction.complete, "{:?}", extraction.diagnostics);
    let chapter = document(&extraction, "OPS/text/chapter.xhtml");
    let paragraph = block(
        chapter,
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(paragraph.units.len(), 1);
    assert!(matches!(
        paragraph.units[0].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(_))
    ));

    let anchor = extraction
        .anchors
        .iter()
        .find(|anchor| {
            anchor.canonical_path == "OPS/text/chapter.xhtml" && anchor.fragment == "target"
        })
        .unwrap();
    assert_eq!(anchor.block_id, paragraph.id);
    assert_eq!(anchor.unit_offset, 0);
    assert_eq!(anchor.locator.element_ordinals, vec![0, 0]);

    let map_input = extraction.chapter_map_input(Vec::new());
    let chapter_map = nt_application::chapter_map::build_candidate_map(&map_input).unwrap();
    assert_eq!(chapter_map.navigation[0].items[0].label, "Ruby base");
    assert!(chapter_map.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic,
        nt_domain::chapter_map::ChapterMapDiagnostic::NavigationTargetUnresolved { label, .. }
            if label == "Ruby base"
    )));
    assert!(chapter_map.volume.main_flow.iter().any(|item| matches!(
        item,
        MainFlowContent::Chapter(chapter)
            if chapter.label.as_deref() == Some("Ruby base")
                && chapter.start_evidence.iter().any(|evidence| {
                    evidence.source == Some(NavigationSource::Epub3Nav)
                        && evidence.block_id == Some(paragraph.id)
                        && evidence.unit_offset == Some(0)
                })
    )));
}

#[test]
fn fixed_layout_xhtml_and_svg_pages_keep_images_and_fragment_anchors() {
    let bytes = fixed_layout_image_pages();
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert_eq!(
        inventory.coverage,
        nt_epub::InventoryCoverage::Complete,
        "{:?}",
        inventory.diagnostics
    );
    assert_eq!(inventory.spine.len(), 3);
    assert_eq!(
        inventory.spine[0].kind,
        SpineContentKind::ReflowableDocument
    );
    assert!(
        inventory.spine[1..]
            .iter()
            .all(|item| { item.kind == SpineContentKind::ImagePage && item.image_uses.len() == 1 })
    );

    let extraction = extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION).unwrap();
    assert!(extraction.complete, "{:?}", extraction.diagnostics);

    let xhtml_page = document(&extraction, "OPS/xhtml-page.xhtml");
    assert_eq!(xhtml_page.domain, CoverageDomain::MainFlow);
    assert_eq!(xhtml_page.linear_spine_positions, vec![1]);
    let xhtml_block = xhtml_page
        .blocks
        .iter()
        .find(|block| {
            block.units.iter().any(|unit| {
                matches!(
                    unit.kind,
                    SourceUnitKind::Protected(ProtectedSourceUnit::Image(_))
                )
            })
        })
        .unwrap();
    let xhtml_image = xhtml_block
        .units
        .iter()
        .find_map(|unit| match &unit.kind {
            SourceUnitKind::Protected(ProtectedSourceUnit::Image(image)) => Some(image),
            _ => None,
        })
        .unwrap();
    assert_eq!(xhtml_image.raw_source.as_deref(), Some("images/pixel.png"));
    assert_eq!(
        xhtml_image.resource_path.as_deref(),
        Some("OPS/images/pixel.png")
    );
    assert_eq!(xhtml_image.alt.as_deref(), Some("inline art"));
    assert!(xhtml_block.attribute_text.iter().any(|attribute| {
        attribute.attribute == "aria-label" && attribute.value == "inline art"
    }));
    let pixel_hash: [u8; 32] = Sha256::digest(b"fixed-page-pixel").into();
    assert!(
        xhtml_block
            .protected_resource_hashes
            .iter()
            .any(|resource| {
                resource.canonical_path == "OPS/images/pixel.png" && resource.sha256 == pixel_hash
            })
    );
    let container_anchor = extraction
        .anchors
        .iter()
        .find(|anchor| {
            anchor.canonical_path == "OPS/xhtml-page.xhtml" && anchor.fragment == "container"
        })
        .unwrap();
    assert_eq!(container_anchor.block_id, xhtml_block.id);
    assert_eq!(container_anchor.unit_offset, 0);

    let svg_page = document(&extraction, "OPS/svg-page.svg");
    assert_eq!(svg_page.domain, CoverageDomain::MainFlow);
    assert_eq!(svg_page.linear_spine_positions, vec![2]);
    assert_eq!(svg_page.blocks.len(), 1);
    let svg_block = &svg_page.blocks[0];
    assert_eq!(
        svg_block.locator,
        BlockLocator::NodePath {
            element_ordinals: vec![],
            order: 0,
        }
    );
    assert!(matches!(
        &svg_block.units[0].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::Image(image))
            if image.resource_path.as_deref() == Some("OPS/svg-page.svg") && image.occurrence == 2
    ));
    let svg_hash: [u8; 32] = Sha256::digest(SVG_IMAGE_PAGE.as_bytes()).into();
    assert!(svg_block.protected_resource_hashes.iter().any(|resource| {
        resource.canonical_path == "OPS/svg-page.svg" && resource.sha256 == svg_hash
    }));
    let panel_anchor = extraction
        .anchors
        .iter()
        .find(|anchor| anchor.canonical_path == "OPS/svg-page.svg" && anchor.fragment == "panel")
        .unwrap();
    assert_eq!(panel_anchor.block_id, svg_block.id);
    assert_eq!(panel_anchor.locator.block, svg_block.locator);
    assert!(panel_anchor.locator.element_ordinals.is_empty());
    assert_eq!(panel_anchor.unit_offset, 0);
    assert!(panel_anchor.is_unique);
    let child_anchor = extraction
        .anchors
        .iter()
        .find(|anchor| {
            anchor.canonical_path == "OPS/svg-page.svg" && anchor.fragment == "embedded-pixel"
        })
        .unwrap();
    assert_eq!(child_anchor.block_id, svg_block.id);
    assert_eq!(child_anchor.locator.element_ordinals, vec![0]);
}

#[test]
fn epub2_ncx_preserves_multifile_order_and_auxiliary_text() {
    let extraction = extract(&epub2());
    let map_input = extraction.chapter_map_input(Vec::new());
    let chapter_map = nt_application::chapter_map::build_candidate_map(&map_input).unwrap();
    assert_eq!(extraction.navigation.len(), 1);
    assert_eq!(extraction.navigation[0].source, NavigationSource::Epub2Ncx);
    assert_eq!(extraction.navigation[0].items.len(), 2);
    assert_eq!(extraction.navigation[0].items[0].label, "One");
    assert_eq!(extraction.navigation[0].items[1].label, "Two");
    let main_paths = extraction
        .documents
        .iter()
        .filter(|document| document.domain == CoverageDomain::MainFlow)
        .map(|document| document.canonical_path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(main_paths, vec!["OPS/text/one.xhtml", "OPS/text/two.xhtml"]);
    let first_oracle = block(
        document(&extraction, "OPS/text/one.xhtml"),
        &BlockLocator::UniqueXhtmlId("first".to_owned()),
    );
    assert_eq!(
        first_oracle.id.to_string(),
        "66b13839edd7b8445ec8b1588bcc1fff2e1ff931a9badc8d28461d55406b9592"
    );
    assert_eq!(
        grapheme_text(block(
            document(&extraction, "OPS/text/one.xhtml"),
            &BlockLocator::UniqueXhtmlId("first".to_owned())
        )),
        "One"
    );
    assert_eq!(
        grapheme_text(block(
            document(&extraction, "OPS/text/two.xhtml"),
            &BlockLocator::UniqueXhtmlId("second".to_owned())
        )),
        "Two"
    );
    assert_eq!(
        grapheme_text(block(
            document(&extraction, "OPS/appendix.xhtml"),
            &BlockLocator::UniqueXhtmlId("appendix".to_owned())
        )),
        "Appendix text."
    );
    assert_eq!(
        document(&extraction, "OPS/appendix.xhtml").domain,
        CoverageDomain::Auxiliary
    );
    assert!(
        !extraction.complete,
        "the existing EPUB 2 metadata parser reports this standards-shaped fixture as partial"
    );

    let first_source = document(&extraction, "OPS/text/one.xhtml");
    let second_source = document(&extraction, "OPS/text/two.xhtml");
    let appendix_source = document(&extraction, "OPS/appendix.xhtml");
    let first_heading = BlockLocator::UniqueXhtmlId("first".to_owned());
    let second_heading = BlockLocator::UniqueXhtmlId("second".to_owned());
    let expected_volume = VolumeTree {
        main_flow: vec![
            MainFlowContent::Chapter(Chapter {
                chapter_id: 0,
                label: Some("One".to_owned()),
                status: MappingStatus::Provisional,
                ranges: vec![
                    source_range(first_source, block(first_source, &first_heading), 3),
                    source_range_by_text(first_source, "First text.", 11),
                ],
                start_evidence: navigation_start_evidence(
                    &extraction,
                    NavigationSource::Epub2Ncx,
                    "OPS/text/one.xhtml",
                    "first",
                    &first_heading,
                    "One",
                    "text/one.xhtml#first",
                ),
            }),
            MainFlowContent::Chapter(Chapter {
                chapter_id: 1,
                label: Some("Two".to_owned()),
                status: MappingStatus::Provisional,
                ranges: vec![
                    source_range(second_source, block(second_source, &second_heading), 3),
                    source_range_by_text(second_source, "Second text.", 12),
                ],
                start_evidence: navigation_start_evidence(
                    &extraction,
                    NavigationSource::Epub2Ncx,
                    "OPS/text/two.xhtml",
                    "second",
                    &second_heading,
                    "Two",
                    "text/two.xhtml#second",
                ),
            }),
        ],
        auxiliary: vec![AuxiliaryDocumentCoverage {
            canonical_path: "OPS/appendix.xhtml".to_owned(),
            ranges: vec![source_range(
                appendix_source,
                block(
                    appendix_source,
                    &BlockLocator::UniqueXhtmlId("appendix".to_owned()),
                ),
                14,
            )],
        }],
    };
    assert_eq!(chapter_map.epub_sha256, extraction.epub_sha256);
    assert_eq!(chapter_map.parser_version, extraction.parser_version);
    assert_eq!(chapter_map.volume, expected_volume);
    assert_eq!(chapter_map.navigation, map_input.navigation);
    assert!(chapter_map.diagnostics.is_empty());
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "nt-epub-source-mapping-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_synced(path: &Path, contents: &[u8]) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(contents).unwrap();
    file.sync_all().unwrap();
}

fn nested_epub(span_count: usize) -> Vec<u8> {
    let nested = format!(
        "{}x{}",
        "<span>".repeat(span_count),
        "</span>".repeat(span_count)
    );
    let chapter = format!(
        r#"<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body><p id="deep">{nested}</p></body></html>"#
    );
    epub3(&chapter)
}

fn deep_epub() -> Vec<u8> {
    nested_epub(300)
}

#[test]
fn accepts_the_maximum_xhtml_depth_and_returns_an_error_one_level_above() {
    let at_limit = extract(&nested_epub(253));
    assert_eq!(
        grapheme_text(block(
            document(&at_limit, "OPS/text/chapter.xhtml"),
            &BlockLocator::UniqueXhtmlId("deep".to_owned())
        )),
        "x"
    );

    let bytes = nested_epub(254);
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert!(matches!(
        extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION),
        Err(nt_epub::SourceExtractionError::XmlDepthLimit { max_depth: 256, .. })
    ));
}
#[test]
fn deep_xhtml_child() {
    let Ok(started_path) = std::env::var("NT_EPUB_SOURCE_MAPPING_CHILD") else {
        return;
    };
    let started_path = PathBuf::from(started_path);
    write_synced(&started_path, b"entered");
    let outcome_path = started_path.with_file_name("outcome");
    let bytes = deep_epub();
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    let result = extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION);
    match result {
        Err(nt_epub::SourceExtractionError::XmlDepthLimit { max_depth: 256, .. }) => {
            write_synced(&outcome_path, b"bounded-error");
        }
        Err(_) => write_synced(&outcome_path, b"unexpected-error"),
        Ok(_) => write_synced(&outcome_path, b"unexpected-success"),
    }
}

#[test]
fn deep_xhtml_returns_a_bounded_error_in_a_synchronized_child_process() {
    let temp = TempDir::new();
    let started_path = temp.path().join("started");
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "deep_xhtml_child", "--nocapture"])
        .env("NT_EPUB_SOURCE_MAPPING_CHILD", &started_path)
        .output()
        .unwrap();
    assert!(
        started_path.exists(),
        "child did not reach the marker before extraction; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "child exited without a bounded result: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(temp.path().join("outcome")).unwrap(),
        b"bounded-error"
    );
}

fn nav_to(href: &str, label: &str) -> String {
    format!(
        r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="{href}">{label}</a></li></ol></nav></body></html>"#
    )
}

fn epub2_with_named_anchor(chapter: &str) -> Vec<u8> {
    let ncx = EPUB2_NCX.replace("text/one.xhtml#first", "text/one.xhtml#entry");
    zip(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OPS/book.opf", EPUB2_PACKAGE.as_bytes()),
        ("OPS/toc.ncx", ncx.as_bytes()),
        ("OPS/text/one.xhtml", chapter.as_bytes()),
        ("OPS/text/two.xhtml", EPUB2_TWO.as_bytes()),
        ("OPS/appendix.xhtml", EPUB2_APPENDIX.as_bytes()),
    ])
}

#[test]
fn malformed_optional_nav_keeps_spine_and_located_provisional_diagnostic() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><h1>Chapter</h1><p>Readable.</p></body></html>"#;
    let broken_nav = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><nav><ol></body></html>"#;
    let bytes = epub3_with_nav(chapter, broken_nav);
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert!(inventory.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::NavigationInvalidXml
            && diagnostic.resource_path.as_deref() == Some("OPS/nav.xhtml")
            && diagnostic.line.is_some()
    }));
    let extraction = extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION).unwrap();
    assert!(!extraction.complete);
    assert!(
        document(&extraction, "OPS/text/chapter.xhtml")
            .blocks
            .iter()
            .any(|block| grapheme_text(block) == "Readable.")
    );
    let map =
        nt_application::chapter_map::build_candidate_map(&extraction.chapter_map_input(Vec::new()))
            .unwrap();
    assert!(map.diagnostics.iter().any(|diagnostic| matches!(diagnostic,
        ChapterMapDiagnostic::InputDiagnostic(item)
        if item.resource_path.as_deref() == Some("OPS/nav.xhtml")
            && item.code == "NavigationInvalidXml" && item.line.is_some())));
    assert!(map.volume.main_flow.iter().any(|entry| matches!(entry,
        MainFlowContent::Chapter(chapter)
        if chapter.label.as_deref() == Some("Chapter") && chapter.status == MappingStatus::Provisional)));
}

#[test]
fn invalid_optional_nav_roots_and_denied_entities_keep_readable_spine() {
    let cases = [
        (
            r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
            "NavigationInvalidXhtmlRoot",
        ),
        (
            r#"<!DOCTYPE html [<!ENTITY ext SYSTEM "file:///not-read">]><html xmlns="http://www.w3.org/1999/xhtml"><body>&ext;</body></html>"#,
            "NavigationXmlUnavailable",
        ),
    ];
    for (nav, expected_code) in cases {
        let extraction = extract(&epub3_with_nav(EPUB3_CHAPTER, nav));
        assert!(!extraction.complete);
        assert!(
            document(&extraction, "OPS/text/chapter.xhtml")
                .blocks
                .iter()
                .any(|block| grapheme_text(block) == "Body")
        );
        assert!(extraction.navigation_diagnostics.iter().any(|diagnostic| {
            diagnostic.resource_path.as_deref() == Some("OPS/nav.xhtml")
                && diagnostic.code == expected_code
        }));
    }
}

#[test]
fn optional_nav_degradation_does_not_hide_required_xml_or_crc_failures() {
    let malformed = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p></body></html>"#;
    let valid_nav = nav_to("text/chapter.xhtml#one", "One");
    let bytes = epub3_with_nav(malformed, &valid_nav);
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert!(
        matches!(extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION),
        Err(SourceExtractionError::Xml { canonical_path }) if canonical_path == "OPS/text/chapter.xhtml")
    );

    let package = EPUB3_PACKAGE.replace("<spine>", "<spine><itemref idref=\"nav\"/>");
    let spine_nav = zip(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OPS/book.opf", package.as_bytes()),
        ("OPS/nav.xhtml", malformed.as_bytes()),
        ("OPS/text/chapter.xhtml", EPUB3_CHAPTER.as_bytes()),
        ("OPS/images/page.png", b"page-image"),
        ("OPS/images/inline.png", b"inline-image"),
        ("OPS/text/end.xhtml", EPUB3_END.as_bytes()),
    ]);
    let mut archive = EpubZip::new(Cursor::new(spine_nav)).unwrap();
    let inventory = inspect(&mut archive);
    assert!(
        matches!(extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION),
        Err(SourceExtractionError::Xml { canonical_path }) if canonical_path == "OPS/nav.xhtml")
    );

    let original = epub3_with_nav(EPUB3_CHAPTER, &valid_nav);
    let mut good_archive = EpubZip::new(Cursor::new(original.clone())).unwrap();
    let good_inventory = inspect(&mut good_archive);
    let mut corrupted = original;
    let nav_offset = corrupted
        .windows(valid_nav.len())
        .position(|window| window == valid_nav.as_bytes())
        .unwrap();
    corrupted[nav_offset + 2] ^= 1;
    let mut bad_archive = EpubZip::new(Cursor::new(corrupted)).unwrap();
    assert!(matches!(
        extract_source(&mut bad_archive, &good_inventory, [7; 32], PARSER_VERSION),
        Err(SourceExtractionError::Archive(_))
    ));
}

#[test]
fn optional_nav_xml_byte_limit_is_still_an_error() {
    let oversized = format!(
        "<html xmlns=\"http://www.w3.org/1999/xhtml\"><body>{}</body></html>",
        " ".repeat(16 * 1024 * 1024)
    );
    let bytes = epub3_with_nav(EPUB3_CHAPTER, &oversized);
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert!(
        matches!(extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION),
        Err(SourceExtractionError::Xml { canonical_path }) if canonical_path == "OPS/nav.xhtml")
    );
}

#[test]
fn navigation_reread_does_not_hide_the_archive_total_limit() {
    let package = r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="3.0" unique-identifier="uid"><metadata><dc:identifier id="uid">limit</dc:identifier><dc:title>Limit</dc:title><dc:language>ja</dc:language></metadata><manifest><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/></manifest><spine><itemref idref="chapter"/></spine></package>"#;
    let chapter =
        r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="one">One.</p></body></html>"#;
    let nav = nav_to("chapter.xhtml#one", "One");
    let bytes = zip(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OPS/book.opf", package.as_bytes()),
        ("OPS/chapter.xhtml", chapter.as_bytes()),
        ("OPS/nav.xhtml", nav.as_bytes()),
    ]);
    let mut unrestricted = EpubZip::new(Cursor::new(bytes.clone())).unwrap();
    let inventory = inspect(&mut unrestricted);
    let limits = ArchiveLimits {
        max_total_bytes: (chapter.len() + nav.len()) as u64,
        ..ArchiveLimits::default()
    };
    let mut constrained = EpubZip::with_limits(Cursor::new(bytes), limits).unwrap();
    assert!(matches!(
        extract_source(&mut constrained, &inventory, [7; 32], PARSER_VERSION),
        Err(SourceExtractionError::Archive(ArchiveError::TotalDeclaredTooLarge { entry, .. }))
            if entry == "OPS/nav.xhtml"
    ));
}

#[test]
fn legacy_name_anchors_match_inventory_collisions_and_exact_offsets() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="paragraph">A<a id="same" name="same"></a>B<a name="entry"></a>C<a name="collision"></a><span id="collision"></span>D<svg xmlns="http://www.w3.org/2000/svg"><a name="foreign"></a></svg></p></body></html>"#;
    let bytes = epub2_with_named_anchor(chapter);
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    assert!(!inventory.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.code,
        DiagnosticCode::NavigationTargetMissing | DiagnosticCode::NavigationTargetAmbiguous
    )));
    let extraction = extract_source(&mut archive, &inventory, [7; 32], PARSER_VERSION).unwrap();
    let paragraph = block(
        document(&extraction, "OPS/text/one.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(grapheme_text(paragraph), "ABCD");
    for (fragment, offset, path) in [("same", 1, vec![0]), ("entry", 2, vec![1])] {
        let anchor = extraction
            .anchors
            .iter()
            .find(|anchor| anchor.fragment == fragment)
            .unwrap();
        assert_eq!(
            (anchor.block_id, anchor.unit_offset),
            (paragraph.id, offset)
        );
        assert_eq!(anchor.locator.block, paragraph.locator);
        assert_eq!(anchor.locator.element_ordinals, path);
        assert!(anchor.is_unique);
    }
    assert!(
        !extraction
            .anchors
            .iter()
            .any(|anchor| matches!(anchor.fragment.as_str(), "collision" | "foreign"))
    );
    let map =
        nt_application::chapter_map::build_candidate_map(&extraction.chapter_map_input(Vec::new()))
            .unwrap();
    assert!(map.boundary_evidence.iter().any(|evidence| evidence.kind
        == BoundaryEvidenceKind::Navigation
        && evidence.source == Some(NavigationSource::Epub2Ncx)
        && evidence.block_id == Some(paragraph.id)
        && evidence.unit_offset == Some(2)));
}

#[test]
fn document_element_and_empty_fragments_point_to_document_start() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml" id="book-start"><head><title>Not source</title></head><body><p id="paragraph">First.</p></body></html>"#;
    for href in [
        "text/chapter.xhtml",
        "text/chapter.xhtml#",
        "text/chapter.xhtml#book-start",
    ] {
        let extraction = extract(&epub3_with_nav(chapter, &nav_to(href, "Start")));
        let paragraph = block(
            document(&extraction, "OPS/text/chapter.xhtml"),
            &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
        );
        let root = extraction
            .anchors
            .iter()
            .find(|anchor| anchor.fragment == "book-start")
            .unwrap();
        assert_eq!((root.block_id, root.unit_offset), (paragraph.id, 0));
        assert_eq!(
            root.locator.block,
            BlockLocator::NodePath {
                element_ordinals: vec![],
                order: 0
            }
        );
        assert!(root.locator.element_ordinals.is_empty());
        let map = nt_application::chapter_map::build_candidate_map(
            &extraction.chapter_map_input(Vec::new()),
        )
        .unwrap();
        assert!(map.boundary_evidence.iter().any(|item| item.kind
            == BoundaryEvidenceKind::Navigation
            && item.raw_href.as_deref() == Some(href)
            && item.block_id == Some(paragraph.id)
            && item.unit_offset == Some(0)));
    }
}

#[test]
fn paired_navigation_keeps_both_sources_and_conflicting_heading() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><h1 id="heading">Chapter One</h1><p id="later">Story.</p><section><h2>First part</h2><p>One.</p></section><section><h2>Second part</h2><p>Two.</p></section></body></html>"#;
    let nav = nav_to("text/chapter.xhtml#later", "Chapter One");
    let package = EPUB3_PACKAGE
        .replace(
            "</manifest>",
            r#"<item id="toc" href="toc.ncx" media-type="application/x-dtbncx+xml"/></manifest>"#,
        )
        .replace("<spine>", "<spine toc=\"toc\">");
    let ncx = r#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap><navPoint id="n1"><navLabel><text>Chapter One</text></navLabel><content src="text/chapter.xhtml#later"/></navPoint></navMap></ncx>"#;
    let paired = zip(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OPS/book.opf", package.as_bytes()),
        ("OPS/nav.xhtml", nav.as_bytes()),
        ("OPS/toc.ncx", ncx.as_bytes()),
        ("OPS/text/chapter.xhtml", chapter.as_bytes()),
        ("OPS/images/page.png", b"page-image"),
        ("OPS/images/inline.png", b"inline-image"),
        ("OPS/text/end.xhtml", EPUB3_END.as_bytes()),
    ]);
    for (bytes, expected_sources) in [(epub3_with_nav(chapter, &nav), 1), (paired, 2)] {
        let extraction = extract(&bytes);
        let heading = block(
            document(&extraction, "OPS/text/chapter.xhtml"),
            &BlockLocator::UniqueXhtmlId("heading".to_owned()),
        );
        let map = nt_application::chapter_map::build_candidate_map(
            &extraction.chapter_map_input(Vec::new()),
        )
        .unwrap();
        assert_eq!(map.navigation.len(), expected_sources);
        assert!(map.diagnostics.iter().any(|item| matches!(item,
            ChapterMapDiagnostic::NavigationHeadingConflict { from_path: Some(path), .. } if path == "OPS/text/chapter.xhtml")));
        assert!(map.volume.main_flow.iter().any(|entry| matches!(entry,
            MainFlowContent::Unclassified { ranges, evidence, .. }
            if ranges.iter().any(|range| range.block_id == heading.id && range.start == 0 && range.end == 11)
                && evidence.iter().any(|item| item.kind == BoundaryEvidenceKind::Heading && item.block_id == Some(heading.id))
                && evidence.iter().filter(|item| item.kind == BoundaryEvidenceKind::Navigation).count() == expected_sources)));
    }
}

#[test]
fn links_wrapping_blocks_keep_ancestor_locator_and_child_ranges() {
    let chapter = r##"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><div id="wrapper"><a id="outer" href="#ending">A<p id="linked">e<em>&#x301;</em><a epub:type="noteref" href="#note">注</a><ruby><rb>漢</rb><rt>かん</rt></ruby><br/>Z</p>Tail<p id="second">2</p></a><p id="ending">End</p></div></body></html>"##;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#linked", "Linked"),
    ));
    let doc = document(&extraction, "OPS/text/chapter.xhtml");
    let linked = block(doc, &BlockLocator::UniqueXhtmlId("linked".to_owned()));
    assert_eq!(linked.units.len(), 5);
    assert_eq!(linked.units[0].spans.len(), 2);
    assert!(matches!(
        linked.units[2].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(_))
    ));
    assert!(matches!(
        linked.units[3].kind,
        SourceUnitKind::Protected(ProtectedSourceUnit::LineBreak { .. })
    ));
    let outer = linked
        .inline_metadata
        .iter()
        .find(|item| {
            matches!(&item.kind,
        InlineMetadataKind::Link { href: Some(href) } if href == "#ending")
        })
        .unwrap();
    assert_eq!(
        outer.locator.block,
        BlockLocator::UniqueXhtmlId("wrapper".to_owned())
    );
    assert_eq!(outer.locator.element_ordinals, vec![0]);
    assert_eq!(outer.ranges, vec![UnitRange { start: 0, end: 5 }]);
    let note = linked
        .inline_metadata
        .iter()
        .find(|item| {
            matches!(&item.kind,
        InlineMetadataKind::NoteReference { href: Some(href) } if href == "#note")
        })
        .unwrap();
    assert_eq!(note.locator.block, linked.locator);
    assert_eq!(note.locator.element_ordinals, vec![1]);
    assert_eq!(note.ranges, vec![UnitRange { start: 1, end: 2 }]);
    let second = block(doc, &BlockLocator::UniqueXhtmlId("second".to_owned()));
    assert!(
        second
            .inline_metadata
            .iter()
            .any(|item| item.locator == outer.locator
                && item.ranges == vec![UnitRange { start: 0, end: 1 }])
    );
    assert_eq!(
        doc.blocks.iter().map(grapheme_text).collect::<String>(),
        "Ae\u{301}注ZTail2End"
    );
}

#[test]
fn nested_block_links_can_use_a_non_emitted_structural_ancestor() {
    let chapter = r##"<html xmlns="http://www.w3.org/1999/xhtml"><body><div><a href="#end"><section><p id="linked">Read</p></section></a><p id="end">End</p></div></body></html>"##;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#linked", "Linked"),
    ));
    let doc = document(&extraction, "OPS/text/chapter.xhtml");
    let linked = block(doc, &BlockLocator::UniqueXhtmlId("linked".to_owned()));
    let link = linked
        .inline_metadata
        .iter()
        .find(|item| matches!(&item.kind, InlineMetadataKind::Link { href: Some(href) } if href == "#end"))
        .unwrap();
    assert_eq!(
        link.locator.block,
        BlockLocator::NodePath {
            element_ordinals: vec![0, 0],
            order: 0,
        }
    );
    assert_eq!(link.locator.element_ordinals, vec![0]);
    assert_eq!(link.ranges, vec![UnitRange { start: 0, end: 4 }]);
    assert!(
        !doc.blocks
            .iter()
            .any(|block| block.locator == link.locator.block)
    );
}

#[test]
fn parser_namespace_changes_ids_without_changing_unaffected_source_content() {
    assert_eq!(PARSER_VERSION, "nt-epub/1.6.0");
    let bytes = epub3(EPUB3_CHAPTER);
    let mut old_archive = EpubZip::new(Cursor::new(bytes.clone())).unwrap();
    let old_inventory = inspect(&mut old_archive);
    let old = extract_source(&mut old_archive, &old_inventory, [7; 32], "nt-epub/1.3.0").unwrap();
    let mut previous_archive = EpubZip::new(Cursor::new(bytes.clone())).unwrap();
    let previous_inventory = inspect(&mut previous_archive);
    let previous = extract_source(
        &mut previous_archive,
        &previous_inventory,
        [7; 32],
        "nt-epub/1.4.0",
    )
    .unwrap();
    let new = extract(&bytes);
    let locator = BlockLocator::UniqueXhtmlId("target".to_owned());
    let old_block = block(document(&old, "OPS/text/chapter.xhtml"), &locator);
    let new_block = block(document(&new, "OPS/text/chapter.xhtml"), &locator);
    assert_eq!(
        old_block.id.to_string(),
        "d170eb030a902dcfd2f1e03c94e3c16d7ae368e3faaa318617083db7d6262045"
    );
    assert_eq!(
        block(document(&previous, "OPS/text/chapter.xhtml"), &locator)
            .id
            .to_string(),
        "b6e6efff9dfb563eb8d678287645a4525586e749b01682331b34bb5ec4041e86"
    );
    assert_eq!(
        new_block.id.to_string(),
        "21f704cc24bf345a206293a7ac7ef954f51059d310169a1e9b6d1740179f4d04"
    );
    assert_eq!(old_block.locator, new_block.locator);
    assert_eq!(old_block.units, new_block.units);
    assert_eq!(old_block.content_hash, new_block.content_hash);
}

#[test]
fn ruby_descendant_anchors_and_grapheme_spans_stay_atomic() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="paragraph">e<em>&#x301;</em><ruby id="ruby"><rb><span id="rb">漢</span></rb><rt id="rt">かん</rt><rp id="rp">)</rp></ruby>Z</p></body></html>"#;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#rt", "Ruby"),
    ));
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(paragraph.units.len(), 3);
    assert_eq!(paragraph.units[0].spans.len(), 2);
    let SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(ruby)) = &paragraph.units[1].kind
    else {
        panic!("expected ruby")
    };
    assert_eq!((&*ruby.base, &*ruby.annotation), ("漢", "かん"));
    assert_eq!(ruby.base_spans[0].node().text_node_order, 2);
    assert_eq!(ruby.annotation_spans[0].node().text_node_order, 3);
    for (fragment, path) in [
        ("ruby", vec![1]),
        ("rb", vec![1, 0, 0]),
        ("rt", vec![1, 1]),
        ("rp", vec![1, 2]),
    ] {
        let anchor = extraction
            .anchors
            .iter()
            .find(|anchor| anchor.fragment == fragment)
            .unwrap();
        assert_eq!((anchor.block_id, anchor.unit_offset), (paragraph.id, 1));
        assert_eq!(anchor.locator.element_ordinals, path);
    }
    let map =
        nt_application::chapter_map::build_candidate_map(&extraction.chapter_map_input(Vec::new()))
            .unwrap();
    assert!(
        map.boundary_evidence
            .iter()
            .any(|item| item.kind == BoundaryEvidenceKind::Navigation
                && item.block_id == Some(paragraph.id)
                && item.unit_offset == Some(1))
    );
}

#[test]
fn nested_ruby_links_and_notes_share_atomic_range_with_outer_link() {
    let chapter = r##"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><p id="paragraph">A<a href="#outer"><ruby><a href="#tail">漢</a><rt><a epub:type="noteref" href="#note">かん</a></rt></ruby></a>Z</p><p id="tail">End.</p><aside id="note">Note.</aside></body></html>"##;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#paragraph", "Chapter"),
    ));
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(paragraph.units.len(), 3);
    let SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(ruby)) = &paragraph.units[1].kind
    else {
        panic!("expected ruby")
    };
    assert_eq!((&*ruby.base, &*ruby.annotation), ("漢", "かん"));
    assert_eq!(ruby.base_spans[0].node().text_node_order, 1);
    assert_eq!(ruby.annotation_spans[0].node().text_node_order, 2);
    for (kind, path) in [
        (
            InlineMetadataKind::Link {
                href: Some("#outer".to_owned()),
            },
            vec![0],
        ),
        (
            InlineMetadataKind::Link {
                href: Some("#tail".to_owned()),
            },
            vec![0, 0, 0],
        ),
        (
            InlineMetadataKind::NoteReference {
                href: Some("#note".to_owned()),
            },
            vec![0, 0, 1, 0],
        ),
    ] {
        assert!(
            paragraph
                .inline_metadata
                .iter()
                .any(|item| item.kind == kind
                    && item.locator.block == paragraph.locator
                    && item.locator.element_ordinals == path
                    && item.ranges == [UnitRange { start: 1, end: 2 }])
        );
    }
}

#[test]
fn ruby_formatting_links_and_notes_keep_exact_atomic_metadata() {
    let chapter = r##"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><p id="paragraph">A<strong id="outer"><ruby id="ruby"><b id="base"><em><i><s><span><a href="#tail">漢</a></span></s></i></em></b><rt><small id="anno"><sub><sup><u><a epub:type="noteref" href="#note">かん</a></u></sup></sub></small></rt></ruby></strong>Z</p><p id="tail">End.</p><aside id="note">Note.</aside></body></html>"##;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#paragraph", "Chapter"),
    ));
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(paragraph.units.len(), 3);
    assert!(matches!(&paragraph.units[0].kind, SourceUnitKind::Grapheme(text) if text == "A"));
    assert!(matches!(&paragraph.units[2].kind, SourceUnitKind::Grapheme(text) if text == "Z"));
    let SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(ruby)) = &paragraph.units[1].kind
    else {
        panic!("expected one atomic ruby");
    };
    assert_eq!((&*ruby.base, &*ruby.annotation), ("漢", "かん"));
    assert_eq!(ruby.node_path, [0, 0]);
    let spans = paragraph
        .units
        .iter()
        .map(|unit| {
            unit.spans
                .iter()
                .map(|span| {
                    assert_eq!(span.node().block, paragraph.locator);
                    (
                        span.node().text_node_order,
                        span.start_byte(),
                        span.end_byte(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        spans,
        [vec![(0, 0, 1)], vec![(1, 0, 3), (2, 0, 6)], vec![(3, 0, 1)]]
    );
    assert_eq!(ruby.base_spans, paragraph.units[1].spans[..1]);
    assert_eq!(ruby.annotation_spans, paragraph.units[1].spans[1..]);

    let format = |element: &str| InlineMetadataKind::Formatting {
        element: element.to_owned(),
    };
    let expected = [
        (format("strong"), vec![0]),
        (format("b"), vec![0, 0, 0]),
        (format("em"), vec![0, 0, 0, 0]),
        (format("i"), vec![0, 0, 0, 0, 0]),
        (format("s"), vec![0, 0, 0, 0, 0, 0]),
        (format("span"), vec![0, 0, 0, 0, 0, 0, 0]),
        (
            InlineMetadataKind::Link {
                href: Some("#tail".to_owned()),
            },
            vec![0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (format("small"), vec![0, 0, 1, 0]),
        (format("sub"), vec![0, 0, 1, 0, 0]),
        (format("sup"), vec![0, 0, 1, 0, 0, 0]),
        (format("u"), vec![0, 0, 1, 0, 0, 0, 0]),
        (
            InlineMetadataKind::NoteReference {
                href: Some("#note".to_owned()),
            },
            vec![0, 0, 1, 0, 0, 0, 0, 0],
        ),
    ];
    assert_eq!(paragraph.inline_metadata.len(), expected.len());
    for (actual, (kind, path)) in paragraph.inline_metadata.iter().zip(expected) {
        assert_eq!(actual.kind, kind);
        assert_eq!(actual.locator.block, paragraph.locator);
        assert_eq!(actual.locator.element_ordinals, path);
        assert_eq!(actual.ranges, [UnitRange { start: 1, end: 2 }]);
    }
    for (fragment, path) in [
        ("outer", vec![0]),
        ("ruby", vec![0, 0]),
        ("base", vec![0, 0, 0]),
        ("anno", vec![0, 0, 1, 0]),
    ] {
        let anchor = extraction
            .anchors
            .iter()
            .find(|item| item.fragment == fragment)
            .unwrap();
        assert_eq!((anchor.block_id, anchor.unit_offset), (paragraph.id, 1));
        assert_eq!(anchor.locator.block, paragraph.locator);
        assert_eq!(anchor.locator.element_ordinals, path);
    }
    assert_eq!(
        paragraph.id.to_string(),
        "3133144974bd21e8bda37de09d28a128e8d8db48932b97e4a362f7f47b779185"
    );
    assert_eq!(
        paragraph
            .content_hash
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "a5ff3f7c5638f2e6bbb68ccba05106ab7d35ad83193df3c67c0c39968537e332"
    );
}

#[test]
fn fallback_locator_order_is_local_to_each_element_path() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body><div>A<p>B</p>C</div><p>D</p></body></html>"#;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml", "Chapter"),
    ));
    let blocks = &document(&extraction, "OPS/text/chapter.xhtml").blocks;
    assert_eq!(
        blocks.iter().map(grapheme_text).collect::<Vec<_>>(),
        ["A", "B", "C", "D"]
    );
    let expected = [
        (
            vec![1, 0],
            0,
            "b95bdae70f7290d7564957bbbdcba9422e78b480ea780a75d740176b67ef6c54",
        ),
        (
            vec![1, 0, 0],
            0,
            "83ce76277fb8fd15aac74bd6ea2525ebe880b716aad4842c904dfb0cf36d1859",
        ),
        (
            vec![1, 0],
            1,
            "75a41ae824a524d9310492bfd4a4b87dc6cc5b127ae5b6338bfec018aa759fb3",
        ),
        (
            vec![1, 1],
            0,
            "9ebf3a41d8280b44ceeaec46ea10d82a0524653bcb0a655488bbd793a1067790",
        ),
    ];
    for (block, (path, order, hash)) in blocks.iter().zip(expected) {
        assert_eq!(
            block.locator,
            BlockLocator::NodePath {
                element_ordinals: path,
                order
            }
        );
        assert_eq!(block.id.to_string(), hash);
    }
}

#[test]
fn historical_fallback_identity_vectors_remain_fixed() {
    for (order, expected) in [
        (
            0,
            "85dcb625ec03ea2a3d3be55c24c27fe3eb3ee6527f9f0633264d8e7e790a4cee",
        ),
        (
            1,
            "c3b27904f31d587254fc7290374cba10efb54529e0409e204ed12273bc0e043f",
        ),
    ] {
        let record = BlockIdentityRecord {
            epub_sha256: [7; 32],
            parser_version: "nt-epub/1.4.0".to_owned(),
            canonical_path: "OPS/chapter.xhtml".to_owned(),
            locator: BlockLocator::NodePath {
                element_ordinals: vec![1, 0, 0],
                order,
            },
        };
        let actual = Sha256::digest(record.canonical_bytes().unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(actual, expected);
    }
}

#[test]
fn fallback_still_promotes_only_the_preferred_heading_level() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><h1>Volume</h1><h2>One</h2><p>A</p><h2>Two</h2><p>B</p></body></html>"#;
    let malformed = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><nav><ol></body></html>"#;
    let extraction = extract(&epub3_with_nav(chapter, malformed));
    let map =
        nt_application::chapter_map::build_candidate_map(&extraction.chapter_map_input(Vec::new()))
            .unwrap();
    let labels = map
        .volume
        .main_flow
        .iter()
        .filter_map(|item| match item {
            MainFlowContent::Chapter(chapter) => chapter.label.as_deref(),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(labels, ["One", "Two"]);
}

#[test]
fn cdata_boundaries_keep_local_offsets_without_breaking_graphemes() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="simple">a<![CDATA[b]]>c</p><p id="combined">e<![CDATA[́]]>x</p><p id="literal">a&amp;<![CDATA[<b>&amp;]]>&amp;z</p></body></html>"#;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#simple", "Chapter"),
    ));
    let doc = document(&extraction, "OPS/text/chapter.xhtml");
    let simple = block(doc, &BlockLocator::UniqueXhtmlId("simple".to_owned()));
    assert_eq!(grapheme_text(simple), "abc");
    assert_eq!(
        simple
            .units
            .iter()
            .flat_map(|unit| &unit.spans)
            .map(|span| (
                span.node().text_node_order,
                span.start_byte(),
                span.end_byte()
            ))
            .collect::<Vec<_>>(),
        [(0, 0, 1), (1, 0, 1), (2, 0, 1)]
    );
    let combined = block(doc, &BlockLocator::UniqueXhtmlId("combined".to_owned()));
    assert_eq!(combined.units.len(), 2);
    assert_eq!(
        combined.units[0].kind,
        SourceUnitKind::Grapheme("e\u{301}".to_owned())
    );
    assert_eq!(
        combined.units[0]
            .spans
            .iter()
            .map(|span| (
                span.node().text_node_order,
                span.start_byte(),
                span.end_byte()
            ))
            .collect::<Vec<_>>(),
        [(0, 0, 1), (1, 0, 2)]
    );
    assert_eq!(
        (
            combined.units[1].spans[0].node().text_node_order,
            combined.units[1].spans[0].start_byte()
        ),
        (2, 0)
    );
    let literal = block(doc, &BlockLocator::UniqueXhtmlId("literal".to_owned()));
    assert_eq!(grapheme_text(literal), "a&<b>&amp;&z");
    assert_eq!(
        (
            literal.units[1].spans[0].node().text_node_order,
            literal.units[1].spans[0].start_byte()
        ),
        (0, 1)
    );
    assert_eq!(
        (
            literal.units[2].spans[0].node().text_node_order,
            literal.units[2].spans[0].start_byte()
        ),
        (1, 0)
    );
    assert_eq!(
        (
            literal.units[10].spans[0].node().text_node_order,
            literal.units[10].spans[0].start_byte()
        ),
        (2, 0)
    );
}

#[test]
fn cdata_boundaries_inside_inline_and_ruby_preserve_text_ordinals() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="paragraph">x<span>y<![CDATA[z]]>w</span><ruby>漢<![CDATA[字]]><rt>か<![CDATA[ん]]></rt></ruby>終</p></body></html>"#;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#paragraph", "Chapter"),
    ));
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(paragraph.units.len(), 6);
    assert_eq!(
        paragraph.units[..4]
            .iter()
            .map(|unit| unit.spans[0].node().text_node_order)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    let SourceUnitKind::Protected(ProtectedSourceUnit::Ruby(ruby)) = &paragraph.units[4].kind
    else {
        panic!("expected ruby")
    };
    assert_eq!((&*ruby.base, &*ruby.annotation), ("漢字", "かん"));
    assert_eq!(
        ruby.base_spans
            .iter()
            .map(|span| (
                span.node().text_node_order,
                span.start_byte(),
                span.end_byte()
            ))
            .collect::<Vec<_>>(),
        [(4, 0, 3), (5, 0, 3)]
    );
    assert_eq!(
        ruby.annotation_spans
            .iter()
            .map(|span| (
                span.node().text_node_order,
                span.start_byte(),
                span.end_byte()
            ))
            .collect::<Vec<_>>(),
        [(6, 0, 3), (7, 0, 3)]
    );
    assert_eq!(paragraph.units[5].spans[0].node().text_node_order, 8);
}

#[test]
fn cdata_in_used_internal_entities_is_split_and_unused_literals_are_ignored() {
    let chapter = r#"<!DOCTYPE html [<!ENTITY piece "b<![CDATA[c]]>d"><!ENTITY outer "&piece;"><!ENTITY unused "<![CDATA[unclosed">]><html xmlns="http://www.w3.org/1999/xhtml"><body><!-- <![CDATA[ignored]]> --><?check keep?><p id="paragraph">a&outer;&outer;z</p></body></html>"#;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#paragraph", "Chapter"),
    ));
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(grapheme_text(paragraph), "abcd bcdz".replace(' ', ""));
    assert_eq!(
        paragraph
            .units
            .iter()
            .map(|unit| (
                unit.spans[0].node().text_node_order,
                unit.spans[0].start_byte()
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0),
            (0, 1),
            (1, 0),
            (2, 0),
            (2, 1),
            (3, 0),
            (4, 0),
            (4, 1)
        ]
    );
}

#[test]
fn empty_and_adjacent_cdata_and_original_comments_keep_node_order() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p id="paragraph">a<!----><![CDATA[b]]><![CDATA[]]><![CDATA[c]]>d</p></body></html>"#;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#paragraph", "Chapter"),
    ));
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(grapheme_text(paragraph), "abcd");
    assert_eq!(
        paragraph
            .units
            .iter()
            .map(|unit| (
                unit.spans[0].node().text_node_order,
                unit.spans[0].start_byte()
            ))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 0), (3, 0), (4, 0)]
    );
}

#[test]
fn cdata_in_entity_generated_markup_uses_original_element_path() {
    let chapter = r#"<!DOCTYPE html [<!ENTITY piece '<span>A<![CDATA[B]]>C</span>'>]><html xmlns="http://www.w3.org/1999/xhtml"><body><p id="paragraph">H&piece;Z</p></body></html>"#;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#paragraph", "Chapter"),
    ));
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(grapheme_text(paragraph), "HABCZ");
    assert_eq!(
        paragraph
            .units
            .iter()
            .map(|unit| (
                unit.spans[0].node().text_node_order,
                unit.spans[0].start_byte()
            ))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
    );
    let format = paragraph.inline_metadata.iter().find(|item| matches!(&item.kind, InlineMetadataKind::Formatting { element } if element == "span")).unwrap();
    assert_eq!(format.locator.block, paragraph.locator);
    assert_eq!(format.locator.element_ordinals, [0]);
    assert_eq!(format.ranges, [UnitRange { start: 1, end: 4 }]);
}

#[test]
fn utf16_document_with_cdata_uses_decoded_utf8_source_offsets() {
    let chapter = r#"<?xml version="1.0" encoding="UTF-16"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p id="paragraph">日<![CDATA[本]]>語</p></body></html>"#;
    let mut encoded = vec![0xff, 0xfe];
    for unit in chapter.encode_utf16() {
        encoded.extend_from_slice(&unit.to_le_bytes());
    }
    let nav = nav_to("text/chapter.xhtml#paragraph", "Chapter");
    let bytes = zip(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", CONTAINER.as_bytes()),
        ("OPS/book.opf", EPUB3_PACKAGE.as_bytes()),
        ("OPS/nav.xhtml", nav.as_bytes()),
        ("OPS/text/chapter.xhtml", &encoded),
        ("OPS/images/page.png", b"page-image"),
        ("OPS/images/inline.png", b"inline-image"),
        ("OPS/text/end.xhtml", EPUB3_END.as_bytes()),
    ]);
    let extraction = extract(&bytes);
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(grapheme_text(paragraph), "日本語");
    assert_eq!(
        paragraph
            .units
            .iter()
            .map(|unit| (
                unit.spans[0].node().text_node_order,
                unit.spans[0].start_byte(),
                unit.spans[0].end_byte()
            ))
            .collect::<Vec<_>>(),
        [(0, 0, 3), (1, 0, 3), (2, 0, 3)]
    );
}

#[test]
fn many_adjacent_cdata_sections_keep_all_source_positions() {
    let chapter = format!(
        "<html xmlns=\"http://www.w3.org/1999/xhtml\"><body><p id=\"paragraph\">{}</p></body></html>",
        "<![CDATA[x]]>".repeat(4096)
    );
    let extraction = extract(&epub3_with_nav(
        &chapter,
        &nav_to("text/chapter.xhtml#paragraph", "Chapter"),
    ));
    let paragraph = block(
        document(&extraction, "OPS/text/chapter.xhtml"),
        &BlockLocator::UniqueXhtmlId("paragraph".to_owned()),
    );
    assert_eq!(paragraph.units.len(), 4096);
    for (index, unit) in paragraph.units.iter().enumerate() {
        assert_eq!(unit.kind, SourceUnitKind::Grapheme("x".to_owned()));
        assert_eq!(
            (
                unit.spans[0].node().text_node_order,
                unit.spans[0].start_byte(),
                unit.spans[0].end_byte()
            ),
            (index as u32, 0, 1)
        );
    }
}

#[test]
fn skipped_nested_and_hidden_cdata_still_count_in_parent_text_ordinals() {
    let chapter = r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><div id="outer">A<p>X<![CDATA[Y]]>Z</p>C</div><div id="hidden">A<script>x<![CDATA[y]]>z</script>C</div></body></html>"#;
    let extraction = extract(&epub3_with_nav(
        chapter,
        &nav_to("text/chapter.xhtml#outer", "Chapter"),
    ));
    let doc = document(&extraction, "OPS/text/chapter.xhtml");
    let parent_tail = doc
        .blocks
        .iter()
        .find(|item| {
            grapheme_text(item) == "C"
                && matches!(&item.locator,
        BlockLocator::NodePath { element_ordinals, order: 1 } if element_ordinals == &[0, 0])
        })
        .unwrap();
    assert_eq!(parent_tail.units[0].spans[0].node().text_node_order, 4);
    let hidden = block(doc, &BlockLocator::UniqueXhtmlId("hidden".to_owned()));
    assert_eq!(grapheme_text(hidden), "AC");
    assert_eq!(hidden.units[1].spans[0].node().text_node_order, 4);
}

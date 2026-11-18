use nt_domain::source_range::{SourceBlock, SourceUnitKind};
use nt_domain::volume_import::{CapturedUnitRun, MAX_IMPORT_WORKING_BYTES};

use super::super::{PARSER_VERSION, UnicodeSegmentation, parse_xhtml};
use super::*;

const PATH: &str = "OPS/Book/chapter.xhtml";

fn request(navigation: bool, budget: usize) -> CapturedSourceRequest<'static> {
    CapturedSourceRequest {
        canonical_path: PATH,
        epub_sha256: [0x31; 32],
        parser_version: PARSER_VERSION,
        is_navigation_document: navigation,
        working_bytes: budget,
    }
}

fn compare_block(actual: &CapturedBlock, original: &SourceBlock) {
    assert_eq!(actual.id, original.id);
    assert_eq!(actual.locator, original.locator);
    assert_eq!(actual.element_name, original.element_name);
    assert_eq!(actual.content_hash, original.content_hash);
    assert_eq!(actual.unit_count as usize, original.units.len());
    assert_eq!(actual.inline_metadata, original.inline_metadata);
    assert_eq!(actual.attribute_text, original.attribute_text);
    let mut units = original.units.iter();
    for run in &actual.runs {
        match run {
            CapturedUnitRun::Text { text, segments, .. } => {
                for (start, value) in text.grapheme_indices(true) {
                    let end = start + value.len();
                    let unit = units.next().unwrap();
                    assert_eq!(unit.kind, SourceUnitKind::Grapheme(value.to_owned()));
                    let mut spans = unit.spans.iter();
                    for &[from, to, node, node_start, _] in segments {
                        let overlap_start = start.max(from as usize);
                        let overlap_end = end.min(to as usize);
                        if overlap_start < overlap_end {
                            let span = spans.next().unwrap();
                            assert_eq!(span.node().block, actual.locator);
                            assert_eq!(span.node().text_node_order, node);
                            assert_eq!(span.start_byte(), node_start + overlap_start as u32 - from);
                            assert_eq!(span.end_byte(), node_start + overlap_end as u32 - from);
                        }
                    }
                    assert!(spans.next().is_none());
                }
            }
            CapturedUnitRun::Protected { unit, spans } => {
                let expected = units.next().unwrap();
                assert_eq!(expected.kind, SourceUnitKind::Protected(unit.clone()));
                assert_eq!(&expected.spans, spans);
            }
        }
    }
    assert!(units.next().is_none());
}

#[test]
fn compact_replay_preserves_original_lexical_spans_units_metadata_and_anchors() {
    const XHTML: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml" id="top">
<head><base href="../Images/"/></head><body>
<div><em>outer<p id="p"><em>A</em><span id="across">&#x301;</span>B<![CDATA[C]]>D
<ruby>本<rp>(</rp><rt>ほん</rt><rp>)</rp></ruby><br/><img src="a.png" alt="説明"/>after</p>continue</em></div>
<p id="same">One</p><p id="same">Two<hr/>Three</p>
</body></html>"#;
    let expected = parse_xhtml(XHTML.as_bytes(), PATH, [0x31; 32], PARSER_VERSION, false)
        .unwrap_or_else(|_| panic!("published source producer rejected fixture"));
    let mut blocks = expected.blocks.iter();
    let (anchors, diagnostics) = SourceExtraction::visit_captured_xhtml(
        XHTML.as_bytes(),
        request(false, MAX_IMPORT_WORKING_BYTES),
        |actual| {
            compare_block(actual, blocks.next().unwrap());
            Ok(())
        },
    )
    .unwrap();
    assert!(blocks.next().is_none());
    assert_eq!(anchors, expected.anchors);
    assert_eq!(diagnostics, expected.diagnostics);
    assert!(anchors.iter().any(|a| a.fragment == "across"));
}

#[test]
fn navigation_replay_excludes_only_the_toc_list() {
    const XHTML: &str = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><h1>Contents</h1><ol><li><a href="ch.xhtml">Chapter</a></li></ol><p>After</p></nav></body></html>"#;
    let expected = parse_xhtml(XHTML.as_bytes(), PATH, [0x31; 32], PARSER_VERSION, true)
        .unwrap_or_else(|_| panic!("published source producer rejected fixture"));
    let mut blocks = expected.blocks.iter();
    assert_eq!(blocks.len(), 2);
    SourceExtraction::visit_captured_xhtml(
        XHTML.as_bytes(),
        request(true, MAX_IMPORT_WORKING_BYTES),
        |actual| {
            compare_block(actual, blocks.next().unwrap());
            Ok(())
        },
    )
    .unwrap();
    assert!(blocks.next().is_none());
}

#[test]
fn raster_and_svg_image_page_replay_preserve_the_atomic_source() {
    let svg =
        br#"<svg xmlns="http://www.w3.org/2000/svg" id="root"><g><rect id="frame"/></g></svg>"#;
    let expected = super::super::parse_svg_image_page(svg, PATH, [0x31; 32], PARSER_VERSION, 3)
        .expect("valid SVG fixture");
    let anchors = SourceExtraction::visit_captured_image_page(
        Some(svg),
        request(false, MAX_IMPORT_WORKING_BYTES),
        3,
        |block| {
            compare_block(block, &expected.blocks[0]);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(anchors, expected.anchors);
    let raster = super::super::image_page_block(PATH, [0x31; 32], PARSER_VERSION, 6).unwrap();
    let anchors = SourceExtraction::visit_captured_image_page(
        None,
        request(false, MAX_IMPORT_WORKING_BYTES),
        6,
        |block| {
            compare_block(block, &raster);
            Ok(())
        },
    )
    .unwrap();
    assert!(anchors.is_empty());
}

#[test]
fn four_million_original_graphemes_use_one_run_and_one_lexical_projection() {
    const LENGTH: usize = 4_000_001;
    let mut xhtml = String::from("<html xmlns=\"http://www.w3.org/1999/xhtml\"><body><p>");
    xhtml.extend(std::iter::repeat_n('a', LENGTH));
    xhtml.push_str("</p></body></html>");
    let mut seen = 0;
    SourceExtraction::visit_captured_xhtml(
        xhtml.as_bytes(),
        request(false, 80 * 1024 * 1024),
        |block| {
            seen += 1;
            assert_eq!(block.unit_count as usize, LENGTH);
            let [CapturedUnitRun::Text { text, segments, .. }] = block.runs.as_slice() else {
                panic!("expected one compact text run");
            };
            assert_eq!(text.len(), LENGTH);
            assert_eq!(segments, &[[0, LENGTH as u32, 0, 0, LENGTH as u32]]);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(seen, 1);
    assert!(matches!(
        SourceExtraction::visit_captured_xhtml(
            xhtml.as_bytes(),
            request(false, 8 * 1024 * 1024),
            |_| { panic!("over-budget input reached the consumer") }
        ),
        Err(Error::ResourceLimit)
    ));
}

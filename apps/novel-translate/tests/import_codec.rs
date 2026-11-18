//! Compact-run byte and Unicode projection oracles. These leaf tests are not
//! an EPUB import, original-node validation, or a full V2 envelope decoder.
use novel_translate::import_codec::{CodecError, TextRun};
use nt_application::volume_import::source_range::{
    BlockLocator, SourceSpan, SourceUnit, TextNodeLocator,
};

fn locator() -> BlockLocator {
    BlockLocator::UniqueXhtmlId("p".into())
}
fn span(node: u32, start: usize, end: usize, text: &str) -> SourceSpan {
    SourceSpan::new(
        TextNodeLocator {
            block: locator(),
            text_node_order: node,
        },
        start,
        end,
        text,
    )
    .unwrap()
}
fn parts(text: &str, lengths: &str, segments: &[[u32; 5]]) -> TextRun {
    // The complete envelope decoder must bound JSON before this leaf DTO is
    // deserialized. Small adversarial literals here exercise the leaf contract.
    serde_json::from_value(serde_json::json!({
        "kind":"TextRun","text":text,"grapheme_lengths":lengths,"segments":segments
    }))
    .unwrap()
}

#[test]
fn signed_ascii_text_run_bytes_match_v2_golden() {
    let units = [SourceUnit::grapheme("A".into(), vec![span(0, 0, 1, "A")])];
    let run = TextRun::from_grapheme_units(&units, &locator()).unwrap();
    assert_eq!(
        serde_json::to_string(&run).unwrap(),
        r#"{"kind":"TextRun","text":"A","grapheme_lengths":"AQ==","segments":[[0,1,0,0,1]]}"#
    );
    assert_eq!(run.validate().unwrap().unit_count(), 1);
}

#[test]
fn canonical_run_reader_composes_lexical_typed_and_grapheme_validation() {
    use std::io::Cursor;
    let valid =
        br#"{"kind":"TextRun","text":"A","grapheme_lengths":"AQ==","segments":[[0,1,0,0,1]]}"#;
    let actual = TextRun::read_canonical_json(&mut Cursor::new(valid), valid.len() as u64).unwrap();
    assert_eq!(actual.validate().unwrap().units().next().unwrap().text, "A");
    let wrong_lengths =
        br#"{"kind":"TextRun","text":"A","grapheme_lengths":"Ag==","segments":[[0,1,0,0,1]]}"#;
    assert_eq!(
        TextRun::read_canonical_json(&mut Cursor::new(wrong_lengths), wrong_lengths.len() as u64),
        Err(CodecError::InvalidTextRun)
    );
    let wrong_order =
        br#"{"text":"A","kind":"TextRun","grapheme_lengths":"AQ==","segments":[[0,1,0,0,1]]}"#;
    assert_eq!(
        TextRun::read_canonical_json(&mut Cursor::new(wrong_order), wrong_order.len() as u64),
        Err(CodecError::NonCanonicalJson)
    );
    let unknown = br#"{"kind":"TextRun","text":"A","grapheme_lengths":"AQ==","segments":[[0,1,0,0,1]],"source_verified":true}"#;
    assert_eq!(
        TextRun::read_canonical_json(&mut Cursor::new(unknown), unknown.len() as u64),
        Err(CodecError::InvalidJson)
    );
}

#[test]
fn combining_and_emoji_contributors_preserve_exact_ordered_spans_lazily() {
    let text0 = "xxAe";
    let text1 = "xxxxx\u{301}";
    let text2 = "xxx👩🏽";
    let text3 = "\u{200d}";
    let text4 = "xxxx💻Z";
    let units = [
        SourceUnit::grapheme("A".into(), vec![span(0, 2, 3, text0)]),
        SourceUnit::grapheme(
            "e\u{301}".into(),
            vec![span(0, 3, 4, text0), span(1, 5, 7, text1)],
        ),
        SourceUnit::grapheme(
            "👩🏽‍💻".into(),
            vec![
                span(2, 3, 11, text2),
                span(3, 0, 3, text3),
                span(4, 4, 8, text4),
            ],
        ),
        SourceUnit::grapheme("Z".into(), vec![span(4, 8, 9, text4)]),
    ];
    let run = TextRun::from_grapheme_units(&units, &locator()).unwrap();
    assert_eq!(run.text(), "Ae\u{301}👩🏽‍💻Z");
    assert_eq!(run.grapheme_lengths(), "AQMPAQ==");
    assert_eq!(
        run.segments(),
        &[
            [0, 2, 0, 2, 4],
            [2, 4, 1, 5, 7],
            [4, 12, 2, 3, 11],
            [12, 15, 3, 0, 3],
            [15, 20, 4, 4, 9]
        ]
    );
    let validated = run.validate().unwrap();
    assert_eq!(validated.unit_count(), 4);
    let actual = validated
        .units()
        .map(|unit| {
            (
                unit.text.to_owned(),
                unit.spans()
                    .map(|s| (s.text_node_order, s.start_byte, s.end_byte))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            ("A".into(), vec![(0, 2, 3)]),
            ("e\u{301}".into(), vec![(0, 3, 4), (1, 5, 7)]),
            ("👩🏽‍💻".into(), vec![(2, 3, 11), (3, 0, 3), (4, 4, 8)]),
            ("Z".into(), vec![(4, 8, 9)]),
        ]
    );
}

#[test]
fn contributors_in_one_grapheme_are_never_coalesced() {
    let node = "e\u{301}";
    let unit = SourceUnit::grapheme(node.into(), vec![span(0, 0, 1, node), span(0, 1, 3, node)]);
    let run = TextRun::from_grapheme_units(&[unit], &locator()).unwrap();
    assert_eq!(run.segments(), &[[0, 1, 0, 0, 1], [1, 3, 0, 1, 3]]);
    assert_eq!(
        run.validate()
            .unwrap()
            .units()
            .next()
            .unwrap()
            .spans()
            .count(),
        2
    );
}

#[test]
fn malformed_and_noncanonical_base64_and_uleb_fail_without_unit_allocation() {
    for lengths in [
        "AR==", "gQA=", "AAE=", "gICAgBA=", "gA==", "AQ", "AQ==AAAA", "AQ==\n", "/////w8=", "Ag==",
    ] {
        assert!(
            matches!(
                parts("A", lengths, &[[0, 1, 0, 0, 1]]).validate(),
                Err(CodecError::InvalidTextRun)
            ),
            "{lengths}"
        );
    }
}

#[test]
fn pinned_grapheme_rules_reject_scalar_and_combining_splits() {
    for (text, lengths) in [("é", "AQE="), ("e\u{301}", "AQI="), ("👩🏽‍💻", "BAQDBA==")] {
        let length = text.len() as u32;
        assert!(matches!(
            parts(text, lengths, &[[0, length, 0, 0, length]]).validate(),
            Err(CodecError::InvalidTextRun)
        ));
    }
}

#[test]
fn gaps_overlaps_node_lengths_and_utf8_projection_edges_are_rejected() {
    for segments in [
        vec![],
        vec![[0, 1, 0, 0, 2]],
        vec![[0, 1, 0, 0, 1], [0, 1, 1, 0, 1]],
        vec![[1, 2, 0, 0, 1]],
        vec![[0, 2, 0, 0, 2]],
    ] {
        assert!(
            parts("A", "AQ==", &segments).validate().is_err(),
            "{segments:?}"
        );
    }
    assert!(
        parts("é", "Ag==", &[[0, 1, 0, 0, 1], [1, 2, 1, 0, 1]])
            .validate()
            .is_err()
    );
    // Adjacent contributors may be separate inside one grapheme, but a
    // redundant split on a grapheme boundary is not canonical V2.
    assert!(
        parts("AB", "AQE=", &[[0, 1, 0, 0, 1], [1, 2, 0, 1, 2]])
            .validate()
            .is_err()
    );
}

#[test]
fn large_node_offsets_are_projected_with_checked_interval_arithmetic() {
    let run = parts(
        "AB",
        "AQE=",
        &[[0, 1, 0, 0, 1], [1, 2, 1, u32::MAX - 1, u32::MAX]],
    );
    let validated = run.validate().unwrap();
    let second = validated.units().nth(1).unwrap();
    let spans = second.spans().collect::<Vec<_>>();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].start_byte, u32::MAX - 1);
    assert_eq!(spans[0].end_byte, u32::MAX);
    // Structural offsets here are not SourceSpan values: a real original-node
    // reader must still reject offsets that exceed its actual decoded node.
}

#[test]
fn wire_tag_fields_duplicates_and_projection_integer_widths_are_closed() {
    for value in [
        r#"{"kind":"Unknown","text":"A","grapheme_lengths":"AQ==","segments":[]}"#,
        r#"{"kind":"TextRun","text":"A","text":"A","grapheme_lengths":"AQ==","segments":[]}"#,
        r#"{"kind":"TextRun","text":"A","grapheme_lengths":"AQ==","segments":[],"extra":0}"#,
        r#"{"kind":"TextRun","text":"A","grapheme_lengths":"AQ==","segments":[[0,1,0,0,4294967296]]}"#,
        r#"{"kind":"TextRun","text":"A","grapheme_lengths":"AQ==","segments":[[0,1,0,0,-1]]}"#,
        r#"{"kind":"TextRun","text":"A","grapheme_lengths":"AQ==","segments":[[0,1,0,0]]}"#,
    ] {
        assert!(serde_json::from_str::<TextRun>(value).is_err());
    }
}

#[test]
fn encoder_rejects_foreign_locators_wrong_span_lengths_and_large_runs_before_copy() {
    let mut foreign = SourceUnit::grapheme("A".into(), vec![span(0, 0, 1, "A")]);
    let wrong = BlockLocator::UniqueXhtmlId("other".into());
    assert_eq!(
        TextRun::from_grapheme_units(&[foreign.clone()], &wrong).unwrap_err(),
        CodecError::InvalidTextRun
    );
    foreign.spans = vec![span(0, 0, 2, "AB")];
    assert_eq!(
        TextRun::from_grapheme_units(&[foreign], &locator()).unwrap_err(),
        CodecError::InvalidTextRun
    );
    let huge = "x".repeat(16 * 1024 * 1024 + 1);
    let bounds = span(0, 0, huge.len(), &huge);
    assert_eq!(
        TextRun::from_grapheme_units(&[SourceUnit::grapheme(huge, vec![bounds])], &locator())
            .unwrap_err(),
        CodecError::ResourceLimit
    );
}

#[test]
fn four_megabyte_and_near_sixteen_megabyte_ascii_runs_iterate_without_a_unit_vec() {
    for count in [4_000_001_usize, 16 * 1024 * 1024 - 1] {
        let mut lengths = "AQEB".repeat(count / 3);
        lengths.push_str(match count % 3 {
            0 => "",
            1 => "AQ==",
            _ => "AQE=",
        });
        let run = parts(
            &"A".repeat(count),
            &lengths,
            &[[0, count as u32, 0, 0, count as u32]],
        );
        let validated = run.validate().unwrap();
        assert_eq!(validated.unit_count() as usize, count);
        assert_eq!(validated.units().count(), count);
        assert_eq!(run.segments().len(), 1);
        let encoded_bytes = serde_json::to_writer(std::io::sink(), &run);
        assert!(encoded_bytes.is_ok());
        // Synthetic leaf calibration only; no eager source extraction is
        // performed, and this does not prove the full import memory budget.
    }
}

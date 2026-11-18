// Pure scope DTO cases share EPUB source identities. Relationship candidates
// here are supplied explicitly; these are not full import-resolution tests.
use super::*;
#[test]
fn scope_wire_preserves_shared_note_relations_history_and_recomputed_handling() {
    use novel_translate::import_codec::{ScopeHandling, ScopeRecord, StoredScope};
    use nt_application::{
        source_scope as app,
        volume_import::{SeriesId, VolumeId, source_scope as scope},
    };
    let source = extracted(3);
    let paragraph = source
        .documents
        .iter()
        .find(|d| d.canonical_path == "OPS/Text/ch.xhtml")
        .unwrap()
        .blocks
        .iter()
        .find(|b| b.element_name == "p")
        .unwrap();
    let note_block = source
        .documents
        .iter()
        .find(|d| d.canonical_path == "OPS/Text/notes.xhtml")
        .unwrap()
        .blocks
        .iter()
        .find(|b| b.unit_count() > 0)
        .unwrap();
    let text = |block: &nt_application::volume_import::source_range::SourceBlock| {
        scope::SourceLocation::text(vec![
            scope::LocatedRange::new(block, 0, block.unit_count()).unwrap(),
        ])
        .unwrap()
    };
    let location = text(paragraph);
    let note_location = text(note_block);
    let context = scope::DecisionContext {
        series_id: SeriesId::parse("1".repeat(32)).unwrap(),
        volume_id: VolumeId::parse("2".repeat(32)).unwrap(),
        epub_sha256: source.epub_sha256,
        mapping_version: [5; 32],
    };
    let evidence = scope::ClassificationEvidence {
        location: location.clone(),
        kind: scope::EvidenceKind::Heading {
            text: "第一章".into(),
        },
    };
    let saved = scope::ConfirmedDecision {
        key: scope::DecisionKey {
            context: context.clone(),
            location: location.clone(),
        },
        kind: scope::ContentKind::Chapter,
        evidence: vec![evidence.clone()],
    };
    let input = app::ContentInput {
        observation: scope::ContentObservation::new(
            location.clone(),
            "Aé👩🏽‍💻".into(),
            vec![
                scope::ContentKind::Chapter,
                scope::ContentKind::Advertisement,
            ],
            vec![evidence.clone()],
        )
        .unwrap(),
        history: vec![saved],
    };
    let ref_at = |order| {
        scope::SourceLocation::element(
            paragraph.canonical_path.clone(),
            nt_application::volume_import::source_range::ElementLocator {
                block: paragraph.locator.clone(),
                element_ordinals: vec![order],
            },
        )
        .unwrap()
    };
    let references = vec![
        app::NoteReference {
            location: ref_at(1),
            from: location.clone(),
            raw_href: Some("notes.xhtml#note".into()),
            target: app::NoteTarget::Resolved(note_location.clone()),
            evidence: vec![evidence.clone()],
        },
        app::NoteReference {
            location: ref_at(2),
            from: location.clone(),
            raw_href: Some("notes.xhtml#note".into()),
            target: app::NoteTarget::Resolved(note_location.clone()),
            evidence: vec![],
        },
        app::NoteReference {
            location: ref_at(3),
            from: location.clone(),
            raw_href: Some("notes.xhtml#missing".into()),
            target: app::NoteTarget::Ambiguous {
                description: "two possible notes".into(),
                candidates: vec![note_location.clone()],
            },
            evidence: vec![],
        },
    ];
    let note = app::NoteInput {
        content: app::ContentInput {
            observation: scope::ContentObservation::new(
                note_location.clone(),
                "注釈".into(),
                vec![scope::ContentKind::OtherAuxiliary],
                vec![],
            )
            .unwrap(),
            history: vec![],
        },
    };
    let related = app::RelatedText {
        location: scope::SourceLocation::attribute(
            paragraph.canonical_path.clone(),
            nt_application::volume_import::source_range::ElementLocator {
                block: paragraph.locator.clone(),
                element_ordinals: vec![2],
            },
            "alt",
        )
        .unwrap(),
        kind: app::RelatedKind::AltText,
        text: "説明".into(),
        relations: vec![app::Relation::Missing {
            description: "not guessed".into(),
        }],
        evidence: vec![evidence],
    };
    let evaluated = app::evaluate_scope(app::ScopeInput {
        context: context.clone(),
        mode: scope::ScopeMode::Story,
        content: vec![input],
        notes: vec![note],
        related: vec![related],
        note_references: references,
    })
    .unwrap();
    assert_eq!(evaluated.notes[0].references.len(), 2);
    assert_eq!(
        evaluated.related[0].handling,
        scope::Handling::NeedsRelationship
    );
    let stored = StoredScope::capture(&evaluated, 0).unwrap();
    let json = serde_json::to_vec(&stored).unwrap();
    let reopened: StoredScope = serde_json::from_slice(&json).unwrap();
    let mut lookup = |ctx: &scope::DecisionContext, path: &str, id: Digest| {
        if ctx.series_id != context.series_id
            || ctx.volume_id != context.volume_id
            || ctx.epub_sha256 != source.epub_sha256
        {
            return Err(CodecError::InvalidRecord);
        }
        source
            .documents
            .iter()
            .filter(|d| d.canonical_path == path)
            .flat_map(|d| &d.blocks)
            .find(|b| *b.id.as_bytes() == id.0)
            .map(|b| b.unit_count() as u32)
            .ok_or(CodecError::InvalidRecord)
    };
    assert_eq!(
        reopened
            .into_evaluation(context.clone(), &mut lookup)
            .unwrap(),
        evaluated
    );
    let mut forged = stored;
    let ScopeRecord::Content(content) = &mut forged.records[0] else {
        panic!("content")
    };
    content.handling = ScopeHandling::PreserveSource;
    assert_eq!(
        forged.into_evaluation(context.clone(), &mut lookup),
        Err(CodecError::InvalidRecord)
    );
}

#[test]
fn scope_wire_keeps_hash_only_and_mapping_only_history_as_comparison() {
    use novel_translate::import_codec::StoredScope;
    use nt_application::{
        source_scope as app,
        volume_import::{SeriesId, VolumeId, source_scope as scope},
    };
    let original = extracted(3);
    let changed_chapter = fixture::CHAPTER.replace("A<em>", "B<em>");
    let changed = parsed_chapter(3, &changed_chapter).1;
    fn block(
        source: &nt_epub::SourceExtraction,
    ) -> &nt_application::volume_import::source_range::SourceBlock {
        source
            .documents
            .iter()
            .find(|d| d.canonical_path == "OPS/Text/ch.xhtml")
            .unwrap()
            .blocks
            .iter()
            .find(|b| b.element_name == "p")
            .unwrap()
    }
    let location = |source: &nt_epub::SourceExtraction| {
        let b = block(source);
        scope::SourceLocation::text(vec![
            scope::LocatedRange::new(b, 0, b.unit_count()).unwrap(),
        ])
        .unwrap()
    };
    let old = scope::DecisionContext {
        series_id: SeriesId::parse("3".repeat(32)).unwrap(),
        volume_id: VolumeId::parse("4".repeat(32)).unwrap(),
        epub_sha256: original.epub_sha256,
        mapping_version: [7; 32],
    };
    for hash_change in [true, false] {
        let source = if hash_change { &changed } else { &original };
        let mut context = old.clone();
        if hash_change {
            context.epub_sha256 = source.epub_sha256;
        } else {
            context.mapping_version = [8; 32];
        }
        let saved = scope::ConfirmedDecision {
            key: scope::DecisionKey {
                context: old.clone(),
                location: location(&original),
            },
            kind: scope::ContentKind::Chapter,
            evidence: vec![],
        };
        let input = app::ContentInput {
            observation: scope::ContentObservation::new(
                location(source),
                "current".into(),
                vec![
                    scope::ContentKind::Chapter,
                    scope::ContentKind::Advertisement,
                ],
                vec![],
            )
            .unwrap(),
            history: vec![saved],
        };
        let evaluated = app::evaluate_scope(app::ScopeInput {
            context: context.clone(),
            mode: scope::ScopeMode::Story,
            content: vec![input],
            notes: vec![],
            related: vec![],
            note_references: vec![],
        })
        .unwrap();
        assert!(matches!(
            evaluated.content[0].decision,
            scope::DecisionMatch::NeedsComparison(_)
        ));
        assert_eq!(
            evaluated.content[0].handling,
            scope::Handling::NeedsClassification
        );
        let json = serde_json::to_vec(&StoredScope::capture(&evaluated, 0).unwrap()).unwrap();
        let reopened: StoredScope = serde_json::from_slice(&json).unwrap();
        let mut lookups = vec![];
        let mut lookup = |ctx: &scope::DecisionContext, path: &str, id: Digest| {
            lookups.push(ctx.epub_sha256);
            let source = if ctx.epub_sha256 == original.epub_sha256 {
                &original
            } else if ctx.epub_sha256 == changed.epub_sha256 {
                &changed
            } else {
                return Err(CodecError::InvalidRecord);
            };
            source
                .documents
                .iter()
                .filter(|d| d.canonical_path == path)
                .flat_map(|d| &d.blocks)
                .find(|b| *b.id.as_bytes() == id.0)
                .map(|b| b.unit_count() as u32)
                .ok_or(CodecError::InvalidRecord)
        };
        assert_eq!(
            reopened.into_evaluation(context, &mut lookup).unwrap(),
            evaluated
        );
        assert!(lookups.contains(&original.epub_sha256));
        assert!(lookups.contains(&source.epub_sha256));
    }
}

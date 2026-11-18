use nt_application::source_scope::{
    ContentInput, NoteInput, NoteReference, NoteTarget, RelatedKind, RelatedText, Relation,
    ScopeError, ScopeInput, evaluate_scope, observe_map_text,
};
use nt_domain::chapter_map::{
    AuxiliaryDocumentCoverage, BlockUnitRange, BoundaryEvidence, BoundaryEvidenceKind, ChapterMap,
    ChapterMapInput, CoverageDomain, MainFlowContent, SourceDocument, VolumeTree,
};
use nt_domain::source_range::{
    BlockLocator, ElementLocator, ImageSource, InlineMetadataKind, InlineSourceMetadata,
    ProtectedSourceUnit, SourceBlock, SourceBlockId, SourceUnit, SourceUnitKind, UnitRange,
};
use nt_domain::source_scope::{
    ClassificationEvidence, ConfirmedDecision, ContentKind, ContentObservation, DecisionContext,
    DecisionKey, DecisionMatch, EvidenceKind, Handling, LocatedRange, ScopeMode, SeriesScope,
    SourceLocation, VolumeScope,
};
use nt_domain::workspace::{SeriesId, VolumeId};

fn series(value: char) -> SeriesId {
    SeriesId::parse(value.to_string().repeat(32)).unwrap()
}

fn volume(value: char) -> VolumeId {
    VolumeId::parse(value.to_string().repeat(32)).unwrap()
}

fn context() -> DecisionContext {
    DecisionContext {
        series_id: series('a'),
        volume_id: volume('b'),
        epub_sha256: [3; 32],
        mapping_version: [4; 32],
    }
}

fn block(id: u8, path: &str, text: &str) -> SourceBlock {
    SourceBlock {
        id: SourceBlockId::from_bytes([id; 32]),
        canonical_path: path.to_owned(),
        element_name: "p".to_owned(),
        locator: BlockLocator::UniqueXhtmlId(format!("p-{id}")),
        parser_version: "nt-epub/1.2.0".to_owned(),
        content_hash: [id.wrapping_add(40); 32],
        units: text
            .chars()
            .map(|ch| SourceUnit {
                kind: SourceUnitKind::Grapheme(ch.to_string()),
                spans: Vec::new(),
            })
            .collect(),
        inline_metadata: Vec::new(),
        attribute_text: Vec::new(),
        protected_resource_hashes: Vec::new(),
    }
}

fn text_location(id: u8) -> SourceLocation {
    let source = block(id, &format!("OPS/{id}.xhtml"), "abcdefgh");
    SourceLocation::text(vec![LocatedRange::new(&source, 0, 8).unwrap()]).unwrap()
}

fn element_location(id: u8) -> SourceLocation {
    SourceLocation::element(
        format!("OPS/{id}.xhtml"),
        ElementLocator {
            block: BlockLocator::UniqueXhtmlId(format!("p-{id}")),
            element_ordinals: vec![1, u32::from(id)],
        },
    )
    .unwrap()
}

fn attribute_location(id: u8) -> SourceLocation {
    SourceLocation::attribute(
        format!("OPS/{id}.xhtml"),
        ElementLocator {
            block: BlockLocator::UniqueXhtmlId(format!("p-{id}")),
            element_ordinals: vec![1],
        },
        "alt",
    )
    .unwrap()
}

fn evidence(location: &SourceLocation, description: &str) -> ClassificationEvidence {
    ClassificationEvidence {
        location: location.clone(),
        kind: EvidenceKind::Other {
            description: description.to_owned(),
        },
    }
}

fn content(location: SourceLocation, kinds: Vec<ContentKind>) -> ContentInput {
    ContentInput {
        observation: ContentObservation::new(
            location.clone(),
            "readable original".to_owned(),
            kinds,
            vec![evidence(&location, "observed source")],
        )
        .unwrap(),
        history: Vec::new(),
    }
}

fn scope(mode: ScopeMode) -> ScopeInput {
    ScopeInput {
        context: context(),
        mode,
        content: Vec::new(),
        notes: Vec::new(),
        related: Vec::new(),
        note_references: Vec::new(),
    }
}

#[test]
fn volume_defaults_are_captured_at_creation_even_before_import() {
    let mut series = SeriesScope::new(series('a'));
    assert_eq!(series.default_mode(), ScopeMode::AllText);
    let mut a = series.capture_for_new_volume(volume('b'));
    let b = series.capture_for_new_volume(volume('c'));
    let created_before_change = series.capture_for_new_volume(volume('d'));

    a.set_mode(ScopeMode::Story);
    assert_eq!(a.mode(), ScopeMode::Story);
    assert_eq!(b.mode(), ScopeMode::AllText);
    assert_eq!(series.default_mode(), ScopeMode::AllText);

    series.set_default(ScopeMode::Story);
    let created_after_change = series.capture_for_new_volume(volume('e'));
    assert_eq!(a.mode(), ScopeMode::Story);
    assert_eq!(b.mode(), ScopeMode::AllText);
    // First import happens after the default changed; creation captured AllText.
    assert_eq!(created_before_change.mode(), ScopeMode::AllText);
    assert_eq!(created_after_change.mode(), ScopeMode::Story);

    series.set_default(ScopeMode::AllText);
    assert_eq!(created_after_change.mode(), ScopeMode::Story);

    let restored_series = SeriesScope::from_saved(series.series_id.clone(), ScopeMode::Story);
    let restored_b = VolumeScope::from_saved(series.series_id.clone(), volume('c'), b.mode());
    assert_eq!(restored_series.default_mode(), ScopeMode::Story);
    assert_eq!(restored_b.mode(), ScopeMode::AllText);
}

#[test]
fn ambiguity_only_blocks_story_when_possible_kinds_change_the_treatment() {
    use ContentKind::{Advertisement, Chapter, Copyright, SideStory};
    let mut input = scope(ScopeMode::Story);
    input.content = vec![
        content(text_location(1), vec![SideStory, Advertisement]),
        content(text_location(2), vec![Chapter, SideStory]),
        content(text_location(3), vec![Copyright, Advertisement]),
    ];
    let story = evaluate_scope(input.clone()).unwrap();
    assert_eq!(
        story
            .content
            .iter()
            .map(|item| item.handling)
            .collect::<Vec<_>>(),
        vec![
            Handling::NeedsClassification,
            Handling::TranslateAndReview,
            Handling::PreserveSource,
        ]
    );
    assert_eq!(
        story.content[0].input.observation.possible_kinds(),
        &[SideStory, Advertisement]
    );
    assert_eq!(
        story.content[0].input.observation.evidence[0].kind,
        evidence(&text_location(1), "observed source").kind
    );

    input.mode = ScopeMode::AllText;
    let all_text = evaluate_scope(input).unwrap();
    assert!(
        all_text
            .content
            .iter()
            .all(|item| item.handling == Handling::TranslateAndReview)
    );
    assert_eq!(all_text.content[0].decision, DecisionMatch::Unconfirmed);
    assert!(ContentObservation::new(text_location(8), String::new(), vec![], vec![]).is_err());
}

#[test]
fn confirmed_decision_reuses_only_the_exact_five_part_key_and_retains_history() {
    use ContentKind::{Advertisement, SideStory};
    let original_location = text_location(1);
    let original_evidence = evidence(&original_location, "user compared the source excerpt");
    let decision = ConfirmedDecision {
        key: DecisionKey {
            context: context(),
            location: original_location.clone(),
        },
        kind: SideStory,
        evidence: vec![original_evidence.clone()],
    };
    let item = ContentInput {
        history: vec![decision.clone()],
        ..content(original_location.clone(), vec![SideStory, Advertisement])
    };
    let mut original = scope(ScopeMode::Story);
    original.content = vec![item.clone()];
    let exact = evaluate_scope(original.clone()).unwrap();
    assert_eq!(exact.content[0].handling, Handling::TranslateAndReview);
    assert_eq!(
        exact.content[0].decision,
        DecisionMatch::Reused(Box::new(decision.clone()))
    );
    assert_eq!(
        exact.content[0].input.observation.evidence[0],
        evidence(&original_location, "observed source")
    );
    assert_eq!(
        exact.content[0].input.history[0].evidence,
        vec![original_evidence]
    );

    // Each case changes one field only. No title or excerpt is used as identity.
    let mut changed_series = original.clone();
    changed_series.context.series_id = series('e');
    let mut changed_volume = original.clone();
    changed_volume.context.volume_id = volume('f');
    let mut changed_hash = original.clone();
    changed_hash.context.epub_sha256 = [5; 32];
    let mut changed_mapping = original.clone();
    changed_mapping.context.mapping_version = [6; 32];
    let mut changed_location = original.clone();
    changed_location.content[0].observation =
        content(text_location(2), vec![SideStory, Advertisement]).observation;

    // The same block and path with just its range shifted must not reuse.
    let same_block = block(1, "OPS/1.xhtml", "abcdefgh");
    let shifted =
        SourceLocation::text(vec![LocatedRange::new(&same_block, 1, 8).unwrap()]).unwrap();
    let mut changed_range = original.clone();
    changed_range.content[0].observation =
        content(shifted, vec![SideStory, Advertisement]).observation;

    for candidate in [
        changed_series,
        changed_volume,
        changed_hash,
        changed_mapping,
        changed_location,
        changed_range,
    ] {
        let result = evaluate_scope(candidate).unwrap();
        assert_eq!(result.content[0].handling, Handling::NeedsClassification);
        assert_eq!(
            result.content[0].decision,
            DecisionMatch::NeedsComparison(vec![decision.clone()])
        );
        assert_eq!(result.content[0].input.history, vec![decision.clone()]);
    }
}

#[test]
fn source_range_location_compares_both_bounds_and_rejects_invalid_map_ranges() {
    let source = block(9, "OPS/chapter.xhtml", "abcdefgh");
    let whole = SourceLocation::text(vec![LocatedRange::new(&source, 0, 8).unwrap()]).unwrap();
    let shorter = SourceLocation::text(vec![LocatedRange::new(&source, 0, 7).unwrap()]).unwrap();
    let shifted = SourceLocation::text(vec![LocatedRange::new(&source, 1, 8).unwrap()]).unwrap();
    assert_ne!(whole, shorter);
    assert_ne!(whole, shifted);
    let split_in_reverse = SourceLocation::text(vec![
        LocatedRange::new(&source, 4, 8).unwrap(),
        LocatedRange::new(&source, 0, 4).unwrap(),
    ])
    .unwrap();
    assert_eq!(whole, split_in_reverse);
    assert!(LocatedRange::new(&source, 4, 4).is_err());
    assert!(LocatedRange::new(&source, 0, 9).is_err());
    assert!(SourceLocation::text(vec![]).is_err());
    assert!(
        SourceLocation::text(vec![
            LocatedRange::new(&source, 0, 5).unwrap(),
            LocatedRange::new(&source, 4, 8).unwrap(),
        ])
        .is_err()
    );
    let mapped = BlockUnitRange {
        canonical_path: "OPS/other.xhtml".to_owned(),
        block_id: source.id,
        start: 0,
        end: 8,
    };
    assert!(LocatedRange::from_map(&source, &mapped).is_err());
    let wrong_block = BlockUnitRange {
        canonical_path: source.canonical_path.clone(),
        block_id: SourceBlockId::from_bytes([8; 32]),
        ..mapped
    };
    assert!(LocatedRange::from_map(&source, &wrong_block).is_err());
}

#[test]
fn shared_note_is_evaluated_once_and_related_text_follows_its_real_owner() {
    use ContentKind::{Advertisement, Chapter, OtherAuxiliary, SideStory};
    let story = text_location(1);
    let auxiliary = text_location(2);
    let uncertain = text_location(3);
    let note = text_location(4);
    let story_ref = NoteReference {
        location: element_location(10),
        from: story.clone(),
        raw_href: Some("../notes.xhtml#n1".to_owned()),
        target: NoteTarget::Resolved(note.clone()),
        evidence: vec![evidence(&element_location(10), "story link")],
    };
    let auxiliary_ref = NoteReference {
        location: element_location(11),
        from: auxiliary.clone(),
        raw_href: Some("../notes.xhtml#n1".to_owned()),
        target: NoteTarget::Resolved(note.clone()),
        evidence: vec![evidence(&element_location(11), "auxiliary link")],
    };
    let make_related = |location, kind, owner| RelatedText {
        location,
        kind,
        text: "related text".to_owned(),
        relations: vec![Relation::Resolved(owner)],
        evidence: vec![],
    };
    let mut input = scope(ScopeMode::Story);
    input.content = vec![
        content(story.clone(), vec![Chapter]),
        content(auxiliary.clone(), vec![Advertisement]),
        content(uncertain.clone(), vec![SideStory, Advertisement]),
    ];
    input.notes = vec![NoteInput {
        content: content(note.clone(), vec![OtherAuxiliary]),
    }];
    input.note_references = vec![story_ref.clone(), auxiliary_ref.clone(), story_ref.clone()];
    input.related = vec![
        make_related(attribute_location(21), RelatedKind::AltText, story.clone()),
        make_related(text_location(22), RelatedKind::Caption, story.clone()),
        make_related(
            text_location(23),
            RelatedKind::NavigationLabel,
            auxiliary.clone(),
        ),
        make_related(text_location(24), RelatedKind::Heading, uncertain.clone()),
        make_related(text_location(25), RelatedKind::OtherLabel, note.clone()),
    ];

    let result = evaluate_scope(input.clone()).unwrap();
    assert_eq!(result.notes.len(), 1);
    assert_eq!(result.notes[0].content.input.observation.location, note);
    assert_eq!(
        result.notes[0].content.handling,
        Handling::TranslateAndReview
    );
    assert_eq!(
        result.notes[0].references,
        vec![story_ref.clone(), auxiliary_ref.clone()]
    );
    assert_eq!(result.note_references, vec![story_ref, auxiliary_ref]);
    assert_eq!(
        result
            .related
            .iter()
            .map(|item| item.handling)
            .collect::<Vec<_>>(),
        vec![
            Handling::TranslateAndReview,
            Handling::TranslateAndReview,
            Handling::PreserveSource,
            Handling::NeedsClassification,
            Handling::TranslateAndReview,
        ]
    );
    assert_eq!(result.content[1].handling, Handling::PreserveSource);
    assert!(
        matches!(&result.related[2].input.relations[0], Relation::Resolved(owner) if owner == &auxiliary)
    );

    input.mode = ScopeMode::AllText;
    let all_text = evaluate_scope(input).unwrap();
    assert!(
        all_text
            .content
            .iter()
            .all(|item| item.handling == Handling::TranslateAndReview)
    );
    assert!(
        all_text
            .related
            .iter()
            .all(|item| item.handling == Handling::TranslateAndReview)
    );
    assert_eq!(all_text.notes.len(), 1);
    assert_eq!(all_text.notes[0].references.len(), 2);
}

#[test]
fn unresolved_note_and_related_targets_keep_evidence_without_guessing() {
    let story = text_location(1);
    let note = text_location(2);
    let missing = NoteReference {
        location: element_location(10),
        from: story.clone(),
        raw_href: Some("#missing".to_owned()),
        target: NoteTarget::Missing {
            description: "no anchor".to_owned(),
        },
        evidence: vec![evidence(&element_location(10), "missing anchor")],
    };
    let ambiguous = NoteReference {
        location: element_location(11),
        from: story.clone(),
        raw_href: Some("#duplicate".to_owned()),
        target: NoteTarget::Ambiguous {
            description: "duplicate anchors".to_owned(),
            candidates: vec![note.clone(), text_location(3)],
        },
        evidence: vec![evidence(&element_location(11), "duplicate anchor")],
    };
    let mut input = scope(ScopeMode::Story);
    input
        .content
        .push(content(story, vec![ContentKind::Chapter]));
    input.notes.push(NoteInput {
        content: content(note, vec![ContentKind::OtherAuxiliary]),
    });
    input.note_references = vec![missing.clone(), ambiguous.clone()];
    input.related.push(RelatedText {
        location: attribute_location(20),
        kind: RelatedKind::AltText,
        text: "illustration description".to_owned(),
        relations: vec![Relation::Ambiguous {
            description: "owner is not uniquely resolved".to_owned(),
            candidates: vec![text_location(1), text_location(2)],
        }],
        evidence: vec![evidence(&attribute_location(20), "two possible owners")],
    });
    let story_result = evaluate_scope(input.clone()).unwrap();
    assert_eq!(
        story_result.notes[0].content.handling,
        Handling::PreserveSource
    );
    assert!(story_result.notes[0].references.is_empty());
    assert_eq!(story_result.note_references, vec![missing, ambiguous]);
    assert_eq!(
        story_result.related[0].handling,
        Handling::NeedsRelationship
    );
    assert_eq!(story_result.related[0].input.evidence.len(), 1);

    input.mode = ScopeMode::AllText;
    let all_text = evaluate_scope(input).unwrap();
    assert_eq!(
        all_text.notes[0].content.handling,
        Handling::TranslateAndReview
    );
    assert_eq!(all_text.related[0].handling, Handling::TranslateAndReview);
    assert_eq!(all_text.note_references.len(), 2);
}

#[test]
fn note_follows_the_confirmed_classification_of_its_ambiguous_caller() {
    let caller = text_location(1);
    let note = text_location(2);
    let mut input = scope(ScopeMode::Story);
    input.content = vec![content(
        caller.clone(),
        vec![ContentKind::SideStory, ContentKind::Advertisement],
    )];
    input.notes = vec![NoteInput {
        content: content(note.clone(), vec![ContentKind::OtherAuxiliary]),
    }];
    input.note_references = vec![NoteReference {
        location: element_location(10),
        from: caller.clone(),
        raw_href: Some("#note".to_owned()),
        target: NoteTarget::Resolved(note),
        evidence: vec![],
    }];
    let pending = evaluate_scope(input.clone()).unwrap();
    assert_eq!(
        pending.notes[0].content.handling,
        Handling::NeedsClassification
    );
    let decision = ConfirmedDecision {
        key: DecisionKey {
            context: context(),
            location: caller,
        },
        kind: ContentKind::SideStory,
        evidence: vec![],
    };
    input.content[0].history.push(decision.clone());
    let story = evaluate_scope(input.clone()).unwrap();
    assert_eq!(
        story.notes[0].content.handling,
        Handling::TranslateAndReview
    );
    // The newest exact decision supersedes an older exact decision, without deleting it.
    input.content[0].history.push(ConfirmedDecision {
        kind: ContentKind::Advertisement,
        ..decision
    });
    let auxiliary = evaluate_scope(input).unwrap();
    assert_eq!(
        auxiliary.notes[0].content.handling,
        Handling::PreserveSource
    );
    assert_eq!(auxiliary.content[0].input.history.len(), 2);
}

#[test]
fn conflicting_or_wrongly_resolved_references_are_rejected() {
    let owner = text_location(1);
    let note = text_location(2);
    let mut input = scope(ScopeMode::Story);
    input
        .content
        .push(content(owner.clone(), vec![ContentKind::Chapter]));
    input.notes.push(NoteInput {
        content: content(note.clone(), vec![ContentKind::OtherAuxiliary]),
    });
    let reference = NoteReference {
        location: element_location(5),
        from: owner.clone(),
        raw_href: Some("#n".to_owned()),
        target: NoteTarget::Resolved(note),
        evidence: vec![],
    };
    let mut conflict = reference.clone();
    conflict.raw_href = Some("#another".to_owned());
    input.note_references = vec![reference.clone(), conflict];
    assert_eq!(
        evaluate_scope(input.clone()),
        Err(ScopeError::ConflictingReference)
    );

    input.note_references = vec![NoteReference {
        target: NoteTarget::Resolved(owner.clone()),
        ..reference.clone()
    }];
    assert_eq!(
        evaluate_scope(input.clone()),
        Err(ScopeError::UnknownResolvedNote)
    );

    input.note_references = vec![NoteReference {
        from: text_location(30),
        ..reference
    }];
    assert_eq!(
        evaluate_scope(input),
        Err(ScopeError::UnknownReferenceOwner)
    );
}

#[test]
fn related_cycles_without_content_stay_unresolved_and_transitive_notes_propagate() {
    let story = text_location(1);
    let first = text_location(2);
    let second = text_location(3);
    let label_a = text_location(4);
    let label_b = text_location(5);
    let mut input = scope(ScopeMode::Story);
    input
        .content
        .push(content(story.clone(), vec![ContentKind::Chapter]));
    input.notes = vec![
        NoteInput {
            content: content(first.clone(), vec![ContentKind::OtherAuxiliary]),
        },
        NoteInput {
            content: content(second.clone(), vec![ContentKind::OtherAuxiliary]),
        },
    ];
    input.note_references = vec![
        NoteReference {
            location: element_location(10),
            from: story,
            raw_href: Some("#first".to_owned()),
            target: NoteTarget::Resolved(first.clone()),
            evidence: vec![],
        },
        NoteReference {
            location: element_location(11),
            from: first,
            raw_href: Some("#second".to_owned()),
            target: NoteTarget::Resolved(second),
            evidence: vec![],
        },
    ];
    input.related = vec![
        RelatedText {
            location: label_a.clone(),
            kind: RelatedKind::Caption,
            text: "a".to_owned(),
            relations: vec![Relation::Resolved(label_b.clone())],
            evidence: vec![],
        },
        RelatedText {
            location: label_b,
            kind: RelatedKind::Caption,
            text: "b".to_owned(),
            relations: vec![Relation::Resolved(label_a)],
            evidence: vec![],
        },
    ];
    let result = evaluate_scope(input.clone()).unwrap();
    assert_eq!(
        result
            .notes
            .iter()
            .map(|note| note.content.handling)
            .collect::<Vec<_>>(),
        vec![Handling::TranslateAndReview, Handling::TranslateAndReview]
    );
    assert_eq!(
        result
            .related
            .iter()
            .map(|text| text.handling)
            .collect::<Vec<_>>(),
        vec![Handling::NeedsRelationship, Handling::NeedsRelationship]
    );
    input.mode = ScopeMode::AllText;
    let all_text = evaluate_scope(input).unwrap();
    assert!(
        all_text
            .related
            .iter()
            .all(|text| text.handling == Handling::TranslateAndReview)
    );
}

#[test]
fn cp04_mapping_provides_ranges_text_and_real_evidence_without_guessing_kind() {
    let mut heading = block(1, "OPS/story.xhtml", "Hello");
    heading.element_name = "h1".to_owned();
    heading.inline_metadata.push(InlineSourceMetadata {
        locator: ElementLocator {
            block: heading.locator.clone(),
            element_ordinals: vec![0],
        },
        kind: InlineMetadataKind::Link {
            href: Some("#note".to_owned()),
        },
        ranges: vec![UnitRange::new(1, 4, 5).unwrap()],
    });
    heading.inline_metadata.push(InlineSourceMetadata {
        locator: ElementLocator {
            block: heading.locator.clone(),
            element_ordinals: vec![1],
        },
        kind: InlineMetadataKind::NoteReference {
            href: Some("#empty".to_owned()),
        },
        ranges: vec![UnitRange::new(5, 5, 5).unwrap()],
    });
    let mut image_only = block(2, "OPS/story.xhtml", "");
    image_only.units.push(SourceUnit::protected(
        ProtectedSourceUnit::Image(ImageSource {
            raw_source: Some("picture.png".to_owned()),
            resource_path: Some("OPS/picture.png".to_owned()),
            alt: None,
            occurrence: 0,
            node_path: vec![3],
            external: false,
        }),
        vec![],
    ));
    let auxiliary = block(3, "OPS/ads.xhtml", "AD");
    let range = |source: &SourceBlock| BlockUnitRange {
        canonical_path: source.canonical_path.clone(),
        block_id: source.id,
        start: 0,
        end: source.unit_count() as u32,
    };
    let input = ChapterMapInput {
        epub_sha256: [3; 32],
        parser_version: "nt-epub/1.2.0".to_owned(),
        documents: vec![
            SourceDocument {
                canonical_path: "OPS/story.xhtml".to_owned(),
                domain: CoverageDomain::MainFlow,
                spine_positions: vec![2],
                linear_spine_positions: vec![2],
                blocks: vec![heading.clone(), image_only.clone()],
            },
            SourceDocument {
                canonical_path: "OPS/ads.xhtml".to_owned(),
                domain: CoverageDomain::Auxiliary,
                spine_positions: vec![],
                linear_spine_positions: vec![],
                blocks: vec![auxiliary.clone()],
            },
        ],
        anchors: vec![],
        navigation: vec![],
        confirmed_boundaries: vec![],
        diagnostics: vec![],
    };
    let map = ChapterMap {
        epub_sha256: [3; 32],
        parser_version: input.parser_version.clone(),
        volume: VolumeTree {
            main_flow: vec![MainFlowContent::OutOfChapter {
                ranges: vec![range(&heading), range(&image_only)],
                reason: "unclassified".to_owned(),
            }],
            auxiliary: vec![AuxiliaryDocumentCoverage {
                canonical_path: auxiliary.canonical_path.clone(),
                ranges: vec![range(&auxiliary)],
            }],
        },
        navigation: vec![],
        boundary_evidence: vec![BoundaryEvidence {
            kind: BoundaryEvidenceKind::Heading,
            source: None,
            canonical_path: heading.canonical_path.clone(),
            block_id: Some(heading.id),
            unit_offset: Some(0),
            label: Some("Hello".to_owned()),
            raw_href: None,
            reason: "heading evidence".to_owned(),
            depth: None,
        }],
        diagnostics: vec![],
    };
    let observations = observe_map_text(&input, &map).unwrap();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].observation.excerpt, "Hello");
    assert_eq!(observations[1].observation.excerpt, "AD");
    assert_eq!(
        observations[0].observation.possible_kinds(),
        &[ContentKind::OtherStory, ContentKind::OtherAuxiliary]
    );
    assert!(
        observations[0]
            .observation
            .evidence
            .iter()
            .any(|item| item.kind == EvidenceKind::SpineOrder { position: 2 })
    );
    assert!(
        observations[0]
            .observation
            .evidence
            .iter()
            .any(|item| item.kind
                == EvidenceKind::Heading {
                    text: "Hello".to_owned()
                })
    );
    assert!(
        observations[0]
            .observation
            .evidence
            .iter()
            .any(|item| item.kind
                == EvidenceKind::Link {
                    href: Some("#note".to_owned()),
                    note_reference: false,
                })
    );
    assert!(
        observations[0]
            .observation
            .evidence
            .iter()
            .any(|item| item.kind
                == EvidenceKind::Link {
                    href: Some("#empty".to_owned()),
                    note_reference: true
                })
    );
    assert!(observations[0].observation.evidence.iter().any(|item| matches!(&item.kind,
        EvidenceKind::MappingBoundary { kind: BoundaryEvidenceKind::Heading, unit_offset: Some(0), reason, .. }
        if reason == "heading evidence"
    )));
    let mut scoped = scope(ScopeMode::Story);
    scoped.content = observations;
    let result = evaluate_scope(scoped).unwrap();
    assert_eq!(result.content[0].handling, Handling::NeedsClassification);
    assert_eq!(result.content[1].handling, Handling::NeedsClassification);

    let mut invalid = map;
    invalid.volume.auxiliary[0].ranges[0].end = 1;
    assert!(matches!(
        observe_map_text(&input, &invalid),
        Err(ScopeError::InvalidMap(_))
    ));
}

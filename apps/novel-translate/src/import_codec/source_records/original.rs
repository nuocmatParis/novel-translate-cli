//! Compare original node and unit evidence without recreating SourceSpan.

use nt_application::volume_import::{CapturedBlock, CapturedUnitRun, source_range as source};

use super::{
    AttributeText, CodecError, ElementLocator, InlineKind, InlineMetadata, MarkerEndpoint,
    ProtectedUnit, Span, StoredSourceBlock, StoredUnitRun,
};

impl StoredSourceBlock {
    /// Compare this block with the compact replay of the captured XML. The
    /// enclosing reader also checks document order, parser version, anchors,
    /// original resource object hashes and mapping coverage.
    pub fn matches_captured_block(&self, block: &CapturedBlock) -> Result<bool, CodecError> {
        if self.id.0 != *block.id.as_bytes()
            || self.element_name != block.element_name
            || !self.locator.matches(&block.locator)
            || self.content_hash.0 != block.content_hash
            || !pairs(
                &self.inline_metadata,
                &block.inline_metadata,
                inline_metadata,
            )
            || !pairs(&self.attribute_text, &block.attribute_text, attribute_text)
            || self.unit_runs.len() != block.runs.len()
        {
            return Ok(false);
        }
        let mut total = 0u32;
        for (saved, original) in self.unit_runs.iter().zip(&block.runs) {
            let units = match (saved, original) {
                (
                    StoredUnitRun::Text(saved),
                    CapturedUnitRun::Text {
                        text,
                        segments,
                        unit_count,
                    },
                ) => {
                    let Some(count) = saved.original_unit_count(text, segments)? else {
                        return Ok(false);
                    };
                    if count != *unit_count {
                        return Ok(false);
                    }
                    count
                }
                (StoredUnitRun::Protected(saved), CapturedUnitRun::Protected { unit, spans })
                    if protected_unit(&saved.unit, unit)
                        && pairs(&saved.spans, spans, Span::matches) =>
                {
                    1
                }
                _ => return Ok(false),
            };
            total = total.checked_add(units).ok_or(CodecError::ResourceLimit)?;
        }
        Ok(total == block.unit_count)
    }
}

fn pairs<A, B>(a: &[A], b: &[B], mut matches: impl FnMut(&A, &B) -> bool) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| matches(a, b))
}

fn element(saved: &ElementLocator, original: &source::ElementLocator) -> bool {
    saved.block.matches(&original.block) && saved.element_ordinals == original.element_ordinals
}

fn inline_metadata(saved: &InlineMetadata, original: &source::InlineSourceMetadata) -> bool {
    use source::InlineMetadataKind as Kind;
    let kind_agrees = match (&saved.kind, &original.kind) {
        (InlineKind::Formatting { element: a }, Kind::Formatting { element: b }) => a == b,
        (InlineKind::Link { href: a }, Kind::Link { href: b })
        | (InlineKind::NoteReference { href: a }, Kind::NoteReference { href: b }) => a == b,
        _ => false,
    };
    kind_agrees
        && element(&saved.locator, &original.locator)
        && pairs(&saved.ranges, &original.ranges, |a, b| {
            a.start == b.start && a.end == b.end
        })
}

fn attribute_text(saved: &AttributeText, original: &source::AttributeSourceText) -> bool {
    element(&saved.locator, &original.locator)
        && saved.attribute == original.attribute
        && saved.value == original.value
        && saved.resource_path == original.resource_path
}

fn marker(saved: &MarkerEndpoint, original: &source::MarkerEndpoint) -> bool {
    saved.node_path == original.node_path
        && pairs(
            &saved.attributes,
            &original.attributes,
            |a, (name, value)| &a.name == name && &a.value == value,
        )
}

fn protected_unit(saved: &ProtectedUnit, original: &source::ProtectedSourceUnit) -> bool {
    use source::ProtectedSourceUnit as Original;
    match (saved, original) {
        (
            ProtectedUnit::Ruby {
                base,
                annotation,
                base_spans,
                annotation_spans,
                node_path,
            },
            Original::Ruby(ruby),
        ) => {
            base == &ruby.base
                && annotation == &ruby.annotation
                && node_path == &ruby.node_path
                && pairs(base_spans, &ruby.base_spans, Span::matches)
                && pairs(annotation_spans, &ruby.annotation_spans, Span::matches)
        }
        (
            ProtectedUnit::Image {
                raw_source,
                resource_path,
                alt,
                occurrence,
                node_path,
                external,
            },
            Original::Image(image),
        ) => {
            raw_source == &image.raw_source
                && resource_path == &image.resource_path
                && alt == &image.alt
                && *occurrence == image.occurrence as u64
                && node_path == &image.node_path
                && *external == image.external
        }
        (ProtectedUnit::MarkerPair { kind, start, end }, Original::MarkerPair(pair)) => {
            kind == &pair.kind && marker(start, &pair.start) && marker(end, &pair.end)
        }
        (ProtectedUnit::LineBreak { element_path: a }, Original::LineBreak { element_path: b })
        | (
            ProtectedUnit::SceneBreak { element_path: a },
            Original::SceneBreak { element_path: b },
        ) => a == b,
        _ => false,
    }
}

use super::super::CodecError;
use super::super::inventory_records::names;
use super::super::records::{Budget, Digest, safe_path};
use nt_epub as epub;
use serde::{Deserialize, Serialize};

names!(
    ReferenceKind,
    epub::ReferenceKind,
    XhtmlImageSrc,
    SvgImageHref,
    SrcsetCandidate,
    StylesheetLink,
    CssImport,
    CssUrl,
    Cover,
    XhtmlLink,
    PackageLink,
    Manifest,
    Fallback,
    MediaOverlay,
    Spine
);

macro_rules! issues{
    ($($variant:ident),+ $(,)?)=>{
        #[derive(Clone,Debug,Eq,PartialEq,Serialize,Deserialize)]
        #[serde(tag="kind",content="data",deny_unknown_fields)]
        pub enum ReferenceIssue{$($variant{}),+,At{source_path:String,line:Option<u32>,column:Option<u32>,reason:Box<ReferenceIssue>}}
        $crate::import_codec::memory::heap_enum!(ReferenceIssue {$($variant{}),+,At{source_path,line,column,reason}});
        impl ReferenceIssue{
            pub(super) fn capture(value:&epub::ReferenceIndexError,budget:&mut Budget)->Result<Self,CodecError>{
                Ok(match value{$(epub::ReferenceIndexError::$variant=>Self::$variant{}),+,epub::ReferenceIndexError::At{source_path,line,column,reason}=>{
                    budget.charge(std::mem::size_of::<Self>(),1)?;
                    Self::At{source_path:budget.text(source_path)?,line:*line,column:*column,reason:Box::new(Self::capture(reason,budget)?)}
                }})
            }
        }
    }
}
issues!(
    InvalidInput,
    Archive,
    ArchiveEntryLimit,
    ArchiveCumulativeLimit,
    XmlMalformed,
    XmlByteLimit,
    XmlNodeLimit,
    XmlDepthLimit,
    ExternalEntity,
    UnknownEntity,
    EntityExpansionLimit,
    UnsupportedEncoding,
    FileCountLimit,
    ReferenceCountLimit,
    ReferenceByteLimit,
    RawEvidenceLimit,
    WorkingMemoryLimit,
    CssByteLimit,
    CssTokenLimit,
    CssFunctionDepthLimit,
    CssImportDepthLimit,
    InvalidCss,
    InvalidSrcset,
    UnsafeUri,
    SourceMismatch,
    Missing,
    Ambiguous,
    External,
    ImportCycle,
    OptionalNavigationSkipped,
    BaseByteLimit,
    BaseNodeLimit,
    BaseWorkLimit
);
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceLocation {
    pub element_ordinals: Vec<u32>,
    pub attribute_namespace: Option<String>,
    pub attribute_local_name: Option<String>,
    pub candidate_ordinal: u32,
    pub line: u32,
    pub column: u32,
    pub decoded_byte_range: Option<(u32, u32)>,
}
impl ReferenceLocation {
    fn capture(v: &epub::ReferenceLocation, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(Self {
            element_ordinals: budget.ordinals(&v.element_ordinals)?,
            attribute_namespace: budget.optional_text(&v.attribute_namespace)?,
            attribute_local_name: budget.optional_text(&v.attribute_local_name)?,
            candidate_ordinal: v.candidate_ordinal,
            line: v.line,
            column: v.column,
            decoded_byte_range: v.decoded_byte_range,
        })
    }
    fn validate(&self) -> Result<(), CodecError> {
        if self.element_ordinals.len() > 256
            || self
                .decoded_byte_range
                .is_some_and(|(start, end)| start > end)
        {
            return Err(CodecError::InvalidRecord);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum ReferenceTarget {
    Internal {
        path: String,
        fragment: Option<String>,
    },
    Missing {
        path: String,
        fragment: Option<String>,
    },
    Ambiguous {},
    External {
        scheme: String,
    },
    Invalid {
        reason: ReferenceIssue,
    },
}
impl ReferenceTarget {
    fn capture(value: &epub::ReferenceTarget, budget: &mut Budget) -> Result<Self, CodecError> {
        Ok(match value {
            epub::ReferenceTarget::Internal { path, fragment } => Self::Internal {
                path: budget.text(path)?,
                fragment: budget.optional_text(fragment)?,
            },
            epub::ReferenceTarget::Missing { path, fragment } => Self::Missing {
                path: budget.text(path)?,
                fragment: budget.optional_text(fragment)?,
            },
            epub::ReferenceTarget::Ambiguous => Self::Ambiguous {},
            epub::ReferenceTarget::External { scheme } => Self::External {
                scheme: budget.text(scheme)?,
            },
            epub::ReferenceTarget::Invalid { reason } => Self::Invalid {
                reason: ReferenceIssue::capture(reason, budget)?,
            },
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceOccurrence {
    pub block_id: Digest,
    pub unit_offset: u32,
}
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceUse {
    pub source_path: String,
    pub location: ReferenceLocation,
    pub kind: ReferenceKind,
    pub target: ReferenceTarget,
    pub raw_reference: String,
    pub spine_positions: Vec<u64>,
    pub inventory_occurrence: Option<u64>,
    pub source_occurrences: Vec<SourceOccurrence>,
}
impl std::fmt::Debug for ResourceUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceUse")
            .field("source_path", &self.source_path)
            .field("location", &self.location)
            .field("kind", &self.kind)
            .field("target", &self.target)
            .field("raw_reference", &"[private]")
            .field("spine_positions", &self.spine_positions)
            .field("inventory_occurrence", &self.inventory_occurrence)
            .field("source_occurrences", &self.source_occurrences)
            .finish()
    }
}
impl ResourceUse {
    pub(in crate::import_codec) fn capture(
        value: &epub::ResourceReference,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        let mut positions = budget.vector(value.spine_positions.len())?;
        for &p in &value.spine_positions {
            positions.push(u64::try_from(p).map_err(|_| CodecError::ResourceLimit)?);
        }
        let mut occurrences = budget.vector(value.source_occurrences.len())?;
        for &(block, unit_offset) in &value.source_occurrences {
            occurrences.push(SourceOccurrence {
                block_id: Digest(block),
                unit_offset,
            });
        }
        Ok(Self {
            source_path: budget.text(&value.source_path)?,
            location: ReferenceLocation::capture(&value.location, budget)?,
            kind: value.kind.into(),
            target: ReferenceTarget::capture(&value.target, budget)?,
            raw_reference: budget.text(&value.raw_reference)?,
            spine_positions: positions,
            inventory_occurrence: value
                .inventory_occurrence
                .map(|n| u64::try_from(n).map_err(|_| CodecError::ResourceLimit))
                .transpose()?,
            source_occurrences: occurrences,
        })
    }
    pub fn validate_structure(&self) -> Result<(), CodecError> {
        if !safe_path(&self.source_path) || self.raw_reference.len() > 16 * 1024 {
            return Err(CodecError::InvalidRecord);
        }
        self.location.validate()?;
        if matches!(&self.target,ReferenceTarget::Internal{path,..}|ReferenceTarget::Missing{path,..} if !safe_path(path))
        {
            return Err(CodecError::InvalidRecord);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceDiagnostic {
    pub source_path: String,
    pub location: Option<ReferenceLocation>,
    pub code: ReferenceIssue,
}
impl ReferenceDiagnostic {
    pub(in crate::import_codec) fn capture(
        value: &epub::ReferenceDiagnostic,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        Ok(Self {
            source_path: budget.text(&value.source_path)?,
            location: value
                .location
                .as_ref()
                .map(|v| ReferenceLocation::capture(v, budget))
                .transpose()?,
            code: ReferenceIssue::capture(&value.code, budget)?,
        })
    }
}

// Retained allocation accounting.
crate::import_codec::memory::heap_struct!(ReferenceLocation {
    element_ordinals,
    attribute_namespace,
    attribute_local_name,
    candidate_ordinal,
    line,
    column,
    decoded_byte_range
});
crate::import_codec::memory::heap_enum!(ReferenceTarget {Internal {path, fragment}, Missing {path, fragment}, Ambiguous {}, External {scheme}, Invalid {reason}});
crate::import_codec::memory::heap_struct!(SourceOccurrence {
    block_id,
    unit_offset
});
crate::import_codec::memory::heap_struct!(ResourceUse {
    source_path,
    location,
    kind,
    target,
    raw_reference,
    spine_positions,
    inventory_occurrence,
    source_occurrences
});
crate::import_codec::memory::heap_struct!(ReferenceDiagnostic {
    source_path,
    location,
    code
});

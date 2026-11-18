//! Complete V2 wire shape. Framing/canonical syntax is not original-EPUB
//! verification. Only the use case may produce a trusted view after all gates.
use super::inventory_records::{InventoryDiagnostic, Severity};
use super::mapping_records::{MapIssue, SourceInputDiagnostic};
use super::records::{Budget, Digest};
use super::resource_records::{ReferenceDiagnostic, ResourceRecord, ResourceUse};
use super::{CodecError, StoredInventory, StoredMapping, StoredScope, StoredSourceIndex};
use nt_application::volume_import::{ObjectReference, chapter_map};
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Seek, SeekFrom, Write};

mod assembly;
mod proof;
pub use assembly::{SnapshotDraft, SnapshotInputs, SnapshotStageError, StagedVolume};
pub use proof::{CaptureEvidence, CaptureVerificationError};

const MAGIC: &[u8; 7] = b"NTCP05\0";
const HEADER: u64 = 17;
const MAX_PAYLOAD: u64 = 512 * 1024 * 1024;
const MAX_JSON: u64 = MAX_PAYLOAD - HEADER;
const VERSION: u16 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VolumeFormat {
    #[serde(rename = "nt.cp05.volume")]
    Volume,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VolumeSource {
    pub epub_sha256: Digest,
    pub original_object_sha256: Digest,
    pub original_object_byte_len: u64,
    pub parser_version: String,
    pub mapping_rule_version: String,
    pub classification_rule_version: String,
    pub mapping_version: Digest,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum SourceIssue {
    InventoryPartial {},
    InvalidOptionalNavigation { canonical_path: String },
    Inventory { value: InventoryDiagnostic },
    UnsupportedFixedLayout { canonical_path: String },
    UnsupportedEncryption { canonical_path: String },
    DuplicateXhtmlId { canonical_path: String, id: String },
    ProtectedResourceMissing { canonical_path: String },
}
impl SourceIssue {
    fn capture(value: &nt_epub::SourceDiagnostic, budget: &mut Budget) -> Result<Self, CodecError> {
        use nt_epub::SourceDiagnostic as D;
        Ok(match value {
            D::InventoryPartial => Self::InventoryPartial {},
            D::InvalidOptionalNavigation { canonical_path } => Self::InvalidOptionalNavigation {
                canonical_path: budget.text(canonical_path)?,
            },
            D::Inventory(value) => Self::Inventory {
                value: InventoryDiagnostic::capture(value, budget)?,
            },
            D::UnsupportedFixedLayout { canonical_path } => Self::UnsupportedFixedLayout {
                canonical_path: budget.text(canonical_path)?,
            },
            D::UnsupportedEncryption { canonical_path } => Self::UnsupportedEncryption {
                canonical_path: budget.text(canonical_path)?,
            },
            D::DuplicateXhtmlId { canonical_path, id } => Self::DuplicateXhtmlId {
                canonical_path: budget.text(canonical_path)?,
                id: budget.text(id)?,
            },
            D::ProtectedResourceMissing { canonical_path } => Self::ProtectedResourceMissing {
                canonical_path: budget.text(canonical_path)?,
            },
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum UnresolvedReference {
    Reference(ResourceUse),
    Diagnostic(ReferenceDiagnostic),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VolumeDiagnostics {
    pub source: Vec<SourceIssue>,
    pub navigation: Vec<SourceInputDiagnostic>,
    pub unresolved_resource_uses: Vec<UnresolvedReference>,
    pub mapping: Vec<MapIssue>,
}
impl VolumeDiagnostics {
    pub fn capture(
        source: &nt_epub::SourceExtraction,
        mapping: &chapter_map::ChapterMap,
        references: &nt_epub::ReferenceIndex,
        reserved_working_bytes: usize,
    ) -> Result<Self, CodecError> {
        Self::capture_with_budget(
            source,
            mapping,
            references,
            &mut Budget::new(reserved_working_bytes)?,
        )
    }
    pub(super) fn capture_with_budget(
        source: &nt_epub::SourceExtraction,
        mapping: &chapter_map::ChapterMap,
        references: &nt_epub::ReferenceIndex,
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        let mut diagnostics = budget.vector(source.diagnostics.len())?;
        for d in &source.diagnostics {
            diagnostics.push(SourceIssue::capture(d, budget)?);
        }
        let mut navigation = budget.vector(source.navigation_diagnostics.len())?;
        for d in &source.navigation_diagnostics {
            navigation.push(SourceInputDiagnostic::capture(d, budget)?);
        }
        let unresolved_resource_uses = Self::reference_issues(references, budget)?;
        let mut map_diagnostics = budget.vector(mapping.diagnostics.len())?;
        for d in &mapping.diagnostics {
            map_diagnostics.push(MapIssue::capture(d, budget)?);
        }
        let result = Self {
            source: diagnostics,
            navigation,
            unresolved_resource_uses,
            mapping: map_diagnostics,
        };
        if result.source_complete() != source.complete {
            return Err(CodecError::InvalidRecord);
        }
        Ok(result)
    }

    fn reference_issues(
        references: &nt_epub::ReferenceIndex,
        budget: &mut Budget,
    ) -> Result<Vec<UnresolvedReference>, CodecError> {
        let count = references
            .references
            .iter()
            .filter(|r| !matches!(r.target, nt_epub::ReferenceTarget::Internal { .. }))
            .count()
            .checked_add(references.diagnostics.len())
            .ok_or(CodecError::ResourceLimit)?;
        let mut unresolved = budget.vector(count)?;
        for r in &references.references {
            if !matches!(r.target, nt_epub::ReferenceTarget::Internal { .. }) {
                unresolved.push(UnresolvedReference::Reference(ResourceUse::capture(
                    r, budget,
                )?));
            }
        }
        for d in &references.diagnostics {
            unresolved.push(UnresolvedReference::Diagnostic(
                ReferenceDiagnostic::capture(d, budget)?,
            ));
        }
        Ok(unresolved)
    }
    pub fn source_complete(&self) -> bool {
        self.source.iter().all(|d| match d {
            SourceIssue::InventoryPartial {}
            | SourceIssue::InvalidOptionalNavigation { .. }
            | SourceIssue::UnsupportedFixedLayout { .. }
            | SourceIssue::UnsupportedEncryption { .. }
            | SourceIssue::ProtectedResourceMissing { .. } => false,
            SourceIssue::Inventory { value } => value.severity != Severity::Incomplete,
            SourceIssue::DuplicateXhtmlId { .. } => true,
        })
    }
}
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VolumeRecord {
    pub format: VolumeFormat,
    pub version: u16,
    pub source: VolumeSource,
    pub inventory: StoredInventory,
    pub source_index: StoredSourceIndex,
    pub mapping: StoredMapping,
    pub scope: StoredScope,
    pub resources: Vec<ResourceRecord>,
    pub diagnostics: VolumeDiagnostics,
}
impl VolumeRecord {
    /// Canonical typed envelope only. The source must be immutable during all
    /// passes; this does not assert that records match the original EPUB.
    pub fn read_unverified(source: &mut (impl Read + Seek)) -> Result<Self, CodecError> {
        Self::read_with_reservation(source, 0)
    }

    pub(crate) fn read_with_reservation(
        source: &mut (impl Read + Seek),
        reserved: usize,
    ) -> Result<Self, CodecError> {
        let limits = Self::read_limits(reserved)?;
        let length = Self::read_header(source)?;
        let record: Self = super::canonical_json::decode(source, length, limits)?;
        if record.version != VERSION {
            return Err(CodecError::UnsupportedWireVersion);
        }
        Ok(record)
    }

    fn check_staged_read_limits(source: &mut (impl Read + Seek)) -> Result<(), CodecError> {
        let length = Self::read_header(source)?;
        super::canonical_json::preflight(source, length, Self::read_limits(0)?).map(|_| ())
    }

    fn read_limits(reserved: usize) -> Result<super::canonical_json::Limits, CodecError> {
        let working = (nt_application::volume_import::MAX_IMPORT_WORKING_BYTES - 64 * 1024)
            .checked_sub(reserved)
            .ok_or(CodecError::ResourceLimit)?;
        Ok(super::canonical_json::Limits {
            working,
            ..Default::default()
        })
    }

    fn read_header(source: &mut (impl Read + Seek)) -> Result<u64, CodecError> {
        source
            .seek(SeekFrom::Start(0))
            .map_err(|_| CodecError::Io)?;
        let mut header = [0u8; HEADER as usize];
        source.read_exact(&mut header).map_err(|error| {
            if error.kind() == io::ErrorKind::UnexpectedEof {
                CodecError::InvalidEnvelope
            } else {
                CodecError::Io
            }
        })?;
        if &header[..7] != MAGIC {
            return Err(CodecError::InvalidEnvelope);
        }
        if u16::from_be_bytes(header[7..9].try_into().expect("header width")) != VERSION {
            return Err(CodecError::UnsupportedWireVersion);
        }
        let length = u64::from_be_bytes(header[9..].try_into().expect("header width"));
        if length == 0 {
            return Err(CodecError::InvalidEnvelope);
        }
        if length > MAX_JSON {
            return Err(CodecError::ResourceLimit);
        }
        let actual = source.seek(SeekFrom::End(0)).map_err(|_| CodecError::Io)?;
        if actual
            != length
                .checked_add(HEADER)
                .ok_or(CodecError::ResourceLimit)?
        {
            return Err(CodecError::InvalidEnvelope);
        }
        source
            .seek(SeekFrom::Start(HEADER))
            .map_err(|_| CodecError::Io)?;
        Ok(length)
    }
    /// Encode for staging. This is not publication or a substitute for the use
    /// case's source/object/coverage/scope checks.
    pub fn write_unverified(
        &self,
        destination: &mut (impl Write + ?Sized),
    ) -> Result<u64, CodecError> {
        if self.version != VERSION {
            return Err(CodecError::UnsupportedWireVersion);
        }
        let length = json_length(self, MAX_JSON)?;
        destination
            .write_all(MAGIC)
            .and_then(|_| destination.write_all(&VERSION.to_be_bytes()))
            .and_then(|_| destination.write_all(&length.to_be_bytes()))
            .map_err(|_| CodecError::Io)?;
        serde_json::to_writer(destination, self).map_err(|_| CodecError::Io)?;
        Ok(length + HEADER)
    }
    pub fn mapping_digest(&self) -> Result<Digest, CodecError> {
        mapping_digest(
            &self.source,
            &self.source_index,
            &self.mapping,
            &self.resources,
        )
    }
    /// Structural identity and coverage. Original node/ownership validation
    /// and stored object dependencies are separate mandatory caller gates.
    pub fn validate_structure(&self) -> Result<(), CodecError> {
        if self.version != VERSION {
            return Err(CodecError::UnsupportedWireVersion);
        }
        let s = &self.source;
        if s.epub_sha256 != s.original_object_sha256
            || s.original_object_byte_len == 0
            || s.parser_version.is_empty()
            || s.mapping_rule_version.is_empty()
            || s.classification_rule_version.is_empty()
        {
            return Err(CodecError::InvalidRecord);
        }
        self.source_index
            .validate_structure(s.epub_sha256, &s.parser_version)?;
        self.mapping
            .validate_coverage(&self.source_index, s.epub_sha256, &s.parser_version)?;
        if self.mapping_digest()? != s.mapping_version
            || self.diagnostics.mapping != self.mapping.diagnostics
        {
            return Err(CodecError::InvalidRecord);
        }
        for resource in &self.resources {
            resource.validate_structure()?;
        }
        Ok(())
    }
}
#[derive(Serialize)]
struct ResourceHash<'a> {
    canonical_path: &'a str,
    sha256: Digest,
    byte_len: u64,
}
#[derive(Serialize)]
struct MappingInputs<'a> {
    epub_sha256: Digest,
    parser_version: &'a str,
    mapping_rule_version: &'a str,
    source_index: &'a StoredSourceIndex,
    mapping: &'a StoredMapping,
    resource_hashes: Vec<ResourceHash<'a>>,
}
fn mapping_digest(
    source: &VolumeSource,
    index: &StoredSourceIndex,
    mapping: &StoredMapping,
    resources: &[ResourceRecord],
) -> Result<Digest, CodecError> {
    let mut hashes = Budget::new(0)?.vector(resources.len())?;
    for resource in resources {
        hashes.push(ResourceHash {
            canonical_path: &resource.canonical_path,
            sha256: resource.object_sha256,
            byte_len: resource.byte_len,
        });
    }
    hashes.sort_unstable_by_key(|r| (r.canonical_path, r.sha256));
    if hashes
        .windows(2)
        .any(|pair| pair[0].canonical_path == pair[1].canonical_path)
    {
        return Err(CodecError::InvalidRecord);
    }
    let input = MappingInputs {
        epub_sha256: source.epub_sha256,
        parser_version: &source.parser_version,
        mapping_rule_version: &source.mapping_rule_version,
        source_index: index,
        mapping,
        resource_hashes: hashes,
    };
    let length = json_length(&input, MAX_PAYLOAD)?;
    let mut sink = nt_storage::ImportDigestSink::new(MAX_PAYLOAD + 1024)
        .map_err(|_| CodecError::ResourceLimit)?;
    sink.write_all(b"NT-CP05-MAPPING-V2\0")
        .and_then(|_| sink.write_all(&length.to_be_bytes()))
        .map_err(|_| CodecError::ResourceLimit)?;
    serde_json::to_writer(&mut sink, &input).map_err(|_| CodecError::ResourceLimit)?;
    let ObjectReference { hash, .. } = sink.finish().map_err(|_| CodecError::ResourceLimit)?;
    Ok(Digest(*hash.as_bytes()))
}
struct Count {
    bytes: u64,
    maximum: u64,
}
impl Write for Count {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .filter(|n| *n <= self.maximum)
            .ok_or_else(|| io::Error::other("Stored data limit exceeded."))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn json_length(value: &impl Serialize, maximum: u64) -> Result<u64, CodecError> {
    let mut count = Count { bytes: 0, maximum };
    serde_json::to_writer(&mut count, value).map_err(|_| CodecError::ResourceLimit)?;
    Ok(count.bytes)
}

// Retained allocation accounting.
crate::import_codec::memory::heap_enum!(VolumeFormat { Volume });
crate::import_codec::memory::heap_struct!(VolumeSource {
    epub_sha256,
    original_object_sha256,
    original_object_byte_len,
    parser_version,
    mapping_rule_version,
    classification_rule_version,
    mapping_version
});
crate::import_codec::memory::heap_enum!(SourceIssue {InventoryPartial {}, InvalidOptionalNavigation {canonical_path}, Inventory {value}, UnsupportedFixedLayout {canonical_path}, UnsupportedEncryption {canonical_path}, DuplicateXhtmlId {canonical_path, id}, ProtectedResourceMissing {canonical_path}});
crate::import_codec::memory::heap_enum!(UnresolvedReference {Reference(value), Diagnostic(value)});
crate::import_codec::memory::heap_struct!(VolumeDiagnostics {
    source,
    navigation,
    unresolved_resource_uses,
    mapping
});
crate::import_codec::memory::heap_struct!(VolumeRecord {
    format,
    version,
    source,
    inventory,
    source_index,
    mapping,
    scope,
    resources,
    diagnostics
});

#[cfg(test)]
mod tests {
    use super::VolumeRecord;
    use crate::import_codec::{CodecError, canonical_json};
    use std::io::Cursor;

    #[test]
    fn historical_decoder_shares_working_bytes_with_the_open_scope() {
        let json = br#"["story","note"]"#;
        let length = json.len() as u64;
        let maximum = VolumeRecord::read_limits(0).unwrap();
        let needed = canonical_json::preflight(&mut &json[..], length, maximum).unwrap();
        let reserved = maximum.working - needed;
        let read = |reserved| -> Result<Vec<String>, CodecError> {
            let limits = VolumeRecord::read_limits(reserved)?;
            canonical_json::decode(&mut Cursor::new(json), length, limits)
        };
        assert_eq!(read(reserved).unwrap(), ["story", "note"]);
        assert_eq!(read(reserved + 1), Err(CodecError::ResourceLimit));
        assert_eq!(read(usize::MAX), Err(CodecError::ResourceLimit));
    }
}

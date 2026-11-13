use std::fmt;

use crate::object::ObjectReference;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeriesId(String);

impl SeriesId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidWorkspaceId> {
        let value = value.into();
        if is_lower_hex_id(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidWorkspaceId)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SeriesId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeId(String);

impl VolumeId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidWorkspaceId> {
        let value = value.into();
        if is_lower_hex_id(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidWorkspaceId)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VolumeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationId(String);

impl GenerationId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidWorkspaceId> {
        let value = value.into();
        if value == "g0" || is_lower_hex_id(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidWorkspaceId)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GenerationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OperationId(String);

impl OperationId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidOperationId> {
        let value = value.into();
        if is_lower_hex_id(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidOperationId)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationWriteRequest {
    operation_id: OperationId,
    expected_parent: GenerationId,
    generation_id: GenerationId,
    objects: Vec<ObjectReference>,
}

impl GenerationWriteRequest {
    pub fn new(
        operation_id: OperationId,
        expected_parent: GenerationId,
        generation_id: GenerationId,
        mut objects: Vec<ObjectReference>,
    ) -> Result<Self, InvalidGenerationWriteRequest> {
        if generation_id.as_str() == "g0" || generation_id == expected_parent {
            return Err(InvalidGenerationWriteRequest);
        }

        objects.sort_by_key(|reference| (reference.hash, reference.byte_len));
        if objects.windows(2).any(|pair| pair[0].hash == pair[1].hash) {
            return Err(InvalidGenerationWriteRequest);
        }

        Ok(Self {
            operation_id,
            expected_parent,
            generation_id,
            objects,
        })
    }

    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    pub fn expected_parent(&self) -> &GenerationId {
        &self.expected_parent
    }

    pub fn generation_id(&self) -> &GenerationId {
        &self.generation_id
    }

    pub fn objects(&self) -> &[ObjectReference] {
        &self.objects
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationCommitResult {
    pub generation_id: GenerationId,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationCommitError {
    Busy,
    StaleGeneration,
    OperationConflict,
    CommitOutcomeUnknown,
    WorkerUnavailable,
    CorruptWorkspace,
    StorageFailure,
}

impl fmt::Display for GenerationCommitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Busy => "The workspace is busy. Try the operation again.",
            Self::StaleGeneration => "The workspace changed. Reopen it before trying again.",
            Self::OperationConflict => "This operation ID was already used for different data.",
            Self::CommitOutcomeUnknown => {
                "The save result is not known. Reopen the workspace to check its operation ID."
            }
            Self::WorkerUnavailable => "The workspace writer is unavailable. Reopen the workspace.",
            Self::CorruptWorkspace => "The workspace data failed an integrity check.",
            Self::StorageFailure => "The workspace could not save this change safely.",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for GenerationCommitError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidOperationId;

impl fmt::Display for InvalidOperationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("operation ID must be 32 lowercase hexadecimal characters")
    }
}

impl std::error::Error for InvalidOperationId {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidGenerationWriteRequest;

impl fmt::Display for InvalidGenerationWriteRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("generation write request has an invalid ID or duplicate object")
    }
}

impl std::error::Error for InvalidGenerationWriteRequest {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeriesMetadata {
    pub id: SeriesId,
    pub created_at_utc_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeMetadata {
    pub id: VolumeId,
    pub series_id: SeriesId,
    pub display_title: Option<String>,
    pub created_at_utc_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMetadata {
    pub series: SeriesMetadata,
    pub schema_version: u32,
    pub current_generation: GenerationId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidWorkspaceId;

impl fmt::Display for InvalidWorkspaceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("workspace ID has an unsupported format")
    }
}

impl std::error::Error for InvalidWorkspaceId {}

fn is_lower_hex_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{GenerationId, SeriesId, VolumeId};

    #[test]
    fn accepts_lower_hex_ids_and_the_bootstrap_generation() {
        assert!(SeriesId::parse("0123456789abcdef0123456789abcdef").is_ok());
        assert!(VolumeId::parse("0123456789abcdef0123456789abcdef").is_ok());
        assert_eq!(GenerationId::parse("g0").unwrap().as_str(), "g0");
    }

    #[test]
    fn rejects_ids_outside_the_selected_format() {
        assert!(SeriesId::parse("0123456789ABCDEF0123456789ABCDEF").is_err());
        assert!(VolumeId::parse("too-short").is_err());
        assert!(GenerationId::parse("").is_err());
    }
}

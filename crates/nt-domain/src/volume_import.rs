//! Typed, storage-independent requests for the T09 generation writer.

use crate::object::ObjectReference;
use crate::source_scope::ScopeMode;
use crate::workspace::{GenerationId, OperationId, SeriesId, VolumeId};

pub const MAX_IMPORT_PAYLOAD_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_IMPORT_WORKING_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_IMPORT_RECORDS: usize = 32_000_000;
pub const MAX_VOLUME_TITLE_BYTES: usize = 512;

#[path = "captured_source.rs"]
mod captured_source;
pub use captured_source::{
    CapturedBlock, CapturedBlockPlacement, CapturedImagePlacement, CapturedNavigationLabel,
    CapturedScopeElement, CapturedSourceError, CapturedSourceRequest, CapturedUnitRun,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DetectedImageFormat {
    Jpeg,
    Png,
    Gif,
    Webp,
    Svg,
}

/// Header-established source information, not a claim of image/export validity.
/// SVG source strings are retained even when absolute pixel dimensions exist.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ImageInformation {
    pub format: Option<DetectedImageFormat>,
    pub pixel_dimensions: Option<(u32, u32)>,
    pub svg_width: Option<String>,
    pub svg_height: Option<String>,
    pub svg_view_box: Option<String>,
}

/// Digest and length of a staged immutable payload object. The application
/// validates its V2 contents before submission; storage streams and rehashes it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StagedPayload {
    pub object: ObjectReference,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedVolumeState {
    pub mode: ScopeMode,
    pub epub_sha256: [u8; 32],
    pub mapping_version: [u8; 32],
    pub payload: StagedPayload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VolumeState {
    Unimported { mode: ScopeMode },
    Imported(ImportedVolumeState),
}

impl VolumeState {
    pub fn mode(&self) -> ScopeMode {
        match self {
            Self::Unimported { mode } => *mode,
            Self::Imported(state) => state.mode,
        }
    }
}

/// A resource is attached to the exact volume version and one object reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeResource {
    pub role: String,
    pub canonical_path: String,
    pub object: ObjectReference,
    pub source_relative_path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportOperation {
    Publish {
        volume_id: VolumeId,
        state: ImportedVolumeState,
        resources: Vec<VolumeResource>,
        default_delta: Option<ScopeMode>,
    },
    SetVolumeScope {
        volume_id: VolumeId,
        state: VolumeState,
        resources: Vec<VolumeResource>,
        default_delta: Option<ScopeMode>,
    },
    SetSeriesDefault {
        mode: ScopeMode,
    },
}

/// Each request supplies complete references for its own generation. Active
/// older volume states are located independently through the generation chain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeImportWriteRequest {
    series_id: SeriesId,
    operation_id: OperationId,
    expected_parent: GenerationId,
    generation_id: GenerationId,
    operation: ImportOperation,
    objects: Vec<ObjectReference>,
    working_bytes: usize,
    registration: Option<VolumeRegistration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeRegistration {
    pub display_title: Option<String>,
}

impl VolumeImportWriteRequest {
    pub fn new(
        series_id: SeriesId,
        operation_id: OperationId,
        expected_parent: GenerationId,
        generation_id: GenerationId,
        mut operation: ImportOperation,
        mut objects: Vec<ObjectReference>,
    ) -> Result<Self, InvalidImportRequest> {
        if generation_id.as_str() == "g0" || generation_id == expected_parent {
            return Err(InvalidImportRequest::Invalid);
        }
        let (resources, resource_capacity) = match &mut operation {
            ImportOperation::Publish { resources, .. }
            | ImportOperation::SetVolumeScope { resources, .. } => {
                resources.sort_unstable_by(|left, right| {
                    (&left.role, &left.canonical_path).cmp(&(&right.role, &right.canonical_path))
                });
                if resources.windows(2).any(|pair| {
                    pair[0].role == pair[1].role && pair[0].canonical_path == pair[1].canonical_path
                }) {
                    return Err(InvalidImportRequest::Invalid);
                }
                (resources.as_slice(), resources.capacity())
            }
            ImportOperation::SetSeriesDefault { .. } => (&[][..], 0),
        };
        // Charge the actual retained representation, including spare capacity,
        // before sorting or making any copy. A record cap alone is not a byte cap.
        let mut allocation = objects
            .capacity()
            .checked_mul(std::mem::size_of::<ObjectReference>())
            .and_then(|bytes| {
                resource_capacity
                    .checked_mul(std::mem::size_of::<VolumeResource>())
                    .and_then(|extra| bytes.checked_add(extra))
            })
            .ok_or(InvalidImportRequest::ResourceLimit)?;
        if resources
            .len()
            .checked_add(objects.len())
            .is_none_or(|count| count > MAX_IMPORT_RECORDS)
        {
            return Err(InvalidImportRequest::ResourceLimit);
        }
        for resource in resources {
            for length in [
                resource.role.capacity(),
                resource.canonical_path.capacity(),
                resource
                    .source_relative_path
                    .as_ref()
                    .map_or(0, String::capacity),
            ] {
                allocation = allocation
                    .checked_add(length)
                    .ok_or(InvalidImportRequest::ResourceLimit)?;
            }
        }
        if allocation > MAX_IMPORT_WORKING_BYTES {
            return Err(InvalidImportRequest::ResourceLimit);
        }
        objects.sort_unstable_by_key(|object| (object.hash, object.byte_len));
        if objects.windows(2).any(|pair| pair[0].hash == pair[1].hash)
            || objects
                .iter()
                .any(|object| object.byte_len > i64::MAX as u64)
        {
            return Err(InvalidImportRequest::Invalid);
        }
        let (state, resources) = match &operation {
            ImportOperation::Publish {
                state, resources, ..
            } => (Some(state), resources.as_slice()),
            ImportOperation::SetVolumeScope {
                state, resources, ..
            } => (
                match state {
                    VolumeState::Imported(state) => Some(state),
                    _ => None,
                },
                resources.as_slice(),
            ),
            ImportOperation::SetSeriesDefault { .. } => (None, &[][..]),
        };
        if let Some(state) = state {
            if state.payload.object.byte_len == 0 {
                return Err(InvalidImportRequest::Invalid);
            }
            if state.payload.object.byte_len > MAX_IMPORT_PAYLOAD_BYTES {
                return Err(InvalidImportRequest::ResourceLimit);
            }
        }
        if matches!(
            &operation,
            ImportOperation::SetVolumeScope {
                state: VolumeState::Unimported { .. },
                ..
            }
        ) && !resources.is_empty()
        {
            return Err(InvalidImportRequest::Invalid);
        }
        if resources.iter().any(|resource| {
            resource.role.is_empty()
                || resource.canonical_path.is_empty()
                || resource
                    .source_relative_path
                    .as_ref()
                    .is_some_and(String::is_empty)
                || objects
                    .binary_search_by_key(&resource.object.hash, |object| object.hash)
                    .ok()
                    .is_none_or(|index| objects[index] != resource.object)
        }) {
            return Err(InvalidImportRequest::Invalid);
        }
        Ok(Self {
            series_id,
            operation_id,
            expected_parent,
            generation_id,
            operation,
            objects,
            working_bytes: allocation,
            registration: None,
        })
    }

    /// A first import must atomically create its volume and keep the parent's
    /// series default. Only an ordinary publication can be a registration.
    pub fn with_registration(
        mut self,
        display_title: Option<String>,
    ) -> Result<Self, InvalidImportRequest> {
        if !matches!(
            self.operation,
            ImportOperation::Publish {
                default_delta: None,
                ..
            }
        ) || self.registration.is_some()
            || display_title
                .as_ref()
                .is_some_and(|title| title.trim().is_empty() || title.chars().any(char::is_control))
        {
            return Err(InvalidImportRequest::Invalid);
        }
        let title_capacity = display_title.as_ref().map_or(0, String::capacity);
        if title_capacity > MAX_VOLUME_TITLE_BYTES {
            return Err(InvalidImportRequest::ResourceLimit);
        }
        self.working_bytes = self
            .working_bytes
            .checked_add(title_capacity)
            .filter(|bytes| *bytes <= MAX_IMPORT_WORKING_BYTES)
            .ok_or(InvalidImportRequest::ResourceLimit)?;
        self.registration = Some(VolumeRegistration { display_title });
        Ok(self)
    }

    pub fn registration(&self) -> Option<&VolumeRegistration> {
        self.registration.as_ref()
    }

    pub fn series_id(&self) -> &SeriesId {
        &self.series_id
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
    pub fn operation(&self) -> &ImportOperation {
        &self.operation
    }
    pub fn objects(&self) -> &[ObjectReference] {
        &self.objects
    }
    /// Owned request buffers already live when storage prepares file handles.
    pub fn working_bytes(&self) -> usize {
        self.working_bytes
    }
}

/// Latest complete state for a volume as seen from one pinned generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeImportVersion {
    pub volume_id: VolumeId,
    /// None denotes a virtual pre-M003 all-text/no-import state.
    pub generation_id: Option<GenerationId>,
    pub state: VolumeState,
    pub resources: Vec<VolumeResource>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidImportRequest {
    Invalid,
    ResourceLimit,
}

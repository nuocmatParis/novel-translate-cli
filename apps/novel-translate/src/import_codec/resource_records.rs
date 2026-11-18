//! Resource records preserve actual objects and every typed reference location.
mod references;
mod staging;
use super::CodecError;
use super::records::{Budget, Digest, safe_path};
use nt_application::volume_import::{DetectedImageFormat, ImageInformation, ObjectReference};
use nt_epub::ResourceReference;
pub use references::{ReferenceDiagnostic, ReferenceIssue, ResourceUse};
use serde::{Deserialize, Serialize};
pub use staging::{ResourceFileError, ResourceFiles};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResourceRole {
    Image,
    Stylesheet,
    Font,
    Xhtml,
    Other,
}
impl ResourceRole {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Stylesheet => "stylesheet",
            Self::Font => "font",
            Self::Xhtml => "xhtml",
            Self::Other => "other",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FormatStatus {
    Known,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatName {
    Jpeg,
    Png,
    Gif,
    Webp,
    Svg,
    Other,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatInfo {
    pub status: FormatStatus,
    pub name: Option<FormatName>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DimensionStatus {
    Known,
    RelativeOrVector,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PixelUnit {
    #[serde(rename = "px")]
    Px,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dimensions {
    pub status: DimensionStatus,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub unit: Option<PixelUnit>,
    pub svg_width: Option<String>,
    pub svg_height: Option<String>,
    pub svg_view_box: Option<String>,
}
pub struct ResourceInput<'a> {
    pub role: ResourceRole,
    pub canonical_path: &'a str,
    pub object: ObjectReference,
    pub source_relative_path: Option<&'a str>,
    pub declared_media_type: Option<&'a str>,
    pub image_information: &'a ImageInformation,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRecord {
    pub role: ResourceRole,
    pub canonical_path: String,
    pub object_sha256: Digest,
    pub byte_len: u64,
    pub source_relative_path: Option<String>,
    pub declared_media_type: Option<String>,
    pub format: FormatInfo,
    pub dimensions: Dimensions,
    pub uses: Vec<ResourceUse>,
}
impl ResourceRecord {
    pub fn capture(
        input: ResourceInput<'_>,
        references: &[&ResourceReference],
        reserved_working_bytes: usize,
    ) -> Result<Self, CodecError> {
        Self::capture_with_budget(input, references, &mut Budget::new(reserved_working_bytes)?)
    }
    pub(super) fn capture_with_budget(
        input: ResourceInput<'_>,
        references: &[&ResourceReference],
        budget: &mut Budget,
    ) -> Result<Self, CodecError> {
        let image = input.image_information;
        let format = FormatInfo {
            status: if image.format.is_some() {
                FormatStatus::Known
            } else {
                FormatStatus::Unknown
            },
            name: image.format.map(|f| match f {
                DetectedImageFormat::Jpeg => FormatName::Jpeg,
                DetectedImageFormat::Png => FormatName::Png,
                DetectedImageFormat::Gif => FormatName::Gif,
                DetectedImageFormat::Webp => FormatName::Webp,
                DetectedImageFormat::Svg => FormatName::Svg,
            }),
        };
        let dimensions = Dimensions {
            status: if image.pixel_dimensions.is_some() {
                DimensionStatus::Known
            } else if image.svg_width.is_some()
                || image.svg_height.is_some()
                || image.svg_view_box.is_some()
            {
                DimensionStatus::RelativeOrVector
            } else {
                DimensionStatus::Unknown
            },
            width: image.pixel_dimensions.map(|d| d.0),
            height: image.pixel_dimensions.map(|d| d.1),
            unit: image.pixel_dimensions.map(|_| PixelUnit::Px),
            svg_width: budget.optional_text(&image.svg_width)?,
            svg_height: budget.optional_text(&image.svg_height)?,
            svg_view_box: budget.optional_text(&image.svg_view_box)?,
        };
        let mut uses = budget.vector(references.len())?;
        for reference in references {
            if !matches!(&reference.target,nt_epub::ReferenceTarget::Internal{path,..} if path==input.canonical_path)
            {
                return Err(CodecError::InvalidRecord);
            }
            uses.push(ResourceUse::capture(reference, budget)?);
        }
        let result = Self {
            role: input.role,
            canonical_path: budget.text(input.canonical_path)?,
            object_sha256: Digest(*input.object.hash.as_bytes()),
            byte_len: input.object.byte_len,
            source_relative_path: input
                .source_relative_path
                .map(|p| budget.text(p))
                .transpose()?,
            declared_media_type: input
                .declared_media_type
                .map(|p| budget.text(p))
                .transpose()?,
            format,
            dimensions,
            uses,
        };
        result.validate_structure()?;
        Ok(result)
    }
    pub fn validate_structure(&self) -> Result<(), CodecError> {
        if !safe_path(&self.canonical_path)
            || self
                .source_relative_path
                .as_deref()
                .is_some_and(|p| !safe_path(p))
            || self.byte_len > i64::MAX as u64
        {
            return Err(CodecError::InvalidRecord);
        }
        match self.format.status {
            FormatStatus::Known if self.format.name.is_some() => {}
            FormatStatus::Unknown if self.format.name.is_none() => {}
            _ => return Err(CodecError::InvalidRecord),
        }
        let d = &self.dimensions;
        let vector = d.svg_width.is_some() || d.svg_height.is_some() || d.svg_view_box.is_some();
        if vector && self.format.name != Some(FormatName::Svg) {
            return Err(CodecError::InvalidRecord);
        }
        match d.status {
            DimensionStatus::Known
                if d.width.is_some_and(|n| n > 0)
                    && d.height.is_some_and(|n| n > 0)
                    && d.unit == Some(PixelUnit::Px) => {}
            DimensionStatus::RelativeOrVector
                if d.width.is_none() && d.height.is_none() && d.unit.is_none() && vector => {}
            DimensionStatus::Unknown
                if d.width.is_none() && d.height.is_none() && d.unit.is_none() && !vector => {}
            _ => return Err(CodecError::InvalidRecord),
        }
        for reference in &self.uses {
            reference.validate_structure()?;
            if !matches!(&reference.target,references::ReferenceTarget::Internal{path,..} if path==&self.canonical_path)
            {
                return Err(CodecError::InvalidRecord);
            }
        }
        Ok(())
    }
}

// Retained allocation accounting.
crate::import_codec::memory::heap_enum!(ResourceRole {
    Image,
    Stylesheet,
    Font,
    Xhtml,
    Other
});
crate::import_codec::memory::heap_enum!(FormatStatus { Known, Unknown });
crate::import_codec::memory::heap_enum!(FormatName {
    Jpeg,
    Png,
    Gif,
    Webp,
    Svg,
    Other
});
crate::import_codec::memory::heap_struct!(FormatInfo { status, name });
crate::import_codec::memory::heap_enum!(DimensionStatus {
    Known,
    RelativeOrVector,
    Unknown
});
crate::import_codec::memory::heap_enum!(PixelUnit { Px });
crate::import_codec::memory::heap_struct!(Dimensions {
    status,
    width,
    height,
    unit,
    svg_width,
    svg_height,
    svg_view_box
});
crate::import_codec::memory::heap_struct!(ResourceRecord {
    role,
    canonical_path,
    object_sha256,
    byte_len,
    source_relative_path,
    declared_media_type,
    format,
    dimensions,
    uses
});

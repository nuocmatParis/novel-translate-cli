use std::fmt;

/// The reason an archive entry name cannot be used as an EPUB-relative path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchivePathError {
    Empty,
    Absolute,
    Backslash,
    ParentTraversal,
    InvalidWindowsCharacter,
    TrailingDotOrSpace,
    ReservedWindowsName,
}

impl fmt::Display for ArchivePathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("entry path is empty"),
            Self::Absolute => f.write_str("absolute entry paths are not allowed"),
            Self::Backslash => f.write_str("backslash separators are not allowed"),
            Self::ParentTraversal => f.write_str("parent traversal is not allowed"),
            Self::InvalidWindowsCharacter => {
                f.write_str("entry path contains a character forbidden on Windows")
            }
            Self::TrailingDotOrSpace => {
                f.write_str("Windows path components cannot end in a dot or space")
            }
            Self::ReservedWindowsName => {
                f.write_str("entry path contains a reserved Windows device name")
            }
        }
    }
}

impl std::error::Error for ArchivePathError {}

/// Canonicalize an archive name without changing its case or Unicode spelling.
/// Repeated separators and `.` components are aliases; `..` is always rejected.
pub fn canonical_archive_path(raw: &str) -> Result<String, ArchivePathError> {
    if raw.is_empty() {
        return Err(ArchivePathError::Empty);
    }
    if raw.starts_with('/') {
        return Err(ArchivePathError::Absolute);
    }
    if raw.contains('\\') {
        return Err(ArchivePathError::Backslash);
    }

    let mut components = Vec::new();
    for component in raw.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            return Err(ArchivePathError::ParentTraversal);
        }
        validate_windows_component(component)?;
        components.push(component);
    }

    if components.is_empty() {
        return Err(ArchivePathError::Empty);
    }

    Ok(components.join("/"))
}

/// Return a conservative Windows case-insensitive key for a destination path.
/// The original canonical path remains available separately for EPUB identity.
pub fn windows_materialization_key(canonical_path: &str) -> String {
    canonical_path.to_lowercase()
}

fn validate_windows_component(component: &str) -> Result<(), ArchivePathError> {
    if component.ends_with('.') || component.ends_with(' ') {
        return Err(ArchivePathError::TrailingDotOrSpace);
    }

    if component.chars().any(|character| {
        character <= '\u{001f}' || matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*')
    }) {
        return Err(ArchivePathError::InvalidWindowsCharacter);
    }

    let stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches([' ', '.'])
        .to_ascii_uppercase();
    if is_reserved_device_name(&stem) {
        return Err(ArchivePathError::ReservedWindowsName);
    }

    Ok(())
}

fn is_reserved_device_name(stem: &str) -> bool {
    if matches!(
        stem,
        "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
    ) {
        return true;
    }

    ["COM", "LPT"].into_iter().any(|prefix| {
        stem.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{ArchivePathError, canonical_archive_path, windows_materialization_key};

    #[test]
    fn canonical_path_preserves_case_unicode_and_collapses_aliases() {
        assert_eq!(
            canonical_archive_path("Text/./第１章.xhtml").unwrap(),
            "Text/第１章.xhtml"
        );
        assert_eq!(
            canonical_archive_path("Text//第１章.xhtml").unwrap(),
            "Text/第１章.xhtml"
        );
        assert_eq!(
            windows_materialization_key("Text/第１章.xhtml"),
            "text/第１章.xhtml"
        );
    }

    #[test]
    fn rejects_absolute_traversal_and_windows_dangerous_names() {
        assert_eq!(
            canonical_archive_path("/META-INF/container.xml"),
            Err(ArchivePathError::Absolute)
        );
        assert_eq!(
            canonical_archive_path("OPS/../secret"),
            Err(ArchivePathError::ParentTraversal)
        );
        for invalid in ["C:/secret", "OPS\\chapter.xhtml", "OPS/CON.txt"] {
            assert!(canonical_archive_path(invalid).is_err(), "{invalid}");
        }
    }
}

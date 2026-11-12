use std::fmt;
use std::path::Path;

use nt_domain::workspace::WorkspaceMetadata;

pub trait WorkspaceRepository {
    fn create(&self, root: &Path) -> Result<WorkspaceMetadata, WorkspaceError>;
    fn open(&self, root: &Path) -> Result<WorkspaceMetadata, WorkspaceError>;
}

pub fn create_workspace(
    repository: &impl WorkspaceRepository,
    root: &Path,
) -> Result<WorkspaceMetadata, WorkspaceError> {
    repository.create(root)
}

pub fn open_workspace(
    repository: &impl WorkspaceRepository,
    root: &Path,
) -> Result<WorkspaceMetadata, WorkspaceError> {
    repository.open(root)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceError {
    InvalidPath,
    UnsupportedFilesystem,
    AlreadyExists,
    DirectoryNotEmpty,
    NotFound,
    PermissionDenied,
    LegacyGoWorkspace,
    UnsupportedFormat,
    NewerSchema { found: u32, supported: u32 },
    CorruptWorkspace,
    StorageFailure,
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath => formatter.write_str(
                "The workspace path is invalid. Choose a directory on a supported local volume.",
            ),
            Self::UnsupportedFilesystem => formatter.write_str(
                "This workspace path is not on a supported local filesystem. Choose a local Windows volume.",
            ),
            Self::AlreadyExists => formatter.write_str(
                "A workspace database already exists here. Open the existing workspace instead.",
            ),
            Self::DirectoryNotEmpty => formatter.write_str(
                "The destination directory is not empty. Choose a new or empty directory.",
            ),
            Self::NotFound => formatter.write_str(
                "The workspace does not exist. Create a Rust workspace or choose an existing one.",
            ),
            Self::PermissionDenied => formatter.write_str(
                "The workspace cannot be accessed with the current permissions. Check local file permissions and close conflicting processes.",
            ),
            Self::LegacyGoWorkspace => formatter.write_str(
                "This is a legacy Go workspace. Create a new Rust workspace and import the EPUB source.",
            ),
            Self::UnsupportedFormat => formatter.write_str(
                "This database is not a supported Rust workspace. Its existing data was left unchanged.",
            ),
            Self::NewerSchema { found, supported } => write!(
                formatter,
                "Workspace schema {found} is newer than the supported schema {supported}. Use a newer application version.",
            ),
            Self::CorruptWorkspace => formatter.write_str(
                "The workspace database failed integrity checks. Restore it from a verified backup.",
            ),
            Self::StorageFailure => formatter.write_str(
                "The workspace could not be opened safely. Check its permissions and filesystem, then try again.",
            ),
        }
    }
}

impl std::error::Error for WorkspaceError {}

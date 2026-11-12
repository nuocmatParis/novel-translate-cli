use std::collections::BTreeSet;
use std::fmt;
use std::io::{self, Read};

use nt_domain::object::{ObjectHash, ObjectReference};

/// A verified view of the files found in the immutable object directory.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectInventory {
    pub referenced: Vec<ObjectReference>,
    pub unreferenced: Vec<ObjectReference>,
    pub incomplete_temps: Vec<String>,
    pub invalid_entries: Vec<String>,
    pub missing_references: Vec<ObjectHash>,
}

/// Storage failures that callers can handle without depending on a filesystem adapter.
#[derive(Debug)]
pub enum ObjectStoreError {
    InvalidWorkspaceRoot,
    UnsupportedFilesystem,
    UnsupportedCapability,
    ObjectNotFound,
    ObjectConflict,
    CorruptObject,
    Io(io::Error),
}

impl fmt::Display for ObjectStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWorkspaceRoot => {
                formatter.write_str("The workspace object store root is invalid.")
            }
            Self::UnsupportedFilesystem => formatter.write_str(
                "The workspace object store path uses an unsupported filesystem feature.",
            ),
            Self::UnsupportedCapability => {
                formatter.write_str("The filesystem cannot safely publish immutable objects.")
            }
            Self::ObjectNotFound => formatter.write_str("The requested object does not exist."),
            Self::ObjectConflict => formatter.write_str(
                "The object name already exists with content that does not match its hash.",
            ),
            Self::CorruptObject => {
                formatter.write_str("The stored object failed its content hash verification.")
            }
            Self::Io(_) => formatter.write_str("Object storage I/O failed."),
        }
    }
}

impl std::error::Error for ObjectStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Application boundary for content-addressed, immutable source files.
pub trait ImmutableObjectStore: Send + Sync {
    fn put(&self, source: &mut dyn Read) -> Result<ObjectReference, ObjectStoreError>;

    fn open_verified(&self, hash: ObjectHash) -> Result<Box<dyn Read + Send>, ObjectStoreError>;

    /// `referenced` comes from a consistent database read. This method never deletes files.
    fn inventory(
        &self,
        referenced: &BTreeSet<ObjectHash>,
    ) -> Result<ObjectInventory, ObjectStoreError>;
}

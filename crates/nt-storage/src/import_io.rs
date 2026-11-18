use nt_application::object_store::ImmutableObjectStore;
use nt_application::volume_import::{
    ImportWriteError, MAX_IMPORT_PAYLOAD_BYTES, ObjectHash, ObjectReference, StagedPayload,
};
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};

/// No-write streaming digest for the compact codec. The caller chooses a bound
/// including its domain/header bytes; no input or whole payload is retained.
pub struct ImportDigestSink {
    hash: Sha256,
    limit: u64,
    length: u64,
    failed: bool,
}
impl ImportDigestSink {
    pub fn new(byte_limit: u64) -> Result<Self, ImportWriteError> {
        if byte_limit > MAX_IMPORT_PAYLOAD_BYTES + 1024 {
            return Err(ImportWriteError::ResourceLimit);
        }
        Ok(Self {
            hash: Sha256::new(),
            limit: byte_limit,
            length: 0,
            failed: false,
        })
    }
    pub fn finish(self) -> Result<ObjectReference, ImportWriteError> {
        if self.failed {
            return Err(ImportWriteError::ResourceLimit);
        }
        Ok(ObjectReference {
            hash: ObjectHash::from_bytes(self.hash.finalize().into()),
            byte_len: self.length,
        })
    }
}
impl Write for ImportDigestSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let length = self.length.checked_add(bytes.len() as u64);
        if self.failed || length.is_none_or(|length| length > self.limit) {
            self.failed = true;
            return Err(io::Error::other("Import digest byte limit exceeded."));
        }
        self.hash.update(bytes);
        self.length = length.expect("checked above");
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn stage_import_payload(
    store: &impl ImmutableObjectStore,
    source: &mut dyn Read,
) -> Result<StagedPayload, ImportWriteError> {
    stage_with_limit(store, source, MAX_IMPORT_PAYLOAD_BYTES)
}
fn stage_with_limit(
    store: &impl ImmutableObjectStore,
    source: &mut dyn Read,
    limit: u64,
) -> Result<StagedPayload, ImportWriteError> {
    struct Bounded<'a> {
        source: &'a mut dyn Read,
        remaining: u64,
        exceeded: bool,
    }
    impl Read for Bounded<'_> {
        fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
            if target.is_empty() {
                return Ok(0);
            }
            if self.exceeded {
                return Err(io::Error::other("Import payload byte limit exceeded."));
            }
            if self.remaining == 0 {
                let mut probe = [0_u8; 1];
                if self.source.read(&mut probe)? == 0 {
                    return Ok(0);
                }
                self.exceeded = true;
                return Err(io::Error::other("Import payload byte limit exceeded."));
            }
            let count = target
                .len()
                .min(usize::try_from(self.remaining).unwrap_or(usize::MAX));
            let read = self.source.read(&mut target[..count])?;
            self.remaining -= read as u64;
            Ok(read)
        }
    }
    let mut bounded = Bounded {
        source,
        remaining: limit,
        exceeded: false,
    };
    let result = store.put(&mut bounded);
    if bounded.exceeded {
        return Err(ImportWriteError::ResourceLimit);
    }
    let object = result.map_err(super::object_error)?;
    if object.byte_len == 0 {
        return Err(ImportWriteError::InvalidRequest);
    }
    Ok(StagedPayload { object })
}

#[cfg(test)]
mod tests {
    use super::{ImportWriteError, stage_with_limit};
    use crate::{FileObjectStore, SqliteWorkspaceRepository};
    use nt_application::object_store::ImmutableObjectStore;
    use nt_application::workspace::create_workspace;
    use std::fs;
    use std::io::{self, Read};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn payload_limit_checks_boundary_before_publish_and_stops_an_unbounded_source() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let base = std::env::temp_dir().join(format!(
            "nt-cp05-payload-limit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&base).unwrap();
        let root = base.join("series");
        create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
        let store = FileObjectStore::new(&root).unwrap();
        let exact = stage_with_limit(&store, &mut &b"abc"[..], 3).unwrap();
        assert_eq!(exact.object.byte_len, 3);
        assert_eq!(
            stage_with_limit(&store, &mut &b"abcd"[..], 3).unwrap_err(),
            ImportWriteError::ResourceLimit
        );

        struct Unbounded(usize);
        impl Read for Unbounded {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                bytes.fill(b'x');
                self.0 += bytes.len();
                Ok(bytes.len())
            }
        }
        let mut infinite = Unbounded(0);
        assert_eq!(
            stage_with_limit(&store, &mut infinite, 7).unwrap_err(),
            ImportWriteError::ResourceLimit
        );
        assert_eq!(infinite.0, 8);
        let inventory = store.inventory(&Default::default()).unwrap();
        assert_eq!(inventory.unreferenced, vec![exact.object]);
        assert!(inventory.incomplete_temps.is_empty());
        drop(store);
        fs::remove_dir_all(&base).unwrap();
    }
}

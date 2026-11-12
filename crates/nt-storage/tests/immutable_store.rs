use std::collections::BTreeSet;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use nt_application::object_store::{ImmutableObjectStore, ObjectStoreError};
use nt_domain::object::ObjectHash;
use nt_storage::FileObjectStore;

static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        for _ in 0..100 {
            let id = NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("nt-object-store-test-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create test directory: {error}"),
            }
        }
        panic!("could not allocate test directory");
    }

    fn workspace(&self) -> PathBuf {
        let root = self.0.join("workspace");
        fs::create_dir(&root).expect("create workspace root");
        root
    }

    fn outside(&self) -> PathBuf {
        let path = self.0.join("outside");
        fs::create_dir(&path).expect("create outside directory");
        path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn stores_verifies_reads_and_deduplicates_exact_bytes() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let store = FileObjectStore::new(&root).unwrap();
    let bytes = b"abc";

    let first = store.put(&mut Cursor::new(bytes)).unwrap();
    assert_eq!(
        first.hash.to_hex(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(first.byte_len, 3);

    let path = root
        .join("objects")
        .join(format!("{}.blob", first.hash.to_hex()));
    assert_eq!(fs::read(&path).unwrap(), bytes);

    let second = store.put(&mut Cursor::new(bytes)).unwrap();
    assert_eq!(second, first);

    let mut reader = store.open_verified(first.hash).unwrap();
    let mut read_back = Vec::new();
    reader.read_to_end(&mut read_back).unwrap();
    assert_eq!(read_back, bytes);

    let entries = fs::read_dir(root.join("objects"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].file_name().to_string_lossy().ends_with(".blob"));
}

#[test]
fn concurrent_puts_publish_one_verified_object() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let store = Arc::new(FileObjectStore::new(&root).unwrap());
    let mut threads = Vec::new();

    for _ in 0..2 {
        let store = Arc::clone(&store);
        threads.push(std::thread::spawn(move || {
            store.put(&mut Cursor::new(b"same concurrent bytes"))
        }));
    }

    let first = threads.remove(0).join().unwrap().unwrap();
    let second = threads.remove(0).join().unwrap().unwrap();
    assert_eq!(first, second);
    assert_eq!(first.byte_len, b"same concurrent bytes".len() as u64);

    let entries = fs::read_dir(root.join("objects"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].file_name().to_string_lossy().ends_with(".blob"));
}

#[test]
fn rejects_and_preserves_a_wrong_existing_target() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let outside = temp.outside().join("sentinel.txt");
    fs::write(&outside, b"outside stays unchanged").unwrap();
    let object_dir = root.join("objects");
    fs::create_dir(&object_dir).unwrap();
    let hash =
        ObjectHash::parse("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
            .unwrap();
    let target = object_dir.join(format!("{hash}.blob"));
    fs::write(&target, b"different pre-existing bytes").unwrap();
    let store = FileObjectStore::new(&root).unwrap();

    let error = store.put(&mut Cursor::new(b"abc")).unwrap_err();
    assert!(matches!(error, ObjectStoreError::ObjectConflict));
    assert_eq!(fs::read(&target).unwrap(), b"different pre-existing bytes");
    assert_eq!(fs::read(&outside).unwrap(), b"outside stays unchanged");
    assert_no_temp_files(&object_dir);
}

#[test]
fn open_verified_rejects_changed_object_bytes() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let store = FileObjectStore::new(&root).unwrap();
    let reference = store.put(&mut Cursor::new(b"original bytes")).unwrap();
    let path = root
        .join("objects")
        .join(format!("{}.blob", reference.hash.to_hex()));
    fs::write(&path, b"changed bytes").unwrap();

    let error = store.open_verified(reference.hash).err().unwrap();
    assert!(matches!(error, ObjectStoreError::CorruptObject));
    assert_eq!(fs::read(&path).unwrap(), b"changed bytes");
}

#[test]
fn inventory_reports_references_orphans_temps_and_invalid_entries() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let store = FileObjectStore::new(&root).unwrap();
    let referenced = store.put(&mut Cursor::new(b"referenced bytes")).unwrap();
    let orphan = store.put(&mut Cursor::new(b"orphan bytes")).unwrap();
    let object_dir = root.join("objects");
    let temp_path = object_dir.join(".nt-object-tmp-abandoned.part");
    let invalid_path = object_dir.join("not-a-hash.blob");
    let corrupt_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    let corrupt_path = object_dir.join(format!("{corrupt_hash}.blob"));
    fs::write(&temp_path, b"partial temp").unwrap();
    fs::write(&invalid_path, b"unrecognized object name").unwrap();
    fs::write(&corrupt_path, b"content does not match the filename").unwrap();
    let missing = ObjectHash::from_bytes([0x11; 32]);
    let referenced_hashes = BTreeSet::from([referenced.hash, missing]);

    let inventory = store.inventory(&referenced_hashes).unwrap();
    assert_eq!(inventory.referenced, vec![referenced]);
    assert_eq!(inventory.unreferenced, vec![orphan]);
    assert_eq!(
        inventory.incomplete_temps,
        vec![
            temp_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        ]
    );
    assert!(
        inventory.invalid_entries.contains(
            &invalid_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        )
    );
    assert!(
        inventory.invalid_entries.contains(
            &corrupt_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        )
    );
    assert_eq!(inventory.missing_references, vec![missing]);
    assert_eq!(fs::read(&temp_path).unwrap(), b"partial temp");
    assert_eq!(
        fs::read(&invalid_path).unwrap(),
        b"unrecognized object name"
    );
    assert_eq!(
        fs::read(&corrupt_path).unwrap(),
        b"content does not match the filename"
    );
}

#[test]
fn rejects_object_directory_junction_without_touching_outside_sentinel() {
    let temp = TestDirectory::new();
    let root = temp.workspace();
    let outside_dir = temp.outside();
    let outside_sentinel = outside_dir.join("sentinel.txt");
    fs::write(&outside_sentinel, b"outside bytes remain").unwrap();
    let junction = root.join("objects");
    create_directory_link(&junction, &outside_dir);
    let store = FileObjectStore::new(&root).unwrap();

    let error = store
        .put(&mut Cursor::new(b"never written outside"))
        .unwrap_err();
    assert!(matches!(error, ObjectStoreError::UnsupportedFilesystem));
    assert_eq!(
        fs::read(&outside_sentinel).unwrap(),
        b"outside bytes remain"
    );

    remove_directory_link(&junction);
}

fn assert_no_temp_files(object_dir: &Path) {
    let entries = fs::read_dir(object_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(entries.iter().all(|entry| {
        !entry
            .file_name()
            .to_string_lossy()
            .starts_with(".nt-object-tmp-")
    }));
}

#[cfg(windows)]
fn create_directory_link(link: &Path, target: &Path) {
    use std::process::Command;

    let output = Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            link.to_str().expect("test path is valid UTF-8"),
            target.to_str().expect("test path is valid UTF-8"),
        ])
        .output()
        .expect("run Windows junction command");
    assert!(
        output.status.success(),
        "could not create test junction: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(windows)]
fn remove_directory_link(link: &Path) {
    fs::remove_dir(link).expect("remove test junction without traversing target");
}

#[cfg(unix)]
fn create_directory_link(link: &Path, target: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create test directory symlink");
}

#[cfg(unix)]
fn remove_directory_link(link: &Path) {
    fs::remove_file(link).expect("remove test directory symlink");
}

#[cfg(not(any(windows, unix)))]
fn create_directory_link(_link: &Path, _target: &Path) {
    panic!("this test requires a platform with directory links");
}

#[cfg(not(any(windows, unix)))]
fn remove_directory_link(_link: &Path) {}

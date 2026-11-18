#![cfg(windows)]

use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use nt_application::object_store::{ImmutableObjectStore, ObjectStoreError};
use nt_application::workspace::create_workspace;
use nt_domain::workspace::VolumeId;
use nt_storage::{FileImportSourceStore, FileObjectStore, SqliteWorkspaceRepository};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nt-cp05-files-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn volume() -> VolumeId {
    VolumeId::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap()
}

#[test]
fn staged_payload_stays_invisible_on_encode_failure_and_seekable_when_sealed() {
    let temp = Temp::new();
    let root = temp.0.join("series");
    create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
    let store = FileObjectStore::new(&root).unwrap();
    let aborted = store.stage_encoded_import(|output| {
        output.write_all(b"partial payload")?;
        output.flush()?;
        Err(io::Error::other("test aborted encode"))
    });
    assert!(matches!(aborted, Err(ObjectStoreError::Io(_))));
    assert!(fs::read_dir(root.join("objects")).unwrap().next().is_none());

    let staged = store
        .stage_encoded_import(|output| {
            output.write_all(b"header")?;
            output.write_all(b"body")
        })
        .unwrap();
    let object_dir = root.join("objects");
    let object = object_dir.join(format!("{}.blob", staged.object.hash));
    assert_eq!(fs::read_dir(&object_dir).unwrap().count(), 1);
    assert_eq!(fs::read(&object).unwrap(), b"headerbody");

    let mut wrong_size = staged.object;
    wrong_size.byte_len -= 1;
    assert!(matches!(
        store.open_import_object(wrong_size),
        Err(ObjectStoreError::CorruptObject)
    ));

    let mut reader = store.open_import_object(staged.object).unwrap();
    assert!(fs::write(&object, b"spoof bytes").is_err());
    assert!(fs::rename(&object_dir, root.join("moved-objects")).is_err());
    reader.seek(SeekFrom::Start(6)).unwrap();
    let mut body = Vec::new();
    reader.read_to_end(&mut body).unwrap();
    assert_eq!(body, b"body");
    drop(reader);
    fs::write(&object, b"bodyheader").unwrap();
    assert!(matches!(
        store.open_import_object(staged.object),
        Err(ObjectStoreError::CorruptObject)
    ));
}

#[test]
fn image_copy_is_snapshot_scoped_pinned_and_does_not_share_object_inode() {
    let temp = Temp::new();
    let root = temp.0.join("series");
    create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
    let store = FileObjectStore::new(&root).unwrap();
    let bytes = b"independent picture data";
    let object = store.put(&mut &bytes[..]).unwrap();
    let images = FileImportSourceStore::new(&root).unwrap();
    let mut input = store.open_verified(object.hash).unwrap();
    let pinned = images
        .materialize(
            &volume(),
            [7; 32],
            "OPS/Images/cover.png",
            object,
            &mut input,
        )
        .unwrap();
    let expected = format!(
        "volumes/{}/snapshots/{}/images/source/OPS/Images/cover.png",
        volume(),
        "07".repeat(32)
    );
    assert_eq!(pinned.relative_path, expected);
    let source_path = root.join(&expected);
    assert_eq!(fs::read(&source_path).unwrap(), bytes);
    assert!(
        fs::write(&source_path, b"malicious change").is_err(),
        "open source must deny writes"
    );
    assert!(
        fs::rename(&source_path, source_path.with_extension("moved")).is_err(),
        "open source must deny replacement"
    );
    assert!(
        fs::rename(
            source_path.parent().unwrap(),
            source_path.parent().unwrap().with_extension("moved")
        )
        .is_err(),
        "open parent must deny replacement"
    );

    drop(pinned);
    fs::write(&source_path, b"changed source").unwrap();
    let original_object = root.join("objects").join(format!("{}.blob", object.hash));
    assert_eq!(
        fs::read(original_object).unwrap(),
        bytes,
        "materialized copy is not a hard link to the object"
    );
    assert!(
        images
            .open_verified(&volume(), [7; 32], "OPS/Images/cover.png", object)
            .is_err()
    );
}

#[test]
fn reimport_reuses_exact_source_preserves_edit_and_rejects_case_collision() {
    let temp = Temp::new();
    let root = temp.0.join("series");
    create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
    let store = FileObjectStore::new(&root).unwrap();
    let bytes = b"picture";
    let object = store.put(&mut &bytes[..]).unwrap();
    let images = FileImportSourceStore::new(&root).unwrap();
    let first = images
        .materialize(
            &volume(),
            [8; 32],
            "OPS/Images/cover.png",
            object,
            &mut &bytes[..],
        )
        .unwrap();
    drop(first);
    let edited = root.join(format!(
        "volumes/{}/snapshots/{}/images/edit/OPS/Images/cover.png",
        volume(),
        "08".repeat(32)
    ));
    fs::create_dir_all(edited.parent().unwrap()).unwrap();
    fs::write(&edited, b"my edited image").unwrap();

    let again = images
        .materialize(
            &volume(),
            [8; 32],
            "OPS/Images/cover.png",
            object,
            &mut &b"wrong input not read on reuse"[..],
        )
        .unwrap();
    drop(again);
    assert_eq!(fs::read(&edited).unwrap(), b"my edited image");
    let collision = images.materialize(
        &volume(),
        [8; 32],
        "OPS/Images/Cover.png",
        object,
        &mut &bytes[..],
    );
    assert!(matches!(collision, Err(ObjectStoreError::ObjectConflict)));
    assert_eq!(fs::read(&edited).unwrap(), b"my edited image");
}

#[test]
fn traversal_reserved_names_and_junctions_cannot_reach_outside_root() {
    let temp = Temp::new();
    let root = temp.0.join("series");
    create_workspace(&SqliteWorkspaceRepository, &root).unwrap();
    let images = FileImportSourceStore::new(&root).unwrap();
    for path in [
        "../outside",
        "/absolute",
        "OPS/CON.txt",
        "OPS\\x.png",
        "OPS/x. ",
    ] {
        assert!(
            FileImportSourceStore::source_relative_path(&volume(), [1; 32], path).is_err(),
            "{path}"
        );
    }
    let outside = temp.0.join("outside");
    fs::create_dir(&outside).unwrap();
    let sentinel = outside.join("sentinel");
    fs::write(&sentinel, b"unchanged").unwrap();
    let junction = root.join("volumes");
    let status = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&junction)
        .arg(&outside)
        .status()
        .unwrap();
    assert!(status.success());
    let object = FileObjectStore::new(&root)
        .unwrap()
        .put(&mut &b"x"[..])
        .unwrap();
    assert!(
        images
            .materialize(&volume(), [1; 32], "OPS/x.png", object, &mut &b"x"[..])
            .is_err()
    );
    assert_eq!(fs::read(sentinel).unwrap(), b"unchanged");
    fs::remove_dir(junction).unwrap();
}

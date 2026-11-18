use super::*;
use novel_translate::import_codec::{ResourceInput, ResourceRecord, ResourceRole};

#[test]
fn resources_keep_actual_image_info_and_every_indexed_occurrence() {
    use nt_application::volume_import::{ImageInformation, VolumeId};
    let bytes = fixture::book(3, fixture::CHAPTER);
    let source = extracted(3);
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let inventory = inspect(&mut archive);
    let references = nt_epub::index_references(
        &mut archive,
        &inventory,
        &source,
        nt_epub::ReferenceIndexLimits::default(),
    )
    .unwrap();
    let id = archive
        .entry_by_path("OPS/Images/a.png")
        .unwrap()
        .unwrap()
        .id();
    let image = archive.read_entry(id).unwrap();
    let info = nt_epub::SourceExtraction::image_information(&image, "image/png").unwrap();
    assert_eq!(info.pixel_dimensions, Some((1, 1)));
    let mut sink = nt_storage::ImportDigestSink::new(image.len() as u64).unwrap();
    sink.write_all(&image).unwrap();
    let object = sink.finish().unwrap();
    let volume = VolumeId::parse("2".repeat(32)).unwrap();
    let relative = nt_storage::FileImportSourceStore::source_relative_path(
        &volume,
        source.epub_sha256,
        "OPS/Images/a.png",
    )
    .unwrap();
    let uses=references.references.iter().filter(|r|matches!(&r.target,nt_epub::ReferenceTarget::Internal{path,..} if path=="OPS/Images/a.png")).collect::<Vec<_>>();
    assert!(uses.len() >= 2);
    let record = ResourceRecord::capture(
        ResourceInput {
            role: ResourceRole::Image,
            canonical_path: "OPS/Images/a.png",
            object,
            source_relative_path: Some(&relative),
            declared_media_type: Some("image/png"),
            image_information: &info,
        },
        &uses,
        0,
    )
    .unwrap();
    assert_eq!(record.uses.len(), uses.len());
    assert_eq!(record.dimensions.width, Some(1));
    let markup = record
        .uses
        .iter()
        .find(|r| !r.source_occurrences.is_empty())
        .unwrap();
    assert_eq!(markup.location.attribute_local_name.as_deref(), Some("src"));
    assert_eq!(markup.spine_positions, [0]);
    assert_eq!(markup.source_occurrences.len(), 1);
    assert!(!format!("{markup:?}").contains("../Images/a.png"));
    let bytes = serde_json::to_vec(&record).unwrap();
    let reopened: ResourceRecord = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(reopened, record);
    reopened.validate_structure().unwrap();
    let mut invalid = record.clone();
    invalid.dimensions.width = Some(0);
    assert_eq!(invalid.validate_structure(), Err(CodecError::InvalidRecord));
    let unknown = ResourceRecord::capture(
        ResourceInput {
            role: ResourceRole::Other,
            canonical_path: "OPS/Other/data.bin",
            object,
            source_relative_path: None,
            declared_media_type: Some("application/octet-stream"),
            image_information: &ImageInformation::default(),
        },
        &[],
        0,
    )
    .unwrap();
    let value = serde_json::to_value(unknown).unwrap();
    assert_eq!(value["format"]["status"], "Unknown");
    assert!(value["format"]["name"].is_null());
    assert_eq!(value["dimensions"]["status"], "Unknown");
    assert!(value["dimensions"]["width"].is_null());
}

#[cfg(windows)]
#[test]
fn staged_resources_reopen_against_original_bytes_and_reject_changed_use_or_copy() {
    use novel_translate::import_codec::{ResourceFileError, ResourceFiles};
    use novel_translate::import_workflow::capture_epub;
    use nt_application::volume_import::{VolumeId, VolumeResource};
    use nt_application::workspace::create_workspace;
    use nt_epub::{ReferenceIndexLimits, index_references};
    use nt_storage::{FileImportSourceStore, FileObjectStore, SqliteWorkspaceRepository};
    use std::fs;
    use std::path::PathBuf;

    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let path = std::env::temp_dir().join(format!("nt-cp05-resource-stage-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    let scratch = Temp(path);
    let workspace = scratch.0.join("series");
    create_workspace(&SqliteWorkspaceRepository, &workspace).unwrap();
    let input = scratch.0.join("source.epub");
    fs::write(&input, fixture::book(3, fixture::CHAPTER)).unwrap();
    let objects = FileObjectStore::new(&workspace).unwrap();
    let original = capture_epub(&objects, &input).unwrap();
    let images = FileImportSourceStore::new(&workspace).unwrap();
    let volume = VolumeId::parse("3".repeat(32)).unwrap();
    let files = ResourceFiles {
        objects: &objects,
        source_images: &images,
        volume: &volume,
        epub: Digest(*original.hash.as_bytes()),
    };
    let mut archive = EpubZip::new(objects.open_import_object(original).unwrap()).unwrap();
    let inventory = inspect(&mut archive);
    let source = extract_source(&mut archive, &inventory, files.epub.0, PARSER_VERSION).unwrap();
    let references = index_references(
        &mut archive,
        &inventory,
        &source,
        ReferenceIndexLimits::default(),
    )
    .unwrap();
    assert!(matches!(
        files.stage(&mut archive, &inventory, &references, 256 * 1024 * 1024 - 1),
        Err(ResourceFileError::Codec(CodecError::ResourceLimit))
    ));
    assert_eq!(fs::read_dir(workspace.join("objects")).unwrap().count(), 1);
    let records = files
        .stage(&mut archive, &inventory, &references, 0)
        .unwrap();
    let rows: Vec<_> = records
        .iter()
        .map(|record| VolumeResource {
            role: record.role.storage_name().into(),
            canonical_path: record.canonical_path.clone(),
            object: record.object(),
            source_relative_path: record.source_relative_path.clone(),
        })
        .collect();
    let image_index = records
        .iter()
        .position(|r| r.canonical_path == "OPS/Images/a.png")
        .unwrap();
    let image = &records[image_index];
    assert!(image.uses.len() >= 2);
    assert_eq!(image.role, ResourceRole::Image);
    let image_path = workspace.join(image.source_relative_path.as_deref().unwrap());
    let entry = archive
        .entry_by_path("OPS/Images/a.png")
        .unwrap()
        .unwrap()
        .id();
    assert_eq!(
        fs::read(&image_path).unwrap(),
        archive.read_entry(entry).unwrap()
    );
    files
        .verify(&mut archive, &inventory, &references, &records, &rows, 0)
        .unwrap();

    let mut tampered = records.clone();
    tampered[image_index].uses.pop().unwrap();
    assert!(matches!(
        files.verify(&mut archive, &inventory, &references, &tampered, &rows, 0),
        Err(ResourceFileError::EvidenceMismatch)
    ));

    fs::write(image_path, b"damaged image copy").unwrap();
    assert!(matches!(
        files.verify(&mut archive, &inventory, &references, &records, &rows, 0),
        Err(ResourceFileError::Storage(_))
    ));
    assert!(objects.open_import_object(image.object()).is_ok());
}

use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use nt_epub::{ArchiveError, ArchiveLimits, EpubZip};
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

static NEXT_SENTINEL: AtomicU64 = AtomicU64::new(0);

fn make_zip(entries: &[(&str, &[u8])], method: CompressionMethod) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        let options = SimpleFileOptions::default().compression_method(method);
        if name.ends_with('/') {
            writer.add_directory(*name, options).unwrap();
        } else {
            writer.start_file(*name, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

fn limits_with(
    max_entries: usize,
    max_entry_bytes: u64,
    max_total_bytes: u64,
    max_central_directory_bytes: u64,
) -> ArchiveLimits {
    ArchiveLimits {
        max_entries,
        max_entry_bytes,
        max_total_bytes,
        max_central_directory_bytes,
    }
}

fn central_directory_size(bytes: &[u8]) -> u64 {
    let eocd = bytes
        .windows(4)
        .rposition(|window| window == [0x50, 0x4b, 0x05, 0x06])
        .unwrap();
    u32::from_le_bytes(bytes[eocd + 12..eocd + 16].try_into().unwrap()) as u64
}

fn upgrade_to_zip64(bytes: &mut Vec<u8>) -> usize {
    let eocd = bytes
        .windows(4)
        .rposition(|window| window == [0x50, 0x4b, 0x05, 0x06])
        .unwrap();
    let entries_on_disk = u16::from_le_bytes(bytes[eocd + 8..eocd + 10].try_into().unwrap()) as u64;
    let entries_total = u16::from_le_bytes(bytes[eocd + 10..eocd + 12].try_into().unwrap()) as u64;
    let directory_bytes =
        u32::from_le_bytes(bytes[eocd + 12..eocd + 16].try_into().unwrap()) as u64;
    let directory_offset =
        u32::from_le_bytes(bytes[eocd + 16..eocd + 20].try_into().unwrap()) as u64;

    let zip64_eocd_offset = eocd as u64;
    let mut records = Vec::with_capacity(76);
    records.extend_from_slice(&0x0606_4b50_u32.to_le_bytes());
    records.extend_from_slice(&44_u64.to_le_bytes());
    records.extend_from_slice(&45_u16.to_le_bytes());
    records.extend_from_slice(&45_u16.to_le_bytes());
    records.extend_from_slice(&0_u32.to_le_bytes());
    records.extend_from_slice(&0_u32.to_le_bytes());
    records.extend_from_slice(&entries_on_disk.to_le_bytes());
    records.extend_from_slice(&entries_total.to_le_bytes());
    records.extend_from_slice(&directory_bytes.to_le_bytes());
    records.extend_from_slice(&directory_offset.to_le_bytes());
    records.extend_from_slice(&0x0706_4b50_u32.to_le_bytes());
    records.extend_from_slice(&0_u32.to_le_bytes());
    records.extend_from_slice(&zip64_eocd_offset.to_le_bytes());
    records.extend_from_slice(&1_u32.to_le_bytes());
    assert_eq!(records.len(), 76);
    bytes.splice(eocd..eocd, records);

    let zip64_eocd = eocd;
    let legacy_eocd = eocd + 76;
    bytes[legacy_eocd + 8..legacy_eocd + 10].copy_from_slice(&u16::MAX.to_le_bytes());
    bytes[legacy_eocd + 10..legacy_eocd + 12].copy_from_slice(&u16::MAX.to_le_bytes());
    bytes[legacy_eocd + 12..legacy_eocd + 16].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[legacy_eocd + 16..legacy_eocd + 20].copy_from_slice(&u32::MAX.to_le_bytes());
    zip64_eocd
}

fn prefixed_zip64_with_decoy_end_record() -> Vec<u8> {
    let mut actual = make_zip(&[("OPS/real.xhtml", b"real")], CompressionMethod::Stored);
    let zip64_eocd = upgrade_to_zip64(&mut actual);
    let locator = zip64_eocd + 56;
    let logical_zip64_eocd =
        u64::from_le_bytes(actual[locator + 8..locator + 16].try_into().unwrap());

    let decoy = make_zip(&[("OPS/d.xhtml", b"fake")], CompressionMethod::Stored);
    let decoy_eocd = decoy
        .windows(4)
        .rposition(|window| window == [0x50, 0x4b, 0x05, 0x06])
        .unwrap();
    let decoy_directory_size =
        u32::from_le_bytes(decoy[decoy_eocd + 12..decoy_eocd + 16].try_into().unwrap()) as u64;
    let decoy_directory_offset =
        u32::from_le_bytes(decoy[decoy_eocd + 16..decoy_eocd + 20].try_into().unwrap()) as u64;

    let prefix_len = logical_zip64_eocd as usize + 64;
    let fake_end = logical_zip64_eocd as usize;
    assert!(decoy_eocd <= fake_end);
    assert!(decoy_directory_offset + decoy_directory_size <= fake_end as u64);

    let mut prefix = vec![0_u8; prefix_len];
    prefix[..decoy_eocd].copy_from_slice(&decoy[..decoy_eocd]);
    let mut fake_record = [0_u8; 56];
    fake_record[..4].copy_from_slice(&0x0606_4b50_u32.to_le_bytes());
    fake_record[4..12].copy_from_slice(&(prefix_len as u64 + 44).to_le_bytes());
    fake_record[12..14].copy_from_slice(&45_u16.to_le_bytes());
    fake_record[14..16].copy_from_slice(&45_u16.to_le_bytes());
    fake_record[24..32].copy_from_slice(&1_u64.to_le_bytes());
    fake_record[32..40].copy_from_slice(&1_u64.to_le_bytes());
    fake_record[40..48].copy_from_slice(&decoy_directory_size.to_le_bytes());
    fake_record[48..56].copy_from_slice(&decoy_directory_offset.to_le_bytes());
    prefix[fake_end..fake_end + fake_record.len()].copy_from_slice(&fake_record);
    prefix.extend_from_slice(&actual);
    prefix
}

fn add_ambiguous_eocd_to_comment(bytes: &mut Vec<u8>) {
    let eocd = bytes
        .windows(4)
        .rposition(|window| window == [0x50, 0x4b, 0x05, 0x06])
        .unwrap();
    bytes[eocd + 20..eocd + 22].copy_from_slice(&22_u16.to_le_bytes());
    let mut fake = [0_u8; 22];
    fake[..4].copy_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
    bytes.extend_from_slice(&fake);
}

fn add_central_directory_digital_signature(bytes: &mut Vec<u8>) {
    let eocd = bytes
        .windows(4)
        .rposition(|window| window == [0x50, 0x4b, 0x05, 0x06])
        .unwrap();
    let directory_size = u32::from_le_bytes(bytes[eocd + 12..eocd + 16].try_into().unwrap());
    let signature_payload = b"sig";
    let mut record = Vec::with_capacity(6 + signature_payload.len());
    record.extend_from_slice(&[0x50, 0x4b, 0x05, 0x05]);
    record.extend_from_slice(&(signature_payload.len() as u16).to_le_bytes());
    record.extend_from_slice(signature_payload);
    bytes.splice(eocd..eocd, record.iter().copied());
    let shifted_eocd = eocd + record.len();
    bytes[shifted_eocd + 12..shifted_eocd + 16]
        .copy_from_slice(&(directory_size + record.len() as u32).to_le_bytes());
}

fn entry_bytes(bytes: Vec<u8>, limits: ArchiveLimits) -> Result<Vec<u8>, ArchiveError> {
    let mut archive = EpubZip::with_limits(Cursor::new(bytes), limits)?;
    let id = archive.entries()[0].id();
    archive.read_entry(id)
}

#[test]
fn accepts_stored_and_deflated_entries() {
    for method in [CompressionMethod::Stored, CompressionMethod::Deflated] {
        let bytes = make_zip(&[("OPS/chapter.xhtml", b"chapter text")], method);
        let output = entry_bytes(bytes, ArchiveLimits::default()).unwrap();
        assert_eq!(output, b"chapter text");
    }
}

#[test]
fn accepts_a_prefixed_archive_and_a_well_formed_directory_signature() {
    let mut prefixed = make_zip(
        &[("OPS/chapter.xhtml", b"chapter")],
        CompressionMethod::Stored,
    );
    prefixed.splice(0..0, b"EPUB preface".iter().copied());
    assert_eq!(
        entry_bytes(prefixed, ArchiveLimits::default()).unwrap(),
        b"chapter"
    );

    let mut signed = make_zip(
        &[("OPS/chapter.xhtml", b"chapter")],
        CompressionMethod::Stored,
    );
    add_central_directory_digital_signature(&mut signed);
    assert_eq!(
        entry_bytes(signed, ArchiveLimits::default()).unwrap(),
        b"chapter"
    );
}

#[test]
fn uses_the_preflight_view_for_a_prefixed_zip64_archive() {
    let bytes = prefixed_zip64_with_decoy_end_record();
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();

    assert_eq!(archive.entries()[0].name(), "OPS/real.xhtml");
    assert_eq!(
        archive.read_entry(archive.entries()[0].id()).unwrap(),
        b"real"
    );
}

#[test]
fn checks_entry_count_below_at_and_above_limit_before_reading_payloads() {
    let bytes = make_zip(
        &[("OPS/one.xhtml", b"one"), ("OPS/two.xhtml", b"two")],
        CompressionMethod::Stored,
    );

    let too_small = limits_with(1, 32, 64, 4096);
    assert!(matches!(
        EpubZip::with_limits(Cursor::new(bytes.clone()), too_small),
        Err(ArchiveError::TooManyEntries {
            actual: 2,
            limit: 1
        })
    ));

    for limit in [2, 3] {
        let archive =
            EpubZip::with_limits(Cursor::new(bytes.clone()), limits_with(limit, 32, 64, 4096))
                .unwrap();
        assert_eq!(archive.entries().len(), 2);
    }
}

#[test]
fn checks_central_directory_metadata_below_at_and_above_limit() {
    let bytes = make_zip(&[("OPS/chapter.xhtml", b"ok")], CompressionMethod::Stored);
    let actual = central_directory_size(&bytes);
    assert!(matches!(
        EpubZip::with_limits(
            Cursor::new(bytes.clone()),
            limits_with(10, 10, 10, actual - 1)
        ),
        Err(ArchiveError::CentralDirectoryTooLarge { actual: size, .. }) if size == actual
    ));

    for limit in [actual, actual + 1] {
        EpubZip::with_limits(Cursor::new(bytes.clone()), limits_with(10, 10, 10, limit)).unwrap();
    }
}

#[test]
fn rejects_central_directory_size_underclaim_overclaim_and_count_mismatch() {
    let original = make_zip(&[("OPS/chapter.xhtml", b"ok")], CompressionMethod::Stored);
    let eocd = original
        .windows(4)
        .rposition(|window| window == [0x50, 0x4b, 0x05, 0x06])
        .unwrap();

    let mut underclaimed = original.clone();
    underclaimed[eocd + 12..eocd + 16].copy_from_slice(&1_u32.to_le_bytes());
    assert!(matches!(
        EpubZip::with_limits(Cursor::new(underclaimed), limits_with(10, 10, 10, 1)),
        Err(ArchiveError::InvalidContainer(_))
    ));

    let mut overclaimed = original.clone();
    let actual_size = u32::from_le_bytes(overclaimed[eocd + 12..eocd + 16].try_into().unwrap());
    overclaimed[eocd + 12..eocd + 16].copy_from_slice(&(actual_size + 1).to_le_bytes());
    assert!(matches!(
        EpubZip::new(Cursor::new(overclaimed)),
        Err(ArchiveError::InvalidContainer(_))
    ));

    let mut wrong_count = original;
    wrong_count[eocd + 8..eocd + 10].copy_from_slice(&2_u16.to_le_bytes());
    wrong_count[eocd + 10..eocd + 12].copy_from_slice(&2_u16.to_le_bytes());
    assert!(matches!(
        EpubZip::new(Cursor::new(wrong_count)),
        Err(ArchiveError::InvalidContainer(_))
    ));

    let mut zip64_wrong_count =
        make_zip(&[("OPS/chapter.xhtml", b"ok")], CompressionMethod::Stored);
    let zip64_eocd = upgrade_to_zip64(&mut zip64_wrong_count);
    zip64_wrong_count[zip64_eocd + 24..zip64_eocd + 32].copy_from_slice(&2_u64.to_le_bytes());
    zip64_wrong_count[zip64_eocd + 32..zip64_eocd + 40].copy_from_slice(&2_u64.to_le_bytes());
    assert!(matches!(
        EpubZip::new(Cursor::new(zip64_wrong_count)),
        Err(ArchiveError::InvalidContainer(_))
    ));

    let mut zip64_underclaimed =
        make_zip(&[("OPS/chapter.xhtml", b"ok")], CompressionMethod::Stored);
    let zip64_eocd = upgrade_to_zip64(&mut zip64_underclaimed);
    zip64_underclaimed[zip64_eocd + 40..zip64_eocd + 48].copy_from_slice(&1_u64.to_le_bytes());
    assert!(matches!(
        EpubZip::with_limits(Cursor::new(zip64_underclaimed), limits_with(10, 10, 10, 1)),
        Err(ArchiveError::InvalidContainer(_))
    ));

    let mut zip64_overclaimed =
        make_zip(&[("OPS/chapter.xhtml", b"ok")], CompressionMethod::Stored);
    let zip64_eocd = upgrade_to_zip64(&mut zip64_overclaimed);
    let actual = u64::from_le_bytes(
        zip64_overclaimed[zip64_eocd + 40..zip64_eocd + 48]
            .try_into()
            .unwrap(),
    );
    zip64_overclaimed[zip64_eocd + 40..zip64_eocd + 48]
        .copy_from_slice(&(actual + 1).to_le_bytes());
    assert!(matches!(
        EpubZip::new(Cursor::new(zip64_overclaimed)),
        Err(ArchiveError::InvalidContainer(_))
    ));
}

#[test]
fn rejects_ambiguous_eocd_inside_the_archive_comment() {
    let mut bytes = make_zip(&[("OPS/chapter.xhtml", b"ok")], CompressionMethod::Stored);
    add_ambiguous_eocd_to_comment(&mut bytes);

    assert!(matches!(
        EpubZip::new(Cursor::new(bytes)),
        Err(ArchiveError::InvalidContainer(_))
    ));
}

#[test]
fn checks_zip64_entry_count_and_metadata_before_indexing() {
    let mut bytes = make_zip(&[("OPS/chapter.xhtml", b"ok")], CompressionMethod::Stored);
    let zip64_eocd = upgrade_to_zip64(&mut bytes);
    let actual_directory_bytes = central_directory_size(&make_zip(
        &[("OPS/chapter.xhtml", b"ok")],
        CompressionMethod::Stored,
    ));

    assert!(matches!(
        EpubZip::with_limits(
            Cursor::new(bytes.clone()),
            limits_with(0, 10, 10, actual_directory_bytes)
        ),
        Err(ArchiveError::TooManyEntries {
            actual: 1,
            limit: 0
        })
    ));

    for count_limit in [1, 2] {
        EpubZip::with_limits(
            Cursor::new(bytes.clone()),
            limits_with(count_limit, 10, 10, actual_directory_bytes),
        )
        .unwrap();
    }

    assert!(matches!(
        EpubZip::with_limits(
            Cursor::new(bytes.clone()),
            limits_with(1, 10, 10, actual_directory_bytes - 1)
        ),
        Err(ArchiveError::CentralDirectoryTooLarge { actual, .. })
            if actual == actual_directory_bytes
    ));
    for directory_limit in [actual_directory_bytes, actual_directory_bytes + 1] {
        EpubZip::with_limits(
            Cursor::new(bytes.clone()),
            limits_with(1, 10, 10, directory_limit),
        )
        .unwrap();
    }

    let mut malformed = bytes.clone();
    malformed[zip64_eocd + 4..zip64_eocd + 12].copy_from_slice(&43_u64.to_le_bytes());
    assert!(matches!(
        EpubZip::new(Cursor::new(malformed)),
        Err(ArchiveError::InvalidContainer(_))
    ));

    let mut extended = bytes.clone();
    extended.splice(zip64_eocd + 56..zip64_eocd + 56, [0xAA, 0xBB, 0xCC, 0xDD]);
    extended[zip64_eocd + 4..zip64_eocd + 12].copy_from_slice(&48_u64.to_le_bytes());
    assert!(matches!(
        EpubZip::new(Cursor::new(extended)),
        Err(ArchiveError::InvalidContainer(_))
    ));

    let mut mismatched_legacy_record = bytes.clone();
    let legacy_eocd = zip64_eocd + 76;
    mismatched_legacy_record[legacy_eocd + 8..legacy_eocd + 10]
        .copy_from_slice(&2_u16.to_le_bytes());
    mismatched_legacy_record[legacy_eocd + 10..legacy_eocd + 12]
        .copy_from_slice(&2_u16.to_le_bytes());
    assert!(matches!(
        EpubZip::new(Cursor::new(mismatched_legacy_record)),
        Err(ArchiveError::InvalidContainer(_))
    ));

    let locator = zip64_eocd + 56;
    bytes[locator + 4..locator + 8].copy_from_slice(&1_u32.to_le_bytes());
    assert!(matches!(
        EpubZip::new(Cursor::new(bytes)),
        Err(ArchiveError::MultiDiskArchive)
    ));
}

#[test]
fn checks_declared_entry_limit_below_at_and_above_threshold() {
    let bytes = make_zip(&[("OPS/chapter.xhtml", b"abc")], CompressionMethod::Stored);

    let too_small = entry_bytes(bytes.clone(), limits_with(10, 2, 10, 4096)).unwrap_err();
    assert!(matches!(
        too_small,
        ArchiveError::EntryDeclaredTooLarge {
            declared: 3,
            limit: 2,
            ..
        }
    ));
    assert!(too_small.to_string().contains("OPS/chapter.xhtml"));

    for limit in [3, 4] {
        assert_eq!(
            entry_bytes(bytes.clone(), limits_with(10, limit, 10, 4096)).unwrap(),
            b"abc"
        );
    }
}

#[test]
fn cumulative_actual_byte_limit_is_shared_across_entry_reads() {
    let bytes = make_zip(
        &[("OPS/one.xhtml", b"12"), ("OPS/two.xhtml", b"34")],
        CompressionMethod::Stored,
    );
    let mut archive =
        EpubZip::with_limits(Cursor::new(bytes), limits_with(10, 8, 4, 4096)).unwrap();
    let first = archive.entries()[0].id();
    let second = archive.entries()[1].id();
    assert_eq!(archive.read_entry(first).unwrap(), b"12");
    assert_eq!(archive.expanded_bytes_consumed(), 2);
    assert_eq!(archive.read_entry(second).unwrap(), b"34");
    assert_eq!(archive.expanded_bytes_consumed(), 4);

    let mut archive = EpubZip::with_limits(
        Cursor::new(make_zip(
            &[("OPS/one.xhtml", b"12"), ("OPS/two.xhtml", b"34")],
            CompressionMethod::Stored,
        )),
        limits_with(10, 8, 5, 4096),
    )
    .unwrap();
    let first = archive.entries()[0].id();
    let second = archive.entries()[1].id();
    archive.read_entry(first).unwrap();
    assert_eq!(archive.read_entry(second).unwrap(), b"34");
    assert_eq!(archive.expanded_bytes_consumed(), 4);

    let mut underdeclared = make_zip(
        &[("OPS/one.xhtml", b"12"), ("OPS/two.xhtml", b"34")],
        CompressionMethod::Stored,
    );
    set_declared_uncompressed_size(&mut underdeclared, "OPS/two.xhtml", 1);
    let mut archive =
        EpubZip::with_limits(Cursor::new(underdeclared), limits_with(10, 8, 3, 4096)).unwrap();
    let first = archive.entries()[0].id();
    let second = archive.entries()[1].id();
    archive.read_entry(first).unwrap();
    let error = archive.read_entry(second).unwrap_err();
    assert!(matches!(
        error,
        ArchiveError::TotalLimitExceeded { .. } | ArchiveError::EntryRead { .. }
    ));
    assert!(error.to_string().contains("OPS/two.xhtml"));
}

#[test]
fn failed_entry_reads_close_the_session_for_stored_and_deflated_data() {
    for method in [CompressionMethod::Stored, CompressionMethod::Deflated] {
        let mut bytes = make_zip(
            &[("OPS/one.xhtml", b"abc"), ("OPS/two.xhtml", b"safe")],
            method,
        );
        set_declared_uncompressed_size(&mut bytes, "OPS/one.xhtml", 1);
        let mut archive =
            EpubZip::with_limits(Cursor::new(bytes), limits_with(10, 16, 32, 4096)).unwrap();
        let first = archive.entries()[0].id();
        let second = archive.entries()[1].id();

        let first_error = archive.read_entry(first).unwrap_err();
        assert!(first_error.to_string().contains("OPS/one.xhtml"));

        let retry = archive.read_entry(first).unwrap_err();
        assert!(matches!(retry, ArchiveError::ReadSessionFailed { .. }));
        assert!(retry.to_string().contains("OPS/one.xhtml"));

        let other_entry = archive.read_entry(second).unwrap_err();
        assert!(matches!(
            other_entry,
            ArchiveError::ReadSessionFailed { .. }
        ));
        assert!(other_entry.to_string().contains("OPS/two.xhtml"));
    }
}

#[test]
fn rejects_unsafe_paths_before_returning_bytes_and_preserves_sentinel() {
    let sentinel = sentinel_path();
    fs::write(&sentinel, b"outside remains unchanged").unwrap();

    for name in [
        "../sentinel",
        "/absolute.xhtml",
        "C:/drive.xhtml",
        "OPS\\backslash.xhtml",
        "OPS/CON.txt",
        "OPS/trailing.",
    ] {
        let bytes = make_zip(&[(name, b"untrusted")], CompressionMethod::Stored);
        let error = EpubZip::new(Cursor::new(bytes))
            .err()
            .expect("unsafe path must be rejected");
        assert!(error.to_string().contains(name));
        assert_eq!(fs::read(&sentinel).unwrap(), b"outside remains unchanged");
    }

    let _ = fs::remove_file(sentinel);
}

#[test]
fn canonical_alias_resolves_to_same_archive_entry_identity() {
    let bytes = make_zip(
        &[("OPS/chapter.xhtml", b"chapter")],
        CompressionMethod::Stored,
    );
    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let direct = archive
        .entry_by_path("OPS/chapter.xhtml")
        .unwrap()
        .unwrap()
        .id();
    let alias = archive
        .entry_by_path("OPS/./chapter.xhtml")
        .unwrap()
        .unwrap()
        .id();
    assert_eq!(direct, alias);
    assert_eq!(archive.read_entry(alias).unwrap(), b"chapter");
}

#[test]
fn duplicate_canonical_names_and_windows_case_collisions_are_reported() {
    let duplicate = make_zip(
        &[
            ("OPS/chapter.xhtml", b"one"),
            ("OPS/./chapter.xhtml", b"two"),
        ],
        CompressionMethod::Stored,
    );
    assert!(matches!(
        EpubZip::new(Cursor::new(duplicate)),
        Err(ArchiveError::DuplicateCanonicalPath { .. })
    ));

    let case_collision = make_zip(
        &[("OPS/Chapter.xhtml", b"one"), ("OPS/chapter.xhtml", b"two")],
        CompressionMethod::Stored,
    );
    let archive = EpubZip::new(Cursor::new(case_collision)).unwrap();
    let error = archive.validate_windows_materialization().unwrap_err();
    assert!(matches!(
        error,
        ArchiveError::WindowsMaterializationCollision { .. }
    ));
    assert!(error.to_string().contains("OPS/Chapter.xhtml"));
    assert!(error.to_string().contains("OPS/chapter.xhtml"));
}

#[test]
fn windows_materialization_detects_file_ancestors_and_allows_directories() {
    for entries in [
        vec![
            ("OPS/chapter", b"file".as_slice()),
            ("OPS/chapter/part.xhtml", b"child".as_slice()),
        ],
        vec![
            ("OPS/chapter/part.xhtml", b"child".as_slice()),
            ("OPS/chapter", b"file".as_slice()),
        ],
        vec![
            ("OPS/Chapter", b"file".as_slice()),
            ("OPS/chapter/part.xhtml", b"child".as_slice()),
        ],
    ] {
        let zip_entries = entries
            .iter()
            .map(|(name, bytes)| (*name, *bytes))
            .collect::<Vec<_>>();
        let archive = EpubZip::new(Cursor::new(make_zip(
            &zip_entries,
            CompressionMethod::Stored,
        )))
        .unwrap();
        let error = archive.validate_windows_materialization().unwrap_err();
        assert!(matches!(
            error,
            ArchiveError::WindowsMaterializationCollision { .. }
        ));
    }

    let explicit_directory = make_zip(
        &[("OPS/chapter/", b""), ("OPS/chapter/part.xhtml", b"child")],
        CompressionMethod::Stored,
    );
    let archive = EpubZip::new(Cursor::new(explicit_directory)).unwrap();
    archive.validate_windows_materialization().unwrap();

    let implicit_parent = make_zip(
        &[("OPS/chapter/part.xhtml", b"child")],
        CompressionMethod::Stored,
    );
    let archive = EpubZip::new(Cursor::new(implicit_parent)).unwrap();
    archive.validate_windows_materialization().unwrap();
}

#[test]
fn reader_creation_errors_keep_the_entry_name_and_source_message() {
    let mut bytes = make_zip(
        &[("OPS/secret.xhtml", b"secret")],
        CompressionMethod::Stored,
    );
    let local = find_signature(&bytes, [0x50, 0x4b, 0x03, 0x04]);
    let central = find_signature(&bytes, [0x50, 0x4b, 0x01, 0x02]);
    let local_flags = u16::from_le_bytes(bytes[local + 6..local + 8].try_into().unwrap()) | 1;
    let central_flags =
        u16::from_le_bytes(bytes[central + 8..central + 10].try_into().unwrap()) | 1;
    bytes[local + 6..local + 8].copy_from_slice(&local_flags.to_le_bytes());
    bytes[central + 8..central + 10].copy_from_slice(&central_flags.to_le_bytes());

    let mut archive = EpubZip::new(Cursor::new(bytes)).unwrap();
    let id = archive.entries()[0].id();
    let error = archive.read_entry(id).unwrap_err();
    assert!(matches!(
        error,
        ArchiveError::EntryRead { ref entry, .. } if entry == "OPS/secret.xhtml"
    ));
    assert!(error.to_string().contains("OPS/secret.xhtml"));
    assert!(error.to_string().to_ascii_lowercase().contains("password"));
}

#[test]
fn corrupt_crc_and_truncated_containers_never_return_partial_payloads() {
    let mut bad_crc = make_zip(
        &[("OPS/chapter.xhtml", b"chapter")],
        CompressionMethod::Stored,
    );
    let local_crc = 14;
    bad_crc[local_crc..local_crc + 4].copy_from_slice(&0x1234_5678_u32.to_le_bytes());
    let central = find_signature(&bad_crc, [0x50, 0x4b, 0x01, 0x02]);
    bad_crc[central + 16..central + 20].copy_from_slice(&0x1234_5678_u32.to_le_bytes());

    let error = entry_bytes(bad_crc, ArchiveLimits::default()).unwrap_err();
    assert!(error.to_string().contains("OPS/chapter.xhtml"));

    let mut truncated_entry = make_zip(
        &[("OPS/chapter.xhtml", b"chapter")],
        CompressionMethod::Stored,
    );
    truncate_entry_payload(&mut truncated_entry, "OPS/chapter.xhtml");
    let error = entry_bytes(truncated_entry, ArchiveLimits::default()).unwrap_err();
    assert!(error.to_string().contains("OPS/chapter.xhtml"));
    assert!(matches!(
        error,
        ArchiveError::EntryRead { .. } | ArchiveError::EntrySizeMismatch { .. }
    ));

    let mut truncated = make_zip(
        &[("OPS/chapter.xhtml", b"chapter")],
        CompressionMethod::Stored,
    );
    truncated.truncate(truncated.len() - 8);
    assert!(EpubZip::new(Cursor::new(truncated)).is_err());
}

#[test]
fn declared_size_mismatch_is_reported_with_the_entry_name() {
    let mut bytes = make_zip(&[("OPS/chapter.xhtml", b"abc")], CompressionMethod::Stored);
    bytes[22..26].copy_from_slice(&4_u32.to_le_bytes());
    let central = find_signature(&bytes, [0x50, 0x4b, 0x01, 0x02]);
    bytes[central + 24..central + 28].copy_from_slice(&4_u32.to_le_bytes());

    let error = entry_bytes(bytes, ArchiveLimits::default()).unwrap_err();
    assert!(error.to_string().contains("OPS/chapter.xhtml"));
    assert!(matches!(
        error,
        ArchiveError::EntrySizeMismatch { .. } | ArchiveError::EntryRead { .. }
    ));
}

fn find_signature(bytes: &[u8], signature: [u8; 4]) -> usize {
    bytes
        .windows(signature.len())
        .position(|window| window == signature)
        .unwrap()
}

fn set_declared_uncompressed_size(bytes: &mut [u8], name: &str, size: u32) {
    let mut cursor = 0;
    loop {
        let central = bytes[cursor..]
            .windows(4)
            .position(|window| window == [0x50, 0x4b, 0x01, 0x02])
            .map(|offset| cursor + offset)
            .unwrap();
        let name_length =
            u16::from_le_bytes(bytes[central + 28..central + 30].try_into().unwrap()) as usize;
        let extra_length =
            u16::from_le_bytes(bytes[central + 30..central + 32].try_into().unwrap()) as usize;
        let comment_length =
            u16::from_le_bytes(bytes[central + 32..central + 34].try_into().unwrap()) as usize;
        let entry_name = &bytes[central + 46..central + 46 + name_length];
        if entry_name == name.as_bytes() {
            let local_offset =
                u32::from_le_bytes(bytes[central + 42..central + 46].try_into().unwrap()) as usize;
            bytes[local_offset + 22..local_offset + 26].copy_from_slice(&size.to_le_bytes());
            bytes[central + 24..central + 28].copy_from_slice(&size.to_le_bytes());
            return;
        }
        cursor = central + 46 + name_length + extra_length + comment_length;
    }
}

fn truncate_entry_payload(bytes: &mut Vec<u8>, name: &str) {
    let central = find_central_entry(bytes, name);
    let local_offset =
        u32::from_le_bytes(bytes[central + 42..central + 46].try_into().unwrap()) as usize;
    let name_length = u16::from_le_bytes(
        bytes[local_offset + 26..local_offset + 28]
            .try_into()
            .unwrap(),
    ) as usize;
    let extra_length = u16::from_le_bytes(
        bytes[local_offset + 28..local_offset + 30]
            .try_into()
            .unwrap(),
    ) as usize;
    let compressed_size = u32::from_le_bytes(
        bytes[local_offset + 18..local_offset + 22]
            .try_into()
            .unwrap(),
    );
    let data_start = local_offset + 30 + name_length + extra_length;
    let remove_at = data_start + compressed_size as usize - 1;
    bytes.remove(remove_at);
    bytes[local_offset + 18..local_offset + 22]
        .copy_from_slice(&(compressed_size - 1).to_le_bytes());

    let shifted_central = central - 1;
    bytes[shifted_central + 20..shifted_central + 24]
        .copy_from_slice(&(compressed_size - 1).to_le_bytes());
    let eocd = bytes
        .windows(4)
        .rposition(|window| window == [0x50, 0x4b, 0x05, 0x06])
        .unwrap();
    bytes[eocd + 16..eocd + 20].copy_from_slice(&(shifted_central as u32).to_le_bytes());
}

fn find_central_entry(bytes: &[u8], name: &str) -> usize {
    let mut cursor = 0;
    loop {
        let central = bytes[cursor..]
            .windows(4)
            .position(|window| window == [0x50, 0x4b, 0x01, 0x02])
            .map(|offset| cursor + offset)
            .unwrap();
        let name_length =
            u16::from_le_bytes(bytes[central + 28..central + 30].try_into().unwrap()) as usize;
        let extra_length =
            u16::from_le_bytes(bytes[central + 30..central + 32].try_into().unwrap()) as usize;
        let comment_length =
            u16::from_le_bytes(bytes[central + 32..central + 34].try_into().unwrap()) as usize;
        let entry_name = &bytes[central + 46..central + 46 + name_length];
        if entry_name == name.as_bytes() {
            return central;
        }
        cursor = central + 46 + name_length + extra_length + comment_length;
    }
}

#[test]
fn actual_expansion_limits_still_apply_when_declared_size_is_too_small() {
    let mut one_entry = make_zip(&[("OPS/chapter.xhtml", b"abc")], CompressionMethod::Stored);
    set_declared_uncompressed_size(&mut one_entry, "OPS/chapter.xhtml", 1);
    let error = entry_bytes(one_entry, limits_with(10, 2, 10, 4096)).unwrap_err();
    assert!(error.to_string().contains("OPS/chapter.xhtml"));
    assert!(matches!(
        error,
        ArchiveError::EntryLimitExceeded { .. } | ArchiveError::EntryRead { .. }
    ));

    for limit in [3, 4] {
        assert_eq!(
            entry_bytes(
                make_zip(&[("OPS/chapter.xhtml", b"abc")], CompressionMethod::Stored),
                limits_with(10, limit, 10, 4096)
            )
            .unwrap(),
            b"abc"
        );
    }

    let mut two_entries = make_zip(
        &[("OPS/one.xhtml", b"12"), ("OPS/two.xhtml", b"34")],
        CompressionMethod::Stored,
    );
    set_declared_uncompressed_size(&mut two_entries, "OPS/two.xhtml", 1);
    let mut archive =
        EpubZip::with_limits(Cursor::new(two_entries), limits_with(10, 8, 3, 4096)).unwrap();
    let first = archive.entries()[0].id();
    let second = archive.entries()[1].id();
    archive.read_entry(first).unwrap();
    let error = archive.read_entry(second).unwrap_err();
    assert!(error.to_string().contains("OPS/two.xhtml"));
    assert!(matches!(
        error,
        ArchiveError::TotalLimitExceeded { .. }
            | ArchiveError::EntryRead { .. }
            | ArchiveError::EntrySizeMismatch { .. }
    ));
}

fn sentinel_path() -> PathBuf {
    let sequence = NEXT_SENTINEL.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "nt-epub-sentinel-{}-{sequence}.txt",
        std::process::id()
    ))
}

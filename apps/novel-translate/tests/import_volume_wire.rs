//! Independent signed V2 encoding bytes. This synthetic object is NOT an EPUB
//! and these tests do not prove import/source/object/DB acceptance.
use novel_translate::import_codec::{CodecError, VolumeRecord};
use std::io::{Cursor, Write};

const JSON: &str = r###"{"format":"nt.cp05.volume","version":2,"source":{"epub_sha256":"7038d580f0a609dfdc150da91f3d9f53c9729c79af653519483ccc7aa1695268","original_object_sha256":"7038d580f0a609dfdc150da91f3d9f53c9729c79af653519483ccc7aa1695268","original_object_byte_len":20,"parser_version":"nt-epub/1.5.0","mapping_rule_version":"map/1","classification_rule_version":"scope/1","mapping_version":"4d56f2e2841cbb80d76c5d62bbe03e56cb06ae97838bed683ad7a0be5f76deef"},"inventory":{"package_path":"OPS/book.opf","package_version":"3.0","metadata":{"values":[],"unique_identifier_id":null},"resources":[{"id":"chapter","path":"OPS/ch.xhtml","media_type":"application/xhtml+xml","properties":[],"fallback":null,"media_overlay":null,"available":true,"encryption":"NotEncrypted","image_uses":[]}],"spine":[{"idref":"chapter","resource_index":0,"reading_order":0,"linear":true,"kind":"ReflowableDocument","image_uses":[]}],"navigation":[],"primary_navigation":null,"coverage":"Complete","diagnostics":[]},"source_index":{"documents":[{"canonical_path":"OPS/ch.xhtml","domain":"MainFlow","spine_positions":[0],"linear_spine_positions":[0],"blocks":[{"id":"848ddd2079ca9157e5fc1c4fb1cde1e6f311b47b15fa0aa1577621ef7fb67140","canonical_path":"OPS/ch.xhtml","element_name":"p","locator":{"kind":"UniqueXhtmlId","data":{"id":"p"}},"parser_version":"nt-epub/1.5.0","content_hash":"a7a92e38a4497513a1a63572579c65fa3cddae0adeb33efcc626a5a67a8a2eed","unit_runs":[{"kind":"TextRun","text":"A","grapheme_lengths":"AQ==","segments":[[0,1,0,0,1]]}],"inline_metadata":[],"attribute_text":[],"protected_resource_hashes":[]}]}],"anchors":[]},"mapping":{"epub_sha256":"7038d580f0a609dfdc150da91f3d9f53c9729c79af653519483ccc7aa1695268","parser_version":"nt-epub/1.5.0","volume":{"main_flow":[{"kind":"Chapter","data":{"chapter_id":0,"label":null,"status":"Provisional","ranges":[{"canonical_path":"OPS/ch.xhtml","block_id":"848ddd2079ca9157e5fc1c4fb1cde1e6f311b47b15fa0aa1577621ef7fb67140","start":0,"end":1}],"start_evidence":[{"kind":"FileEdge","source":null,"canonical_path":"OPS/ch.xhtml","block_id":"848ddd2079ca9157e5fc1c4fb1cde1e6f311b47b15fa0aa1577621ef7fb67140","unit_offset":0,"label":null,"raw_href":null,"reason":"synthetic","depth":null}]}}],"auxiliary":[]},"navigation":[],"boundary_evidence":[{"kind":"FileEdge","source":null,"canonical_path":"OPS/ch.xhtml","block_id":"848ddd2079ca9157e5fc1c4fb1cde1e6f311b47b15fa0aa1577621ef7fb67140","unit_offset":0,"label":null,"raw_href":null,"reason":"synthetic","depth":null}],"confirmed_boundaries":[],"diagnostics":[]},"scope":{"effective_mode":"AllText","records":[],"notes":[]},"resources":[{"role":"Xhtml","canonical_path":"OPS/ch.xhtml","object_sha256":"8cc6cd285540feded32dbcbbacec7941dacc69316e93d786206897133559da44","byte_len":78,"source_relative_path":null,"declared_media_type":"application/xhtml+xml","format":{"status":"Unknown","name":null},"dimensions":{"status":"Unknown","width":null,"height":null,"unit":null,"svg_width":null,"svg_height":null,"svg_view_box":null},"uses":[]}],"diagnostics":{"source":[],"navigation":[],"unresolved_resource_uses":[],"mapping":[]}}"###;
const PAYLOAD_SHA: &str = "e8e067bde244e6ba0d182a379304b7b5802982720027c6fd2a5d8f6ad600e662";
const MAPPING_SHA: &str = "4d56f2e2841cbb80d76c5d62bbe03e56cb06ae97838bed683ad7a0be5f76deef";
fn wire(json: &str) -> Vec<u8> {
    let mut out = b"NTCP05\0".to_vec();
    out.extend_from_slice(&2u16.to_be_bytes());
    out.extend_from_slice(&(json.len() as u64).to_be_bytes());
    out.extend_from_slice(json.as_bytes());
    out
}
fn golden() -> VolumeRecord {
    VolumeRecord::read_unverified(&mut Cursor::new(wire(JSON))).unwrap()
}
#[test]
fn signed_complete_v2_bytes_and_mapping_digest_match_independent_oracle() {
    let record = golden();
    let mut actual = vec![];
    assert_eq!(record.write_unverified(&mut actual).unwrap(), 3104);
    assert_eq!(actual, wire(JSON));
    let mut sink = nt_storage::ImportDigestSink::new(actual.len() as u64).unwrap();
    sink.write_all(&actual).unwrap();
    assert_eq!(sink.finish().unwrap().hash.to_string(), PAYLOAD_SHA);
    assert_eq!(
        serde_json::to_string(&record.mapping_digest().unwrap()).unwrap(),
        format!("\"{MAPPING_SHA}\"")
    );
    record.validate_structure().unwrap();
}

fn error(bytes: Vec<u8>) -> CodecError {
    match VolumeRecord::read_unverified(&mut Cursor::new(bytes)) {
        Ok(_) => panic!("invalid wire accepted"),
        Err(error) => error,
    }
}

#[test]
fn envelope_rejects_wrong_magic_versions_lengths_truncation_and_trailing_data() {
    let valid = wire(JSON);
    assert_eq!(error(vec![]), CodecError::InvalidEnvelope);
    let mut wrong = valid.clone();
    wrong[0] = 0;
    assert_eq!(error(wrong), CodecError::InvalidEnvelope);
    for version in [0u16, 1, 3, u16::MAX] {
        let mut wrong = valid.clone();
        wrong[7..9].copy_from_slice(&version.to_be_bytes());
        assert_eq!(error(wrong), CodecError::UnsupportedWireVersion);
    }
    for length in [0, JSON.len() as u64 - 1, JSON.len() as u64 + 1] {
        let mut wrong = valid.clone();
        wrong[9..17].copy_from_slice(&length.to_be_bytes());
        assert_eq!(error(wrong), CodecError::InvalidEnvelope);
    }
    let mut wrong = valid.clone();
    wrong.pop();
    assert_eq!(error(wrong), CodecError::InvalidEnvelope);
    let mut wrong = valid;
    wrong.push(0);
    assert_eq!(error(wrong), CodecError::InvalidEnvelope);
}

#[test]
fn declared_limit_rejects_after_header_without_reading_or_seeking_a_body() {
    use std::io::{self, Read, Seek, SeekFrom};
    struct HeaderOnly(Cursor<Vec<u8>>);
    impl Read for HeaderOnly {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.0.position() >= 17 {
                return Err(io::Error::other("body must not be read"));
            }
            self.0.read(out)
        }
    }
    impl Seek for HeaderOnly {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            match position {
                SeekFrom::Start(0) => self.0.seek(position),
                _ => Err(io::Error::other("body must not be sought")),
            }
        }
    }
    for length in [512 * 1024 * 1024 - 16, u64::MAX] {
        let mut header = b"NTCP05\0".to_vec();
        header.extend_from_slice(&2u16.to_be_bytes());
        header.extend_from_slice(&length.to_be_bytes());
        let mut source = HeaderOnly(Cursor::new(header));
        assert!(matches!(
            VolumeRecord::read_unverified(&mut source),
            Err(CodecError::ResourceLimit)
        ));
        assert_eq!(source.0.position(), 17);
    }
}

#[test]
fn whole_payload_uses_closed_fields_checked_widths_and_one_canonical_byte_form() {
    for json in [
        JSON.replacen("\"version\":2,", "\"version\":2,\"version\":2,", 1),
        JSON.replacen("\"format\":", "\"unknown\":0,\"format\":", 1),
        JSON.replacen("\"version\":2", "\"version\":18446744073709551616", 1),
        JSON.replace("\"kind\":\"TextRun\"", "\"kind\":\"FutureTextRun\""),
        JSON.replace("\"unit_offset\":0", "\"unit_offset\":4294967296"),
    ] {
        assert_eq!(error(wire(&json)), CodecError::InvalidJson);
    }
    for json in [
        JSON.replacen(
            "\"format\":\"nt.cp05.volume\",\"version\":2",
            "\"version\":2,\"format\":\"nt.cp05.volume\"",
            1,
        ),
        format!(" {JSON}"),
        JSON.replace("\"text\":\"A\"", "\"text\":\"\\u0041\""),
    ] {
        assert_eq!(error(wire(&json)), CodecError::NonCanonicalJson);
    }
    assert_eq!(
        error(wire(&JSON.replacen("\"version\":2", "\"version\":3", 1))),
        CodecError::UnsupportedWireVersion
    );
}

#[test]
fn syntax_decode_never_implies_projection_or_mapping_acceptance() {
    let wrong = JSON.replace(
        "\"grapheme_lengths\":\"AQ==\"",
        "\"grapheme_lengths\":\"Ag==\"",
    );
    let data = VolumeRecord::read_unverified(&mut Cursor::new(wire(&wrong))).unwrap();
    assert_eq!(data.validate_structure(), Err(CodecError::InvalidTextRun));
    let mut data = golden();
    data.source.mapping_version.0[0] ^= 1;
    assert_eq!(data.validate_structure(), Err(CodecError::InvalidRecord));
    let mut value = serde_json::to_value(golden()).unwrap();
    value["mapping"]["volume"]["main_flow"][0]["data"]["ranges"][0]["end"] = 0.into();
    let data: VolumeRecord = serde_json::from_value(value).unwrap();
    assert_eq!(data.validate_structure(), Err(CodecError::InvalidRecord));
}

#[test]
fn mapping_digest_changes_with_source_mapping_and_resource_identity_but_not_scope() {
    let mut data = golden();
    let original = data.mapping_digest().unwrap();
    let mut value = serde_json::to_value(&data).unwrap();
    value["mapping"]["volume"]["main_flow"][0]["data"]["label"] = "renamed chapter".into();
    let renamed: VolumeRecord = serde_json::from_value(value).unwrap();
    assert_ne!(renamed.mapping_digest().unwrap(), original);
    assert_eq!(
        renamed.source_index.documents[0].blocks[0].id,
        data.source_index.documents[0].blocks[0].id
    );
    data.resources[0].byte_len += 1;
    assert_ne!(data.mapping_digest().unwrap(), original);
    let mut data = golden();
    let mut extra = data.resources[0].clone();
    extra.canonical_path = "OPS/second.xhtml".into();
    data.resources.push(extra);
    let digest = data.mapping_digest().unwrap();
    data.resources.reverse();
    assert_eq!(data.mapping_digest().unwrap(), digest);
    data.resources[0].canonical_path = data.resources[1].canonical_path.clone();
    assert_eq!(data.mapping_digest(), Err(CodecError::InvalidRecord));
    let mut value = serde_json::to_value(golden()).unwrap();
    value["scope"]["effective_mode"] = "Story".into();
    let changed: VolumeRecord = serde_json::from_value(value).unwrap();
    assert_eq!(changed.mapping_digest().unwrap(), original);
}

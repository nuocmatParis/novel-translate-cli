use roxmltree::Document;

use crate::inventory::ResourceEncryption;
use crate::xml::{XmlFailure, node_position, with_xml_document};

const OCF_NAMESPACE: &str = "urn:oasis:names:tc:opendocument:xmlns:container";
const XML_ENCRYPTION_NAMESPACE: &str = "http://www.w3.org/2001/04/xmlenc#";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EncryptionRecord {
    pub href: String,
    pub kind: ResourceEncryption,
    pub line: u32,
    pub column: u32,
}

pub(crate) fn parse_encryption(bytes: &[u8]) -> Result<Vec<EncryptionRecord>, XmlFailure> {
    let (valid_root, position, records) = with_xml_document(
        bytes,
        super::xml::DEFAULT_XML_MAX_BYTES,
        super::xml::DEFAULT_XML_MAX_NODES,
        |document| {
            let root = document.root_element();
            let valid_root = root.tag_name().name() == "encryption"
                && root.tag_name().namespace() == Some(OCF_NAMESPACE);
            let position = document.text_pos_at(root.range().start);
            (valid_root, position, extract_records(document))
        },
    )?;
    if !valid_root {
        return Err(XmlFailure::new(
            super::xml::XmlFailureKind::Malformed,
            Some(position),
        ));
    }
    Ok(records)
}

fn extract_records(document: &Document<'_>) -> Vec<EncryptionRecord> {
    document
        .descendants()
        .filter(|node| {
            node.is_element()
                && node.tag_name().name() == "EncryptedData"
                && node.tag_name().namespace() == Some(XML_ENCRYPTION_NAMESPACE)
        })
        .map(|encrypted| {
            let method = encrypted.descendants().find(|node| {
                node.is_element()
                    && node.tag_name().name() == "EncryptionMethod"
                    && node.tag_name().namespace() == Some(XML_ENCRYPTION_NAMESPACE)
            });
            let reference = encrypted.descendants().find(|node| {
                node.is_element()
                    && node.tag_name().name() == "CipherReference"
                    && node.tag_name().namespace() == Some(XML_ENCRYPTION_NAMESPACE)
            });
            let href = reference
                .and_then(|node| node.attribute("URI"))
                .unwrap_or_default()
                .trim()
                .to_owned();
            let algorithm = method
                .and_then(|node| node.attribute("Algorithm"))
                .unwrap_or_default();
            let kind = match algorithm {
                "http://www.idpf.org/2008/embedding" => ResourceEncryption::IdpfFontObfuscation,
                "http://ns.adobe.com/pdf/enc#RC" => ResourceEncryption::AdobeFontObfuscation,
                _ => ResourceEncryption::Unsupported,
            };
            let (line, column) = node_position(reference.unwrap_or(encrypted));
            EncryptionRecord {
                href,
                kind,
                line,
                column,
            }
        })
        .collect()
}

use crate::xml::{XmlFailure, XmlFailureKind, with_xml_document};

const OCF_NAMESPACE: &str = "urn:oasis:names:tc:opendocument:xmlns:container";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Rootfile {
    pub full_path: String,
    pub media_type: Option<String>,
}

pub(crate) fn parse_container(bytes: &[u8]) -> Result<Vec<Rootfile>, XmlFailure> {
    with_xml_document(
        bytes,
        super::xml::DEFAULT_XML_MAX_BYTES,
        super::xml::DEFAULT_XML_MAX_NODES,
        |document| {
            let root = document.root_element();
            if root.tag_name().name() != "container"
                || root.tag_name().namespace() != Some(OCF_NAMESPACE)
            {
                return Err(XmlFailure::new(
                    XmlFailureKind::Malformed,
                    Some(document.text_pos_at(root.range().start)),
                ));
            }
            Ok(rootfiles(root))
        },
    )?
}

fn rootfiles(root: roxmltree::Node<'_, '_>) -> Vec<Rootfile> {
    root.children()
        .find(|node| {
            node.is_element()
                && node.tag_name().name() == "rootfiles"
                && node.tag_name().namespace() == Some(OCF_NAMESPACE)
        })
        .into_iter()
        .flat_map(|rootfiles| rootfiles.children())
        .filter(|node| node.is_element())
        .filter(|node| {
            node.tag_name().name() == "rootfile"
                && node.tag_name().namespace() == Some(OCF_NAMESPACE)
        })
        .filter_map(|node| {
            let full_path = node.attribute("full-path")?.trim();
            if full_path.is_empty() {
                return None;
            }
            Some(Rootfile {
                full_path: full_path.to_owned(),
                media_type: node.attribute("media-type").map(str::to_owned),
            })
        })
        .collect()
}

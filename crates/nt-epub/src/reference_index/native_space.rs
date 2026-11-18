use std::cell::{Cell, RefCell};
use std::rc::Rc;

use nt_domain::volume_import::{CapturedSourceError, MAX_IMPORT_WORKING_BYTES};
use roxmltree::{Document, Node};

use crate::{EpubInventory, EpubZip, inspect};

thread_local! {
    static CURRENT: RefCell<Option<Rc<Space>>> = const { RefCell::new(None) };
}

struct Space {
    free: Cell<usize>,
    denied: Cell<bool>,
    reserve_legacy_projection: bool,
}

#[derive(Debug)]
pub(crate) struct Limit;

pub(crate) struct Lease {
    space: Option<Rc<Space>>,
    bytes: usize,
}

impl Lease {
    pub(crate) fn release_scratch(&mut self, retained: usize) -> Result<(), Limit> {
        let scratch = self.bytes.checked_sub(retained).ok_or(Limit)?;
        if let Some(space) = &self.space {
            space.free.set(space.free.get() + scratch);
        }
        self.bytes = retained;
        Ok(())
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if let Some(space) = &self.space {
            space.free.set(space.free.get() + self.bytes);
        }
    }
}

struct Session;

impl Drop for Session {
    fn drop(&mut self) {
        CURRENT.with(|current| current.replace(None));
    }
}

pub(crate) fn scoped<T>(
    available: usize,
    body: impl FnOnce() -> T,
) -> Result<T, CapturedSourceError> {
    run_scope(available, true, body)
}

pub(crate) fn scoped_replay<T>(
    available: usize,
    body: impl FnOnce() -> T,
) -> Result<T, CapturedSourceError> {
    run_scope(available, false, body)
}

fn run_scope<T>(
    available: usize,
    reserve_legacy_projection: bool,
    body: impl FnOnce() -> T,
) -> Result<T, CapturedSourceError> {
    if active() {
        return Err(CapturedSourceError::ResourceLimit);
    }
    let space = Rc::new(Space {
        free: Cell::new(available.min(MAX_IMPORT_WORKING_BYTES)),
        denied: Cell::new(false),
        reserve_legacy_projection,
    });
    CURRENT.with(|current| current.replace(Some(Rc::clone(&space))));
    let _session = Session;
    let output = body();
    if space.denied.get() {
        return Err(CapturedSourceError::ResourceLimit);
    }
    Ok(output)
}

pub(crate) fn remaining() -> Result<usize, CapturedSourceError> {
    CURRENT.with(|current| {
        current
            .borrow()
            .as_ref()
            .filter(|space| !space.denied.get())
            .map(|space| space.free.get())
            .ok_or(CapturedSourceError::ResourceLimit)
    })
}

pub(crate) fn active() -> bool {
    CURRENT.with(|current| current.borrow().is_some())
}

pub(crate) fn denied() -> bool {
    CURRENT.with(|current| {
        current
            .borrow()
            .as_ref()
            .is_some_and(|space| space.denied.get())
    })
}

fn spend(space: &Space, bytes: usize) -> Result<(), Limit> {
    match space.free.get().checked_sub(bytes) {
        Some(remaining) if !space.denied.get() => {
            space.free.set(remaining);
            Ok(())
        }
        _ => {
            space.denied.set(true);
            Err(Limit)
        }
    }
}

pub(crate) fn retain(bytes: usize) -> Result<(), Limit> {
    CURRENT.with(|current| {
        let active = current.borrow();
        active.as_ref().map_or(Ok(()), |space| spend(space, bytes))
    })
}

pub(crate) fn hold(bytes: usize) -> Result<Lease, Limit> {
    CURRENT.with(|current| {
        let space = current.borrow().clone();
        if let Some(active) = &space {
            spend(active, bytes)?;
        }
        Ok(Lease { space, bytes })
    })
}

pub(crate) fn xml_decode(bytes: &[u8]) -> Result<Lease, Limit> {
    // Account for UTF-16 units, the growing output and encoding declaration
    // checks during conversion. The caller then keeps the actual output capacity.
    hold(bytes.len().saturating_mul(6).saturating_add(4096))
}

pub(crate) fn xml_entity_tables(text: &str) -> Result<Lease, Limit> {
    let declarations = text.as_bytes().iter().filter(|byte| **byte == b'<').count();
    hold(
        text.len()
            .saturating_mul(4)
            .saturating_add(declarations.saturating_mul(512))
            .saturating_add(4096),
    )
}

pub(crate) fn xml_document(
    text: &str,
    expanded: usize,
    namespace_markers: usize,
    internal_entities: bool,
) -> Result<Lease, Limit> {
    let mut openings = 0;
    let mut equals = 0;
    let mut references = 0;
    for byte in text.bytes() {
        openings += usize::from(byte == b'<');
        equals += usize::from(byte == b'=');
        references += usize::from(byte == b'&');
    }
    let bytes = text.len().max(expanded);
    let expanding = internal_entities && references > 0;
    let structures = if expanding {
        bytes.saturating_mul(1024)
    } else {
        openings
            .saturating_mul(512)
            .saturating_add(equals.saturating_mul(512))
            .saturating_add(references.saturating_mul(128))
    };
    hold(
        structures
            .saturating_add(namespace_order_bytes(namespace_markers))
            .saturating_add(bytes.saturating_mul(8))
            .saturating_add(16 * 1024),
    )
}

fn namespace_order_bytes(declarations: usize) -> usize {
    // roxmltree 0.21.2 shares the parent's namespace range until a declaration
    // changes it. Only then can it append inherited indices to tree_order.
    // There are at most `declarations` new ranges, each with at most the same
    // number of declarations plus the built-in xml binding. Allow the old and
    // new Vec buffers to coexist during growth (u16 indices), plus initial space.
    let indices = declarations
        .saturating_mul(declarations.saturating_add(1))
        .saturating_add(1);
    indices.saturating_mul(8).saturating_add(64)
}

pub(crate) fn retain_xml_projection(document: &Document<'_>) -> Result<(), Limit> {
    // Compact replay computes its own projection with the allowance left
    // while the parser is still live. Legacy visitors reserve it here.
    if !CURRENT.with(|current| {
        current
            .borrow()
            .as_ref()
            .is_some_and(|space| space.reserve_legacy_projection)
    }) {
        return Ok(());
    }
    let navigation = document.descendants().any(|node| {
        node.has_tag_name(("http://www.w3.org/1999/xhtml", "nav")) || node.has_tag_name("navMap")
    });
    let mut bytes = 4096usize;
    for node in document.descendants() {
        if node.is_element() {
            bytes = bytes.saturating_add(1024);
            let tag = node.tag_name();
            let names = tag.name().len() + tag.namespace().map_or(0, str::len);
            bytes = bytes.saturating_add(names.saturating_mul(8));
            for attr in node.attributes() {
                let length = attr.name().len() + attr.value().len();
                let words = if attr.name() == "properties" {
                    attr.value().split_ascii_whitespace().count()
                } else {
                    0
                };
                bytes = bytes
                    .saturating_add(length.saturating_mul(16))
                    .saturating_add(words.saturating_mul(256));
            }
        } else if node.is_text() {
            let length = node.text().map_or(0, str::len);
            let clone_bytes = if navigation {
                let ancestors = node.ancestors().filter(Node::is_element).take(257).count();
                length.saturating_mul(40 * ancestors)
            } else {
                length.saturating_mul(8)
            };
            bytes = bytes.saturating_add(clone_bytes);
        }
    }
    retain(bytes)
}

pub(crate) fn retain_path_copies(path: &str, copies: usize) -> Result<(), Limit> {
    retain(path.len().saturating_mul(copies))
}

pub(crate) fn uri(base: &str, href: &str) -> Result<Lease, Limit> {
    let combined = base.len().saturating_add(href.len());
    retain(combined.saturating_mul(8))?;
    hold(combined.saturating_add(32).saturating_mul(128))
}

pub(crate) fn retain_navigation_merge(inventory: &EpubInventory) -> Result<(), Limit> {
    fn items(navigation: &[crate::NavigationItem]) -> usize {
        navigation.iter().fold(0usize, |sum, item| {
            let target = item.target.as_ref().map_or(0, |target| {
                target.resource_path.as_ref().map_or(0, String::len)
                    + target.fragment.as_ref().map_or(0, String::len)
            });
            sum.saturating_add(512)
                .saturating_add(item.label.len())
                .saturating_add(target)
                .saturating_add(items(&item.children))
        })
    }
    for tree in &inventory.navigation {
        retain_path_copies(&tree.resource_path, 4)?;
        retain(items(&tree.items))?;
    }
    for diagnostic in &inventory.diagnostics {
        if let Some(path) = &diagnostic.resource_path {
            retain_path_copies(path, 1)?;
        }
        retain(512)?;
    }
    Ok(())
}

impl EpubInventory {
    pub fn inspect_captured(
        archive: &mut EpubZip<impl std::io::Read + std::io::Seek>,
        working_bytes: usize,
    ) -> Result<Self, CapturedSourceError> {
        scoped(working_bytes, || inspect(archive))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_scratch_is_returned_while_the_output_stays_reserved() {
        scoped(128, || {
            let mut buffer = hold(96).unwrap();
            assert_eq!(remaining().unwrap(), 32);
            assert!(buffer.release_scratch(97).is_err());
            buffer.release_scratch(24).unwrap();
            assert_eq!(remaining().unwrap(), 104);
            let parser = hold(90).unwrap();
            assert_eq!(remaining().unwrap(), 14);
            drop(parser);
            drop(buffer);
            assert_eq!(remaining().unwrap(), 128);
        })
        .unwrap();
    }

    #[test]
    fn temporary_and_retained_space_share_the_allowance() {
        let result = scoped(64, || {
            retain(16).unwrap();
            {
                let _buffer = hold(32).unwrap();
                retain(8).unwrap();
            }
            retain(40).unwrap();
            assert!(hold(1).is_err());
        });
        assert_eq!(result, Err(CapturedSourceError::ResourceLimit));
        assert!(hold(65).is_ok());
        assert!(scoped(64, || hold(64).is_ok()).unwrap());
    }

    #[test]
    fn xml_namespace_order_spends_space_before_the_parser() {
        let namespaces = "<root xmlns='urn:book'><section xmlns:i='urn:image' /></root>";
        let markers = namespaces.matches("xmlns").count();
        let baseline = namespace_order_bytes(0);
        let required = namespace_order_bytes(markers);
        assert_eq!(baseline, 72);
        assert_eq!(required, baseline + 6 * 8);
        // Equal-size inputs with the same '<' and '=' counts, where only the
        // namespaces create extra table entries in the real parser.
        let attributes = namespaces
            .replace("xmlns", "plain")
            .replace("plain:i", "plain-i");
        assert_eq!(attributes.len(), namespaces.len());
        let openings = attributes.matches('<').count();
        let equals = attributes.matches('=').count();
        let allowance = (openings + equals) * 512 + attributes.len() * 8 + 16 * 1024 + baseline;
        assert!(
            scoped(allowance, || xml_document(
                &attributes,
                attributes.len(),
                0,
                false
            ))
            .is_ok()
        );
        assert!(matches!(
            scoped(allowance, || xml_document(
                namespaces,
                namespaces.len(),
                markers,
                false
            )),
            Err(CapturedSourceError::ResourceLimit)
        ));
        assert!(
            scoped(allowance + required - baseline, || {
                xml_document(namespaces, namespaces.len(), markers, false).is_ok()
            })
            .unwrap()
        );
    }
}

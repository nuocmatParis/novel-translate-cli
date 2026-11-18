//! One bounded base context for each decoded EPUB XML document.
//! HTML URL attributes use the document base; SVG URL attributes additionally
//! use inherited XML Base. CSS URLs use the document or stylesheet base.

use std::cell::Cell;
use std::mem::size_of;
use std::rc::Rc;

use roxmltree::{Document, Node, NodeId};

use crate::package::{HrefError, ResolvedHref, resolve_href};

const XHTML: &str = "http://www.w3.org/1999/xhtml";
const SVG: &str = "http://www.w3.org/2000/svg";
const XML: &str = "http://www.w3.org/XML/1998/namespace";
const DIRECTORY_ANCHOR: &str = ".__nt_directory_base__";
pub(crate) const MAX_REFERENCE_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReferenceKind {
    Html,
    Svg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseLimit {
    Bytes,
    Nodes,
    Work,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseError {
    Invalid(HrefError),
    Limit(BaseLimit),
}

impl BaseError {
    pub(crate) fn is_limit(self) -> bool {
        matches!(self, Self::Limit(_))
    }
}

impl From<HrefError> for BaseError {
    fn from(value: HrefError) -> Self {
        Self::Invalid(value)
    }
}

#[derive(Debug)]
enum Base {
    // A directory keeps a private file anchor solely for joining descendants.
    // A fragment/empty reference must not alias a real file at that anchor.
    Internal { anchor: String, directory: bool },
    External,
    Invalid(HrefError),
}

#[derive(Clone, Copy)]
struct Limits {
    bytes: usize,
    nodes: usize,
    work: usize,
}

const LIMITS: Limits = Limits {
    bytes: 64 * 1024 * 1024,
    nodes: 1_000_000,
    work: 256 * 1024 * 1024,
};

pub(crate) struct BaseContext {
    document: Rc<Base>,
    svg: Vec<Option<Rc<Base>>>,
    retained: Cell<usize>,
    work: Cell<usize>,
    limits: Limits,
}

impl BaseContext {
    pub(crate) fn from_document(path: &str, document: &Document<'_>) -> Result<Self, BaseError> {
        Self::from_document_with_limits(path, document, LIMITS)
    }

    fn from_document_with_limits(
        path: &str,
        document: &Document<'_>,
        limits: Limits,
    ) -> Result<Self, BaseError> {
        let count = document.descendants().count();
        if count > limits.nodes {
            return Err(BaseError::Limit(BaseLimit::Nodes));
        }
        if document.input_text().len() > crate::xml::DEFAULT_XML_MAX_BYTES {
            return Err(BaseError::Limit(BaseLimit::Bytes));
        }
        let mut context = Self::for_html_base_with_limits(path, first_html_base(document), limits)?;
        context.charge_work(document.input_text().len())?;
        context.charge_work(
            count
                .checked_mul(4)
                .ok_or(BaseError::Limit(BaseLimit::Work))?,
        )?;

        // HTML attributes never consult this table. Non-HTML/SVG document
        // profiles (OPF/NCX) have no base declarations handled by this helper.
        let root = document.root_element();
        let content_profile = matches!(root.tag_name().namespace(), Some(XHTML | SVG));
        if !content_profile
            || !document
                .descendants()
                .any(|n| n.tag_name().namespace() == Some(SVG))
            || !document
                .descendants()
                .any(|n| n.attribute((XML, "base")).is_some())
        {
            return Ok(context);
        }

        // Node IDs are dense indices in roxmltree. Allocate once, after checking
        // the full vector and traversal-stack footprint. No ancestor rescans.
        let slots = count
            .checked_mul(size_of::<Option<Rc<Base>>>() + size_of::<(NodeId, Rc<Base>)>())
            .ok_or(BaseError::Limit(BaseLimit::Bytes))?;
        context.charge_retained(slots)?;
        context
            .svg
            .try_reserve_exact(count)
            .map_err(|_| BaseError::Limit(BaseLimit::Bytes))?;
        context.svg.resize_with(count, || None);
        let mut stack = Vec::<(NodeId, Rc<Base>)>::new();
        stack
            .try_reserve_exact(count)
            .map_err(|_| BaseError::Limit(BaseLimit::Bytes))?;
        for node in document.descendants().filter(Node::is_element) {
            let parent_id = node.parent_element().map(|parent| parent.id());
            while stack.last().is_some_and(|(id, _)| Some(*id) != parent_id) {
                stack.pop();
            }
            let parent = stack
                .last()
                .map(|(_, base)| base)
                .unwrap_or(&context.document);
            let current = match node.attribute((XML, "base")) {
                Some(raw) => context.derive(parent, raw)?,
                None => Rc::clone(parent),
            };
            if node.tag_name().namespace() == Some(SVG) {
                context.svg[node.id().get_usize()] = Some(Rc::clone(&current));
            }
            stack.push((node.id(), current));
        }
        Ok(context)
    }

    /// Nav supplies its first HTML base. NCX and external CSS supply None.
    pub(crate) fn for_html_base(path: &str, raw: Option<&str>) -> Result<Self, BaseError> {
        Self::for_html_base_with_limits(path, raw, LIMITS)
    }

    fn for_html_base_with_limits(
        path: &str,
        raw: Option<&str>,
        limits: Limits,
    ) -> Result<Self, BaseError> {
        check_reference_bytes(path)?;
        let overhead = size_of::<Base>() + 2 * size_of::<usize>();
        let initial = path
            .len()
            .checked_add(overhead)
            .ok_or(BaseError::Limit(BaseLimit::Bytes))?;
        if initial > limits.bytes {
            return Err(BaseError::Limit(BaseLimit::Bytes));
        }
        let mut context = Self {
            document: Rc::new(Base::Internal {
                anchor: path.to_owned(),
                directory: false,
            }),
            svg: Vec::new(),
            retained: Cell::new(initial),
            work: Cell::new(0),
            limits,
        };
        if let Some(raw) = raw {
            check_reference_bytes(raw)?;
            context.charge_work(raw.len())?;
            let raw = raw.trim_matches(|ch: char| ch.is_ascii_whitespace());
            // HTML explicitly falls back for these base schemes. The first
            // base still wins; a later base is not substituted.
            let raw = raw.trim_matches(|c: char| c.is_ascii_whitespace());
            let forbidden = raw.split_once(':').is_some_and(|(scheme, _)| {
                scheme.eq_ignore_ascii_case("data") || scheme.eq_ignore_ascii_case("javascript")
            });
            if !forbidden {
                context.document = context.derive(&context.document, raw)?;
            }
        }
        Ok(context)
    }

    /// Node must belong to the document used to construct this context.
    pub(crate) fn resolve(
        &self,
        node: Node<'_, '_>,
        href: &str,
        kind: ReferenceKind,
    ) -> Result<ResolvedHref, BaseError> {
        let base = if kind == ReferenceKind::Svg && !self.svg.is_empty() {
            self.svg
                .get(node.id().get_usize())
                .and_then(Option::as_ref)
                .ok_or(BaseError::Invalid(HrefError::InvalidArchivePath))?
        } else {
            &self.document
        };
        self.resolve_on(base, href)
    }

    pub(crate) fn resolve_document_reference(&self, href: &str) -> Result<ResolvedHref, BaseError> {
        self.resolve_on(&self.document, href)
    }

    fn charge_work(&self, amount: usize) -> Result<(), BaseError> {
        charge(&self.work, amount, self.limits.work, BaseLimit::Work)
    }

    fn charge_retained(&self, amount: usize) -> Result<(), BaseError> {
        charge(&self.retained, amount, self.limits.bytes, BaseLimit::Bytes)
    }

    fn new_base(&self, base: Base) -> Result<Rc<Base>, BaseError> {
        let bytes = match &base {
            Base::Internal { anchor, .. } => anchor.len(),
            _ => 0,
        };
        self.charge_retained(
            bytes
                .checked_add(size_of::<Base>() + 2 * size_of::<usize>())
                .ok_or(BaseError::Limit(BaseLimit::Bytes))?,
        )?;
        Ok(Rc::new(base))
    }

    fn derive(&self, parent: &Rc<Base>, raw: &str) -> Result<Rc<Base>, BaseError> {
        check_reference_bytes(raw)?;
        self.charge_work(
            raw.len()
                .checked_mul(32)
                .ok_or(BaseError::Limit(BaseLimit::Work))?,
        )?;
        if raw.is_empty() {
            return Ok(Rc::clone(parent));
        }
        if has_scheme(raw) || raw.starts_with("//") {
            return self.new_base(Base::External);
        }
        let (path, fragment) = raw.split_once('#').unwrap_or((raw, ""));
        if raw.contains('#') {
            // Fragment is discarded from the base, but malformed percent
            // encoding remains rejected by the existing URI contract.
            match resolve_href("base", &format!("#{fragment}")) {
                Ok(_) => {}
                Err(error) => return self.new_base(Base::Invalid(error)),
            }
        }
        if path.is_empty() {
            return Ok(Rc::clone(parent));
        }
        let last = path.rsplit('/').next().unwrap_or_default();
        let decoded_dot = last.to_ascii_lowercase().replace("%2e", ".");
        let directory = path.ends_with('/') || decoded_dot == "." || decoded_dot == "..";
        let reference = if directory {
            format!(
                "{path}{}{DIRECTORY_ANCHOR}",
                if path.ends_with('/') { "" } else { "/" }
            )
        } else {
            path.to_owned()
        };
        // Include the sentinel only in the internal join, not the user limit.
        match self.resolve_on_inner(parent, &reference) {
            Ok(target) if target.external => self.new_base(Base::External),
            Ok(target) => match target.path {
                Some(anchor) => {
                    check_reference_bytes(&anchor)?;
                    // The next resolution's retained string has already been
                    // bounded; reserve its accounting before storing the Rc.
                    self.new_base(Base::Internal { anchor, directory })
                }
                None => self.new_base(Base::Invalid(HrefError::InvalidArchivePath)),
            },
            Err(BaseError::Invalid(error)) => self.new_base(Base::Invalid(error)),
            Err(error) => Err(error),
        }
    }

    fn resolve_on(&self, base: &Base, href: &str) -> Result<ResolvedHref, BaseError> {
        check_reference_bytes(href)?;
        self.resolve_on_inner(base, href)
    }

    fn resolve_on_inner(&self, base: &Base, href: &str) -> Result<ResolvedHref, BaseError> {
        let anchor_bytes = match base {
            Base::Internal { anchor, .. } => anchor.len(),
            _ => 0,
        };
        let allocation_work = anchor_bytes
            .checked_add(href.len())
            .and_then(|n| n.checked_add(1024))
            .and_then(|n| n.checked_mul(32))
            .ok_or(BaseError::Limit(BaseLimit::Work))?;
        self.charge_work(allocation_work)?;
        if href.is_empty() {
            return Err(HrefError::Empty.into());
        }
        if has_scheme(href) || href.starts_with("//") {
            return Ok(ResolvedHref {
                path: None,
                fragment: None,
                external: true,
            });
        }
        match base {
            Base::External => Ok(ResolvedHref {
                path: None,
                fragment: None,
                external: true,
            }),
            Base::Invalid(error) => Err((*error).into()),
            Base::Internal { anchor, directory } => {
                if *directory && href.starts_with('#') {
                    return Err(HrefError::InvalidArchivePath.into());
                }
                Ok(resolve_href(anchor, href)?)
            }
        }
    }
}

fn charge(
    counter: &Cell<usize>,
    amount: usize,
    maximum: usize,
    kind: BaseLimit,
) -> Result<(), BaseError> {
    let next = counter
        .get()
        .checked_add(amount)
        .ok_or(BaseError::Limit(kind))?;
    if next > maximum {
        return Err(BaseError::Limit(kind));
    }
    counter.set(next);
    Ok(())
}

pub(crate) fn check_reference_bytes(value: &str) -> Result<(), BaseError> {
    if value.len() > MAX_REFERENCE_BYTES {
        Err(BaseError::Limit(BaseLimit::Bytes))
    } else {
        Ok(())
    }
}

pub(crate) fn first_html_base<'a>(document: &'a Document<'_>) -> Option<&'a str> {
    let root = document.root_element();
    if !matches!(
        (root.tag_name().namespace(), root.tag_name().name()),
        (Some(XHTML), "html") | (Some(SVG), "svg")
    ) {
        return None;
    }
    document
        .descendants()
        .find(|node| {
            node.is_element()
                && node.tag_name().namespace() == Some(XHTML)
                && node.tag_name().name() == "base"
                && unqualified(*node, "href").is_some()
        })
        .and_then(|node| unqualified(node, "href"))
}

fn has_scheme(value: &str) -> bool {
    value.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
    })
}

fn unqualified<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attribute| attribute.namespace().is_none() && attribute.name() == name)
        .map(|attribute| attribute.value())
}

/// Plain URL attributes must not match a prefixed local-name lookalike.
pub(crate) fn unqualified_attribute<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attribute| attribute.namespace().is_none() && attribute.name() == name)
        .map(|attribute| attribute.value())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(base: &str, href: &str) -> Result<ResolvedHref, BaseError> {
        BaseContext::for_html_base("OPS/Text/ch.xhtml", Some(base))?
            .resolve_document_reference(href)
    }

    #[test]
    fn directory_file_empty_fragment_and_first_base_are_distinct() {
        for base in [
            "../Images/",
            "../Images/.",
            "../Images/%2e",
            "../Images/sub/..",
        ] {
            assert_eq!(
                target(base, "cover%20one.png").unwrap().path.as_deref(),
                Some("OPS/Images/cover one.png")
            );
            assert!(target(base, "#x").is_err());
        }
        assert_eq!(
            target("../Images/cover.png#old", "#new")
                .unwrap()
                .fragment
                .as_deref(),
            Some("new")
        );
        for base in ["", "#ignored", "data:text/plain,x", "javascript:void(0)"] {
            assert_eq!(
                target(base, "#p").unwrap().path.as_deref(),
                Some("OPS/Text/ch.xhtml")
            );
        }
        let doc = Document::parse(r#"<html xmlns="http://www.w3.org/1999/xhtml"><body><img src="cover.png"/></body><head><base href="../Images/"/><base href="wrong/"/></head></html>"#).unwrap();
        let context = BaseContext::from_document("OPS/Text/ch.xhtml", &doc).unwrap();
        let image = doc
            .descendants()
            .find(|n| n.has_tag_name((XHTML, "img")))
            .unwrap();
        assert_eq!(
            context
                .resolve(image, "cover.png", ReferenceKind::Html)
                .unwrap()
                .path
                .as_deref(),
            Some("OPS/Images/cover.png")
        );
    }

    #[test]
    fn svg_xml_base_inherits_without_changing_html_or_css_base() {
        let doc = Document::parse(r#"<html xmlns="http://www.w3.org/1999/xhtml" xml:base="../Images/"><head><base href="../"/></head><body><img src="x.png"/><svg xmlns="http://www.w3.org/2000/svg" xml:base="nested/"><g xml:base="../"><image href="x.png"/></g></svg></body></html>"#).unwrap();
        let context = BaseContext::from_document("OPS/Text/ch.xhtml", &doc).unwrap();
        let svg = doc
            .descendants()
            .find(|n| n.has_tag_name((SVG, "image")))
            .unwrap();
        let html = doc
            .descendants()
            .find(|n| n.has_tag_name((XHTML, "img")))
            .unwrap();
        assert_eq!(
            context
                .resolve(svg, "x.png", ReferenceKind::Svg)
                .unwrap()
                .path
                .as_deref(),
            Some("Images/x.png")
        );
        assert_eq!(
            context
                .resolve(html, "x.png", ReferenceKind::Html)
                .unwrap()
                .path
                .as_deref(),
            Some("OPS/x.png")
        );
        assert_eq!(
            context
                .resolve_document_reference("x.png")
                .unwrap()
                .path
                .as_deref(),
            Some("OPS/x.png")
        );
    }

    #[test]
    fn foreign_base_and_ncx_opf_declarations_do_not_change_document_base() {
        for xml in [
            r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><base xmlns="urn:foreign" href="wrong/"/></head></html>"#,
            r#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" xml:base="wrong/"><base xmlns="http://www.w3.org/1999/xhtml" href="wrong/"/></ncx>"#,
            r#"<package xmlns="http://www.idpf.org/2007/opf" xml:base="wrong/"/>"#,
        ] {
            let doc = Document::parse(xml).unwrap();
            let context = BaseContext::from_document("OPS/Text/ch.xhtml", &doc).unwrap();
            assert_eq!(
                context
                    .resolve_document_reference("x.png")
                    .unwrap()
                    .path
                    .as_deref(),
                Some("OPS/Text/x.png")
            );
        }
    }

    #[test]
    fn security_contract_and_external_no_fetch_are_preserved() {
        for base in [
            "../../..",
            "/Images/",
            "../Images%2f/",
            "../Images%5C/",
            "../%GG/",
            "../%FF/",
            "../Images/?x",
            "../%00/",
        ] {
            assert!(target(base, "cover.png").is_err(), "{base}");
        }
        for base in [
            "https://example.invalid/art/",
            "//example.invalid/art/",
            "file:///tmp/",
        ] {
            assert!(target(base, "cover.png").unwrap().external);
        }
        assert!(
            target("../%GG/", "https://example.invalid/art")
                .unwrap()
                .external
        );
        assert_eq!(
            target("../Images/", "a%252fb.png").unwrap().path.as_deref(),
            Some("OPS/Images/a%2fb.png")
        );
        assert!(target("../Images/", "").is_err());
    }

    #[test]
    fn byte_node_work_and_overflow_limits_are_checked() {
        let huge = "x".repeat(MAX_REFERENCE_BYTES + 1);
        assert_eq!(target(&huge, "x"), Err(BaseError::Limit(BaseLimit::Bytes)));
        let doc = Document::parse("<root><child/></root>").unwrap();
        assert!(matches!(
            BaseContext::from_document_with_limits("x", &doc, Limits { nodes: 1, ..LIMITS }),
            Err(BaseError::Limit(BaseLimit::Nodes))
        ));
        assert!(matches!(
            BaseContext::for_html_base_with_limits("x", None, Limits { bytes: 0, ..LIMITS }),
            Err(BaseError::Limit(BaseLimit::Bytes))
        ));
        let c = BaseContext::for_html_base_with_limits("x", None, Limits { work: 1, ..LIMITS })
            .unwrap();
        assert_eq!(
            c.resolve_document_reference("y"),
            Err(BaseError::Limit(BaseLimit::Work))
        );
        let counter = Cell::new(usize::MAX);
        assert_eq!(
            charge(&counter, 1, usize::MAX, BaseLimit::Work),
            Err(BaseError::Limit(BaseLimit::Work))
        );
    }
}

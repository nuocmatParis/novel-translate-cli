use roxmltree::{Document, Node, TextPos};

use crate::inventory::NavigationSource;
use crate::xml::{XmlFailure, XmlFailureKind, node_position, with_xml_document};

const EPUB_OPS_NAMESPACE: &str = "http://www.idpf.org/2007/ops";
const XHTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const NCX_NAMESPACE: &str = "http://www.daisy.org/z3986/2005/ncx/";
const MAX_NAVIGATION_DEPTH: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawNavigationTarget {
    pub href: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawNavigationItem {
    pub label: String,
    pub target: Option<RawNavigationTarget>,
    pub children: Vec<RawNavigationItem>,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParsedNavigation {
    pub source: NavigationSource,
    pub items: Vec<RawNavigationItem>,
    pub has_toc: bool,
}

pub(crate) fn parse_navigation(
    bytes: &[u8],
    source: NavigationSource,
) -> Result<ParsedNavigation, XmlFailure> {
    with_xml_document(
        bytes,
        super::xml::DEFAULT_XML_MAX_BYTES,
        super::xml::DEFAULT_XML_MAX_NODES,
        |document| match source {
            NavigationSource::Epub3Nav => parse_epub3(document),
            NavigationSource::Epub2Ncx => parse_ncx(document),
        },
    )?
}

fn parse_epub3(document: &Document<'_>) -> Result<ParsedNavigation, XmlFailure> {
    if !is_named_in_namespace(document.root_element(), XHTML_NAMESPACE, "html") {
        return Ok(empty_navigation(NavigationSource::Epub3Nav));
    }
    let nav = document
        .descendants()
        .filter(|node| is_named_in_namespace(*node, XHTML_NAMESPACE, "nav"))
        .find(|node| {
            attribute_in_namespace(*node, EPUB_OPS_NAMESPACE, "type")
                .is_some_and(|value| value.split_ascii_whitespace().any(|token| token == "toc"))
        });
    let Some(nav) = nav else {
        return Ok(ParsedNavigation {
            source: NavigationSource::Epub3Nav,
            items: Vec::new(),
            has_toc: false,
        });
    };
    let list = nav
        .children()
        .find(|node| is_named_in_namespace(*node, XHTML_NAMESPACE, "ol"));
    let items = if let Some(list) = list {
        validate_html_depth(list)?;
        parse_html_list(list)
    } else {
        Vec::new()
    };
    Ok(ParsedNavigation {
        source: NavigationSource::Epub3Nav,
        items,
        has_toc: true,
    })
}

fn validate_html_depth(list: Node<'_, '_>) -> Result<(), XmlFailure> {
    let mut stack = list
        .children()
        .filter(|node| is_named_in_namespace(*node, XHTML_NAMESPACE, "li"))
        .map(|node| (node, 1usize))
        .collect::<Vec<_>>();
    while let Some((item, depth)) = stack.pop() {
        if depth > MAX_NAVIGATION_DEPTH {
            return Err(navigation_depth_failure(item));
        }
        if let Some(nested) = item
            .children()
            .find(|node| is_named_in_namespace(*node, XHTML_NAMESPACE, "ol"))
        {
            stack.extend(
                nested
                    .children()
                    .filter(|node| is_named_in_namespace(*node, XHTML_NAMESPACE, "li"))
                    .map(|node| (node, depth + 1)),
            );
        }
    }
    Ok(())
}

fn parse_html_list(list: Node<'_, '_>) -> Vec<RawNavigationItem> {
    list.children()
        .filter(|node| is_named_in_namespace(*node, XHTML_NAMESPACE, "li"))
        .map(parse_html_item)
        .collect()
}

fn parse_html_item(item: Node<'_, '_>) -> RawNavigationItem {
    let link = item.children().find(|node| {
        node.is_element()
            && node.tag_name().namespace() == Some(XHTML_NAMESPACE)
            && matches!(node.tag_name().name(), "a" | "span")
    });
    let label = link.map(collect_text).unwrap_or_else(|| collect_text(item));
    let target = link
        .filter(|node| node.tag_name().name() == "a")
        .and_then(|node| node.attribute("href"))
        .map(|href| raw_target(item, href));
    let children = item
        .children()
        .find(|node| is_named_in_namespace(*node, XHTML_NAMESPACE, "ol"))
        .map(parse_html_list)
        .unwrap_or_default();
    let (line, column) = node_position(item);
    RawNavigationItem {
        label,
        target,
        children,
        line,
        column,
    }
}

fn parse_ncx(document: &Document<'_>) -> Result<ParsedNavigation, XmlFailure> {
    if !is_named_in_namespace(document.root_element(), NCX_NAMESPACE, "ncx") {
        return Ok(empty_navigation(NavigationSource::Epub2Ncx));
    }
    let nav_map = document
        .descendants()
        .find(|node| is_named_in_namespace(*node, NCX_NAMESPACE, "navMap"));
    let Some(nav_map) = nav_map else {
        return Ok(ParsedNavigation {
            source: NavigationSource::Epub2Ncx,
            items: Vec::new(),
            has_toc: false,
        });
    };
    validate_ncx_depth(nav_map)?;
    let items = nav_map
        .children()
        .filter(|node| is_named_in_namespace(*node, NCX_NAMESPACE, "navPoint"))
        .map(parse_ncx_item)
        .collect();
    Ok(ParsedNavigation {
        source: NavigationSource::Epub2Ncx,
        items,
        has_toc: true,
    })
}

fn validate_ncx_depth(nav_map: Node<'_, '_>) -> Result<(), XmlFailure> {
    let mut stack = nav_map
        .children()
        .filter(|node| is_named_in_namespace(*node, NCX_NAMESPACE, "navPoint"))
        .map(|node| (node, 1usize))
        .collect::<Vec<_>>();
    while let Some((item, depth)) = stack.pop() {
        if depth > MAX_NAVIGATION_DEPTH {
            return Err(navigation_depth_failure(item));
        }
        stack.extend(
            item.children()
                .filter(|node| is_named_in_namespace(*node, NCX_NAMESPACE, "navPoint"))
                .map(|node| (node, depth + 1)),
        );
    }
    Ok(())
}

fn navigation_depth_failure(node: Node<'_, '_>) -> XmlFailure {
    let (line, column) = node_position(node);
    XmlFailure::new(
        XmlFailureKind::NavigationDepthLimit,
        Some(TextPos::new(line, column)),
    )
}

fn empty_navigation(source: NavigationSource) -> ParsedNavigation {
    ParsedNavigation {
        source,
        items: Vec::new(),
        has_toc: false,
    }
}

fn parse_ncx_item(item: Node<'_, '_>) -> RawNavigationItem {
    let label = item
        .children()
        .find(|node| is_named_in_namespace(*node, NCX_NAMESPACE, "navLabel"))
        .and_then(|label| {
            label
                .descendants()
                .find(|node| is_named_in_namespace(*node, NCX_NAMESPACE, "text"))
        })
        .map(collect_text)
        .unwrap_or_default();
    let content = item
        .children()
        .find(|node| is_named_in_namespace(*node, NCX_NAMESPACE, "content"));
    let target = content.and_then(|node| node.attribute("src").map(|href| raw_target(node, href)));
    let children = item
        .children()
        .filter(|node| is_named_in_namespace(*node, NCX_NAMESPACE, "navPoint"))
        .map(parse_ncx_item)
        .collect();
    let (line, column) = node_position(item);
    RawNavigationItem {
        label,
        target,
        children,
        line,
        column,
    }
}

fn raw_target(node: Node<'_, '_>, href: &str) -> RawNavigationTarget {
    let (line, column) = node_position(node);
    RawNavigationTarget {
        href: href.to_owned(),
        line,
        column,
    }
}

fn collect_text(node: Node<'_, '_>) -> String {
    node.descendants()
        .filter(Node::is_text)
        .filter_map(|descendant| descendant.text())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn attribute_in_namespace<'a>(node: Node<'a, 'a>, namespace: &str, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attribute| attribute.name() == name && attribute.namespace() == Some(namespace))
        .map(|attribute| attribute.value())
}

fn is_named_in_namespace(node: Node<'_, '_>, namespace: &str, name: &str) -> bool {
    node.is_element()
        && node.tag_name().name() == name
        && node.tag_name().namespace() == Some(namespace)
}

#[cfg(test)]
mod tests {
    use crate::inventory::NavigationSource;
    use crate::xml::XmlFailureKind;

    use super::parse_navigation;

    fn nested_html(depth: usize) -> String {
        let mut html = String::from(
            "<html xmlns='http://www.w3.org/1999/xhtml' xmlns:epub='http://www.idpf.org/2007/ops'><body><nav epub:type='toc'><ol>",
        );
        for level in 0..depth {
            html.push_str("<li><a>item</a>");
            if level + 1 < depth {
                html.push_str("<ol>");
            }
        }
        for level in (0..depth).rev() {
            html.push_str("</li>");
            if level > 0 {
                html.push_str("</ol>");
            }
        }
        html.push_str("</ol></nav></body></html>");
        html
    }

    fn nested_ncx(depth: usize) -> String {
        let mut xml = String::from("<ncx xmlns='http://www.daisy.org/z3986/2005/ncx/'><navMap>");
        for level in 0..depth {
            xml.push_str("<navPoint><navLabel><text>item</text></navLabel><content src='x'/>");
            if level + 1 < depth {
                continue;
            }
        }
        for _ in 0..depth {
            xml.push_str("</navPoint>");
        }
        xml.push_str("</navMap></ncx>");
        xml
    }

    #[test]
    fn rejects_overdeep_epub3_navigation_and_accepts_the_limit() {
        let at_limit = parse_navigation(&nested_html(256).into_bytes(), NavigationSource::Epub3Nav);
        assert!(at_limit.is_ok(), "{at_limit:?}");
        assert_eq!(
            parse_navigation(&nested_html(257).into_bytes(), NavigationSource::Epub3Nav)
                .unwrap_err()
                .kind,
            XmlFailureKind::NavigationDepthLimit
        );
    }

    #[test]
    fn rejects_overdeep_epub2_navigation_before_recursive_projection() {
        assert!(
            parse_navigation(&nested_ncx(256).into_bytes(), NavigationSource::Epub2Ncx).is_ok()
        );
        assert_eq!(
            parse_navigation(&nested_ncx(257).into_bytes(), NavigationSource::Epub2Ncx)
                .unwrap_err()
                .kind,
            XmlFailureKind::NavigationDepthLimit
        );
    }
}

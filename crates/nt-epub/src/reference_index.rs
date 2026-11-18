//! Bounded, offline resource-reference indexing for a captured EPUB.
//!
//! The caller must supply the inventory and extraction produced from this same
//! captured archive session and must independently verify the capture hash.
//! This API checks the relationships it consumes; it cannot prove arbitrary
//! supplied structures were produced from identical original ZIP bytes.

mod css;
mod grapheme_validation;
mod image_info;
mod markup;
pub(crate) mod native_space;
mod package_refs;
mod srcset;
mod working;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::io::{Read, Seek};

use nt_domain::source_range::BlockIdentityRecord;
use nt_domain::volume_import::CapturedImagePlacement;
use roxmltree::Node;
use sha2::{Digest, Sha256};

use crate::archive::WorkingEntry;
use crate::inventory::{EpubInventory, InventoryCoverage, Resource};
use crate::package::resolve_href;
use crate::source_extract::{SourceDiagnostic, SourceDocument, SourceExtraction};
use crate::xml::{XmlFailure, XmlFailureKind, node_position, with_xml_document};
use crate::{ArchiveError, EpubZip, canonical_archive_path};

const XHTML: &str = "http://www.w3.org/1999/xhtml";
const SVG: &str = "http://www.w3.org/2000/svg";
const XLINK: &str = "http://www.w3.org/1999/xlink";
const OPF: &str = "http://www.idpf.org/2007/opf";

/// Hard maxima apply even if a caller supplies larger values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceIndexLimits {
    /// Total space available to the index while its caller retains other data.
    pub max_working_bytes: usize,
    pub max_files: usize,
    pub max_references: usize,
    pub max_raw_reference_bytes: usize,
    pub max_reference_bytes: usize,
    pub max_css_bytes: usize,
    pub max_css_tokens: usize,
    pub max_css_function_depth: usize,
    pub max_css_import_depth: usize,
    pub max_xml_bytes: usize,
    pub max_xml_nodes: u32,
    pub max_xml_depth: usize,
}

impl Default for ReferenceIndexLimits {
    fn default() -> Self {
        Self {
            max_working_bytes: nt_domain::volume_import::MAX_IMPORT_WORKING_BYTES,
            max_files: 10_000,
            max_references: 1_000_000,
            max_raw_reference_bytes: 64 * 1024 * 1024,
            max_reference_bytes: 16 * 1024,
            max_css_bytes: 16 * 1024 * 1024,
            max_css_tokens: 2_000_000,
            max_css_function_depth: 256,
            max_css_import_depth: 64,
            max_xml_bytes: 16 * 1024 * 1024,
            max_xml_nodes: 1_000_000,
            max_xml_depth: 256,
        }
    }
}

impl ReferenceIndexLimits {
    /// Size the graph and its XML or CSS parser against the space left by the
    /// caller's live source, index and history.
    pub fn for_working_bytes(bytes: usize) -> Self {
        let hard = Self::default();
        let bytes = bytes.min(hard.max_working_bytes);
        let scanner = bytes - bytes / 2;
        Self {
            max_working_bytes: bytes,
            max_references: hard
                .max_references
                .min(scanner / (6 * size_of::<ResourceReference>())),
            max_raw_reference_bytes: hard.max_raw_reference_bytes.min(scanner / 12),
            max_xml_nodes: hard
                .max_xml_nodes
                .min(u32::try_from(scanner / 1536).unwrap_or(u32::MAX)),
            max_xml_bytes: hard.max_xml_bytes.min(scanner / 12),
            max_css_bytes: hard.max_css_bytes.min(scanner / 12),
            ..hard
        }
    }

    fn bounded(mut self) -> Self {
        let hard = Self::default();
        self.max_working_bytes = self.max_working_bytes.min(hard.max_working_bytes);
        self.max_files = self.max_files.min(hard.max_files);
        self.max_references = self.max_references.min(hard.max_references);
        self.max_raw_reference_bytes = self
            .max_raw_reference_bytes
            .min(hard.max_raw_reference_bytes);
        self.max_reference_bytes = self.max_reference_bytes.min(hard.max_reference_bytes);
        self.max_css_bytes = self.max_css_bytes.min(hard.max_css_bytes);
        self.max_css_tokens = self.max_css_tokens.min(hard.max_css_tokens);
        self.max_css_function_depth = self.max_css_function_depth.min(hard.max_css_function_depth);
        self.max_css_import_depth = self.max_css_import_depth.min(hard.max_css_import_depth);
        self.max_xml_bytes = self.max_xml_bytes.min(hard.max_xml_bytes);
        self.max_xml_nodes = self.max_xml_nodes.min(hard.max_xml_nodes);
        self.max_xml_depth = self.max_xml_depth.min(hard.max_xml_depth);
        self
    }
}

/// Stable error categories. They never include source text or external URLs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceIndexError {
    InvalidInput,
    Archive,
    ArchiveEntryLimit,
    ArchiveCumulativeLimit,
    XmlMalformed,
    XmlByteLimit,
    XmlNodeLimit,
    XmlDepthLimit,
    ExternalEntity,
    UnknownEntity,
    EntityExpansionLimit,
    UnsupportedEncoding,
    FileCountLimit,
    ReferenceCountLimit,
    ReferenceByteLimit,
    RawEvidenceLimit,
    WorkingMemoryLimit,
    CssByteLimit,
    CssTokenLimit,
    CssFunctionDepthLimit,
    CssImportDepthLimit,
    InvalidCss,
    InvalidSrcset,
    UnsafeUri,
    SourceMismatch,
    Missing,
    Ambiguous,
    External,
    ImportCycle,
    OptionalNavigationSkipped,
    BaseByteLimit,
    BaseNodeLimit,
    BaseWorkLimit,
    At {
        source_path: String,
        line: Option<u32>,
        column: Option<u32>,
        reason: Box<ReferenceIndexError>,
    },
}

impl fmt::Display for ReferenceIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for ReferenceIndexError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceCoverage {
    Complete,
    NeedsReview,
    Incomplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceKind {
    XhtmlImageSrc,
    SvgImageHref,
    SrcsetCandidate,
    StylesheetLink,
    CssImport,
    CssUrl,
    Cover,
    XhtmlLink,
    PackageLink,
    Manifest,
    Fallback,
    MediaOverlay,
    Spine,
}

/// Paths use element-child ordinals from the document element, starting at zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceLocation {
    pub element_ordinals: Vec<u32>,
    pub attribute_namespace: Option<String>,
    pub attribute_local_name: Option<String>,
    pub candidate_ordinal: u32,
    pub line: u32,
    pub column: u32,
    pub decoded_byte_range: Option<(u32, u32)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceTarget {
    Internal {
        path: String,
        fragment: Option<String>,
    },
    Missing {
        path: String,
        fragment: Option<String>,
    },
    Ambiguous,
    External {
        scheme: String,
    },
    Invalid {
        reason: ReferenceIndexError,
    },
}

#[derive(Clone, Eq, PartialEq)]
pub struct ResourceReference {
    pub source_path: String,
    pub location: ReferenceLocation,
    pub kind: ReferenceKind,
    pub target: ReferenceTarget,
    /// Private source evidence. Do not include this field in user diagnostics or logs.
    pub raw_reference: String,
    pub spine_positions: Vec<usize>,
    pub inventory_occurrence: Option<usize>,
    /// Matched source block ID and unit offset, when CP04 emitted the image.
    pub source_occurrences: Vec<([u8; 32], u32)>,
}

impl fmt::Debug for ResourceReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResourceReference")
            .field("source_path", &self.source_path)
            .field("location", &self.location)
            .field("kind", &self.kind)
            .field("target", &self.target)
            .field("raw_reference", &"[private]")
            .field("spine_positions", &self.spine_positions)
            .field("inventory_occurrence", &self.inventory_occurrence)
            .field("source_occurrences", &self.source_occurrences)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceDiagnostic {
    pub source_path: String,
    pub location: Option<ReferenceLocation>,
    pub code: ReferenceIndexError,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceIndex {
    pub references: Vec<ResourceReference>,
    pub diagnostics: Vec<ReferenceDiagnostic>,
    pub coverage: ReferenceCoverage,
}

struct Budget {
    limits: ReferenceIndexLimits,
    graph: working::GraphSpace,
    files: usize,
    references: usize,
    raw: usize,
    tokens: usize,
}
impl Budget {
    fn reference(&mut self, bytes: usize) -> Result<(), ReferenceIndexError> {
        if bytes > self.limits.max_reference_bytes {
            return Err(ReferenceIndexError::ReferenceByteLimit);
        }
        self.references = self
            .references
            .checked_add(1)
            .ok_or(ReferenceIndexError::ReferenceCountLimit)?;
        if self.references > self.limits.max_references {
            return Err(ReferenceIndexError::ReferenceCountLimit);
        }
        self.raw = self
            .raw
            .checked_add(bytes)
            .ok_or(ReferenceIndexError::RawEvidenceLimit)?;
        if self.raw > self.limits.max_raw_reference_bytes {
            return Err(ReferenceIndexError::RawEvidenceLimit);
        }
        Ok(())
    }
    fn token(&mut self) -> Result<(), ReferenceIndexError> {
        self.tokens = self
            .tokens
            .checked_add(1)
            .ok_or(ReferenceIndexError::CssTokenLimit)?;
        if self.tokens > self.limits.max_css_tokens {
            return Err(ReferenceIndexError::CssTokenLimit);
        }
        Ok(())
    }
    fn file(&mut self) -> Result<(), ReferenceIndexError> {
        self.files = self
            .files
            .checked_add(1)
            .ok_or(ReferenceIndexError::FileCountLimit)?;
        if self.files > self.limits.max_files {
            return Err(ReferenceIndexError::FileCountLimit);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Resolution {
    Internal(String, Option<String>),
    External(String),
    Invalid,
    MissingIdentifier,
    Ambiguous,
}

struct RawReference {
    location: ReferenceLocation,
    kind: ReferenceKind,
    value: String,
    raw: String,
    resolved: Option<Resolution>,
    spine_override: Option<Vec<usize>>,
    inventory_occurrence: Option<usize>,
    source_occurrences: Vec<([u8; 32], u32)>,
}

struct Indexer<'a, R> {
    archive: &'a mut EpubZip<R>,
    inventory: &'a EpubInventory,
    source: IndexSource<'a>,
    budget: Budget,
    result: ReferenceIndex,
    manifest_paths: HashMap<String, usize>,
    seen_files: HashSet<String>,
}

enum IndexSource<'a> {
    Extracted(&'a SourceExtraction),
    Replayed {
        images: &'a BTreeMap<String, Vec<CapturedImagePlacement>>,
        diagnostics: &'a [SourceDiagnostic],
    },
}

enum ImageEvidence<'a> {
    Extracted(&'a SourceDocument),
    Replayed(&'a [CapturedImagePlacement]),
}

impl IndexSource<'_> {
    fn images(&self, path: &str) -> Option<ImageEvidence<'_>> {
        match self {
            Self::Extracted(source) => source
                .documents
                .iter()
                .find(|doc| doc.canonical_path == path)
                .map(ImageEvidence::Extracted),
            Self::Replayed { images, .. } => images
                .get(path)
                .map(|placements| ImageEvidence::Replayed(placements)),
        }
    }

    fn diagnostics(&self) -> &[SourceDiagnostic] {
        match self {
            Self::Extracted(source) => &source.diagnostics,
            Self::Replayed { diagnostics, .. } => diagnostics,
        }
    }

    fn complete(&self) -> bool {
        match self {
            Self::Extracted(source) => source.complete,
            Self::Replayed { diagnostics, .. } => {
                diagnostics.iter().all(|diagnostic| match diagnostic {
                    SourceDiagnostic::DuplicateXhtmlId { .. } => true,
                    SourceDiagnostic::Inventory(d) => {
                        d.severity != crate::DiagnosticSeverity::Incomplete
                    }
                    _ => false,
                })
            }
        }
    }
}

/// Index the already captured EPUB without network access.
///
/// The same archive session, inventory, extraction and caller-verified capture
/// hash must be used. Source IDs, spine positions and image evidence are
/// reconciled, but these checks are not a substitute for the capture hash.
pub fn index_references<R: Read + Seek>(
    archive: &mut EpubZip<R>,
    inventory: &EpubInventory,
    extracted: &SourceExtraction,
    limits: ReferenceIndexLimits,
) -> Result<ReferenceIndex, ReferenceIndexError> {
    if extracted.parser_version.trim().is_empty() {
        return Err(ReferenceIndexError::InvalidInput);
    }
    index(
        archive,
        inventory,
        IndexSource::Extracted(extracted),
        limits,
    )
}

impl ReferenceIndex {
    /// Rebuild the same index from original markup and compact image placements.
    /// Placements and diagnostics must come from replay of this captured EPUB.
    pub fn from_replayed(
        archive: &mut EpubZip<impl Read + Seek>,
        inventory: &EpubInventory,
        images: &BTreeMap<String, Vec<CapturedImagePlacement>>,
        diagnostics: &[SourceDiagnostic],
        limits: ReferenceIndexLimits,
    ) -> Result<Self, ReferenceIndexError> {
        index(
            archive,
            inventory,
            IndexSource::Replayed {
                images,
                diagnostics,
            },
            limits,
        )
    }
}

fn index<R: Read + Seek>(
    archive: &mut EpubZip<R>,
    inventory: &EpubInventory,
    source: IndexSource<'_>,
    limits: ReferenceIndexLimits,
) -> Result<ReferenceIndex, ReferenceIndexError> {
    let limits = limits.bounded();
    if inventory.resources.len() > 1_000_000 {
        return Err(ReferenceIndexError::InvalidInput);
    }
    let graph_bytes = limits.max_working_bytes / 2;
    let scanner_bytes = limits.max_working_bytes - graph_bytes;
    let package = inventory.package_path.as_deref();
    native_space::scoped(scanner_bytes, || {
        let mut indexer = Indexer {
            archive,
            inventory,
            source,
            budget: Budget {
                graph: working::GraphSpace::new(graph_bytes),
                limits,
                files: 0,
                references: 0,
                raw: 0,
                tokens: 0,
            },
            result: ReferenceIndex {
                references: Vec::new(),
                diagnostics: Vec::new(),
                coverage: ReferenceCoverage::Complete,
            },
            manifest_paths: HashMap::new(),
            seen_files: HashSet::new(),
        };
        indexer.validate_inputs()?;
        if inventory.coverage != InventoryCoverage::Complete || !indexer.source.complete() {
            indexer.result.coverage = ReferenceCoverage::Incomplete;
        }
        indexer.run()?;
        Ok(indexer.result)
    })
    .map_err(|_| {
        let reason = ReferenceIndexError::WorkingMemoryLimit;
        package.map_or(reason.clone(), |path| at(path, None, None, reason))
    })?
}

impl<R: Read + Seek> Indexer<'_, R> {
    fn run(&mut self) -> Result<(), ReferenceIndexError> {
        let package = self
            .inventory
            .package_path
            .clone()
            .ok_or(ReferenceIndexError::InvalidInput)?;
        let bytes = self.read(&package, self.budget.limits.max_xml_bytes, false)?;
        let raw = with_xml_document(
            &bytes,
            self.budget.limits.max_xml_bytes,
            self.budget.limits.max_xml_nodes,
            |document| package_refs::scan(document, &package, self.inventory, &mut self.budget),
        )
        .map_err(|failure| xml_error(&package, failure))?
        .map_err(|reason| at(&package, None, None, reason))?;
        for item in raw {
            self.add(&package, &[], item)?;
        }

        let mut markup_files = Vec::new();
        let mut stylesheet_files = Vec::new();
        let mut selected = HashSet::new();
        for (index, resource) in self.inventory.resources.iter().enumerate() {
            let Some(path) = resource.path.as_ref().filter(|_| resource.available) else {
                continue;
            };
            match resource.media_type.as_str() {
                "application/xhtml+xml" | "image/svg+xml" if selected.insert(path.clone()) => {
                    markup_files.push((path.clone(), index))
                }
                "text/css" if selected.insert(path.clone()) => stylesheet_files.push(path.clone()),
                _ => {}
            }
        }
        let mut seeds = Vec::new();
        for (path, resource_index) in markup_files {
            let optional_navigation_failure = self.inventory.resources[resource_index]
                .properties.iter().any(|p|p=="nav")
                && !self.inventory.spine.iter().any(|item|item.resource_index==Some(resource_index))
                && self.source.diagnostics().iter().any(|d|matches!(d,SourceDiagnostic::InvalidOptionalNavigation{canonical_path} if canonical_path==&path));
            if optional_navigation_failure {
                // The producer already diagnosed this optional Nav and kept a
                // usable chapter fallback. Do not turn it into mandatory XML.
                // A referenced fragment is still checked below and may fail.
                let diagnostic = self.budget.graph.diagnostic(
                    &path,
                    None,
                    ReferenceIndexError::OptionalNavigationSkipped,
                )?;
                self.record_diagnostic(diagnostic, false)?;
                continue;
            }
            let bytes = self.read(&path, self.budget.limits.max_xml_bytes, false)?;
            let resource = &self.inventory.resources[resource_index];
            let images = self.source.images(&path);
            let raw = with_xml_document(
                &bytes,
                self.budget.limits.max_xml_bytes,
                self.budget.limits.max_xml_nodes,
                |document| markup::scan(document, &path, Some(resource), images, &mut self.budget),
            )
            .map_err(|failure| xml_error(&path, failure))?
            .map_err(|reason| at(&path, None, None, reason))?;
            let spine = self
                .inventory
                .spine
                .iter()
                .filter(|item| item.resource_index == Some(resource_index))
                .map(|item| item.reading_order)
                .collect::<Vec<_>>();
            for item in raw {
                if let Some((target, location)) = self.add(&path, &spine, item)? {
                    self.budget.graph.take(path.len())?;
                    seeds.push((target, path.clone(), Some(location)));
                }
            }
        }
        for path in stylesheet_files {
            seeds.push((path.clone(), path, None));
        }
        self.stylesheets(seeds)?;
        self.reconcile_fragments()?;
        Ok(())
    }

    fn stylesheets(
        &mut self,
        seeds: Vec<(String, String, Option<ReferenceLocation>)>,
    ) -> Result<(), ReferenceIndexError> {
        enum Job {
            Enter {
                path: String,
                origin: String,
                location: Option<ReferenceLocation>,
                depth: usize,
            },
            Exit(String),
        }
        let mut jobs = seeds
            .into_iter()
            .rev()
            .map(|(path, origin, location)| Job::Enter {
                path,
                origin,
                location,
                depth: 0,
            })
            .collect::<Vec<_>>();
        let mut active = HashSet::new();
        let mut done = HashSet::new();
        let mut cycles = HashSet::new();
        while let Some(job) = jobs.pop() {
            let (path, origin, location, depth) = match job {
                Job::Exit(path) => {
                    active.remove(&path);
                    done.insert(path);
                    continue;
                }
                Job::Enter {
                    path,
                    origin,
                    location,
                    depth,
                } => (path, origin, location, depth),
            };
            if active.contains(&path) {
                if cycles.insert(path) {
                    let diagnostic = self.budget.graph.diagnostic(
                        &origin,
                        location.as_ref(),
                        ReferenceIndexError::ImportCycle,
                    )?;
                    self.record_diagnostic(diagnostic, false)?;
                }
                continue;
            }
            if done.contains(&path) {
                continue;
            }
            if depth > self.budget.limits.max_css_import_depth {
                return Err(at(
                    &origin,
                    location.as_ref().map(|l| l.line),
                    location.as_ref().map(|l| l.column),
                    ReferenceIndexError::CssImportDepthLimit,
                ));
            }
            active.insert(path.clone());
            let bytes = self.read(&path, self.budget.limits.max_css_bytes, true)?;
            let _decoder = native_space::xml_decode(&bytes)
                .map_err(|_| ReferenceIndexError::WorkingMemoryLimit)?;
            let text = css::decode(&bytes, self.budget.limits.max_css_bytes)
                .map_err(|reason| at(&path, None, None, reason))?;
            let refs = css::scan(&text, &mut self.budget)
                .map_err(|reason| at(&path, None, None, reason))?;
            let mut imports = Vec::new();
            for (ordinal, reference) in refs.into_iter().enumerate() {
                let (line, column) = markup::css_position(&text, reference.position, 1, 1);
                let location = ReferenceLocation {
                    element_ordinals: Vec::new(),
                    attribute_namespace: None,
                    attribute_local_name: None,
                    candidate_ordinal: u32::try_from(ordinal)
                        .map_err(|_| ReferenceIndexError::ReferenceCountLimit)?,
                    line,
                    column,
                    decoded_byte_range: Some((
                        u32::try_from(reference.range.0)
                            .map_err(|_| ReferenceIndexError::ReferenceByteLimit)?,
                        u32::try_from(reference.range.1)
                            .map_err(|_| ReferenceIndexError::ReferenceByteLimit)?,
                    )),
                };
                let kind = reference.kind;
                let raw = RawReference {
                    location,
                    kind,
                    value: reference.value,
                    raw: reference.raw,
                    resolved: None,
                    spine_override: None,
                    inventory_occurrence: None,
                    source_occurrences: Vec::new(),
                };
                if let Some(import) = self.add(&path, &[], raw)? {
                    imports.push(import);
                }
            }
            jobs.push(Job::Exit(path.clone()));
            for (target, location) in imports.into_iter().rev() {
                let next_depth = depth
                    .checked_add(1)
                    .ok_or(ReferenceIndexError::CssImportDepthLimit)?;
                jobs.push(Job::Enter {
                    path: target,
                    origin: path.clone(),
                    location: Some(location),
                    depth: next_depth,
                });
            }
        }
        Ok(())
    }

    fn reconcile_fragments(&mut self) -> Result<(), ReferenceIndexError> {
        let mut requested: BTreeMap<String, BTreeMap<String, Vec<usize>>> = BTreeMap::new();
        for (index, item) in self.result.references.iter().enumerate() {
            if let ReferenceTarget::Internal {
                path,
                fragment: Some(fragment),
            } = &item.target
                && !fragment.is_empty()
                && self.inventory.resources.iter().any(|resource| {
                    resource.path.as_deref() == Some(path)
                        && matches!(
                            resource.media_type.as_str(),
                            "application/xhtml+xml" | "image/svg+xml"
                        )
                })
            {
                requested
                    .entry(path.clone())
                    .or_default()
                    .entry(fragment.clone())
                    .or_default()
                    .push(index);
            }
        }
        for (path, names) in requested {
            let bytes = self.read(&path, self.budget.limits.max_xml_bytes, false)?;
            let counts = with_xml_document(
                &bytes,
                self.budget.limits.max_xml_bytes,
                self.budget.limits.max_xml_nodes,
                |document| {
                    let mut counts: HashMap<&str, usize> = HashMap::new();
                    for node in document.descendants().filter(Node::is_element) {
                        let id = node
                            .attribute("id")
                            .filter(|value| names.contains_key(*value));
                        let name = (node.tag_name().namespace() == Some(XHTML)
                            && node.tag_name().name() == "a")
                            .then(|| node.attribute("name"))
                            .flatten()
                            .filter(|value| Some(*value) != id && names.contains_key(*value));
                        for value in id.into_iter().chain(name) {
                            *counts.entry(value).or_default() += 1;
                        }
                    }
                    names
                        .keys()
                        .map(|name| {
                            (
                                name.clone(),
                                counts.get(name.as_str()).copied().unwrap_or(0),
                            )
                        })
                        .collect::<HashMap<_, _>>()
                },
            )
            .map_err(|failure| xml_error(&path, failure))?;
            for (fragment, indices) in names {
                let count = counts.get(&fragment).copied().unwrap_or(0);
                if count == 1 {
                    continue;
                }
                for index in indices {
                    let item = &mut self.result.references[index];
                    item.target = if count == 0 {
                        ReferenceTarget::Missing {
                            path: path.clone(),
                            fragment: Some(fragment.clone()),
                        }
                    } else {
                        ReferenceTarget::Ambiguous
                    };
                    let code = if count == 0 {
                        ReferenceIndexError::Missing
                    } else {
                        ReferenceIndexError::Ambiguous
                    };
                    let diagnostic = self.budget.graph.diagnostic(
                        &item.source_path,
                        Some(&item.location),
                        code,
                    )?;
                    self.record_diagnostic(diagnostic, true)?;
                }
            }
        }
        Ok(())
    }

    fn validate_inputs(&mut self) -> Result<(), ReferenceIndexError> {
        let Some(package_path) = self.inventory.package_path.as_deref() else {
            return Err(ReferenceIndexError::InvalidInput);
        };
        validate_canonical(package_path)?;
        let mut index_bytes = self
            .inventory
            .spine
            .len()
            .saturating_mul(4 * size_of::<usize>());
        for resource in &self.inventory.resources {
            if let Some(path) = &resource.path {
                index_bytes = index_bytes.saturating_add(1024 + 8 * path.len());
            }
        }
        native_space::retain(index_bytes).map_err(|_| ReferenceIndexError::WorkingMemoryLimit)?;
        for resource in &self.inventory.resources {
            if let Some(path) = &resource.path {
                validate_canonical(path)?;
                *self.manifest_paths.entry(path.clone()).or_default() += 1;
            }
        }
        for item in &self.inventory.spine {
            if let Some(index) = item.resource_index
                && (index >= self.inventory.resources.len()
                    || self.inventory.resources[index].id != item.idref)
            {
                return Err(ReferenceIndexError::SourceMismatch);
            }
        }
        let IndexSource::Extracted(extracted) = self.source else {
            return Ok(());
        };
        let mut documents = HashSet::new();
        for document in &extracted.documents {
            validate_canonical(&document.canonical_path)?;
            if !documents.insert(&document.canonical_path) {
                return Err(ReferenceIndexError::SourceMismatch);
            }
            let indices = self
                .inventory
                .resources
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    (item.path.as_deref() == Some(document.canonical_path.as_str())).then_some(i)
                })
                .collect::<Vec<_>>();
            if indices.is_empty() {
                return Err(ReferenceIndexError::SourceMismatch);
            }
            let positions = self
                .inventory
                .spine
                .iter()
                .filter(|item| item.resource_index.is_some_and(|i| indices.contains(&i)))
                .map(|item| item.reading_order)
                .collect::<Vec<_>>();
            let linear = self
                .inventory
                .spine
                .iter()
                .filter(|item| {
                    item.linear && item.resource_index.is_some_and(|i| indices.contains(&i))
                })
                .map(|item| item.reading_order)
                .collect::<Vec<_>>();
            if positions != document.spine_positions || linear != document.linear_spine_positions {
                return Err(ReferenceIndexError::SourceMismatch);
            }
            for block in &document.blocks {
                if block.canonical_path != document.canonical_path
                    || block.parser_version != extracted.parser_version
                {
                    return Err(ReferenceIndexError::SourceMismatch);
                }
                let record = BlockIdentityRecord {
                    epub_sha256: extracted.epub_sha256,
                    parser_version: extracted.parser_version.clone(),
                    canonical_path: document.canonical_path.clone(),
                    locator: block.locator.clone(),
                };
                let bytes = record
                    .canonical_bytes()
                    .map_err(|_| ReferenceIndexError::SourceMismatch)?;
                let digest: [u8; 32] = Sha256::digest(bytes).into();
                if block.id.as_bytes() != &digest {
                    return Err(ReferenceIndexError::SourceMismatch);
                }
            }
        }
        Ok(())
    }

    fn read(
        &mut self,
        path: &str,
        maximum: usize,
        css_file: bool,
    ) -> Result<WorkingEntry, ReferenceIndexError> {
        validate_canonical(path)?;
        if !self.seen_files.contains(path) {
            self.budget
                .file()
                .map_err(|reason| at(path, None, None, reason))?;
            self.seen_files.insert(path.to_owned());
        }
        let Some(entry) = self
            .archive
            .entry_by_path(path)
            .map_err(|_| at(path, None, None, ReferenceIndexError::Archive))?
        else {
            return Err(at(path, None, None, ReferenceIndexError::Missing));
        };
        if entry.is_directory() {
            return Err(at(path, None, None, ReferenceIndexError::Missing));
        }
        if entry.declared_size() > maximum as u64 {
            return Err(at(
                path,
                None,
                None,
                if css_file {
                    ReferenceIndexError::CssByteLimit
                } else {
                    ReferenceIndexError::XmlByteLimit
                },
            ));
        }
        let id = entry.id();
        self.archive.read_working_entry(id).map_err(|error| {
            let reason = match error {
                ArchiveError::TotalDeclaredTooLarge { .. }
                | ArchiveError::TotalLimitExceeded { .. } => {
                    ReferenceIndexError::ArchiveCumulativeLimit
                }
                ArchiveError::EntryDeclaredTooLarge { .. }
                | ArchiveError::EntryLimitExceeded { .. } => ReferenceIndexError::ArchiveEntryLimit,
                _ => ReferenceIndexError::Archive,
            };
            at(path, None, None, reason)
        })
    }

    fn add(
        &mut self,
        source: &str,
        spine: &[usize],
        raw: RawReference,
    ) -> Result<Option<(String, ReferenceLocation)>, ReferenceIndexError> {
        let (line, column) = (raw.location.line, raw.location.column);
        self.push_reference(source, spine, raw)
            .map_err(|reason| at(source, Some(line), Some(column), reason))
    }

    fn push_reference(
        &mut self,
        source: &str,
        spine: &[usize],
        mut raw: RawReference,
    ) -> Result<Option<(String, ReferenceLocation)>, ReferenceIndexError> {
        let published = raw
            .resolved
            .take()
            .unwrap_or_else(|| resolve(source, &raw.value));
        let target = self.classify(published)?;
        let space = &mut self.budget.graph;
        space.take(source.len())?;
        space.location(&raw.location)?;
        space.target(&target)?;
        space.take(raw.raw.capacity())?;
        space.array::<([u8; 32], u32)>(raw.source_occurrences.capacity())?;
        let follow = match (&target, raw.kind) {
            (
                ReferenceTarget::Internal { path, .. },
                ReferenceKind::StylesheetLink | ReferenceKind::CssImport,
            ) => {
                space.take(path.len())?;
                space.location(&raw.location)?;
                Some((path.clone(), raw.location.clone()))
            }
            _ => None,
        };
        let code = match &target {
            ReferenceTarget::Missing { .. } => Some((ReferenceIndexError::Missing, true)),
            ReferenceTarget::Ambiguous => Some((ReferenceIndexError::Ambiguous, true)),
            ReferenceTarget::External { .. } => Some((ReferenceIndexError::External, false)),
            ReferenceTarget::Invalid { reason } => Some((reason.clone(), true)),
            ReferenceTarget::Internal { .. } => None,
        };
        if let Some((code, incomplete)) = code {
            let diagnostic = self
                .budget
                .graph
                .diagnostic(source, Some(&raw.location), code)?;
            self.record_diagnostic(diagnostic, incomplete)?;
        }
        self.budget.graph.slot(&mut self.result.references)?;
        let spine_positions = self.budget.graph.positions(raw.spine_override, spine)?;
        self.result.references.push(ResourceReference {
            source_path: source.to_owned(),
            location: raw.location,
            kind: raw.kind,
            target,
            raw_reference: raw.raw,
            spine_positions,
            inventory_occurrence: raw.inventory_occurrence,
            source_occurrences: raw.source_occurrences,
        });
        Ok(follow)
    }

    fn classify(&self, resolution: Resolution) -> Result<ReferenceTarget, ReferenceIndexError> {
        Ok(match resolution {
            Resolution::Invalid => ReferenceTarget::Invalid {
                reason: ReferenceIndexError::UnsafeUri,
            },
            Resolution::MissingIdentifier => ReferenceTarget::Invalid {
                reason: ReferenceIndexError::Missing,
            },
            Resolution::Ambiguous => ReferenceTarget::Ambiguous,
            Resolution::External(scheme) => ReferenceTarget::External { scheme },
            Resolution::Internal(path, fragment) => {
                if self.manifest_paths.get(&path).copied().unwrap_or(0) > 1 {
                    ReferenceTarget::Ambiguous
                } else if self
                    .archive
                    .entry_by_path(&path)
                    .map_err(|_| ReferenceIndexError::Archive)?
                    .is_some_and(|entry| !entry.is_directory())
                {
                    ReferenceTarget::Internal { path, fragment }
                } else {
                    ReferenceTarget::Missing { path, fragment }
                }
            }
        })
    }

    fn record_diagnostic(
        &mut self,
        diagnostic: ReferenceDiagnostic,
        incomplete: bool,
    ) -> Result<(), ReferenceIndexError> {
        self.budget.graph.slot(&mut self.result.diagnostics)?;
        if incomplete {
            self.result.coverage = ReferenceCoverage::Incomplete;
        } else if self.result.coverage == ReferenceCoverage::Complete {
            self.result.coverage = ReferenceCoverage::NeedsReview;
        }
        self.result.diagnostics.push(diagnostic);
        Ok(())
    }
}

fn validate_canonical(path: &str) -> Result<(), ReferenceIndexError> {
    if canonical_archive_path(path).ok().as_deref() != Some(path) {
        return Err(ReferenceIndexError::InvalidInput);
    }
    Ok(())
}
fn at(
    source_path: &str,
    line: Option<u32>,
    column: Option<u32>,
    reason: ReferenceIndexError,
) -> ReferenceIndexError {
    match reason {
        ReferenceIndexError::At {
            source_path: inner,
            line: inner_line,
            column: inner_column,
            reason,
        } => ReferenceIndexError::At {
            source_path: if inner.is_empty() {
                source_path.to_owned()
            } else {
                inner
            },
            line: line.or(inner_line),
            column: column.or(inner_column),
            reason,
        },
        reason => ReferenceIndexError::At {
            source_path: source_path.to_owned(),
            line,
            column,
            reason: Box::new(reason),
        },
    }
}
fn xml_error(path: &str, failure: XmlFailure) -> ReferenceIndexError {
    let reason = match failure.kind {
        XmlFailureKind::ByteLimit => ReferenceIndexError::XmlByteLimit,
        XmlFailureKind::NodeLimit => ReferenceIndexError::XmlNodeLimit,
        XmlFailureKind::UnsupportedEncoding => ReferenceIndexError::UnsupportedEncoding,
        XmlFailureKind::ExternalEntity => ReferenceIndexError::ExternalEntity,
        XmlFailureKind::UnknownEntity => ReferenceIndexError::UnknownEntity,
        XmlFailureKind::EntityExpansionLimit => ReferenceIndexError::EntityExpansionLimit,
        XmlFailureKind::NavigationDepthLimit => ReferenceIndexError::XmlDepthLimit,
        XmlFailureKind::Malformed => ReferenceIndexError::XmlMalformed,
    };
    at(path, failure.line, failure.column, reason)
}
fn scheme(value: &str) -> String {
    let Some((name, _)) = value.split_once(':') else {
        return "network-path".to_owned();
    };
    if !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
    {
        name.to_ascii_lowercase()
    } else {
        "external".to_owned()
    }
}
fn resolve(path: &str, value: &str) -> Resolution {
    match resolve_href(path, value) {
        Ok(target) if target.external => Resolution::External(scheme(value)),
        Ok(target) => target.path.map_or(Resolution::Invalid, |p| {
            Resolution::Internal(p, target.fragment)
        }),
        Err(_) => Resolution::Invalid,
    }
}

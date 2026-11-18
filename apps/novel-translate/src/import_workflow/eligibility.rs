use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use nt_epub::{
    Diagnostic, DiagnosticCode, EpubInventory, ReferenceIndex, ReferenceKind, ReferenceTarget,
    ResourceEncryption, SourceDiagnostic, SpineContentKind,
};

/// The archive location is safe to report without exposing its raw links.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockingSource {
    MissingPackage,
    MissingSpine,
    Spine(usize),
    Inventory(Diagnostic),
    Resource {
        canonical_path: String,
    },
    Reference {
        canonical_path: String,
        kind: ReferenceKind,
        line: u32,
        column: u32,
    },
}

pub(super) fn check_spine(inventory: &EpubInventory) -> Result<(), BlockingSource> {
    if inventory.package_path.is_none() || inventory.package_version.is_none() {
        return Err(BlockingSource::MissingPackage);
    }
    if !inventory.spine.iter().any(|entry| entry.linear) {
        return Err(BlockingSource::MissingSpine);
    }
    for entry in &inventory.spine {
        let resource = entry
            .resource_index
            .and_then(|index| inventory.resources.get(index));
        let supported = matches!(
            entry.kind,
            SpineContentKind::ReflowableDocument | SpineContentKind::ImagePage
        );
        if !supported
            || !resource.is_some_and(|resource| {
                resource.available
                    && resource.path.is_some()
                    && resource.encryption != ResourceEncryption::Unsupported
            })
        {
            return Err(BlockingSource::Spine(entry.reading_order));
        }
    }
    Ok(())
}

/// Resolve dependencies from the real included source set. An ordinary
/// hyperlink and a reference required to render source have different gates.
pub(super) fn check_dependencies(
    inventory: &EpubInventory,
    references: &ReferenceIndex,
    diagnostics: &[SourceDiagnostic],
) -> Result<(), BlockingSource> {
    check_spine(inventory)?;
    let spine_paths: BTreeSet<_> = inventory
        .spine
        .iter()
        .filter_map(|entry| entry.resource_index)
        .filter_map(|index| inventory.resources.get(index)?.path.as_deref())
        .collect();
    let non_spine_nav = |path: &str| {
        !spine_paths.contains(path)
            && (inventory.resources.iter().any(|r| {
                r.path.as_deref() == Some(path)
                    && (r.media_type == "application/x-dtbncx+xml"
                        || r.properties.iter().any(|property| property == "nav"))
            }) || inventory
                .navigation
                .iter()
                .any(|tree| tree.resource_path == path))
    };
    let mut outgoing = HashMap::<&str, Vec<_>>::new();
    for reference in &references.references {
        if file_dependency(reference.kind) {
            outgoing
                .entry(&reference.source_path)
                .or_default()
                .push(reference);
        }
    }
    let mut pending = VecDeque::new();
    pending.extend(spine_paths.iter().copied());
    pending.extend(inventory.package_path.as_deref());
    for resource in &inventory.resources {
        if let Some(path) = resource.path.as_deref()
            && resource.available
            && resource.media_type == "application/xhtml+xml"
            && !non_spine_nav(path)
        {
            pending.push_back(path);
        }
    }
    let mut required = HashSet::new();
    while let Some(path) = pending.pop_front() {
        if !required.insert(path) {
            continue;
        }
        if let Some(uses) = outgoing.get(path) {
            for reference in uses {
                match &reference.target {
                    ReferenceTarget::Internal { path, .. } => pending.push_back(path),
                    ReferenceTarget::External { scheme }
                        if scheme == "data" && embedded_asset(reference.kind) => {}
                    _ => {
                        return Err(BlockingSource::Reference {
                            canonical_path: reference.source_path.clone(),
                            kind: reference.kind,
                            line: reference.location.line,
                            column: reference.location.column,
                        });
                    }
                }
            }
        }
    }

    for diagnostic in &inventory.diagnostics {
        let path = diagnostic.resource_path.as_deref();
        // Package diagnostics can identify their affected manifest resource
        // separately from the OPF file holding the invalid reference.
        let subject = diagnostic
            .related_resource
            .as_deref()
            .and_then(|name| {
                inventory
                    .resources
                    .iter()
                    .find(|resource| resource.id == name || resource.path.as_deref() == Some(name))
                    .and_then(|resource| resource.path.as_deref())
            })
            .or(diagnostic.related_resource.as_deref())
            .or(path);
        let role = match diagnostic.code {
            DiagnosticCode::ResourceMissing
            | DiagnosticCode::ExternalResourceNotRead
            | DiagnosticCode::UnsupportedEncryption
            | DiagnosticCode::ResourceHrefInvalid => subject,
            _ => path,
        };
        let on_required_path = role.is_none_or(|path| required.contains(path));
        let optional_navigation = path.is_some_and(non_spine_nav);
        if diagnostic_blocks(diagnostic.code, on_required_path, optional_navigation) {
            return Err(BlockingSource::Inventory(diagnostic.clone()));
        }
    }
    for diagnostic in diagnostics {
        let blocked = match diagnostic {
            SourceDiagnostic::UnsupportedFixedLayout { canonical_path }
            | SourceDiagnostic::UnsupportedEncryption { canonical_path }
                if required.contains(canonical_path.as_str()) =>
            {
                Some(canonical_path)
            }
            SourceDiagnostic::ProtectedResourceMissing { canonical_path } => Some(canonical_path),
            SourceDiagnostic::InvalidOptionalNavigation { canonical_path }
                if !non_spine_nav(canonical_path) || required.contains(canonical_path.as_str()) =>
            {
                Some(canonical_path)
            }
            _ => None,
        };
        if let Some(path) = blocked {
            return Err(BlockingSource::Resource {
                canonical_path: path.clone(),
            });
        }
    }
    Ok(())
}

fn embedded_asset(kind: ReferenceKind) -> bool {
    matches!(
        kind,
        ReferenceKind::XhtmlImageSrc
            | ReferenceKind::SvgImageHref
            | ReferenceKind::SrcsetCandidate
            | ReferenceKind::CssUrl
    )
}

fn file_dependency(kind: ReferenceKind) -> bool {
    embedded_asset(kind)
        || matches!(
            kind,
            ReferenceKind::StylesheetLink
                | ReferenceKind::CssImport
                | ReferenceKind::Cover
                | ReferenceKind::Fallback
                | ReferenceKind::MediaOverlay
                | ReferenceKind::Spine
        )
}

fn diagnostic_blocks(code: DiagnosticCode, required: bool, optional_navigation: bool) -> bool {
    use DiagnosticCode::*;
    match code {
        MetadataMissing
        | UniqueIdentifierMissing
        | UniqueIdentifierUnresolved
        | NavigationMissing
        | SpineTocReferenceInvalid
        | NavigationSourcesConflict
        | NavigationTargetMissing
        | NavigationTargetUnlisted
        | NavigationTargetAmbiguous
        | ImageOnlyContentNeedsImageWorkflow
        | FontObfuscationDetected
        | ReferencedImageUnlisted => false,
        NavigationInvalidXml | NavigationStructureUnsupported | XmlMalformed => {
            !optional_navigation
        }
        ResourceMissing
        | ExternalResourceNotRead
        | UnsupportedEncryption
        | ExternalReferenceNotRead
        | ReferencedImageMissing
        | ReferencedImageAmbiguous => required,
        // These compromise the package, a required document, or the scope of
        // safe decoding itself. In particular, limits always block.
        ArchiveEntryUnreadable
        | ContainerMissing
        | ContainerInvalidXml
        | ContainerRootfileMissing
        | ContainerRootfileInvalid
        | PackageMissing
        | PackageInvalidXml
        | PackageRootInvalid
        | PackageVersionMissing
        | PackageVersionUnsupported
        | ManifestMissing
        | ManifestIdMissing
        | ManifestIdDuplicate
        | ManifestItemInvalid
        | ResourceHrefInvalid
        | ResourcePathAmbiguous
        | SpineMissing
        | SpineEmpty
        | SpineReferenceInvalid
        | SpineResourceMissing
        | SpineLinearInvalid
        | XhtmlNamespaceInvalid
        | UnsupportedSpineMediaType
        | FixedLayoutTextUnsupported
        | FixedLayoutPageUnsupported
        | NavigationDepthLimitExceeded
        | EncryptionMetadataInvalid
        | EncryptedResourceMissing
        | EncryptedResourceUnlisted
        | ScriptedContentUnsupported
        | RemoteResourcesUnsupported
        | BaseResolutionLimitExceeded
        | XmlByteLimitExceeded
        | XmlNodeLimitExceeded
        | XmlEncodingUnsupported
        | XmlExternalEntityRejected
        | XmlEntityReferenceUnknown
        | XmlEntityExpansionLimit => true,
    }
}

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;
use std::process::Command;

const PRODUCT_PACKAGES: [&str; 5] = [
    "novel-translate",
    "nt-application",
    "nt-domain",
    "nt-storage",
    "nt-tui",
];
const ARCHITECTURE_TARGET: &str = "x86_64-pc-windows-msvc";

const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";
const TRACING_PACKAGE_ID: &str =
    "registry+https://github.com/rust-lang/crates.io-index#tracing@0.1.44";
const TRACING_SUBSCRIBER_PACKAGE_ID: &str =
    "registry+https://github.com/rust-lang/crates.io-index#tracing-subscriber@0.3.23";
const CROSSTERM_WINAPI_PACKAGE_ID: &str =
    "registry+https://github.com/rust-lang/crates.io-index#crossterm_winapi@0.9.1";

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    #[serde(default)]
    workspace_members: Vec<String>,
    #[serde(default)]
    resolve: Option<CargoResolve>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    #[serde(default)]
    id: String,
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    source: Option<String>,
    manifest_path: String,
    #[serde(default)]
    dependencies: Vec<CargoDependency>,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoDependency {
    /// Cargo metadata's `name` is the actual package name. `rename` is the
    /// local crate alias and must never be used to decide architecture edges.
    name: String,
    #[serde(default)]
    rename: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoResolve {
    #[serde(default)]
    nodes: Vec<CargoNode>,
}

#[derive(Debug, Deserialize)]
struct CargoNode {
    id: String,
    #[serde(default)]
    deps: Vec<CargoResolvedDependency>,
}

#[derive(Debug, Deserialize)]
struct CargoResolvedDependency {
    /// Cargo's resolved dependency name is the manifest alias; `pkg` is the
    /// identity of the package Cargo actually selected.
    name: String,
    pkg: String,
    #[serde(default)]
    dep_kinds: Vec<CargoResolvedDependencyKind>,
}

#[derive(Debug, Deserialize)]
struct CargoResolvedDependencyKind {
    kind: Option<String>,
    target: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum DependencyKind {
    Normal,
    Build,
    Dev,
}

impl DependencyKind {
    fn parse(kind: Option<&str>) -> Result<Self, String> {
        match kind {
            None | Some("normal") => Ok(Self::Normal),
            Some("build") => Ok(Self::Build),
            Some("dev") => Ok(Self::Dev),
            Some(other) => Err(format!("unsupported Cargo dependency kind `{other}`")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Build => "build",
            Self::Dev => "dev",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Edge {
    source: String,
    target: String,
    kind: DependencyKind,
    alias: Option<String>,
    target_condition: Option<String>,
    optional: bool,
    features: Vec<String>,
    package_source: Option<String>,
    package_path: Option<String>,
}

impl Edge {
    fn witness(&self) -> String {
        let alias = self
            .alias
            .as_deref()
            .map(|name| format!(", alias=`{name}`"))
            .unwrap_or_default();
        let target = self.target_condition.as_deref().unwrap_or("all targets");
        let features = if self.features.is_empty() {
            "none".to_owned()
        } else {
            self.features.join(",")
        };
        format!(
            "path=`{} -> {}` (kind={}, target={target}, optional={}, features={features}{alias})",
            self.source,
            self.target,
            self.kind.as_str(),
            self.optional,
        )
    }
}

#[derive(Debug, Default)]
struct ArchitectureReport {
    runtime_edges: Vec<Edge>,
    dev_edges: Vec<Edge>,
    violations: Vec<String>,
}

impl ArchitectureReport {
    fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    fn describe_violations(&self) -> String {
        if self.violations.is_empty() {
            "no forbidden dependency paths".to_owned()
        } else {
            self.violations.join("\n")
        }
    }
}

fn allowed_internal_edge(source: &str, target: &str) -> bool {
    matches!(
        (source, target),
        ("nt-application", "nt-domain")
            | ("nt-storage", "nt-application")
            | ("nt-storage", "nt-domain")
            | ("nt-tui", "nt-application")
            | ("novel-translate", "nt-application")
            | ("novel-translate", "nt-tui")
            | ("novel-translate", "nt-storage")
    )
}

fn allowed_external_dependency(
    owner: &str,
    dependency: &str,
    target: Option<&str>,
    kind: DependencyKind,
) -> bool {
    match owner {
        "nt-domain" => matches!(dependency, "thiserror"),
        "nt-application" => matches!(dependency, "thiserror" | "serde"),
        "nt-storage" => {
            dependency == "rusqlite"
                || (dependency == "sha2" && target.is_none() && kind == DependencyKind::Normal)
                || (dependency == "windows-sys"
                    && target == Some("cfg(windows)")
                    && kind == DependencyKind::Normal)
        }
        "nt-tui" => {
            matches!(
                dependency,
                "ratatui" | "crossterm" | "thiserror" | "unicode-segmentation"
            ) || (dependency == "crossterm_winapi"
                && target == Some("cfg(windows)")
                && kind == DependencyKind::Normal)
        }
        "novel-translate" => matches!(
            dependency,
            "anyhow"
                | "clap"
                | "serde"
                | "serde_json"
                | "thiserror"
                | "tracing"
                | "tracing-subscriber"
        ),
        _ => false,
    }
}

fn analyze_metadata(json: &str) -> Result<ArchitectureReport, serde_json::Error> {
    // Inspect all member declarations, including optional and target-specific
    // edges, then use Cargo's resolved graph to verify package identity/source
    // and catch a product crate hidden behind an external package.
    let metadata: CargoMetadata = serde_json::from_str(json)?;
    let package_by_id: BTreeMap<_, _> = metadata
        .packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect();
    let workspace_member_ids: BTreeSet<_> = if metadata.workspace_members.is_empty() {
        metadata
            .packages
            .iter()
            .filter(|package| PRODUCT_PACKAGES.contains(&package.name.as_str()))
            .map(|package| package.id.clone())
            .collect()
    } else {
        metadata.workspace_members.iter().cloned().collect()
    };
    let workspace_packages: Vec<_> = metadata
        .packages
        .iter()
        .filter(|package| workspace_member_ids.contains(&package.id))
        .collect();
    let expected_packages: BTreeSet<_> = PRODUCT_PACKAGES.into_iter().collect();
    let actual_packages: BTreeSet<_> = workspace_packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    let workspace_package_paths: BTreeMap<_, _> = workspace_packages
        .iter()
        .map(|package| {
            (
                package.name.clone(),
                normalize_path(&package.manifest_path)
                    .trim_end_matches("/cargo.toml")
                    .to_owned(),
            )
        })
        .collect();
    let mut report = ArchitectureReport::default();

    if actual_packages != expected_packages {
        report.violations.push(format!(
            "workspace package inventory differs: expected [{}], got [{}]",
            expected_packages.into_iter().collect::<Vec<_>>().join(", "),
            actual_packages.into_iter().collect::<Vec<_>>().join(", "),
        ));
    }

    for package in workspace_packages {
        for dependency in &package.dependencies {
            let kind = match DependencyKind::parse(dependency.kind.as_deref()) {
                Ok(kind) => kind,
                Err(error) => {
                    report.violations.push(format!(
                        "package `{}` dependency `{}`: {error}",
                        package.name, dependency.name
                    ));
                    continue;
                }
            };
            let edge = Edge {
                source: package.name.clone(),
                target: dependency.name.clone(),
                kind,
                alias: dependency.rename.clone(),
                target_condition: dependency.target.clone(),
                optional: dependency.optional,
                features: dependency.features.clone(),
                package_source: dependency.source.clone(),
                package_path: dependency.path.clone(),
            };

            // Dev dependencies are harness-only edges. Keep them visible in
            // the report, but do not turn them into product runtime paths.
            if edge.kind == DependencyKind::Dev {
                report.dev_edges.push(edge);
                continue;
            }

            let is_workspace_package = edge.package_source.is_none()
                && workspace_package_paths
                    .get(&edge.target)
                    .is_some_and(|path| {
                        edge.package_path.as_deref().is_some_and(|dependency_path| {
                            normalize_path(dependency_path) == path.as_str()
                        })
                    });
            if is_workspace_package {
                if !allowed_internal_edge(&edge.source, &edge.target) {
                    report.violations.push(format!(
                        "forbidden internal dependency: source=`{}`, destination=`{}`; {}",
                        edge.source,
                        edge.target,
                        edge.witness(),
                    ));
                }
            } else if !allowed_external_dependency(
                &edge.source,
                &edge.target,
                edge.target_condition.as_deref(),
                edge.kind,
            ) {
                report.violations.push(format!(
                    "unapproved external dependency: owner=`{}`, package=`{}`; {}",
                    edge.source,
                    edge.target,
                    edge.witness(),
                ));
            } else if edge.package_source.as_deref() != Some(CRATES_IO)
                || edge.package_path.is_some()
            {
                report.violations.push(format!(
                    "external dependency source/path is not approved: owner=`{}`, package=`{}`, declared_source={:?}, declared_path={:?}; {}",
                    edge.source,
                    edge.target,
                    edge.package_source,
                    edge.package_path,
                    edge.witness(),
                ));
            } else if metadata.resolve.is_some() {
                let resolved_packages =
                    resolved_packages_for_declaration(&metadata, package, dependency);
                if resolved_packages.is_empty() {
                    report.violations.push(format!(
                        "approved external dependency is missing from Cargo's resolved graph: owner=`{}`, package=`{}`; {}",
                        edge.source,
                        edge.target,
                        edge.witness(),
                    ));
                }
                for resolved_package in resolved_packages {
                    if resolved_package.name != edge.target
                        || resolved_package.source.as_deref() != Some(CRATES_IO)
                    {
                        report.violations.push(format!(
                            "resolved external package identity/source is not approved: owner=`{}`, package=`{}`, resolved_name=`{}`, resolved_source={:?}, resolved_path=`{}`; {}",
                            edge.source,
                            edge.target,
                            resolved_package.name,
                            resolved_package.source,
                            resolved_package.manifest_path,
                            edge.witness(),
                        ));
                    }
                }
            }
            report.runtime_edges.push(edge);
        }
    }

    if let Some(resolve) = &metadata.resolve {
        report.violations.extend(forbidden_transitive_product_edges(
            &resolve.nodes,
            &workspace_member_ids,
            &package_by_id,
        ));
    }

    report.runtime_edges.sort();
    report.dev_edges.sort();
    report.violations.sort();
    Ok(report)
}

fn resolved_packages_for_declaration<'a>(
    metadata: &'a CargoMetadata,
    package: &CargoPackage,
    dependency: &CargoDependency,
) -> Vec<&'a CargoPackage> {
    let Some(resolve) = &metadata.resolve else {
        return Vec::new();
    };
    let alias = normalize_resolved_dependency_name(
        dependency.rename.as_deref().unwrap_or(&dependency.name),
    );
    let Some(node) = resolve.nodes.iter().find(|node| node.id == package.id) else {
        return Vec::new();
    };
    let resolved_ids: BTreeSet<_> = node
        .deps
        .iter()
        .filter(|resolved| {
            normalize_resolved_dependency_name(&resolved.name) == alias
                && resolved_matches_declaration(resolved, dependency)
        })
        .map(|resolved| resolved.pkg.as_str())
        .collect();

    metadata
        .packages
        .iter()
        .filter(|resolved_package| resolved_ids.contains(resolved_package.id.as_str()))
        .collect()
}

fn normalize_resolved_dependency_name(name: &str) -> String {
    name.replace('-', "_")
}

fn resolved_matches_declaration(
    resolved: &CargoResolvedDependency,
    dependency: &CargoDependency,
) -> bool {
    resolved.dep_kinds.is_empty()
        || resolved.dep_kinds.iter().any(|kind| {
            kind.kind.as_deref() == dependency.kind.as_deref()
                && kind.target.as_deref() == dependency.target.as_deref()
        })
}

fn forbidden_transitive_product_edges(
    nodes: &[CargoNode],
    workspace_member_ids: &BTreeSet<String>,
    package_by_id: &BTreeMap<&str, &CargoPackage>,
) -> Vec<String> {
    let nodes_by_id: BTreeMap<_, _> = nodes.iter().map(|node| (node.id.as_str(), node)).collect();
    let mut violations = BTreeSet::new();

    for root_id in workspace_member_ids {
        let Some(root) = package_by_id.get(root_id.as_str()) else {
            continue;
        };
        let mut pending = VecDeque::from([(root_id.clone(), vec![root.name.clone()])]);
        let mut visited = BTreeSet::from([root_id.clone()]);

        while let Some((source_id, path)) = pending.pop_front() {
            let Some(node) = nodes_by_id.get(source_id.as_str()) else {
                continue;
            };
            let Some(source_package) = package_by_id.get(source_id.as_str()) else {
                continue;
            };

            for dependency in &node.deps {
                if !resolved_dependency_is_product_edge(dependency) {
                    continue;
                }
                let Some(destination) = package_by_id.get(dependency.pkg.as_str()) else {
                    continue;
                };
                let mut next_path = path.clone();
                next_path.push(destination.name.clone());

                if workspace_member_ids.contains(&destination.id)
                    && !workspace_member_ids.contains(&source_package.id)
                {
                    violations.insert(format!(
                        "forbidden transitive product dependency: source=`{}`, destination=`{}`; path=`{}`",
                        root.name,
                        destination.name,
                        next_path.join(" -> "),
                    ));
                }

                if visited.insert(destination.id.clone()) {
                    pending.push_back((destination.id.clone(), next_path));
                }
            }
        }
    }

    violations.into_iter().collect()
}

fn resolved_dependency_is_product_edge(dependency: &CargoResolvedDependency) -> bool {
    dependency.dep_kinds.is_empty()
        || dependency
            .dep_kinds
            .iter()
            .any(|kind| kind.kind.as_deref() != Some("dev"))
}

fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/").trim_end_matches('/').to_owned();
    if cfg!(windows) {
        normalized.to_ascii_lowercase()
    } else {
        normalized
    }
}

fn dependency(
    name: &str,
    rename: Option<&str>,
    kind: Option<&str>,
    target: Option<&str>,
    optional: bool,
    features: &[&str],
    source: Option<&str>,
) -> serde_json::Value {
    let local_path = if source.is_none() && PRODUCT_PACKAGES.contains(&name) {
        Some(format!("C:/workspace/{name}"))
    } else {
        None
    };
    serde_json::json!({
        "name": name,
        "rename": rename,
        "kind": kind,
        "target": target,
        "optional": optional,
        "features": features,
        "source": source,
        "path": local_path,
    })
}

fn package(name: &str, dependencies: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "id": name,
        "name": name,
        "version": "0.1.0",
        "source": null,
        "manifest_path": format!("C:/workspace/{name}/Cargo.toml"),
        "dependencies": dependencies,
    })
}

fn fixture(packages: Vec<serde_json::Value>) -> String {
    let mut packages = packages;
    if !packages
        .iter()
        .any(|package| package["name"] == "nt-storage")
    {
        packages.push(package("nt-storage", vec![]));
    }
    let workspace_members: Vec<_> = packages
        .iter()
        .filter_map(|package| package.get("id").and_then(serde_json::Value::as_str))
        .collect();
    serde_json::json!({
        "packages": packages,
        "workspace_members": workspace_members,
    })
    .to_string()
}

fn resolved_fixture(
    mut packages: Vec<serde_json::Value>,
    workspace_members: &[&str],
    mut nodes: Vec<serde_json::Value>,
) -> String {
    if !packages
        .iter()
        .any(|package| package["name"] == "nt-storage")
    {
        packages.push(package("nt-storage", vec![]));
    }
    if !workspace_members.contains(&"nt-storage") {
        let mut members = workspace_members.to_vec();
        members.push("nt-storage");
        return resolved_fixture(packages, &members, nodes);
    }
    if !nodes.iter().any(|node| node["id"] == "nt-storage") {
        nodes.push(resolved_node("nt-storage", vec![]));
    }
    serde_json::json!({
        "packages": packages,
        "workspace_members": workspace_members,
        "resolve": { "nodes": nodes },
    })
    .to_string()
}

fn resolved_node(id: &str, deps: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({ "id": id, "deps": deps })
}

fn resolved_dependency(
    name: &str,
    package_id: &str,
    kind: Option<&str>,
    target: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "pkg": package_id,
        "dep_kinds": [{ "kind": kind, "target": target }],
    })
}

fn registry_package(package_id: &str, name: &str, version: &str) -> serde_json::Value {
    serde_json::json!({
        "id": package_id,
        "name": name,
        "version": version,
        "source": CRATES_IO,
        "manifest_path": format!("C:/cargo/registry/src/{name}-{version}/Cargo.toml"),
        "dependencies": [],
    })
}

fn app_registry_fixture(
    dependencies: Vec<serde_json::Value>,
    resolved_dependencies: Vec<serde_json::Value>,
) -> String {
    resolved_fixture(
        vec![
            package("novel-translate", dependencies),
            package("nt-application", vec![]),
            package("nt-domain", vec![]),
            package("nt-tui", vec![]),
            registry_package(TRACING_PACKAGE_ID, "tracing", "0.1.44"),
            registry_package(
                TRACING_SUBSCRIBER_PACKAGE_ID,
                "tracing-subscriber",
                "0.3.23",
            ),
        ],
        &["novel-translate", "nt-application", "nt-domain", "nt-tui"],
        vec![
            resolved_node("novel-translate", resolved_dependencies),
            resolved_node("nt-application", vec![]),
            resolved_node("nt-domain", vec![]),
            resolved_node("nt-tui", vec![]),
            resolved_node(TRACING_PACKAGE_ID, vec![]),
            resolved_node(TRACING_SUBSCRIBER_PACKAGE_ID, vec![]),
        ],
    )
}

fn registry() -> Option<&'static str> {
    Some(CRATES_IO)
}

#[test]
fn real_cargo_metadata_obeys_product_boundaries() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("app package must be under the repository root");
    let output = Command::new("cargo")
        .current_dir(workspace_root)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--offline",
            "--all-features",
            "--filter-platform",
            ARCHITECTURE_TARGET,
            "--manifest-path",
        ])
        .arg(workspace_root.join("Cargo.toml"))
        .output()
        .expect("run cargo metadata for the real workspace");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata = String::from_utf8(output.stdout).expect("cargo metadata must emit UTF-8 JSON");
    let full_metadata: CargoMetadata =
        serde_json::from_str(&metadata).expect("parse full Cargo metadata JSON");
    assert!(
        full_metadata.resolve.is_some(),
        "full Cargo metadata must include the resolved dependency graph"
    );
    let report = analyze_metadata(&metadata).expect("parse cargo metadata JSON");
    assert!(report.is_clean(), "{}", report.describe_violations());
    assert!(
        report
            .dev_edges
            .iter()
            .all(|edge| edge.kind == DependencyKind::Dev)
    );
    for expected_app_dependency in ["clap", "serde", "serde_json"] {
        assert!(
            report.runtime_edges.iter().any(|edge| {
                edge.source == "novel-translate"
                    && edge.target == expected_app_dependency
                    && edge.kind == DependencyKind::Normal
            }),
            "CLI runtime dependency `{expected_app_dependency}` must be reported as a normal app edge"
        );
    }
    assert!(
        report.runtime_edges.iter().any(|edge| {
            edge.source == "nt-storage"
                && edge.target == "sha2"
                && edge.kind == DependencyKind::Normal
                && edge.target_condition.is_none()
        }),
        "object hashing must use sha2 as a normal dependency of nt-storage"
    );
    let sha2_package = full_metadata
        .packages
        .iter()
        .find(|package| package.name == "sha2")
        .expect("the exact SHA-256 package must be resolved");
    assert_eq!(sha2_package.version, "0.11.1");
    assert_eq!(sha2_package.source.as_deref(), Some(CRATES_IO));
}

#[test]
fn allows_sha2_only_as_a_normal_all_target_storage_dependency() {
    assert!(allowed_external_dependency(
        "nt-storage",
        "sha2",
        None,
        DependencyKind::Normal
    ));
    assert!(!allowed_external_dependency(
        "nt-storage",
        "sha2",
        Some("cfg(windows)"),
        DependencyKind::Normal
    ));
    assert!(!allowed_external_dependency(
        "nt-storage",
        "sha2",
        None,
        DependencyKind::Build
    ));
    assert!(!allowed_external_dependency(
        "nt-application",
        "sha2",
        None,
        DependencyKind::Normal
    ));
}

#[test]
fn accepts_allowed_edges_and_uses_real_package_names_for_renamed_dependencies() {
    let metadata = fixture(vec![
        package(
            "novel-translate",
            vec![
                dependency(
                    "nt-tui",
                    Some("terminal_ui"),
                    None,
                    Some("cfg(windows)"),
                    true,
                    &["crossterm_0_29"],
                    None,
                ),
                dependency("nt-application", None, None, None, false, &[], None),
                dependency("clap", None, None, None, false, &["derive"], registry()),
                dependency("tracing", None, None, None, false, &["std"], registry()),
            ],
        ),
        package(
            "nt-application",
            vec![dependency(
                "nt-domain",
                Some("core"),
                None,
                Some("cfg(unix)"),
                false,
                &[],
                None,
            )],
        ),
        package(
            "nt-tui",
            vec![
                dependency(
                    "nt-application",
                    None,
                    Some("build"),
                    Some("cfg(windows)"),
                    false,
                    &[],
                    None,
                ),
                dependency(
                    "ratatui",
                    None,
                    None,
                    None,
                    false,
                    &["crossterm_0_29"],
                    registry(),
                ),
            ],
        ),
        package(
            "nt-domain",
            vec![dependency(
                "novel-translate",
                None,
                Some("dev"),
                None,
                false,
                &[],
                None,
            )],
        ),
        package(
            "nt-storage",
            vec![
                dependency("nt-application", None, None, None, false, &[], None),
                dependency("nt-domain", None, None, None, false, &[], None),
            ],
        ),
    ]);

    let report = analyze_metadata(&metadata).expect("parse fixture metadata");
    assert!(report.is_clean(), "{}", report.describe_violations());
    assert_eq!(report.runtime_edges.len(), 9);
    assert_eq!(report.dev_edges.len(), 1);
    let renamed_tui = report
        .runtime_edges
        .iter()
        .find(|edge| edge.source == "novel-translate" && edge.target == "nt-tui")
        .expect("the renamed workspace package must retain its real identity");
    assert_eq!(renamed_tui.alias.as_deref(), Some("terminal_ui"));
    assert_eq!(
        renamed_tui.target_condition.as_deref(),
        Some("cfg(windows)")
    );
    assert!(renamed_tui.optional);
    assert_eq!(renamed_tui.features, ["crossterm_0_29"]);
    assert_eq!(report.dev_edges[0].target, "novel-translate");
}

#[test]
fn accepts_crossterm_winapi_only_as_a_normal_windows_tui_dependency() {
    let package_set = |target: &str, kind: Option<&str>| {
        resolved_fixture(
            vec![
                package("novel-translate", vec![]),
                package("nt-application", vec![]),
                package("nt-domain", vec![]),
                package(
                    "nt-tui",
                    vec![dependency(
                        "crossterm_winapi",
                        None,
                        kind,
                        Some(target),
                        false,
                        &[],
                        registry(),
                    )],
                ),
                registry_package(CROSSTERM_WINAPI_PACKAGE_ID, "crossterm_winapi", "0.9.1"),
            ],
            &["novel-translate", "nt-application", "nt-domain", "nt-tui"],
            vec![
                resolved_node("novel-translate", vec![]),
                resolved_node("nt-application", vec![]),
                resolved_node("nt-domain", vec![]),
                resolved_node(
                    "nt-tui",
                    vec![resolved_dependency(
                        "crossterm_winapi",
                        CROSSTERM_WINAPI_PACKAGE_ID,
                        kind,
                        Some(target),
                    )],
                ),
                resolved_node(CROSSTERM_WINAPI_PACKAGE_ID, vec![]),
            ],
        )
    };

    let windows = analyze_metadata(&package_set("cfg(windows)", None))
        .expect("parse Windows adapter fixture");
    assert!(windows.is_clean(), "{}", windows.describe_violations());
    assert_eq!(windows.runtime_edges.len(), 1);
    assert_eq!(
        windows.runtime_edges[0].target_condition.as_deref(),
        Some("cfg(windows)")
    );

    let unix = analyze_metadata(&package_set("cfg(unix)", None))
        .expect("parse non-Windows adapter fixture");
    assert_eq!(unix.violations.len(), 1);
    assert!(unix.violations[0].contains("owner=`nt-tui`, package=`crossterm_winapi`"));
    assert!(unix.violations[0].contains("target=cfg(unix)"));

    let build = analyze_metadata(&package_set("cfg(windows)", Some("build")))
        .expect("parse build-dependency adapter fixture");
    assert_eq!(build.violations.len(), 1);
    assert!(build.violations[0].contains("kind=build, target=cfg(windows)"));
}

#[test]
fn matches_hyphenated_package_and_alias_names_to_cargo_resolve_names() {
    let metadata = app_registry_fixture(
        vec![
            dependency(
                "tracing-subscriber",
                None,
                None,
                None,
                false,
                &["env-filter", "fmt"],
                registry(),
            ),
            dependency(
                "tracing",
                Some("probe-logging"),
                None,
                None,
                false,
                &["std"],
                registry(),
            ),
        ],
        vec![
            resolved_dependency(
                "tracing_subscriber",
                TRACING_SUBSCRIBER_PACKAGE_ID,
                None,
                None,
            ),
            resolved_dependency("probe_logging", TRACING_PACKAGE_ID, None, None),
        ],
    );

    let report = analyze_metadata(&metadata).expect("parse resolved metadata fixture");
    assert!(report.is_clean(), "{}", report.describe_violations());
    assert_eq!(report.runtime_edges.len(), 2);
}

#[test]
fn matches_a_reused_alias_by_resolved_target_and_dependency_kind() {
    let target_metadata = app_registry_fixture(
        vec![
            dependency(
                "tracing",
                Some("logging"),
                None,
                Some("cfg(unix)"),
                false,
                &["std"],
                registry(),
            ),
            dependency(
                "tracing-subscriber",
                Some("logging"),
                None,
                Some("cfg(windows)"),
                false,
                &["fmt"],
                registry(),
            ),
        ],
        vec![
            resolved_dependency("logging", TRACING_PACKAGE_ID, None, Some("cfg(unix)")),
            resolved_dependency(
                "logging",
                TRACING_SUBSCRIBER_PACKAGE_ID,
                None,
                Some("cfg(windows)"),
            ),
        ],
    );
    let target_report =
        analyze_metadata(&target_metadata).expect("parse target-specific metadata fixture");
    assert!(
        target_report.is_clean(),
        "{}",
        target_report.describe_violations()
    );

    let kind_metadata = app_registry_fixture(
        vec![
            dependency(
                "tracing",
                Some("logging"),
                None,
                None,
                false,
                &["std"],
                registry(),
            ),
            dependency(
                "tracing-subscriber",
                Some("logging"),
                Some("build"),
                None,
                false,
                &["fmt"],
                registry(),
            ),
        ],
        vec![
            resolved_dependency("logging", TRACING_PACKAGE_ID, None, None),
            resolved_dependency(
                "logging",
                TRACING_SUBSCRIBER_PACKAGE_ID,
                Some("build"),
                None,
            ),
        ],
    );
    let kind_report = analyze_metadata(&kind_metadata).expect("parse dependency-kind fixture");
    assert!(
        kind_report.is_clean(),
        "{}",
        kind_report.describe_violations()
    );
}

#[test]
fn reports_forbidden_internal_edges_with_source_destination_and_path() {
    let metadata = fixture(vec![
        package(
            "novel-translate",
            vec![dependency("nt-tui", None, None, None, false, &[], None)],
        ),
        package(
            "nt-application",
            vec![dependency(
                "nt-tui",
                Some("presentation"),
                None,
                Some("cfg(windows)"),
                false,
                &[],
                None,
            )],
        ),
        package(
            "nt-domain",
            vec![dependency(
                "nt-application",
                None,
                None,
                None,
                false,
                &[],
                None,
            )],
        ),
        package(
            "nt-tui",
            vec![dependency(
                "novel-translate",
                None,
                None,
                None,
                false,
                &[],
                None,
            )],
        ),
        package(
            "nt-storage",
            vec![dependency("nt-tui", None, None, None, false, &[], None)],
        ),
    ]);

    let report = analyze_metadata(&metadata).expect("parse fixture metadata");
    assert_eq!(
        report.violations.len(),
        4,
        "{}",
        report.describe_violations()
    );
    let violations = report.describe_violations();
    assert!(violations.contains("source=`nt-application`, destination=`nt-tui`"));
    assert!(violations.contains("path=`nt-application -> nt-tui`"));
    assert!(violations.contains("alias=`presentation`"));
    assert!(violations.contains("source=`nt-domain`, destination=`nt-application`"));
    assert!(violations.contains("source=`nt-storage`, destination=`nt-tui`"));
    assert!(violations.contains("source=`nt-tui`, destination=`novel-translate`"));
}

#[test]
fn does_not_confuse_a_same_named_external_path_with_a_workspace_crate() {
    let mut shadowed_domain = dependency("nt-domain", Some("core"), None, None, false, &[], None);
    shadowed_domain["path"] = serde_json::json!("C:/third-party/nt-domain");
    let metadata = fixture(vec![
        package("novel-translate", vec![shadowed_domain]),
        package("nt-application", vec![]),
        package("nt-domain", vec![]),
        package("nt-tui", vec![]),
    ]);

    let report = analyze_metadata(&metadata).expect("parse fixture metadata");
    assert_eq!(report.violations.len(), 1);
    assert!(report.violations[0].contains("owner=`novel-translate`, package=`nt-domain`"));
}

#[test]
fn rejects_an_allowlisted_external_package_declared_from_a_path() {
    let mut path_thiserror = dependency(
        "thiserror",
        Some("error_impl"),
        None,
        Some("cfg(unix)"),
        true,
        &[],
        None,
    );
    path_thiserror["path"] = serde_json::json!("C:/third-party/thiserror");
    let metadata = fixture(vec![
        package("novel-translate", vec![]),
        package("nt-application", vec![]),
        package("nt-domain", vec![path_thiserror]),
        package("nt-tui", vec![]),
    ]);

    let report = analyze_metadata(&metadata).expect("parse fixture metadata");
    assert_eq!(
        report.violations.len(),
        1,
        "{}",
        report.describe_violations()
    );
    assert!(report.violations[0].contains("external dependency source/path is not approved"));
    assert!(report.violations[0].contains("owner=`nt-domain`, package=`thiserror`"));
    assert!(report.violations[0].contains("declared_path=Some(\"C:/third-party/thiserror\")"));
}

#[test]
fn rejects_a_registry_patch_resolved_to_a_path_crate_that_depends_on_product_ui() {
    let patched_id = "path+file:///C:/third-party/thiserror#2.0.21";
    let metadata = resolved_fixture(
        vec![
            package("novel-translate", vec![]),
            package("nt-application", vec![]),
            package(
                "nt-domain",
                vec![dependency(
                    "thiserror",
                    Some("error_impl"),
                    None,
                    None,
                    false,
                    &[],
                    registry(),
                )],
            ),
            package("nt-tui", vec![]),
            serde_json::json!({
                "id": patched_id,
                "name": "thiserror",
                "version": "2.0.21",
                "source": null,
                "manifest_path": "C:/third-party/thiserror/Cargo.toml",
                "dependencies": [dependency("nt-tui", None, None, None, false, &[], None)],
            }),
        ],
        &["novel-translate", "nt-application", "nt-domain", "nt-tui"],
        vec![
            resolved_node("novel-translate", vec![]),
            resolved_node("nt-application", vec![]),
            resolved_node(
                "nt-domain",
                vec![resolved_dependency("error_impl", patched_id, None, None)],
            ),
            resolved_node("nt-tui", vec![]),
            resolved_node(
                patched_id,
                vec![resolved_dependency("nt-tui", "nt-tui", None, None)],
            ),
        ],
    );

    let report = analyze_metadata(&metadata).expect("parse fixture metadata");
    let violations = report.describe_violations();
    assert!(
        violations.contains("resolved external package identity/source is not approved"),
        "{violations}"
    );
    assert!(violations.contains("owner=`nt-domain`, package=`thiserror`"));
    assert!(violations.contains("source=`nt-domain`, destination=`nt-tui`"));
    assert!(violations.contains("path=`nt-domain -> thiserror -> nt-tui`"));
}

#[test]
fn rejects_unapproved_normal_and_target_specific_build_dependencies() {
    let metadata = fixture(vec![
        package("novel-translate", vec![]),
        package("nt-application", vec![]),
        package(
            "nt-domain",
            vec![
                dependency("serde", None, None, None, false, &["derive"], registry()),
                dependency(
                    "tokio",
                    Some("async_runtime"),
                    Some("build"),
                    Some("cfg(windows)"),
                    true,
                    &["rt"],
                    registry(),
                ),
            ],
        ),
        package(
            "nt-tui",
            vec![dependency(
                "tracing",
                Some("tui_log"),
                None,
                Some("cfg(unix)"),
                true,
                &["std"],
                registry(),
            )],
        ),
    ]);

    let report = analyze_metadata(&metadata).expect("parse fixture metadata");
    assert_eq!(
        report.violations.len(),
        3,
        "{}",
        report.describe_violations()
    );
    let violations = report.describe_violations();
    assert!(violations.contains("owner=`nt-domain`, package=`serde`"));
    assert!(violations.contains("owner=`nt-domain`, package=`tokio`"));
    assert!(violations.contains("owner=`nt-tui`, package=`tracing`"));
    assert!(violations.contains(
        "kind=build, target=cfg(windows), optional=true, features=rt, alias=`async_runtime`"
    ));
}

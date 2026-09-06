//! Running `cargo metadata` and reading a workspace's own first-party dependency edges
//! out of it.
//!
//! The only process this crate ever runs, and the only provider this workspace ships whose
//! `Materialize` needs more than bytes a caller already supplied — `nomos-lang-rust` and
//! `nomos-lang-rust-scan` are both pure functions over text. That process runs through a
//! caller-supplied [`ProcessLauncher`] rather than `std::process::Command` directly, the
//! same [`nomos_platform`] port `nomos-ledger` and `nomos-work-orchestration` already run
//! their own subprocesses through, so this crate depends on `nomos-platform` and not on any
//! concrete implementation of it — the composition root chooses that, same as it chooses a
//! [`nomos_platform::FileSystem`] for anything that walks a directory.
//!
//! Deliberately the same invocation `tests/contract/src/workspace.rs` already runs and
//! already established works over this workspace — `--no-deps` is kept, not dropped: for
//! a first-party, path-only dependency, `cargo metadata`'s per-package `dependencies`
//! array already names the exact package a manifest's `dependencies = {...}` entry
//! resolves to, kind and optionality included, without needing the full transitive
//! resolve graph `--no-deps` omits. What changes from `workspace.rs` is only that this
//! reader keeps the `kind`/`optional` fields that one discards.

use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[path = "metadata/discovered_package.rs"]
mod discovered_package;

pub use discovered_package::DiscoveredPackage;

/// `cargo metadata` over this workspace finishes in well under a second; a full minute is
/// generous headroom, the same bound `nomos-surface-provenance`'s own quick subprocess
/// calls use for the same reason.
const TIMEOUT: Duration = Duration::from_secs(60);

/// `cargo metadata` could not be run or its answer could not be read as this reader
/// expects.
///
/// A boundary this repository already lives by: `tests/contract/src/workspace.rs`'s own
/// doc comment says a check that cannot see the graph must fail loudly rather than verify
/// nothing. This type is that same refusal, returned rather than panicked, because a
/// provider's `Materialize` reports its own failures to a caller instead of aborting the
/// run that asked it a question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataError
{
    pub reason: String,
}

impl core::fmt::Display for MetadataError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Every first-party workspace member's own dependency edges, restricted to edges that
/// name another *workspace* member.
///
/// External (registry) dependencies are read by `cargo metadata` like any other, and are
/// deliberately not returned: this capability answers what `bands.rs`'s comparison
/// actually needs — whether one member of *this* workspace reaches another — and an edge
/// to a crate this workspace does not declare a band for is not a fact a
/// dependency-direction rule could ever act on. Filtering here, once, is what keeps that
/// choice from being reinvented at every caller of this capability.
///
/// # Errors
///
/// [`MetadataError`] if the `cargo` binary cannot be run, exits non-zero, is killed for
/// exceeding [`TIMEOUT`] or going idle for that long, or its stdout is not the JSON
/// document `--format-version 1` promises.
pub fn Discover_Workspace<Launcher: ProcessLauncher>(root: &Path, launcher: &Launcher) -> Result<Vec<DiscoveredPackage>, MetadataError>
{
    let document = Run_Cargo_Metadata(root, launcher)?;
    Require_Workspace_Root_Is(&document, root)?;
    let members = Member_Ids(&document)?;
    let packages = Packages_Array(&document)?;
    let member_names = Member_Names(packages, &members);
    let discovered = Discovered_Packages(packages, &members, &member_names, root)?;

    return Require_Nonempty(discovered);
}

/// Refuses if `document`'s own `"workspace_root"` does not resolve to `root` -- the exact
/// escape `P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT` measured directly: `cargo
/// metadata`, pointed at a directory with no `Cargo.toml` of its own that sits inside a
/// larger cargo workspace, silently walks upward and answers about that enclosing
/// workspace instead of refusing. `cargo metadata`'s own document always names exactly
/// which workspace root it actually resolved to, so this check reads that field directly
/// rather than inferring the escape indirectly from which packages came back.
///
/// Both sides are canonicalized before comparing: `workspace_root` is cargo's own already-
/// resolved absolute path, and `root` is often a relative `.` in a real invocation (`nomos
/// check`'s own CLI default), which `std::fs::canonicalize` resolves against the real
/// process working directory the same way `cargo metadata` itself did. Canonicalizing both
/// sides is safe even though `std::fs::canonicalize`'s own Windows implementation returns a
/// `\\?\`-prefixed verbatim path (`nomos_lang_rust_compiler::reading::Load_Crate`'s own doc
/// names this as a real footgun for a *prefix* comparison) -- this is a plain equality
/// check, and the identical transformation applied to both sides cancels out.
fn Require_Workspace_Root_Is(document: &serde_json::Value, root: &Path) -> Result<(), MetadataError>
{
    let workspace_root = document
        .get("workspace_root")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| MetadataError {
            reason: "cargo metadata's document has no \"workspace_root\" string".to_owned(),
        })?;

    let resolved = std::fs::canonicalize(workspace_root).map_err(|error| MetadataError {
        reason: format!("cargo metadata's own \"workspace_root\" ({workspace_root}) could not be read: {error}"),
    })?;
    let expected = std::fs::canonicalize(root).map_err(|error| MetadataError {
        reason: format!("the root cargo metadata was asked about ({}) could not be read: {error}", root.display()),
    })?;

    if resolved != expected
    {
        return Err(MetadataError {
            reason: format!(
                "cargo metadata escaped the workspace it was asked about: asked over \"{}\", it \
                 answered for \"{}\" instead -- refusing rather than silently reporting on the \
                 wrong tree",
                expected.display(),
                resolved.display()
            ),
        });
    }

    return Ok(());
}

fn Run_Cargo_Metadata<Launcher: ProcessLauncher>(root: &Path, launcher: &Launcher) -> Result<serde_json::Value, MetadataError>
{
    let command = Cargo_Metadata_Command(root);
    let output = launcher.Run(&command).map_err(|error| MetadataError {
        reason: format!("cargo metadata could not be run: {error}"),
    })?;

    Require_Clean_Exit(&output.outcome, &output.stderr)?;

    return Parse_Metadata_Document(&output.stdout);
}

/// The `cargo metadata` invocation `tests/contract/src/workspace.rs` already established
/// works over this workspace, run from `root`.
fn Cargo_Metadata_Command(root: &Path) -> Command
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut command = Command::New(
        vec![
            cargo,
            "metadata".to_owned(),
            "--format-version".to_owned(),
            "1".to_owned(),
            "--no-deps".to_owned(),
            "--all-features".to_owned(),
        ],
        TIMEOUT,
    );
    command.working_directory = Some(root.to_path_buf());

    return command;
}

/// Refuses every outcome a launched process can report other than a clean, zero exit.
fn Require_Clean_Exit(outcome: &ExitOutcome, stderr: &str) -> Result<(), MetadataError>
{
    return match outcome
    {
        ExitOutcome::Exited { code: 0 } => Ok(()),
        ExitOutcome::Exited { code } => Err(MetadataError {
            reason: format!("cargo metadata failed (exit {code}): {stderr}"),
        }),
        ExitOutcome::TimedOut => Err(MetadataError {
            reason: format!("cargo metadata was still running after {TIMEOUT:?} and was killed"),
        }),
        ExitOutcome::Stalled { idle_elapsed } => Err(MetadataError {
            reason: format!("cargo metadata produced no output for {idle_elapsed:?} and was judged stalled"),
        }),
        ExitOutcome::Terminated => Err(MetadataError {
            reason: "cargo metadata was terminated before it could finish".to_owned(),
        }),
    };
}

/// `cargo metadata`'s stdout, parsed as the JSON document `--format-version 1` promises.
fn Parse_Metadata_Document(stdout: &str) -> Result<serde_json::Value, MetadataError>
{
    return serde_json::from_str(stdout).map_err(|error| MetadataError {
        reason: format!("cargo metadata's stdout was not the JSON it promised: {error}"),
    });
}

/// The raw package-id strings `workspace_members` names, as opposed to registry
/// dependencies.
fn Member_Ids(document: &serde_json::Value) -> Result<BTreeSet<String>, MetadataError>
{
    let members = document
        .get("workspace_members")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| MetadataError {
            reason: "cargo metadata's document has no \"workspace_members\" array".to_owned(),
        })?;

    return Ok(members
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_owned)
        .collect());
}

/// The "packages" array `cargo metadata`'s document promises alongside `workspace_members`.
fn Packages_Array(document: &serde_json::Value) -> Result<&Vec<serde_json::Value>, MetadataError>
{
    return document
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| MetadataError {
            reason: "cargo metadata's document has no \"packages\" array".to_owned(),
        });
}

/// Every workspace member's own package name — the set `Read_Dependency` checks a
/// dependency's target against to decide whether it names another workspace member.
fn Member_Names(packages: &[serde_json::Value], members: &BTreeSet<String>) -> BTreeSet<String>
{
    return packages
        .iter()
        .filter(|package| Is_Member(package, members))
        .filter_map(Package_Name)
        .collect();
}

/// Each workspace member's own package, read into a [`DiscoveredPackage`]; a package that
/// is not a workspace member is skipped rather than read.
fn Discovered_Packages(
    packages: &[serde_json::Value],
    members: &BTreeSet<String>,
    member_names: &BTreeSet<String>,
    root: &Path,
) -> Result<Vec<DiscoveredPackage>, MetadataError>
{
    let mut discovered = Vec::new();
    for package in packages
    {
        if !Is_Member(package, members)
        {
            continue;
        }

        let read = Read_Package(package, member_names, root)?;
        discovered.push(read);
    }

    return Ok(discovered);
}

fn Read_Package(
    package: &serde_json::Value,
    member_names: &BTreeSet<String>,
    root: &Path,
) -> Result<DiscoveredPackage, MetadataError>
{
    let name = Package_Name(package).ok_or_else(|| MetadataError {
        reason: "a package in cargo metadata's document has no \"name\"".to_owned(),
    })?;

    let manifest_relative_root = Manifest_Relative_Root(package, root)?;
    let edges = Dependency_Edges(package, member_names);

    return Ok(DiscoveredPackage {
        payload: DependencyPayload {
            package: name,
            edges,
        },
        manifest_relative_root,
    });
}

fn Package_Name(package: &serde_json::Value) -> Option<String>
{
    return package
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
}

/// This package's manifest directory, relative to `root` and normalized to forward
/// slashes — the same convention every other subject in this workspace is addressed by.
fn Manifest_Relative_Root(package: &serde_json::Value, root: &Path) -> Result<String, MetadataError>
{
    let manifest_path = package
        .get("manifest_path")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| MetadataError {
            reason: "a package in cargo metadata's document has no \"manifest_path\"".to_owned(),
        })?;

    let absolute = PathBuf::from(manifest_path);
    let directory = absolute.parent().unwrap_or(&absolute);
    // `Require_Workspace_Root_Is` has already refused unless `root` is genuinely the
    // workspace root `cargo metadata` itself resolved, and every package this function
    // reads is one `cargo metadata` reported as a member of that same workspace -- so
    // `directory` is always really under `root` by the time execution reaches here. The
    // `unwrap_or` fallback below is therefore dead in practice, kept only because a
    // caller that skipped straight to this function without going through
    // `Discover_Workspace` first would have no such guarantee, and reporting a package's
    // own absolute path is a safer failure than a panic for that caller.
    let relative = directory.strip_prefix(root).unwrap_or(directory);

    return Ok(relative.to_string_lossy().replace('\\', "/"));
}

/// This package's own dependency edges, restricted to ones naming another workspace
/// member.
fn Dependency_Edges(package: &serde_json::Value, member_names: &BTreeSet<String>) -> Vec<DependencyEdge>
{
    let Some(dependencies) = package.get("dependencies").and_then(serde_json::Value::as_array)
    else
    {
        return Vec::new();
    };

    let mut edges: Vec<DependencyEdge> = dependencies
        .iter()
        .filter_map(|dependency| Read_Dependency(dependency, member_names))
        .collect();

    // Canonical order: the fact's bytes must not depend on the order cargo happened to
    // emit dependencies in, which is declaration order in the manifest and therefore an
    // authoring detail rather than anything this fact is about.
    edges.sort_by(|left, right| (&left.target, left.kind.Label(), left.optional).cmp(&(&right.target, right.kind.Label(), right.optional)));

    return edges;
}

fn Read_Dependency(dependency: &serde_json::Value, member_names: &BTreeSet<String>) -> Option<DependencyEdge>
{
    let target = dependency.get("name").and_then(serde_json::Value::as_str)?;
    if !member_names.contains(target)
    {
        return None;
    }

    let kind = Dependency_Kind(dependency)?;
    let optional = Is_Dependency_Optional(dependency);

    return Some(DependencyEdge {
        target: target.to_owned(),
        kind,
        optional,
    });
}

/// This dependency's kind, or `None` for a kind this reader does not recognize.
fn Dependency_Kind(dependency: &serde_json::Value) -> Option<DependencyKind>
{
    return match dependency.get("kind").and_then(serde_json::Value::as_str)
    {
        None | Some("null") => Some(DependencyKind::Normal),
        Some("dev") => Some(DependencyKind::Dev),
        Some("build") => Some(DependencyKind::Build),
        Some(_other) => None,
    };
}

fn Is_Dependency_Optional(dependency: &serde_json::Value) -> bool
{
    return dependency
        .get("optional")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
}

/// Refuses an empty result: `cargo metadata` reporting no workspace members means this
/// reader saw nothing, not that the workspace's own graph is empty.
fn Require_Nonempty(discovered: Vec<DiscoveredPackage>) -> Result<Vec<DiscoveredPackage>, MetadataError>
{
    if discovered.is_empty()
    {
        return Err(MetadataError {
            reason: "cargo metadata reported no workspace members; refusing to report a \
                     clean result over an empty graph"
                .to_owned(),
        });
    }

    return Ok(discovered);
}

fn Is_Member(package: &serde_json::Value, members: &BTreeSet<String>) -> bool
{
    return package
        .get("id")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|id| members.contains(id));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_platform_std::StdProcessLauncher;

    #[test]
    fn Test_Discover_Workspace_Should_Find_This_Crates_Real_Dependency_On_Nomos_Contracts()
    {
        let discovered = Discover_Workspace(&Repository_Root(), &StdProcessLauncher).expect("a real cargo workspace");

        let this_crate = discovered
            .iter()
            .find(|package| package.payload.package == "nomos-lang-rust-cargo")
            .expect("this crate is itself a workspace member");

        assert!(
            this_crate
                .payload
                .edges
                .iter()
                .any(|edge| edge.target == "nomos-contracts" && edge.kind == DependencyKind::Normal),
            "got {:?}",
            this_crate.payload.edges
        );
    }

    #[test]
    fn Test_Nomos_Contracts_Should_Have_No_First_Party_Edges()
    {
        let discovered = Discover_Workspace(&Repository_Root(), &StdProcessLauncher).expect("a real cargo workspace");

        let contracts = discovered
            .iter()
            .find(|package| package.payload.package == "nomos-contracts")
            .expect("nomos-contracts is a workspace member");

        assert!(
            contracts.payload.edges.is_empty(),
            "nomos-contracts must depend on nothing else in this workspace: {:?}",
            contracts.payload.edges
        );
    }

    #[test]
    fn Test_Every_Discovered_Package_Should_Carry_A_Manifest_Relative_Root()
    {
        let discovered = Discover_Workspace(&Repository_Root(), &StdProcessLauncher).expect("a real cargo workspace");

        let this_crate = discovered
            .iter()
            .find(|package| package.payload.package == "nomos-lang-rust-cargo")
            .expect("this crate is itself a workspace member");

        assert_eq!(
            this_crate.manifest_relative_root,
            "crates/languages/nomos-lang-rust-cargo"
        );
    }

    #[test]
    fn Test_Edges_Should_Be_In_Canonical_Order()
    {
        let discovered = Discover_Workspace(&Repository_Root(), &StdProcessLauncher).expect("a real cargo workspace");

        for package in &discovered
        {
            let mut sorted = package.payload.edges.clone();
            sorted.sort_by(|left, right| (&left.target, left.kind.Label(), left.optional).cmp(&(&right.target, right.kind.Label(), right.optional)));
            assert_eq!(package.payload.edges, sorted, "{}", package.payload.package);
        }
    }

    /// Run over this workspace's own real root, the same standard `tests/contract`
    /// already holds this exact invocation to: a boundary reader that cannot be checked
    /// against a real graph is checked against nothing.
    fn Repository_Root() -> PathBuf
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .map(PathBuf::from)
            .expect("this crate sits three levels below the workspace root");
    }
}

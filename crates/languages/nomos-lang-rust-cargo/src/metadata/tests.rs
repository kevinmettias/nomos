//! What this module promises, exercised -- a real `cargo metadata` run over this
//! workspace, read back through the same reader the composition root calls.

use super::*;
use nomos_platform_std::{StdEnvironment, StdProgramLauncher};

#[test]
fn Test_Discover_Workspace_Should_Find_This_Crates_Real_Dependency_On_Nomos_Contracts()
{
    let discovered = Discover_Workspace(&Repository_Root(), &StdProgramLauncher, &StdEnvironment).expect("a real cargo workspace");

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
    let discovered = Discover_Workspace(&Repository_Root(), &StdProgramLauncher, &StdEnvironment).expect("a real cargo workspace");

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
    let discovered = Discover_Workspace(&Repository_Root(), &StdProgramLauncher, &StdEnvironment).expect("a real cargo workspace");

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
    let discovered = Discover_Workspace(&Repository_Root(), &StdProgramLauncher, &StdEnvironment).expect("a real cargo workspace");

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
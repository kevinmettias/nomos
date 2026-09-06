//! What the dependency graph is allowed to do, and the vacuity guard in front of it.

use crate::bands::{Permits, Zone_Of, SAME_ZONE_EDGES, ZONES};
use nomos_contract_tests::Workspace;

/// Everything `nomos-contracts` is permitted to reach, transitively.
///
/// `serde_core` and `serde_derive` are serde's own decomposition rather than choices
/// made here.
const CONTRACTS_ALLOWLIST: &[&str] = &["serde", "serde_core", "serde_derive"];

/// The only crate permitted to name the sibling platform workspace.
///
/// It does not exist yet. Naming it here now means that when it arrives, the exception
/// is already a decision somebody wrote down rather than a line added to make a failing
/// test pass.
const PLATFORM_ADAPTER: &[&str] = &["nomos-platform-xvpe"];

/// The crates permitted to name the sibling knowledge workspace.
///
/// Empty: no such adapter exists yet, and unlike the platform crossing this repository
/// has not even provisionally named one. When one arrives it belongs here explicitly, the
/// same way `PLATFORM_ADAPTER` already names `nomos-platform-xvpe` before that crate
/// exists — AGT-006 names this as the one enforced crossing missing its `kwb-` twin.
const KNOWLEDGE_ADAPTER: &[&str] = &[];

/// Guards every other test in this suite against passing vacuously.
///
/// If package-id parsing breaks — cargo has changed that format more than once — the
/// member set comes back empty, every loop below iterates zero times, and all five
/// assertions report a clean result over nothing. The sibling workspace hit exactly
/// this: `check-standards-tree /nonexistent` walked nothing, found nothing, and
/// reported CLEAN, and the same defect was later found in three other checks.
///
/// A check that cannot find its subject must fail loudly, not quietly verify nothing.
#[test]
fn Test_The_Workspace_Should_Not_Appear_Empty()
{
    let workspace = Workspace::Load();
    let members = workspace.Members();

    assert!(
        members.len() >= ZONES.len(),
        "found {} workspace members but {} zones are declared: {:?}.\n\
         Every other assertion in this suite iterates over these members, so an empty or \
         truncated set makes all of them pass having checked nothing.",
        members.len(),
        ZONES.len(),
        members.iter().map(|member| &member.name).collect::<Vec<_>>()
    );

    assert!(
        workspace.Get("nomos-contracts").is_some(),
        "nomos-contracts must be visible in the graph; the allowlist assertion is \
         meaningless without it"
    );
}

/// The load-bearing one.
///
/// Every type in `nomos-contracts` is reimplemented by systems that will never compile
/// this crate — a knowledge service in another language, a client in TypeScript, a
/// platform in another workspace. A dependency here makes the protocol Nomos-shaped and
/// forces those peers to vendor a Rust crate in order to agree with us.
///
/// `serde` is the deliberate exception: the artifact a peer actually reads is the JSON
/// Schema generated from these declarations, and the neutrality that matters is that no
/// *Nomos* and no *platform* type appears in the protocol.
#[test]
fn Test_Contracts_Should_Depend_On_The_Allowlist_And_Nothing_Else()
{
    use std::collections::BTreeSet;

    let workspace = Workspace::Load();
    let allowed: BTreeSet<String> = CONTRACTS_ALLOWLIST
        .iter()
        .map(|name| (*name).to_owned())
        .collect();

    let actual = workspace.Transitive_Dependencies("nomos-contracts");
    let unexpected: Vec<&String> = actual.difference(&allowed).collect();

    assert!(
        unexpected.is_empty(),
        "nomos-contracts grew a dependency: {unexpected:?}.\n\
         Every type in that crate is copied into implementations that have never seen \
         this repository. A dependency here makes the protocol Nomos-shaped and forces a \
         peer to vendor a Rust crate in order to speak it."
    );
}

/// Nothing below the host band may reach the sibling platform workspace.
///
/// When `nomos-platform-xvpe` exists it will be the single exception, and it will be
/// named here explicitly so that the exception is a decision rather than an oversight.
#[test]
fn Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace()
{
    let workspace = Workspace::Load();

    for member in workspace.Members()
    {
        if PLATFORM_ADAPTER.contains(&member.name.as_str())
        {
            continue;
        }

        let leaked: Vec<String> = workspace
            .Transitive_Dependencies(&member.name)
            .into_iter()
            .filter(|dependency| dependency.starts_with("xvpe-"))
            .collect();

        assert!(
            leaked.is_empty(),
            "{} reaches {leaked:?}.\n\
             The sibling workspace is a downward implementation dependency behind the \
             platform port, not something the domain may name directly.",
            member.name
        );
    }
}

/// Nothing may depend on the sibling knowledge workspace at runtime.
///
/// AGT-006 requires neither system depend on the other for its core function; this is
/// the mechanical half of that for the KWB crossing, the twin of
/// `Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` above. There is no
/// adapter exception yet because `KNOWLEDGE_ADAPTER` is empty — every crate is checked.
#[test]
fn Test_No_Crate_May_Name_The_Sibling_Knowledge_Workbench()
{
    let workspace = Workspace::Load();

    for member in workspace.Members()
    {
        if KNOWLEDGE_ADAPTER.contains(&member.name.as_str())
        {
            continue;
        }

        let leaked: Vec<String> = workspace
            .Transitive_Dependencies(&member.name)
            .into_iter()
            .filter(|dependency| dependency.starts_with("kwb-"))
            .collect();

        assert!(
            leaked.is_empty(),
            "{} reaches {leaked:?}.\n\
             The sibling knowledge workspace is KWB's, not something Nomos may depend on \
             at runtime for its core function (AGT-006); integration crosses through \
             neutral versioned contracts instead.",
            member.name
        );
    }
}

/// A crate may depend only on a zone its own zone permits, or a same-zone peer named in
/// `SAME_ZONE_EDGES`.
///
/// Cargo already forbids cycles. This forbids the legal-but-wrong edges: a kernel crate
/// reaching up into a service, a transport reaching past the service layer into
/// analysis. Those compile perfectly and dissolve the architecture. `OD-RULES-020`
/// replaced the numeric band comparison this assertion used to make with the identical
/// zone-and-named-edge judgment `nomos-rules`' own `Check_Dependency_Direction` makes over
/// a real `nomos check` run, so a forbidden edge is refused in both places from the one
/// declaration.
#[test]
fn Test_Dependencies_Should_Run_Strictly_Downward()
{
    let workspace = Workspace::Load();

    for member in workspace.Members()
    {
        let Some(zone) = Zone_Of(&member.name)
        else
        {
            continue;
        };

        for dependency in &member.direct_dependencies
        {
            let Some(dependency_zone) = Zone_Of(dependency)
            else
            {
                continue;
            };

            let permitted = if zone == dependency_zone
            {
                SAME_ZONE_EDGES.contains(&(member.name.as_str(), dependency.as_str()))
            }
            else
            {
                Permits(zone, dependency_zone)
            };

            assert!(
                permitted,
                "{} ({zone:?}) depends on {dependency} ({dependency_zone:?}).\n\
                 This edge is neither a permitted zone crossing nor a named same-zone \
                 exception; an unchecked edge like this one is how a layered architecture \
                 becomes a graph nobody can reason about.",
                member.name
            );
        }
    }
}

/// Every `SAME_ZONE_EDGES` pair is a real, direct dependency — a declared exception with
/// nothing behind it is worse than no exception, since it would permit an edge nobody's
/// code actually draws.
#[test]
fn Test_Every_Same_Zone_Edge_Should_Be_A_Real_Dependency()
{
    let workspace = Workspace::Load();

    for (from, to) in SAME_ZONE_EDGES
    {
        let Some(member) = workspace.Get(from)
        else
        {
            panic!("{from} names no real workspace member");
        };

        assert!(
            member.direct_dependencies.contains(*to),
            "SAME_ZONE_EDGES names {from} -> {to}, but {from}'s own Cargo.toml declares no \
             such dependency. A named exception with nothing behind it permits an edge \
             nobody's code actually draws."
        );
    }
}

/// A crate cannot join the workspace without declaring where it sits.
#[test]
fn Test_Every_Member_Should_Declare_A_Band()
{
    let workspace = Workspace::Load();

    let undeclared: Vec<&str> = workspace
        .Members()
        .iter()
        .map(|member| member.name.as_str())
        .filter(|name| Zone_Of(name).is_none())
        .collect();

    assert!(
        undeclared.is_empty(),
        "these crates declare no zone: {undeclared:?}.\n\
         Add them to nomos-rules' own ZONES. A crate outside the declared architecture is \
         a crate the architecture does not constrain."
    );
}

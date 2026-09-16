//! What the dependency graph is allowed to do, and the vacuity guard in front of it.

use crate::bands::Declared_Architecture;
use nomos_cap_architecture::{Depended, Depending};
use nomos_contract_tests::Workspace;

/// Everything `nomos-contracts` is permitted to reach, transitively.
///
/// `serde_core` and `serde_derive` are serde's own decomposition rather than choices
/// made here.
const CONTRACTS_ALLOWLIST: &[&str] = &["serde", "serde_core", "serde_derive"];

/// The crates permitted to name the sibling knowledge workspace.
///
/// Empty: no such adapter exists, and this repository has never provisionally named one.
/// When one arrives it belongs here explicitly, so the exception is a decision somebody
/// wrote down rather than a line added to make a failing test pass.
///
/// The platform crossing once had this same shape. It no longer does — nomos is built on
/// top of XVPE, so naming `xvpe-` is ordinary rather than an exception. That says nothing
/// about this crossing, which remains closed.
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
        members.len() >= Declared_Architecture().membership.len(),
        "found {} workspace members but {} zones are declared: {:?}.\n\
         Every other assertion in this suite iterates over these members, so an empty or \
         truncated set makes all of them pass having checked nothing.",
        members.len(),
        Declared_Architecture().membership.len(),
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

// `Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` was here, and is
// retired as of 2026-09-10 by the owner's decision.
//
// It forbade any crate but a single named adapter from transitively reaching an `xvpe-`
// dependency, on AGT-006's premise that neither system depends on the other. That premise
// is no longer this project's: **nomos is built on top of XVPE**, as the knowledge
// workbench is. XVPE is the engine; this workspace is an application over it. A rule
// forbidding that dependency does not describe an architecture worth keeping.
//
// Retired rather than widened, deliberately. Adding nine crates to an allow-list would
// have left a rule that still reads as a boundary while enforcing nothing, which is worse
// than no rule: the next reader would take it for a constraint that holds.
//
// The knowledge crossing below is a different question and is untouched. Nothing here says
// this workspace may name `kwb-`; only that it may name `xvpe-`.

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
    let architecture = Declared_Architecture();

    for member in workspace.Members()
    {
        let Some(component) = architecture.Component_Of(&member.name)
        else
        {
            continue;
        };

        for dependency in &member.direct_dependencies
        {
            let Some(dependency_component) = architecture.Component_Of(dependency)
            else
            {
                continue;
            };

            let permitted = if component == dependency_component
            {
                architecture.Excepts(Depending(&member.name), Depended(dependency))
            }
            else
            {
                architecture.Permits(Depending(component), Depended(dependency_component))
            };

            assert!(
                permitted,
                "{} ({component}) depends on {dependency} ({dependency_component}).\n\
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

    for exception in &Declared_Architecture().exceptions
    {
        let (from, to) = (exception.from.as_str(), exception.to.as_str());
        let Some(member) = workspace.Get(from)
        else
        {
            panic!("{from} names no real workspace member");
        };

        assert!(
            member.direct_dependencies.contains(to),
            "nomos-architecture.json excepts {from} -> {to}, but {from}'s own Cargo.toml declares no \
             such dependency. A named exception with nothing behind it permits an edge \
             nobody's code actually draws."
        );
    }
}

/// Every declared exception names two packages the declaration places in one component.
///
/// The name is the one `OD-RULES-028` cites, kept deliberately through the move out of
/// `nomos-rules`: the property is unchanged and a citation that stopped resolving would be a
/// worse thing to leave behind than a name that says "zone" where the declaration now says
/// "component". An exception exists to lift the peer refusal *within* one component, so a pair
/// straddling two components is not an exception to anything -- the declaration's own
/// permissions already answer it -- and a pair naming a package the declaration does not place
/// is an exception to a rule that never applied.
#[test]
fn Test_Same_Zone_Edges_Should_Each_Name_Two_Members_Of_The_Same_Zone()
{
    let architecture = Declared_Architecture();

    for exception in &architecture.exceptions
    {
        let from = architecture
            .Component_Of(&exception.from)
            .unwrap_or_else(|| panic!("{} is excepted and placed in no component", exception.from));
        let to = architecture
            .Component_Of(&exception.to)
            .unwrap_or_else(|| panic!("{} is excepted and placed in no component", exception.to));

        assert_eq!(
            from, to,
            "{} -> {}: an exception must name two members of one component, and these are in {from} and {to}",
            exception.from, exception.to
        );
    }
}

/// `OD-RULES-023`'s own write-authority table names only doors that are real dependencies.
#[test]
fn Test_Every_Write_Door_Should_Be_A_Real_Dependency()
{
    let workspace = Workspace::Load();

    for declared in &Declared_Architecture().authorities
    {
        let (authority, doors) = (declared.package.as_str(), declared.doors.as_slice());
        for door in doors
        {
            let door = door.as_str();
            let Some(member) = workspace.Get(door)
            else
            {
                panic!("{door} names no real workspace member");
            };

            assert!(
                member.direct_dependencies.contains(authority),
                "nomos-architecture.json names {door} as a door into {authority}, but {door}'s own \
                 Cargo.toml declares no such dependency. A named door with nothing behind \
                 it permits an edge nobody's code actually draws."
            );
        }
    }
}

/// A crate cannot join the workspace without declaring where it sits.
#[test]
fn Test_Every_Member_Should_Declare_A_Band()
{
    let workspace = Workspace::Load();
    let architecture = Declared_Architecture();

    let undeclared: Vec<&str> = workspace
        .Members()
        .iter()
        .map(|member| member.name.as_str())
        .filter(|name| architecture.Component_Of(name).is_none())
        .collect();

    assert!(
        undeclared.is_empty(),
        "these crates declare no zone: {undeclared:?}.\n\
         Place them in nomos-architecture.json's own members. A crate outside the declared \
         architecture is \
         a crate the architecture does not constrain."
    );
}

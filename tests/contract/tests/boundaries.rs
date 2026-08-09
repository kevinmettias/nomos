//! The boundary assertions. Each one is a property the architecture claims, made
//! checkable.

use nomos_contract_tests::Workspace;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The declared band of every workspace member.
///
/// Authored here rather than derived, because a band is a design decision and there is
/// nothing in the source to infer it from. Adding a crate without adding it here fails
/// [`Test_Every_Member_Should_Declare_A_Band`] — so a new crate cannot quietly join the
/// workspace outside the ordering.
///
/// Bands are scaled by ten so that a band can be inserted between two others without
/// renumbering everything, which is the sort of churn that turns a table nobody wants
/// to touch into a table nobody updates.
const BANDS: &[(&str, u32)] = &[
    ("nomos-contracts", 0),
    ("nomos-model", 10),
    ("nomos-store", 12),
    // The port and its implementations are not peers. The traits sit below, so that
    // swapping an implementation cannot recompile anything that only knows the port —
    // which is the entire reason the seam exists.
    ("nomos-platform", 15),
    ("nomos-platform-std", 16),
    // What the workspace currently is: above the document store it records into, below
    // everything that keys a fact on a snapshot or a build variant.
    ("nomos-workspace", 18),
    ("nomos-ledger", 20),
    // The product substrate. Capability sits above the model and below everything
    // that resolves a provider through it.
    ("nomos-capability", 21),
    ("nomos-analysis", 22),
    // Language providers sit above analysis because they produce the facts it stores,
    // and nothing sits above them but a composition root. They reach each other not at
    // all: two languages are two providers of one capability, and the registry is the
    // only thing that knows both.
    ("nomos-lang-rust", 25),
    // Its peer, deliberately at the same band. Two providers of one capability must not
    // be able to name each other: this file's downward rule forbids an edge between
    // crates at one band, which is what stops the second answer from being derived from
    // the first. Two providers that shared a parser could not disagree.
    ("nomos-lang-rust-scan", 25),
    // The spec system sits beside the kernel, not above it. It reaches the product only
    // through a KnowledgeCapability, so nothing in the product may name it directly.
    ("nomos-spec-model", 11),
    ("nomos-spec-store", 12),
    // Bundle and ingest are peers over the store: one reads the corpus in, the other
    // writes it back out. Neither may name the other.
    ("nomos-spec-bundle", 13),
    ("nomos-spec-ingest", 13),
    ("nomos-spec-validate", 14),
    ("nomos-spec-project", 14),
    ("nomos-cli", 90),
    // The contract tests sit at the top: they observe the workspace and nothing
    // observes them.
    ("nomos-contract-tests", 100),
    // The vertical slice is their peer, not their superior. Both are terminal, and
    // sharing a band is what makes them unable to name each other: this file's
    // downward rule forbids an edge between two crates at the same band. That is the
    // property wanted — an observer of the workspace that also participates in it
    // could no longer be trusted to report on it.
    ("nomos-integration-tests", 100),
];

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

fn Declared_Band(name: &str) -> Option<u32>
{
    return BANDS
        .iter()
        .find(|(crate_name, _)| *crate_name == name)
        .map(|(_, band)| *band);
}

/// Guards every other test in this file against passing vacuously.
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
        members.len() >= BANDS.len(),
        "found {} workspace members but {} bands are declared: {:?}.\n\
         Every other assertion in this file iterates over these members, so an empty or \
         truncated set makes all of them pass having checked nothing.",
        members.len(),
        BANDS.len(),
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

/// A crate may depend only on crates in a strictly lower band.
///
/// Cargo already forbids cycles. This forbids the legal-but-wrong edges: a kernel crate
/// reaching up into a service, a transport reaching past the service layer into
/// analysis. Those compile perfectly and dissolve the architecture.
#[test]
fn Test_Dependencies_Should_Run_Strictly_Downward()
{
    let workspace = Workspace::Load();

    for member in workspace.Members()
    {
        let Some(band) = Declared_Band(&member.name)
        else
        {
            continue;
        };

        for dependency in &member.direct_dependencies
        {
            let Some(dependency_band) = Declared_Band(dependency)
            else
            {
                continue;
            };

            assert!(
                dependency_band < band,
                "{} (band {band}) depends on {dependency} (band {dependency_band}).\n\
                 Dependencies run strictly downward; equal or upward edges are how a \
                 layered architecture becomes a graph nobody can reason about.",
                member.name
            );
        }
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
        .filter(|name| Declared_Band(name).is_none())
        .collect();

    assert!(
        undeclared.is_empty(),
        "these crates declare no band: {undeclared:?}.\n\
         Add them to BANDS in this file. A crate outside the ordering is a crate the \
         ordering does not constrain."
    );
}

/// Every `.rs` file under a crate's `src/` must be reachable from its root by following
/// `mod` declarations.
///
/// Adopted from the sibling workspace, where it found 95 orphaned files across seven
/// crates — including a complete, documented, four-test module that had never once
/// compiled, and a test file that had never run a single assertion.
///
/// The compiler says nothing about a file no `mod` names. It never parses it. So the
/// file does not type-check, its tests do not run, its lints do not fire and its denies
/// do not apply — while reading, to every human, as finished work. An orphan is strictly
/// worse than a missing file: a missing file is an absence, and an orphan is a false
/// claim of coverage.
#[test]
fn Test_Every_Source_File_Should_Be_Reachable()
{
    let workspace = Workspace::Load();
    let mut orphans = Vec::new();

    for member in workspace.Members()
    {
        let source_root = member.root.join("src");
        if !source_root.is_dir()
        {
            continue;
        }

        let declared = Declared_Modules(&source_root);

        for file in Source_Files(&source_root)
        {
            let Some(stem) = file.file_stem().and_then(|stem| stem.to_str())
            else
            {
                continue;
            };

            // Crate roots and module roots are reached by cargo and by their parent
            // directory's declaration respectively, not by a `mod` naming their stem.
            if matches!(stem, "lib" | "main" | "mod")
            {
                continue;
            }

            if !declared.contains(stem)
            {
                orphans.push(
                    file.strip_prefix(&member.root)
                        .unwrap_or(&file)
                        .display()
                        .to_string(),
                );
            }
        }
    }

    assert!(
        orphans.is_empty(),
        "these files are not reachable from any crate root: {orphans:#?}.\n\
         rustc never parses an undeclared file, so its tests do not run and its lints do \
         not fire, while it reads as finished work. Declare it or delete it."
    );
}

/// Every `mod` name declared anywhere under a source root.
///
/// Deliberately a flat set rather than a resolved tree. The precise version would walk
/// declarations from each root, and would need to handle `#[path]`, `#[cfg]` and inline
/// modules to avoid false positives. This approximation cannot report a false orphan —
/// it only misses the case where a module is declared in one place and the file lives in
/// an unrelated one, which is a naming problem rather than an invisibility problem.
fn Declared_Modules(source_root: &Path) -> BTreeSet<String>
{
    let mut declared = BTreeSet::new();

    for file in Source_Files(source_root)
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };

        for line in text.lines()
        {
            let trimmed = line.trim();
            // A commented-out declaration is not a declaration. The sibling workspace
            // names this specifically as a decoy that made an orphan look declared.
            if trimmed.starts_with("//")
            {
                continue;
            }

            let Some(rest) = trimmed
                .strip_prefix("mod ")
                .or_else(|| trimmed.strip_prefix("pub mod "))
                .or_else(|| trimmed.strip_prefix("pub(crate) mod "))
            else
            {
                continue;
            };

            // `mod foo;` is a declaration of a file. `mod foo {` is an inline module,
            // which declares nothing on disk.
            if let Some(name) = rest.strip_suffix(';')
            {
                declared.insert(name.trim().to_owned());
            }
        }
    }

    return declared;
}

/// Every `.rs` file under a directory, recursively.
fn Source_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };

        for entry in entries.flatten()
        {
            let path = entry.path();
            if path.is_dir()
            {
                pending.push(path);
            }
            else if path.extension().is_some_and(|extension| extension == "rs")
            {
                found.push(path);
            }
        }
    }

    return found;
}

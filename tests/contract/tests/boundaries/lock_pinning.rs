//! The committed lock still pins the XVPE crossing, and a local override cannot publish a
//! lock that does not.
//!
//! `D-130`'s surviving clause requires this crossing be adopted by git reference and
//! commit. The manifests say so, and `Cargo.lock` is what turns that from a statement into
//! a recorded fact: it names the revision every build resolved. `OD-PLATFORM-004` permits a
//! local patch override redirecting those crates to a sibling checkout, and names the debt
//! this module pays.
//!
//! # Why a lock needs its own guard
//!
//! Measured 2026-09-14: with such an override active, one `cargo metadata` call rewrote the
//! tracked `Cargo.lock` and deleted the `source` line from all twelve `xvpe-` packages,
//! reporting nothing. Every other clause of `OD-PLATFORM-004` fails safely -- a wrong
//! override breaks a local build and its author finds out. This one fails silently, in the
//! committed state, in the direction the pinning exists to prevent. `Cargo.lock` is tracked
//! on purpose, so `.gitignore` cannot protect it the way it protects the override file, and
//! the scoped `git add` `AGENTS.md` asks for everywhere else is the gesture that would
//! publish the damage.
//!
//! # Why this reads manifest text rather than the resolved graph
//!
//! Every other module here reads `cargo metadata`, deliberately, because a resolved graph
//! cannot be fooled by a dependency hidden under a feature or a target. Here that reasoning
//! inverts: the resolved graph is exactly what an override rewrites, so a check reading it
//! would agree with the override and report the tree clean. The manifests are what the
//! override does not touch, so they are the authority for what the revision is supposed to
//! be, and the lock is compared against them.
//!
//! The expected revision is therefore never written down here. A legitimate bump edits the
//! manifests and the lock together and this module follows it; a hardcoded revision would
//! fail on every bump and teach the next reader to edit the test rather than look.

use crate::bands::Repository_Root;
use std::collections::BTreeSet;

/// The repository the XVPE crossing is adopted from.
const XVPE_GIT_URL: &str = "https://github.com/kevinmettias/xvpe.git";

/// The prefix naming a package that belongs to that crossing.
const XVPE_PREFIX: &str = "xvpe-";

/// The value of `rev = "..."` in `line`, if it carries one.
fn Revision_In(line: &str) -> Option<String>
{
    const KEY: &str = "rev = \"";

    let from = line.find(KEY)?.checked_add(KEY.len())?;
    let rest = line.get(from..)?;
    let to = rest.find('\"')?;

    return Some(rest.get(..to)?.to_string());
}

/// Every revision this workspace's own manifests pin the crossing to.
///
/// Read as text. This module's own documentation says why the resolved graph is the wrong
/// authority for this one question.
fn Declared_Revisions() -> BTreeSet<String>
{
    use nomos_contract_tests::Workspace;

    let workspace = Workspace::Load();
    let mut found = BTreeSet::new();

    for member in workspace.Members()
    {
        let manifest = member.root.join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest)
        else
        {
            continue;
        };

        for line in text.lines()
        {
            if line.contains(XVPE_GIT_URL)
            {
                if let Some(revision) = Revision_In(line)
                {
                    found.insert(revision);
                }
            }
        }
    }

    return found;
}

/// Every `xvpe-` package in the committed lock, paired with its `source` value if it has one.
fn Locked_Crossing_Packages() -> Vec<(String, Option<String>)>
{
    const NAME_KEY: &str = "name = \"";
    const SOURCE_KEY: &str = "source = \"";

    let path = Repository_Root().join("Cargo.lock");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    let mut found = Vec::new();

    for block in text.split("[[package]]")
    {
        let mut name: Option<String> = None;
        let mut source: Option<String> = None;

        for line in block.lines()
        {
            let trimmed = line.trim();

            if let Some(rest) = trimmed.strip_prefix(NAME_KEY)
            {
                name = rest.strip_suffix('\"').map(str::to_string);
            }
            else if let Some(rest) = trimmed.strip_prefix(SOURCE_KEY)
            {
                source = rest.strip_suffix('\"').map(str::to_string);
            }
        }

        if let Some(name) = name
        {
            if name.starts_with(XVPE_PREFIX)
            {
                found.push((name, source));
            }
        }
    }

    return found;
}

/// The manifests agree on one revision for the crossing.
///
/// Two revisions would mean one workspace adopting two snapshots of one engine, which is a
/// defect on its own and would also leave the comparison below without a single answer to
/// check against.
#[test]
fn Test_Every_Manifest_Should_Pin_The_Crossing_To_One_Revision()
{
    let declared = Declared_Revisions();

    assert!(
        !declared.is_empty(),
        "no manifest pins the XVPE crossing to a git revision.\n\
         Either this workspace stopped depending on XVPE -- in which case this module and \
         D-130's surviving clause both need revisiting -- or a dependency was respelled as \
         a path, which is the form D-130 refuses."
    );

    assert!(
        declared.len() == 1,
        "the XVPE crossing is pinned to {} different revisions: {declared:?}.\n\
         One workspace adopting two snapshots of one engine is a defect by itself, and it \
         leaves no single revision for the committed lock to be checked against.",
        declared.len()
    );
}

/// The committed lock names that revision for every crate of the crossing.
///
/// This is the assertion an active local patch override defeats, silently, by deleting the
/// `source` line the moment any cargo command runs.
#[test]
fn Test_The_Committed_Lock_Should_Pin_Every_Crossing_Package_To_That_Revision()
{
    let declared = Declared_Revisions();
    let Some(revision) = declared.iter().next()
    else
    {
        panic!(
            "no declared revision. \
             Test_Every_Manifest_Should_Pin_The_Crossing_To_One_Revision says why."
        );
    };

    let locked = Locked_Crossing_Packages();

    assert!(
        !locked.is_empty(),
        "Cargo.lock names no {XVPE_PREFIX} package at all.\n\
         This assertion would pass over an empty set while proving nothing, which is the \
         failure mode this workspace's own records are about. If the crossing really is \
         gone, remove this module rather than let it report a clean result over nothing."
    );

    let mut unpinned = Vec::new();

    for (name, source) in &locked
    {
        let pinned = source
            .as_deref()
            .is_some_and(|source| source.starts_with("git+") && source.contains(revision));

        if !pinned
        {
            let seen = source.as_deref().unwrap_or("no source, so a local path");
            unpinned.push(format!("{name} -> {seen}"));
        }
    }

    unpinned.sort();

    assert!(
        unpinned.is_empty(),
        "{} of {} packages in the XVPE crossing are not pinned to {revision} in the \
         committed lock:\n  {}\n\n\
         A package with no source at all resolves from a local path. The usual cause is not \
         an edited manifest: it is a local Cargo patch override, which rewrites Cargo.lock \
         on any cargo command and reports nothing. OD-PLATFORM-004 records that this is the \
         one clause of it that fails silently and in the committed direction.\n\n\
         To restore, park the override and check the lock out again:\n  \
         mv .cargo/config.toml <somewhere> && git checkout -- Cargo.lock",
        unpinned.len(),
        locked.len(),
        unpinned.join("\n  ")
    );
}

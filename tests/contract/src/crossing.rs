//! The XVPE crossing's declared revision, read once for every guard that asks.
//!
//! Two test targets need this fact and they need it for different reasons.
//! `boundaries::lock_pinning` asks whether the manifests agree with each other and with the
//! committed lock; `crossing_cold` asks whether a cold build actually resolved the crates from
//! the revision the manifests name. Before this module each carried its own scan, and on
//! 2026-09-21 that cost exactly what `OD-GATE-011` says a duplicated answer costs: `e0197368`
//! moved the declaration into the root manifest's `[workspace.dependencies]` table and taught
//! one of the two readers to look there, leaving the other scanning
//! `crates/platform/nomos-platform-xvpe/Cargo.toml` for a `rev = "` that inheritance no longer
//! spells. That reader panics rather than failing an assertion, and it is `#[ignore]`d, so an
//! ordinary `cargo test` never reached it and only the gate's own `--ignored` step did.
//!
//! So this is the authority both derive from, named outside either, which is the arrangement
//! `OD-GATE-011` asks for and `OD-COMPLETENESS-001` explains: a guard whose universe is
//! narrower than the claim beside it does not fail, it stops noticing.
//!
//! # What counts as a declaring site
//!
//! The workspace root, because `[workspace.dependencies]` is where the crossing is declared,
//! and every member, because inheriting with `workspace = true` is a convention rather than a
//! constraint and a member may spell its own `git` and `rev` at any time. A member that does is
//! a second snapshot of one engine, which is precisely what the guards refuse — so the set is
//! derived rather than listed, and neither reader chooses it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The repository the XVPE crossing is adopted from.
pub const XVPE_GIT_URL: &str = "https://github.com/kevinmettias/xvpe.git";

/// The prefix naming a package that belongs to that crossing.
pub const XVPE_PREFIX: &str = "xvpe-";

/// The workspace root, from the contract crate's own manifest directory.
#[must_use]
pub fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
}

/// The value of `rev = "..."` in `line`, if it carries one.
fn Revision_In(line: &str) -> Option<String>
{
    const KEY: &str = "rev = \"";

    let from = line.find(KEY)?.checked_add(KEY.len())?;
    let rest = line.get(from..)?;
    let to = rest.find('\"')?;

    return Some(rest.get(..to)?.to_string());
}

/// Every revision `text` pins the crossing to.
///
/// Over text rather than over a path, so a disagreement can be constructed. A tree that
/// satisfies the rule cannot demonstrate that the rule has teeth.
#[must_use]
pub fn Revisions_In(text: &str) -> BTreeSet<String>
{
    let mut found = BTreeSet::new();

    for line in text.lines()
    {
        // A guard clause rather than a nested `if`: the two conditions one inside the other
        // put this at four levels of control flow, which `nesting-depth` reports, and
        // flattening with an early `continue` is the first remedy it names.
        if !line.contains(XVPE_GIT_URL)
        {
            continue;
        }

        if let Some(revision) = Revision_In(line)
        {
            found.insert(revision);
        }
    }

    return found;
}

/// Every manifest that can pin the crossing: the workspace root first, then every member.
///
/// # Panics
///
/// Panics if the workspace cannot be loaded. A guard that cannot see its subject must fail
/// loudly rather than quietly quantify over nothing.
#[must_use]
pub fn Pinning_Manifests() -> Vec<PathBuf>
{
    use crate::Workspace;

    let mut manifests = vec![Repository_Root().join("Cargo.toml")];

    for member in Workspace::Load().Members()
    {
        manifests.push(member.root.join("Cargo.toml"));
    }

    return manifests;
}

/// Every revision this workspace's own manifests pin the crossing to.
#[must_use]
pub fn Declared_Revisions() -> BTreeSet<String>
{
    let mut found = BTreeSet::new();

    for manifest in Pinning_Manifests()
    {
        let Ok(text) = std::fs::read_to_string(&manifest)
        else
        {
            continue;
        };

        found.extend(Revisions_In(&text));
    }

    return found;
}

/// The one revision this workspace adopts the crossing at.
///
/// # Panics
///
/// Panics when the manifests name none, or name more than one. A caller asking for *the*
/// revision has no sensible answer in either case, and both are defects this repository's
/// guards exist to report rather than conditions to tolerate: none means the crossing was
/// respelled as a path, which `D-130` refuses, or that this workspace stopped depending on
/// XVPE; more than one means a workspace adopting two snapshots of one engine.
#[must_use]
pub fn Declared_Revision() -> String
{
    let declared = Declared_Revisions();

    assert!(
        !declared.is_empty(),
        "no manifest declares a revision for the XVPE crossing. The declaring site is the root \
         manifest's [workspace.dependencies] table, which every member inherits from with \
         `workspace = true`; a reader that finds nothing there is reading the wrong file or the \
         crossing was respelled as a path, which is the form D-130 refuses."
    );

    assert!(
        declared.len() == 1,
        "the XVPE crossing is declared at {} different revisions: {declared:?}. One workspace \
         adopting two snapshots of one engine is a defect by itself, and it leaves no single \
         revision for a build or a lock to be checked against.",
        declared.len()
    );

    return declared.into_iter().next().unwrap_or_default();
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A member that re-pins is seen as a second revision, and one that inherits adds none.
    #[test]
    fn Test_A_Member_Re_Pinning_The_Crossing_Should_Be_Seen_As_A_Second_Revision()
    {
        let root = "[workspace.dependencies]\n\
                    xvpe-primitives = { git = \"https://github.com/kevinmettias/xvpe.git\", rev = \"82a3c8fccf4ef7f3759f36d3f320a91d0f96341c\" }\n";
        let inheriting = "[dependencies]\nxvpe-primitives = { workspace = true }\n";
        let re_pinning = "[dependencies]\n\
                          xvpe-primitives = { git = \"https://github.com/kevinmettias/xvpe.git\", rev = \"0000000000000000000000000000000000000000\" }\n";

        let mut inherited = Revisions_In(root);
        inherited.extend(Revisions_In(inheriting));

        assert!(
            inherited.len() == 1,
            "a member inheriting the crossing added a revision of its own: {inherited:?}. \
             Inheritance carries no `rev`, so the declaring site must be the only answer."
        );

        let mut disagreeing = Revisions_In(root);
        disagreeing.extend(Revisions_In(re_pinning));

        assert!(
            disagreeing.len() == 2,
            "a member that re-pinned the crossing to its own revision was not seen: \
             {disagreeing:?}. Every guard built on this would then pass over a workspace holding \
             two snapshots of one engine."
        );
    }
}

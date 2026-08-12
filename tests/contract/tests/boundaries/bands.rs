//! The band table, and the walk two of the assertions need.
//!
//! `BANDS` is here because it is the subject of two different questions — whether the README
//! restates it and whether the dependency graph respects it — and a second copy of it would
//! let those two answers disagree.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The declared band of every workspace member.
///
/// Authored here rather than derived, because a band is a design decision and there is
/// nothing in the source to infer it from. Adding a crate without adding it here fails
/// [`crate::graph::Test_Every_Member_Should_Declare_A_Band`] — so a new crate cannot quietly
/// join the workspace outside the ordering.
///
/// Bands are scaled by ten so that a band can be inserted between two others without
/// renumbering everything, which is the sort of churn that turns a table nobody wants
/// to touch into a table nobody updates.
pub(crate) const BANDS: &[(&str, u32)] = &[
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
    // A capability contract sits above the registry that resolves it and below every
    // provider that offers against it. Not beside the providers: an agreement that lives
    // with one party to it is that party's to change, and the other cannot see the file.
    ("nomos-cap-syntax", 23),
    // Language providers sit above analysis because they produce the facts it stores,
    // and nothing sits above them but a composition root. They reach each other not at
    // all: two languages are two providers of one capability, and the registry is the
    // only thing that knows both.
    ("nomos-lang-rust", 25),
    // Its peer, deliberately at the same band. Two providers of one capability must not
    // be able to name each other: this suite's downward rule forbids an edge between
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
    // Rules sit above everything they could ever need to judge and below the only thing
    // that runs them. Deliberately well clear of the language providers at 25: a rule is
    // a pure function from source text to findings and names no provider today, but the
    // moment one needs a parsed tree it must be able to reach a provider rather than
    // vendor a second parser — and a band below them would have forbidden that edge and
    // made the second parser the easy answer.
    ("nomos-rules", 30),
    ("nomos-cli", 90),
    // The contract tests sit at the top: they observe the workspace and nothing
    // observes them.
    ("nomos-contract-tests", 100),
    // The vertical slice is their peer, not their superior. Both are terminal, and
    // sharing a band is what makes them unable to name each other: this suite's
    // downward rule forbids an edge between two crates at the same band. That is the
    // property wanted — an observer of the workspace that also participates in it
    // could no longer be trusted to report on it.
    ("nomos-integration-tests", 100),
];

pub(crate) fn Declared_Band(name: &str) -> Option<u32>
{
    return BANDS
        .iter()
        .find(|(crate_name, _)| *crate_name == name)
        .map(|(_, band)| *band);
}

/// The workspace root, from this crate's manifest directory.
pub(crate) fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
}

/// Every `.rs` file under a directory, recursively.
pub(crate) fn Source_Files(root: &Path) -> Vec<PathBuf>
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
            Sort_One_Entry(&entry.path(), &mut pending, &mut found);
        }
    }

    return found;
}

/// A directory to descend into later, a Rust file to keep, or neither.
fn Sort_One_Entry(path: &Path, pending: &mut Vec<PathBuf>, found: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path.to_path_buf());
    }
    else if path.extension().is_some_and(|extension| extension == "rs")
    {
        found.push(path.to_path_buf());
    }
}

/// Every `mod` name declared anywhere under a source root.
///
/// Deliberately a flat set rather than a resolved tree. The precise version would walk
/// declarations from each root, and would need to handle `#[path]`, `#[cfg]` and inline
/// modules to avoid false positives. This approximation cannot report a false orphan —
/// it only misses the case where a module is declared in one place and the file lives in
/// an unrelated one, which is a naming problem rather than an invisibility problem.
pub(crate) fn Declared_Modules(source_root: &Path) -> BTreeSet<String>
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
            let named = Module_Declared_By(line);

            declared.extend(named);
        }
    }

    return declared;
}

/// The file name one `mod` line declares.
///
/// A commented-out declaration is not a declaration. The sibling workspace names this
/// specifically as a decoy that made an orphan look declared. And `mod foo;` is a declaration
/// of a file where `mod foo {` is an inline module, which declares nothing on disk.
fn Module_Declared_By(line: &str) -> Option<String>
{
    let trimmed = line.trim();
    if trimmed.starts_with("//")
    {
        return None;
    }

    let rest = trimmed
        .strip_prefix("mod ")
        .or_else(|| trimmed.strip_prefix("pub mod "))
        .or_else(|| trimmed.strip_prefix("pub(crate) mod "))?;

    return rest.strip_suffix(';').map(|name| return name.trim().to_owned());
}

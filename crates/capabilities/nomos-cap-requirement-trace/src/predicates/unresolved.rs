//! The three predicates that resolve a citation against the real tree: every site, every
//! `Partial` gap and every named record that is not where its entry says it is.
//!
//! Each is called by [`crate::provider::Discover_Workspace`] and by a test control -- held
//! apart from both so the control and the provider run the same code. A control that
//! re-implemented the comparison would prove the copy right and say nothing about the guard.
//!
//! Ported from `tests/contract/tests/requirement_trace/predicates.rs`. Two changes from that
//! file: [`Unresolved_Sites`], [`Unresolved_Gaps`] and [`Unresolved_Records`] now take a
//! [`nomos_platform::FileSystem`] rather than reading `std::fs` directly, since this module
//! is reachable from a real provider as well as from tests; and every predicate returns
//! [`Problem`]s rather than pre-rendered `String`s, so `nomos-rules`' own
//! `Check_Requirement_Trace_Staleness` can build a `Finding` with a real `subject_name` and
//! `locations` from structured data instead of re-parsing a sentence.
//!
//! `Unresolved_Sites` and `Unresolved_Gaps` share one message shape and differ only in
//! which field of an [`Assessment`] they walk and which of the two disambiguating words
//! (`site`/`gap`) they use — the original test suite's own `Unresolved` helper reported the
//! identical text for both, relying on the caller's own name to tell them apart; this
//! module says which one in the message itself; a `Finding`'s own summary has to stand on
//! its own outside the two functions that produced it.

use crate::assessment::{Assessment, Site};
use crate::payload::{Problem, ProblemKind};
use nomos_platform::FileSystem;
use std::path::Path;

/// Where `Unresolved` reads from -- `root` and `filesystem` always travel together, since
/// every call site resolves a path through the same pair.
struct Workspace<'a, Fs: FileSystem>
{
    root: &'a Path,
    filesystem: &'a Fs,
}

/// Which kind of problem `Unresolved` reports, and the word its own message uses for what
/// it was checking -- always chosen together, since a caller naming one already knows the
/// other.
struct ProblemShape<'a>
{
    kind: ProblemKind,
    label: &'a str,
}

/// Every site that is not where its entry says it is.
pub fn Unresolved_Sites<Fs: FileSystem>(root: &Path, assessments: &[Assessment], filesystem: &Fs) -> Vec<Problem>
{
    let mut missing = Vec::new();
    let workspace = Workspace { root, filesystem };

    for assessment in assessments
    {
        for site in &assessment.sites
        {
            let shape = ProblemShape { kind: ProblemKind::UnresolvedSite, label: "site" };
            let unresolved = Unresolved(&workspace, &assessment.requirement, site, shape);
            missing.extend(unresolved);
        }
    }

    return missing;
}

/// Every `Partial` gap that is not where its entry says it is.
///
/// The same check [`Unresolved_Sites`] runs, over `Partial`'s own field, so a partial entry
/// decays exactly the way a `Met` one does: if the code at a named gap moves or the gap
/// closes, the entry stops resolving instead of quietly going on describing nothing.
pub fn Unresolved_Gaps<Fs: FileSystem>(root: &Path, assessments: &[Assessment], filesystem: &Fs) -> Vec<Problem>
{
    let mut missing = Vec::new();
    let workspace = Workspace { root, filesystem };

    for assessment in assessments
    {
        for gap in &assessment.gaps
        {
            let shape = ProblemShape { kind: ProblemKind::UnresolvedGap, label: "gap" };
            let unresolved = Unresolved(&workspace, &assessment.requirement, gap, shape);
            missing.extend(unresolved);
        }
    }

    return missing;
}

/// Every named record that is not a registered governing record.
pub fn Unresolved_Records<Fs: FileSystem>(root: &Path, assessments: &[Assessment], filesystem: &Fs) -> Vec<Problem>
{
    let mut unresolved = Vec::new();

    for assessment in assessments
    {
        let Some(record) = assessment.record.as_deref()
        else
        {
            continue;
        };

        let missing = Unregistered(root, assessment, record, filesystem);
        unresolved.extend(missing);
    }

    return unresolved;
}

/// Why one named record does not resolve to a registered governing record, if it does not.
fn Unregistered<Fs: FileSystem>(root: &Path, assessment: &Assessment, record: &str, filesystem: &Fs) -> Option<Problem>
{
    if !Registration_Exists(root, record, filesystem)
    {
        return Some(Problem {
            kind: ProblemKind::UnresolvedRecord,
            requirement: assessment.requirement.clone(),
            message: format!(
                "{}: {record} has no registration under \
                 crates/spec/nomos-spec-store/records/",
                assessment.requirement
            ),
        });
    }
    if Document_Exists(root, record, filesystem)
    {
        return None;
    }

    return Some(Problem {
        kind: ProblemKind::UnresolvedRecord,
        requirement: assessment.requirement.clone(),
        message: format!(
            "{}: {record} is registered and its document is not under docs/records/",
            assessment.requirement
        ),
    });
}

/// Whether a record identifier has a registration file.
fn Registration_Exists<Fs: FileSystem>(root: &Path, record: &str, filesystem: &Fs) -> bool
{
    return filesystem.Exists(&root.join("crates/spec/nomos-spec-store/records").join(format!("{record}.record")));
}

/// Whether a record identifier has a document under `docs/records`.
///
/// By stem prefix, because the slug is not derivable from the identifier — the same reason
/// `OD-SPEC-007` puts the path inside the registration rather than computing it.
fn Document_Exists<Fs: FileSystem>(root: &Path, record: &str, filesystem: &Fs) -> bool
{
    let prefix = format!("{record}-");
    let Ok(entries) = filesystem.Read_Directory(&root.join("docs/records"))
    else
    {
        return false;
    };

    return entries.iter().any(|path| {
        if path
            .extension()
            .is_none_or(|extension| return extension != "md")
        {
            return false;
        }

        return path
            .file_name()
            .and_then(|name| return name.to_str())
            .is_some_and(|name| return name.starts_with(&prefix));
    });
}

/// Why one site (or gap) is not where its entry says it is, if it is not. `shape.label`
/// names which field `site` came from ("site" or "gap"), so the message stands on its own.
fn Unresolved<Fs: FileSystem>(workspace: &Workspace<'_, Fs>, requirement: &str, site: &Site, shape: ProblemShape<'_>) -> Option<Problem>
{
    let path = workspace.root.join(&site.path);
    let Ok(text) = workspace.filesystem.Read_To_String(&path)
    else
    {
        return Some(Problem {
            kind: shape.kind,
            requirement: requirement.to_owned(),
            message: format!("{requirement}: {} {} is not a file in this workspace", shape.label, site.path),
        });
    };
    if text.contains(&site.symbol)
    {
        return None;
    }

    return Some(Problem {
        kind: shape.kind,
        requirement: requirement.to_owned(),
        message: format!("{requirement}: {} no longer occurs in {} {}", site.symbol, shape.label, site.path),
    });
}

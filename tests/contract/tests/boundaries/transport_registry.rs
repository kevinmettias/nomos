//! What the API transport may project, asserted rather than left to a list that is short today.
//!
//! `OD-HOST-007` decided that an external surface over `nomos-api` projects Gate's three
//! verbs, `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2` widened that to a fourth for Correction,
//! and neither projects the many handlers belonging to crates `README.md` marks
//! `[repo tooling]`, and said in as many words how that exclusion has to hold: "The exclusion
//! is structural rather than advisory: the transport crate declares its tool registry
//! explicitly, and `tests/contract` asserts that the registry names no handler belonging to a
//! `[repo tooling]` crate -- the same shape
//! `Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` already uses to keep a
//! boundary from being crossed by accident instead of by decision. A registry that is merely
//! short today, with nothing stopping a later increment from lengthening it, would be the
//! absence-as-boundary `OD-CONNECTOR-001` refuses."
//!
//! That record left itself open until a transport existed, because "a contract test asserting
//! a registry that does not exist would be the vacuous-truth trap `OD-CONTRACTS-001`'s
//! honesty vocabularies refuse". This module is the artifact it was waiting for.
//!
//! The subject is a *name*, the same way the sibling-workspace check's subject is a name.
//! Reaching for the dependency graph instead would prove nothing here: the transport depends
//! on `nomos-api`, which depends on every orchestration crate, so `nomos-work-orchestration`
//! is in its transitive closure whatever its registry says. Writing a call to
//! `Handle_Work_Finish` in the transport's own source is what widening the registry actually
//! costs, and that is what is refused.
//!
//! It is the *qualified* name that is refused -- `nomos_api::Handle_Work_Finish`, the shape a
//! call takes -- rather than the bare identifier anywhere in the file. The bare form appears
//! legitimately in two places this boundary should not fight: the transport's own module doc,
//! which names the excluded handlers in order to explain what it excludes, and a test
//! asserting that a caller reaching for one is refused. A check that could not tell a call
//! from the sentence explaining why there is no call would be satisfied by deleting the
//! explanation, which is the wrong thing to reward. What makes the narrower subject
//! sufficient rather than merely narrower is
//! [`Test_The_Transport_Should_Reach_Nomos_Api_Only_By_Qualified_Path`] below: with no import
//! of that crate anywhere, there is no unqualified spelling for a call to hide in.

use crate::bands::Repository_Root;
use std::path::{Path, PathBuf};

/// The crate whose registry `OD-HOST-007` bounds.
const TRANSPORT_SOURCE: &str = "crates/host/nomos-api-transport/src";

/// The crate the transport projects.
const PROJECTED_CRATE: &str = "nomos_api";

/// The blessed surface of the crate the transport projects.
///
/// Read rather than restated: this file is checked in and
/// `Test_Every_Crates_Public_Surface_Should_Match_Its_Snapshot` keeps it equal to what
/// `nomos-api` really exports, so a handler added there arrives here without anyone
/// remembering to add it.
const PROJECTED_SURFACE: &str = "tests/contract/surface/nomos-api.txt";

/// The handlers `OD-HOST-007` and `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2` admit.
///
/// Authored here rather than derived, because which verbs are a product surface and which are
/// how this repository is developed is a decision, and there is nothing in the source to
/// infer it from. Changing this list is changing what that record decided, which is what a
/// failure below is meant to make somebody notice.
const ADMITTED: [&str; 5] = [
    "Handle_Gate_Plan",
    "Handle_Gate_Run",
    "Handle_Gate_Explain",
    "Handle_Gate_Compare",
    "Handle_Correction_Run",
];

/// The transport calls no handler its registry does not admit.
///
/// Quantified over `nomos-api`'s real exported surface rather than over a prefix guess, so a
/// future handler family -- a `Handle_Corrections_*`, say -- is excluded by default and has to
/// be admitted deliberately, which is the direction `OD-CONNECTOR-001` asks a boundary to fail
/// in.
#[test]
fn Test_The_Transport_Should_Name_No_Repo_Tooling_Handler()
{
    let handlers = Projected_Handlers();
    let text = Transport_Text(&Transport_Sources());

    let leaked: Vec<&String> = handlers
        .iter()
        .filter(|handler| return !ADMITTED.contains(&handler.as_str()))
        .filter(|handler| return text.contains(&Qualified(handler)))
        .collect();

    assert!(
        leaked.is_empty(),
        "the API transport calls {leaked:?}.\n\
         OD-HOST-007 bounds its registry to {ADMITTED:?}: the other handlers nomos-api \
         exports belong to crates README.md marks [repo tooling], which exist to develop \
         this repository rather than to answer a question an end-user repository would ask. \
         Widening the registry is that record's decision to revisit, not this crate's to \
         make."
    );
}

/// The transport reaches `nomos-api` only by qualified path, so the check above sees every
/// call.
///
/// Without this, `use nomos_api::{Handle_Gate_Run, Handle_Work_Finish}` would bring a handler
/// into scope under a bare name that the qualified search never matches — the boundary would
/// still be crossed by decision, but silently, which is the half of
/// `Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace`'s value this module would
/// otherwise lose. The constraint costs the transport nothing: a handler call is one
/// expression per dispatch arm, and spelling it fully is what makes the registry legible in
/// one read of one file.
#[test]
fn Test_The_Transport_Should_Reach_Nomos_Api_Only_By_Qualified_Path()
{
    let sources = Transport_Sources();

    let importing: Vec<String> = sources
        .iter()
        .filter(|source| return Read(source).contains(&format!("use {PROJECTED_CRATE}::")))
        .map(|source| return source.display().to_string())
        .collect();

    assert!(
        importing.is_empty(),
        "{importing:?} imports from {PROJECTED_CRATE} rather than calling through it.\n\
         An imported handler is reachable under a bare name, which \
         Test_The_Transport_Should_Name_No_Repo_Tooling_Handler above does not search for. \
         Call nomos_api::Handle_… by its full path so the registry stays greppable."
    );
}

/// The two assertions above have real subjects on both sides.
///
/// Without this they pass over a transport that calls nothing at all, and over a surface
/// snapshot that yielded no handlers to exclude — the two ways they could go quiet while
/// reading nothing. `OD-CONTRACTS-001`'s own objection to a vacuous check, applied to the
/// check that record asked for.
#[test]
fn Test_The_Registry_Assertion_Should_Have_Subjects_On_Both_Sides()
{
    let handlers = Projected_Handlers();
    let sources = Transport_Sources();
    let text = Transport_Text(&sources);

    let excluded = handlers.iter().filter(|handler| return !ADMITTED.contains(&handler.as_str())).count();
    let called: Vec<&&str> = ADMITTED.iter().filter(|handler| return text.contains(&Qualified(handler))).collect();

    assert!(!sources.is_empty(), "no source was read from {TRANSPORT_SOURCE}");
    assert!(
        excluded > 0,
        "nomos-api exported {} handlers and every one of them is admitted, so the exclusion \
         above has nothing to exclude and would pass having judged nothing",
        handlers.len()
    );
    assert_eq!(
        called.len(),
        ADMITTED.len(),
        "the transport calls {called:?} of the {ADMITTED:?} its registry admits. A transport \
         that calls none of them satisfies the exclusion by serving nothing, which is not \
         what OD-HOST-007 decided."
    );
}

/// `handler` as a call to it is spelled.
fn Qualified(handler: &str) -> String
{
    return format!("{PROJECTED_CRATE}::{handler}");
}

/// Every `Handle_*` function `nomos-api` exports, from its own blessed snapshot.
fn Projected_Handlers() -> Vec<String>
{
    let path = Repository_Root().join(PROJECTED_SURFACE);
    let text = std::fs::read_to_string(&path)
        // Unreadable, this file yields no handlers, and two assertions above quantify over
        // them — one would pass having excluded nothing, and the third is what catches it.
        // Failing here says which of the two happened.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    let mut handlers = Vec::new();
    for line in text.lines()
    {
        handlers.extend(Handler_Name(line));
    }

    return handlers;
}

/// The handler a surface line declares, or nothing if the line declares something else.
///
/// A line is `pub fn nomos_api::Handle_Gate_Run(command: &GateCommand) -> GateRunResponse`.
fn Handler_Name(line: &str) -> Option<String>
{
    let declaration = line.strip_prefix(&format!("pub fn {PROJECTED_CRATE}::"))?;
    let name = declaration.split('(').next()?;

    return name.starts_with("Handle_").then(|| return name.to_owned());
}

/// Every source file the transport crate is built from.
fn Transport_Sources() -> Vec<PathBuf>
{
    let mut sources = Vec::new();
    let mut pending = vec![Repository_Root().join(TRANSPORT_SOURCE)];

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
            Collect(path, &mut pending, &mut sources);
        }
    }

    sources.sort();
    return sources;
}

/// One walked entry: descended into if it is a directory, collected if it is Rust source.
fn Collect(path: PathBuf, pending: &mut Vec<PathBuf>, sources: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path);

        return;
    }

    if path.extension().is_some_and(|extension| return extension == "rs")
    {
        sources.push(path);
    }
}

/// Every source of the transport crate as one text to search.
fn Transport_Text(sources: &[PathBuf]) -> String
{
    let mut text = String::new();
    for source in sources
    {
        text.push_str(&Read(source));
        text.push('\n');
    }

    return text;
}

/// One source, read or reported.
fn Read(path: &Path) -> String
{
    return std::fs::read_to_string(path)
        // A source that cannot be read is a source the assertions above did not search, which
        // is indistinguishable from a source that named nothing.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

//! What the API transport may project, asserted rather than left to a list that is short today.
//!
//! `OD-HOST-007` decided that an external surface over `nomos-api` projects Gate's verbs,
//! `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2` widened that for Correction and `OD-HOST-014`
//! for Check, and none of them projects the many handlers belonging to crates `README.md`
//! marks `[repo tooling]`, and said in as many words how that exclusion has to hold: "The
//! exclusion is structural rather than advisory: the transport crate declares its tool registry
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

/// The handlers `OD-HOST-007`, `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2` and `OD-HOST-014`
/// admit.
///
/// Authored here rather than derived, because which verbs are a product surface and which are
/// how this repository is developed is a decision, and there is nothing in the source to
/// infer it from. Changing this list is changing what those records decided, which is what a
/// failure below is meant to make somebody notice. `OD-HOST-014`'s decision 5 names this
/// array as the third of the three places an admitting increment must edit, "which is the
/// point of there being three".
const ADMITTED: [&str; 6] = [
    "Handle_Gate_Plan",
    "Handle_Gate_Run",
    "Handle_Gate_Explain",
    "Handle_Gate_Compare",
    "Handle_Correction_Run",
    "Handle_Check_Run",
];

/// The three product operations `OD-HOST-014` refuses, by the handler each is exported as.
///
/// Named rather than left to the quantification above, because their exclusion is a decision
/// with a stated reason rather than a gap: `Handle_Agent_Execute` and
/// `Handle_Agent_Judge_Role` "start a subprocess and spend against a ceiling" that bounds one
/// dispatch and not a caller making many, and `Handle_Workflow_Run` is refused "while a step
/// may carry an agent body", which would admit the first two transitively. The quantification
/// keeps every unnamed handler out; this keeps these three out *by name*, so admitting one
/// cannot read as the same edit as admitting a handler nobody had considered.
const REFUSED: [&str; 3] = ["Handle_Agent_Execute", "Handle_Agent_Judge_Role", "Handle_Workflow_Run"];

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
         OD-HOST-007 and OD-HOST-014 together bound its registry to {ADMITTED:?}. The other \
         handlers nomos-api exports are excluded by one of the two: the twenty-one \
         Handle_Work_* and Handle_Spec_* ones belong to crates README.md marks \
         [repo tooling], which exist to develop this repository rather than to answer a \
         question an end-user repository would ask, and the agent and workflow ones start or \
         reach a metered external process, which OD-HOST-014's own criterion refuses. \
         Widening the registry is those records' decision to revisit, not this crate's to \
         make."
    );
}

/// The three operations `OD-HOST-014` refuses are still refused.
///
/// The quantification above keeps every unnamed handler out, and would keep these out too.
/// This says so *by name*, because their exclusion is a decision with a stated reason rather
/// than a gap nobody has filled: admitting one is the record's own decision 6 to revisit, and
/// editing this array is how somebody would say one of the four events it names had happened.
#[test]
fn Test_The_Agent_And_Workflow_Operations_Should_Stay_Refused()
{
    let text = Transport_Text(&Transport_Sources());

    let called: Vec<&&str> = REFUSED.iter().filter(|handler| return text.contains(&Qualified(handler))).collect();

    assert!(
        called.is_empty(),
        "the API transport calls {called:?}.\n\
         OD-HOST-014 refuses Agent_Execute and Agent_Judge_Role because each starts a \
         subprocess and spends against a per-dispatch ceiling that bounds no number of calls \
         an unauthenticated wire caller makes, and refuses Workflow_Run while a step may \
         carry an agent body. Its decision 6 names what would reopen either; none of it is a \
         registry's own decision to make."
    );
}

/// The refusal above has a real subject: `nomos-api` still exports all three.
///
/// Without this it passes over a handler that has been renamed or removed, which is a refusal
/// with nothing left to refuse -- the same vacuity objection
/// [`Test_The_Registry_Assertion_Should_Have_Subjects_On_Both_Sides`] makes about the
/// quantified one.
#[test]
fn Test_Every_Refused_Operation_Should_Still_Be_Exported()
{
    let handlers = Projected_Handlers();

    let exported: Vec<&&str> = REFUSED
        .iter()
        .filter(|handler| return handlers.iter().any(|exported| return exported == *handler))
        .collect();

    assert_eq!(
        exported.len(),
        REFUSED.len(),
        "nomos-api exports {exported:?} of the {REFUSED:?} OD-HOST-014 refuses. A handler this \
         array names and nomos-api no longer exports makes the refusal above vacuous, so the \
         array is stale rather than satisfied."
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

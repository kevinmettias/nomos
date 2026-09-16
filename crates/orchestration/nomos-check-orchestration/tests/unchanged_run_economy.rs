//! An unchanged second `Run` produces no new facts.
//!
//! # Why this is not what the neighbouring suite proves
//!
//! `incremental_equivalence.rs` proves that a reused workspace and store reach the claim and
//! findings a clean recomputation reaches, through the real `Run`, across three classes of
//! input. That is correctness of reuse, and its own documentation states why it is not
//! economy: *recomputing everything and recomputing nothing both pass an unchanged
//! comparison*. A regression that silently re-materialized every fact on every call would
//! leave that suite, and every other test in this workspace, green.
//!
//! # Why the observable is the store's counter and not `Examined`
//!
//! [`nomos_analysis::MemoryFactStore::Materializations`] counts store writes: it is
//! incremented inside `Materialize`, and `Is_Already_Current` short-circuits before that call,
//! so a reused subject does not increment it. It is exactly production.
//!
//! `CheckOutcome::Judged`'s own `Examined { files, facts }` is deliberately *not* that
//! observable and must not be turned into one. `Materialize_Syntax` counts a subject as
//! covered whether it was reused or produced -- its own doc says a caller reusing a store
//! "pays for a real parse only for a subject whose own bytes moved since the store last saw
//! it, but still counts that subject as covered either way." Asserting zero there would force
//! the wrong semantics onto a counter answering a different question, and would make an
//! honest run look like a broken one.
//!
//! # Why one narrow selection
//!
//! `COMPLETENESS_MIRROR` declares only `RequiredFact::SyntaxItems`, so this drives the
//! source-derived path and no subprocess. The claim is about whether reuse is economical at
//! all; proving it for every family would mean paying for `cargo metadata` and a clippy run
//! twice to learn the same thing about the same store.

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::{CheckOutcome, Claim, Run, RunContext};
use nomos_contracts::{Finding, RuleId};
use nomos_model::Subject_Of_Path;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, Workspace};
use std::path::Path;

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// The text half of a [`Source_File`]. A distinct type from the path half, so the two adjacent
/// string positions cannot be transposed at a call site and still compile.
struct SourceText<'a>(&'a str);

fn Source_File(path: &str, text: SourceText<'_>) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text.0);
}

/// What one `Run` answered, and what it cost the store to answer it.
struct Answer
{
    findings: Vec<Finding>,
    claim: Claim,
    produced: u32,
}

/// The workspace and store a caller carries from one `Run` to the next, held together as one
/// value so a call site cannot thread half the pair to a different run than the other half.
struct Carried
{
    workspace: Option<Workspace>,
    store: MemoryFactStore,
}

impl Carried
{
    fn New() -> Self
    {
        return Self { workspace: None, store: MemoryFactStore::New() };
    }

    /// One `Run` over `sources` at `root` under `selected`, reusing this value's own workspace
    /// and store, reduced to an [`Answer`] carrying how many facts the store gained while the
    /// run was in flight.
    fn Answered(&mut self, sources: &[SourceFile], root: &Path, selected: &[RuleId]) -> Answer
    {
        let before = self.store.Materializations();

        let outcome = Run(
            sources,
            RunContext { variant: Test_Variant(), root, launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut self.workspace, store: &mut self.store },
            selected,
        );

        let produced = self.store.Materializations().saturating_sub(before);

        let CheckOutcome::Judged { findings, claim, .. } = outcome
        else
        {
            panic!("the fixture is readable, so a run over it must reach Judged");
        };

        return Answer { findings, claim, produced };
    }
}

/// Guards the vacuity every economy assertion below would otherwise hide: a store that never
/// materializes anything trivially materializes nothing the second time.
fn Assert_First_Run_Produced_Something(first: &Answer)
{
    assert!(
        first.produced > 0,
        "the first run produced no facts at all, so this fixture proves nothing about reuse: \
         a store that never materializes anything trivially materializes nothing the second \
         time"
    );
}

/// Asserts a second run paid the store for no new fact and still answered exactly what the
/// first run answered.
fn Assert_Second_Run_Cost_Nothing(first: &Answer, second: &Answer)
{
    assert_eq!(
        second.produced, 0,
        "an unchanged second run produced {} new fact(s). Nothing about the tree, the \
         selection or the policy moved between the two calls, so every fact the second run \
         needed was already live in the store it was handed. Facts produced again here are \
         work a caller pays for twice, and no existing test would have reported it -- \
         recomputing everything and recomputing nothing both satisfy an equivalence \
         comparison.",
        second.produced
    );
    assert_eq!(
        second.findings, first.findings,
        "the second run answered different findings, so whatever economy it bought was bought \
         by answering a different question"
    );
    assert_eq!(
        second.claim, first.claim,
        "the second run reached a different claim, so its coverage is not the first run's even \
         though it produced nothing"
    );
}

/// A second run over an unchanged tree produces nothing, and answers the same.
#[test]
fn Test_An_Unchanged_Second_Run_Should_Produce_No_New_Facts()
{
    let sources = [
        Source_File("src/lib.rs", SourceText("pub fn Judged_Thing() -> u8\n{\n    return 1;\n}\n")),
        Source_File("src/other.rs", SourceText("pub fn Second_Thing() -> u8\n{\n    return 2;\n}\n")),
    ];
    let selected = [RuleId::New(nomos_rules::COMPLETENESS_MIRROR)];
    let root = Path::new(".");

    let mut carried = Carried::New();
    let first = carried.Answered(&sources, root, &selected);
    let second = carried.Answered(&sources, root, &selected);

    Assert_First_Run_Produced_Something(&first);
    Assert_Second_Run_Cost_Nothing(&first, &second);
}

/// A second run over a *changed* tree does produce a fact.
///
/// The control that keeps the assertion above from being vacuous in the other direction. A
/// store whose counter never moved, or a `Run` that never materialized on the source-derived
/// path at all, would satisfy "an unchanged second run produces nothing" while proving
/// nothing about reuse. This is the same discipline `incremental_equivalence.rs` uses when it
/// requires the mutation to be observable before comparing the two answers.
#[test]
fn Test_A_Second_Run_Over_A_Changed_Source_Should_Produce_A_Fact()
{
    let before = [Source_File("src/lib.rs", SourceText("pub fn Judged_Thing() -> u8\n{\n    return 1;\n}\n"))];
    let after = [Source_File("src/lib.rs", SourceText("pub fn Judged_Thing() -> u8\n{\n    return 99;\n}\n"))];
    let selected = [RuleId::New(nomos_rules::COMPLETENESS_MIRROR)];
    let root = Path::new(".");

    let mut carried = Carried::New();
    let first = carried.Answered(&before, root, &selected);
    let changed = carried.Answered(&after, root, &selected);

    Assert_First_Run_Produced_Something(&first);
    assert!(
        changed.produced > 0,
        "a source whose bytes moved produced no new fact, so the counter this file asserts \
         zero on does not move for a real change -- which would make the zero meaningless"
    );
}

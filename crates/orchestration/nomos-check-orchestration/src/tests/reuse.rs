//! A workspace and a store carried from one `Run` to the next: what reuse must agree with,
//! and what it must skip.

use nomos_analysis::MemoryFactStore;
use nomos_contracts::{GenerationId, RuleId};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::Workspace;

use crate::{CheckOutcome, RuleReassessmentCache, Run, RunContext};

use super::{Repository_Root, Source_File, SourceText, Test_Variant};

/// How many real rule invocations `RuleReassessmentCache` records once the one source the rule
/// reads has been edited: one for the call that recorded it, one for the call the edit forces.
const REAL_RULE_INVOCATIONS_AFTER_AN_EDIT: u32 = 2;

/// One `Run` over `sources`, carrying `workspace` and `store` across calls -- the reuse the
/// equivalence test below is about.
fn Run_Carrying(
    sources: &[SourceFile], selected: &[RuleId], workspace: &mut Option<Workspace>, store: &mut MemoryFactStore,
) -> CheckOutcome
{
    return Run(
        sources,
        RunContext {
            variant: Test_Variant(),
            root: &Repository_Root(),
            launcher: &StdProgramLauncher,
            filesystem: &StdFileSystem,
            environment: &StdEnvironment,
            workspace,
            store,
        },
        selected,
    );
}

/// The generation `Run` last left on a reused workspace, which a real edit must advance.
fn Workspace_Generation(workspace: &Option<Workspace>) -> GenerationId
{
    return workspace.as_ref().expect("Run must leave a workspace behind").Generation();
}

/// Asserts a reused run reached the claim and the findings a clean recomputation reached,
/// which is the whole of `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS`'s safety claim.
fn Assert_Reuse_Agrees_With_Recomputation(reused: &CheckOutcome, clean: &CheckOutcome)
{
    let CheckOutcome::Judged { findings: reused_findings, claim: reused_claim, .. } = reused
    else
    {
        panic!("the reused workspace and store call over a readable tree must be judged");
    };
    let CheckOutcome::Judged { findings: clean_findings, claim: clean_claim, .. } = clean
    else
    {
        panic!("the fresh workspace and store call over a readable tree must be judged");
    };

    assert_eq!(
        reused_claim, clean_claim,
        "a workspace and store reused across an edit must reach the same claim a clean recomputation reaches"
    );
    assert_eq!(
        reused_findings, clean_findings,
        "a workspace and store reused across an edit must report the same findings a clean recomputation reports"
    );
}

/// Asserts a second call over unmoved sources was still judged, holding a current fact for
/// every source and no findings of its own -- the state
/// `P40-INCREMENTAL-SKIP-COVERAGE-REGRESSION` would have broken by reporting `NoFacts` for a
/// store whose every subject it skipped as already current.
fn Assert_Second_Call_Still_Judged(second: &CheckOutcome, source_count: usize)
{
    let CheckOutcome::Judged { examined, findings, .. } = second
    else
    {
        panic!("a second call with no content change at all must still be judged, not NoFacts: skipping an already-current subject must not read as failing to find one");
    };
    assert_eq!(
        examined.facts,
        source_count,
        "every source still has a current fact after the second call, none of them newly written"
    );
    assert!(findings.is_empty(), "{findings:?}");
}

/// `OD-ANALYSIS-009`'s first real increment, exercised end to end: `crates/substrate/
/// nomos-analysis/tests/recomputation_equivalence.rs` already proves, over synthetic
/// `FactKey`s built by hand, that `MemoryFactStore::Invalidate` and `Materialize` agree with
/// a clean rebuild. What that file cannot prove is that this crate's own real composition --
/// the real syntax provider, the real `Workspace`, `Run`'s real ingestion and judging -- ever
/// asks a store and a workspace to survive across two calls at all; before `Run` accepted
/// either through `RunContext`, nothing in this workspace ever did. This is that second real
/// caller: the same `workspace` and `store` carried across a call that edits one file and
/// leaves another alone, checked against an independent third call that recomputes the
/// post-edit tree from nothing.
///
/// `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS` closed the gap this doc used to name here:
/// `Materialize_Syntax` no longer re-derives a fact for a subject whose own bytes did not
/// move since the reused `store` last saw it. What this test still proves is the
/// precondition that fix needed: carrying a workspace and a store across a real edit is
/// *safe* -- reusing them agrees with throwing them away and starting over, on both the
/// claim and the findings, and the generation the reused workspace reports genuinely
/// advances rather than repeating itself. A caller could not have relied on either fact
/// before this increment, because no caller had ever reused either object.
/// [`Test_A_Store_And_Workspace_Reused_With_No_Change_Between_Two_Calls_Should_Still_Be_Judged`],
/// below, is the test that exercises the skip itself, over the zero-change case this one
/// does not reach.
///
/// `REQUIREMENT_TRACE_STALENESS` rides beside `NAMING_CONVENTION`, and it is here for what
/// `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY` changed. Both selected rules read a
/// non-syntax family, but the naming one judges a source and so is re-judged by the edit
/// anyway; the requirement-trace one judges the whole workspace, reads a family whose fact the
/// reused call does **not** re-file, and is not re-judged at all. So the reused half of this
/// comparison now answers partly from a fact carried over rather than one produced, which is
/// exactly the state a currency check creates and exactly what equivalence has to survive. A
/// check that skipped a write it should have made shows up here as a disagreement with the
/// clean recomputation -- `OD-ANALYSIS-005`'s property, and the reason this is the test
/// `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY` names.
#[test]
fn Test_A_Store_And_Workspace_Reused_Across_An_Edit_Agrees_With_A_Clean_Recomputation()
{
    let unedited = Source_File("a.rs", SourceText("pub fn Ok() {}\n"));
    let edited = Source_File("a.rs", SourceText("pub fn Ok() {}\npub fn Also_Ok() {}\n"));
    let untouched = Source_File("b.rs", SourceText("pub fn Untouched() {}\n"));
    let selected = [RuleId::New(nomos_rules::NAMING_CONVENTION), RuleId::New(nomos_rules::REQUIREMENT_TRACE_STALENESS)];

    let mut workspace = None;
    let mut store = MemoryFactStore::New();
    let first = Run_Carrying(&[unedited, untouched.clone()], &selected, &mut workspace, &mut store);
    assert!(matches!(first, CheckOutcome::Judged { .. }), "the first call over a readable tree must be judged");
    let generation_before = Workspace_Generation(&workspace);

    let reused = Run_Carrying(&[edited.clone(), untouched.clone()], &selected, &mut workspace, &mut store);
    let generation_after = Workspace_Generation(&workspace);
    assert!(
        generation_after > generation_before,
        "a real edit reusing the same workspace must advance its generation, not repeat it"
    );

    let clean = Run_Carrying(&[edited, untouched], &selected, &mut None, &mut MemoryFactStore::New());
    Assert_Reuse_Agrees_With_Recomputation(&reused, &clean);
}

/// `P40-INCREMENTAL-SKIP-COVERAGE-REGRESSION`: a second `Run` over a reused workspace and
/// store, with *no* content change at all between the two calls, must still report
/// `CheckOutcome::Judged` with `examined.facts` equal to the file count -- not
/// `CheckOutcome::NoFacts`, which is what `Materialize_Syntax`'s own return value briefly
/// meant "newly written" rather than "current" would have produced here, since skipping
/// every source as already-current would have made that count zero. This is the exact
/// usage `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS`'s own done_when asked a caller to be
/// free to adopt, so it is the one case that regression's own fix owes a real test.
#[test]
fn Test_A_Store_And_Workspace_Reused_With_No_Change_Between_Two_Calls_Should_Still_Be_Judged()
{
    let sources = [Source_File("a.rs", SourceText("pub fn Ok() {}\n")), Source_File("b.rs", SourceText("pub fn Also_Ok() {}\n"))];
    let selected = [RuleId::New(nomos_rules::NAMING_CONVENTION)];

    let mut workspace = None;
    let mut store = MemoryFactStore::New();
    let first = Run_Carrying(&sources, &selected, &mut workspace, &mut store);
    let CheckOutcome::Judged { examined: first_examined, .. } = first else { panic!("the first call over a readable tree must be judged") };
    assert_eq!(first_examined.facts, sources.len(), "a fresh store must report a current fact for every real source");

    let second = Run_Carrying(&sources, &selected, &mut workspace, &mut store);
    Assert_Second_Call_Still_Judged(&second, sources.len());
}

/// The workspace, store and reassessment cache one cache-retention test carries across its
/// three calls.
struct ReassessmentFixture
{
    workspace: Option<Workspace>,
    store: MemoryFactStore,
    cache: RuleReassessmentCache,
}

impl ReassessmentFixture
{
    fn New() -> Self
    {
        return Self {
            workspace: None,
            store: MemoryFactStore::New(),
            cache: RuleReassessmentCache::New(),
        };
    }

    /// One [`crate::Run_Reassessing`] over `sources`, reusing this fixture's workspace, store
    /// and cache.
    fn Reassess(&mut self, sources: &[SourceFile], selected: &[RuleId]) -> CheckOutcome
    {
        return crate::run_context::Run_Reassessing(
            sources,
            RunContext {
                variant: Test_Variant(),
                root: &Repository_Root(),
                launcher: &StdProgramLauncher,
                filesystem: &StdFileSystem,
                environment: &StdEnvironment,
                workspace: &mut self.workspace,
                store: &mut self.store,
            },
            selected,
            &mut self.cache,
        );
    }

    /// How many real rule invocations the cache has recorded.
    fn Recorded(&self) -> u32
    {
        return self.cache.Recorded();
    }
}

/// Asserts the cache has recorded exactly `expected` rule invocations, quoting `context` in
/// the failure. The count is the only evidence that tells a skipped rule from one that ran
/// and reported the same thing, which is why these tests read `Recorded` rather than compare
/// findings.
fn Assert_Recorded(fixture: &ReassessmentFixture, expected: u32, context: &str)
{
    assert_eq!(fixture.Recorded(), expected, "{context}");
}

/// Asserts a skipped rule still reports what it reported when it last ran, rather than
/// reporting nothing because it did not run.
fn Assert_Skip_Repeats_Its_Last_Findings(first: CheckOutcome, second: CheckOutcome)
{
    let (CheckOutcome::Judged { findings: first_findings, .. }, CheckOutcome::Judged { findings: second_findings, .. }) = (first, second)
    else
    {
        panic!("both calls over a readable tree must be judged");
    };
    assert_eq!(first_findings, second_findings, "a skipped rule must report what it reported when it last ran, not nothing");
}

/// A retained cache skips a rule whose families did not move, and stops skipping when one
/// does.
///
/// `Run_Reassessing` and `run_context::RuleReassessmentCache` had no caller and no test anywhere
/// when this was written -- grepped across every `.rs` under `crates/` and `tests/`, the only
/// mentions outside `run_context.rs` were inside the cache's own file. The mechanism was
/// implemented, exported, and had never been executed, so nothing stood behind its
/// behavioural claim. This is that claim, run.
///
/// `COMPLETENESS_MIRROR` is the rule under test because `DESCRIPTORS` gives it exactly one
/// required family, `RequiredFact::SyntaxItems`. When this was written, a rule reading a
/// policy family would have proven less here: those sections re-filed their fact on every
/// call, so their family appeared in `changed` every time and the skip could never be
/// observed. `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY` closed that, and
/// `crate::run_context::tests` is where a policy-declaring rule's own skip is now counted.
/// The narrow selection stays because the claim here is about the mechanism rather than about
/// which families reach it.
///
/// The count comes from `RuleReassessmentCache::Recorded` rather than from comparing
/// findings, for the reason that method's own doc gives: a rule whose output does not change
/// is indistinguishable from a skipped one by content alone.
#[test]
fn Test_A_Retained_Cache_Should_Skip_A_Rule_Whose_Families_Did_Not_Move()
{
    let selected = [RuleId::New(nomos_rules::COMPLETENESS_MIRROR)];
    let unedited = [Source_File("a.rs", SourceText("pub fn Ok() {}\n"))];
    let edited = [Source_File("a.rs", SourceText("pub fn Ok() {}\npub fn Also_Ok() {}\n"))];

    let mut fixture = ReassessmentFixture::New();

    let first = fixture.Reassess(&unedited, &selected);
    assert!(matches!(first, CheckOutcome::Judged { .. }), "the first call over a readable tree must be judged");
    Assert_Recorded(&fixture, 1, "the first call must run the rule for real and record what it found");

    let second = fixture.Reassess(&unedited, &selected);
    Assert_Recorded(&fixture, 1, "a second call over unmoved sources must reuse the recorded findings rather than run the rule again; a count of 2 means the cache was threaded and never consulted");

    let third = fixture.Reassess(&edited, &selected);
    Assert_Recorded(&fixture, REAL_RULE_INVOCATIONS_AFTER_AN_EDIT, "editing the one source moves SyntaxItems, which COMPLETENESS_MIRROR reads, so the rule must run again rather than serve a stale finding");

    Assert_Skip_Repeats_Its_Last_Findings(first, second);
    assert!(matches!(third, CheckOutcome::Judged { .. }), "the edited call must still be judged");
}

/// The empty selection, which is what every real caller passes and what
/// `NomosDiagnosticProvider` passes in particular. Named rather than written as `&[]` at the
/// call, because an empty slice at a call site reads as "no rules" and means the opposite.
const EVERY_COMPOSED_RULE: &[RuleId] = &[];

/// A retained cache skips a large part of the composed table under the selection production
/// actually uses, and stops skipping for the rules an edit reaches.
///
/// The test above selects `COMPLETENESS_MIRROR` alone and its own doc says why. That is the
/// right choice for the claim it makes, and it leaves the production claim unmade. This is
/// that claim.
///
/// Nothing asserted it before. A regression putting `SyntaxItems` into `changed`
/// unconditionally would take the skip to nothing and leave the narrow test above, the LSP
/// suite, and every equivalence test in this workspace green, because skipping a
/// deterministic rule and re-running it produce identical findings. `Recorded` is the only
/// observable that tells them apart, and `0c3b1013` established that nothing outside this
/// crate can distinguish the two -- so a claim not asserted here is asserted nowhere.
///
/// Written as inequalities, not as the figures measured when it was first run. The composed
/// table grows, and a test that fails because somebody added a rule fails for the wrong
/// reason. What is fixed is the shape: an unchanged call skips part of the table, and an
/// edit puts rules back.
///
/// How much, measured twice and recorded here as a reading rather than as an assertion.
///
/// Before `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY`: 71 rules cold, 33 on the unchanged
/// call, 71 again after the edit. So the skip was 38 of 71, and the edit was a *total*
/// invalidation -- editing one of two sources put the whole table back in play, not the part
/// of it that reads the edited file.
///
/// After it, measured 2026-09-21: 71 cold, **0** on the unchanged call, 69 after the edit. The
/// unchanged call now runs no rule closure at all, because no family reports itself changed
/// when nothing it reads has moved -- which, over this repository's own root and with the
/// empty selection, includes the three subprocess families, so a second `cargo metadata`,
/// `cargo clippy` and `cargo deny` each produced the payload the store was already serving.
/// (That those three answer at all over this root is `super::materialization`'s claim, not
/// this one's; a family whose provider refused would also be absent from `changed`.) And the
/// edit is no longer total: two of the seventy-one stay skipped, the whole-workspace rules
/// whose own families a source edit cannot reach.
///
/// Both readings are deliberately not asserted. Whether an edit should invalidate the rules
/// that cannot see it is a question about the materialization, and pinning today's answer in
/// this test would freeze it. `0c3b1013`'s own commit message read 71, then plus 19, then plus
/// 68 over a different table, which is the other reason these are inequalities.
#[test]
fn Test_A_Retained_Cache_Under_The_Production_Selection_Should_Skip_Most_Of_The_Composed_Table()
{
    let unedited = [
        Source_File("a.rs", SourceText("pub fn Ok() {}\n")),
        Source_File("b.rs", SourceText("pub fn Also_Ok() {}\n")),
    ];
    let edited = [
        Source_File("a.rs", SourceText("pub fn Ok() {}\npub fn Newly_Added() {}\n")),
        Source_File("b.rs", SourceText("pub fn Also_Ok() {}\n")),
    ];

    let mut fixture = ReassessmentFixture::New();

    let _cold_outcome = fixture.Reassess(&unedited, EVERY_COMPOSED_RULE);
    let cold = fixture.Recorded();
    let _unchanged_outcome = fixture.Reassess(&unedited, EVERY_COMPOSED_RULE);
    let unchanged = fixture.Recorded().saturating_sub(cold);
    let _edited_outcome = fixture.Reassess(&edited, EVERY_COMPOSED_RULE);
    let after_edit = fixture.Recorded().saturating_sub(cold).saturating_sub(unchanged);

    assert!(
        cold > 0,
        "the cold call recorded no rule invocation at all, so the two comparisons below would \
         compare nothing -- the empty selection must mean every composed rule, not none"
    );
    assert!(
        unchanged < cold,
        "a second call over unmoved sources must run strictly fewer rules than the cold one: \
         cold ran {cold}, unchanged ran {unchanged}. Equal counts mean nothing skipped, which is \
         what a cache threaded and never consulted looks like"
    );
    assert!(
        after_edit > unchanged,
        "editing one of the two sources must put more rules back in play than an unchanged call \
         does: unchanged ran {unchanged}, the edited call ran {after_edit}"
    );
}

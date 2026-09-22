//! What this module promises, exercised: `Run` composes and judges, `Run_Reassessing`
//! reuses a rule whose families held still, and `Recognized_Sources` enriches a path with
//! the syntax provider that will read it.

use super::*;
use nomos_contracts::ProviderId;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
// Named here rather than beside the module's own imports: production code no longer
// names either rule, and putting them back up there to satisfy a test would undo exactly
// what `OD-RULES-027` and `P102` each removed -- the second of them by deriving the
// rule-to-family relation from `DESCRIPTORS`, which is what left fifteen module-scope
// imports here naming nothing.
use nomos_rules::{COMPLETENESS_MIRROR, REQUIREMENT_TRACE_STALENESS};

/// The two rules every reassessment test below selects, and why exactly these two.
///
/// `COMPLETENESS_MIRROR` declares `RequiredFact::SyntaxItems` and judges a walked source, so
/// it is the control: an edit to one of the sources must always put it back in play.
/// `REQUIREMENT_TRACE_STALENESS` declares `RequiredFact::RequirementTrace` and its subject is
/// `nomos_rules::SubjectKind::Workspace`, so one source's text moving is not a change to what
/// it read -- it is the rule under test.
///
/// The requirement-trace family rather than any of the seven `standards.json`-backed ones
/// because `nomos_cap_requirement_trace::Materialize_Workspace` is infallible by its own
/// design: it produces a fact over any root, including one with no
/// `tests/contract/requirements/` directory at all. So the first call below genuinely
/// materializes the fact and genuinely reports the family as changed, and the skip on the
/// second is a skip of a real write rather than the vacuous silence of a family that never
/// materialized.
const REASSESSED_RULES: [&str; 2] = [COMPLETENESS_MIRROR, REQUIREMENT_TRACE_STALENESS];

/// [`REASSESSED_RULES`] as `Run_Reassessing` takes a selection.
fn Reassessed_Selection() -> Vec<RuleId>
{
    return REASSESSED_RULES.iter().map(|rule| return RuleId::New(*rule)).collect();
}

/// How many real closures a cold call over [`REASSESSED_RULES`] must record: one per selected
/// rule, since nothing is cached the first time.
fn Reassessed_Rule_Count() -> u32
{
    return u32::try_from(REASSESSED_RULES.len()).expect("two rules fit in a u32");
}

/// `root` is never read: `COMPLETENESS_MIRROR` alone selects none of the
/// dependency-edges, lint-diagnostics or dependency-policy materializations, so this
/// stays a fast, self-contained proof of `Run`'s own composing-and-judging contract
/// rather than a second real-repository integration test -- `src/tests.rs` already
/// carries that one, over the real syntax provider and the real rule.
#[test]
fn Test_Run_Should_Judge_A_Clean_Source_With_No_Findings()
{
    let sources = vec![SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n")];
    let selected = [RuleId::New(COMPLETENESS_MIRROR)];
    let mut workspace = None;
    let mut store = MemoryFactStore::New();

    let context = Test_Context(Path::new("."), &mut workspace, &mut store);
    let outcome = Run(&sources, context, &selected);

    let CheckOutcome::Judged { findings, examined, claim, .. } = outcome
    else
    {
        // this fixture's own source is well-formed and the provider recognizes it; a
        // refusal here is a bug in the test's own setup, not a caller-facing failure.
        panic!("a tree the provider can read must be judged");
    };
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(examined, crate::examined::Examined { files: 1, facts: 1 });
    assert_eq!(claim, crate::examined::Claim::Complete);
}

/// `P40-INCREMENTAL-SKIP-UNCHANGED-RULES`'s own `done_when`: a second [`Run_Reassessing`]
/// call over a workspace, store and cache all reused from the first, with the same
/// selection and no source changed at all, must run neither selected rule's closure again.
///
/// Both, now. Before `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY` this test asserted the
/// syntax-only rule was skipped and the policy-declaring one ran again, because its family's
/// own materializer re-filed its fact on every call whether the fact had moved or not, and
/// that re-file reported the family as changed. The policy fact has not moved between these
/// two calls and is no longer re-filed, so it is no longer reported as changed and the rule
/// reading it is no longer re-judged.
///
/// That leaves this test without an in-selection control against the whole table going
/// quiet, which is why it is paired with
/// [`Test_Run_Reassessing_Should_Skip_A_Policy_Declaring_Workspace_Rule_When_Only_An_Unrelated_Source_Changed`]
/// below: the same two rules, the same fixture, an edit to one source, and one of the two
/// running again. A cache that skipped everything unconditionally passes this test and fails
/// that one.
#[test]
fn Test_Run_Reassessing_Should_Skip_Every_Selected_Rule_When_Nothing_Changed()
{
    let sources = Reassessed_Sources();
    let selected = Reassessed_Selection();
    let mut fixture = Test_Reassessing_Fixture();

    let first = Reassessing_Outcome(&sources, &selected, &mut fixture);
    assert!(matches!(first, CheckOutcome::Judged { .. }), "a tree the provider can read must be judged");
    let recorded_after_first = fixture.reassessment.Recorded();
    assert_eq!(recorded_after_first, Reassessed_Rule_Count(), "both selected rules must run their real closure the first time, with nothing yet cached");

    let second = Reassessing_Outcome(&sources, &selected, &mut fixture);
    assert!(matches!(second, CheckOutcome::Judged { .. }), "a tree the provider can read must be judged");
    assert_eq!(
        fixture.reassessment.Recorded(), recorded_after_first,
        "nothing about the sources, the selection or the repository's own declarations moved between the two calls, \
         so neither rule's prior findings could have gone stale and neither closure should have run again"
    );
}

/// `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY`'s own `done_when`, and the whole point of
/// giving a policy family a currency check: a rule that declares a policy family and judges
/// the workspace rather than a source is *not* re-judged when only a source changed.
///
/// The count is the evidence and nothing else can be. `REQUIREMENT_TRACE_STALENESS` reads one
/// whole-workspace fact that did not move, so a skipped invocation and a real one report
/// identical findings -- which is the reason `RuleReassessmentCache::Recorded` exists at all.
///
/// Before the currency check, the third call here recorded two invocations rather than one:
/// `Materialize_Requirement_Trace` re-filed a byte-identical fact, the family landed in
/// `changed`, and a rule that could not see the edited source was re-judged because of it.
/// That is the measurement `P40-INCREMENTAL-DEMAND-DRIVEN-RECOMPUTE-2`'s decline recorded,
/// stated as a test.
#[test]
fn Test_Run_Reassessing_Should_Skip_A_Policy_Declaring_Workspace_Rule_When_Only_An_Unrelated_Source_Changed()
{
    let selected = Reassessed_Selection();
    let mut fixture = Test_Reassessing_Fixture();

    let first = Reassessing_Outcome(&Reassessed_Sources(), &selected, &mut fixture);
    assert!(matches!(first, CheckOutcome::Judged { .. }), "a tree the provider can read must be judged");
    let recorded_after_first = fixture.reassessment.Recorded();
    assert_eq!(recorded_after_first, Reassessed_Rule_Count(), "both selected rules must run their real closure the first time, with nothing yet cached");

    let edited = Reassessing_Outcome(&Reassessed_Sources_With_One_Edited(), &selected, &mut fixture);
    assert!(matches!(edited, CheckOutcome::Judged { .. }), "a tree the provider can read must be judged");
    assert_eq!(
        fixture.reassessment.Recorded(), recorded_after_first + 1,
        "exactly one closure should have run again: the source-judging rule, because the text it judges moved. \
         A count of two means the whole-workspace rule was re-judged for an edit it cannot see, which is the \
         cost a family that re-files an unmoved fact imposes; a count of zero means the source edit did not \
         invalidate the rule that reads it"
    );
}

/// The two sources every reassessment test above judges. Two rather than one so that an edit
/// to the first leaves a second, genuinely untouched subject beside it.
fn Reassessed_Sources() -> Vec<SourceFile>
{
    return vec![
        SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n"),
        SourceFile::New("b.rs", nomos_model::Subject_Of_Path("b.rs"), "pub fn Also_Ok() {}\n"),
    ];
}

/// [`Reassessed_Sources`] with the first source's own text moved and the second left exactly
/// as it was -- the "only an unrelated source changed" the test above names.
fn Reassessed_Sources_With_One_Edited() -> Vec<SourceFile>
{
    return vec![
        SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\npub fn Newly_Added() {}\n"),
        SourceFile::New("b.rs", nomos_model::Subject_Of_Path("b.rs"), "pub fn Also_Ok() {}\n"),
    ];
}

/// `OD-CAPABILITY-009`'s corrected fix, exercised directly: a `.rs` path's enrichment
/// must resolve to `nomos_lang_rust`'s own identity, the same identity
/// [`crate::facts::dependency_materialization::Materialize_Syntax`]'s write side
/// dispatches on, so the two sides agree by construction.
#[test]
fn Test_Recognized_Sources_Should_Populate_Preferred_Syntax_Provider()
{
    let sources = vec![SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n")];

    let recognized = Recognized_Sources(&sources, &crate::composition::provider_table::Composed_Syntax_Providers());

    assert_eq!(
        recognized.first().expect("one source in, one source out").preferred_syntax_provider,
        Some(ProviderId::New(nomos_lang_rust::PROVIDER))
    );
}

/// A path neither syntax provider recognizes must enrich to `None` rather than to a
/// guess -- the same "carried rather than derived" contract
/// [`Recognized_Syntax_Provider`] states for the field this populates.
#[test]
fn Test_Recognized_Sources_Should_Leave_An_Unrecognized_Path_With_No_Preferred_Provider()
{
    let sources = vec![SourceFile::New("readme.md", nomos_model::Subject_Of_Path("readme.md"), "# hi\n")];

    let recognized = Recognized_Sources(&sources, &crate::composition::provider_table::Composed_Syntax_Providers());

    assert_eq!(recognized.first().expect("one source in, one source out").preferred_syntax_provider, None);
}

/// One `Run_Reassessing` call over a caller-held [`ReassessingFixture`], composed the
/// identical way the test above assembles it -- extracted because the two calls it makes
/// differ only in what the caller asserts about them, not in how they are made.
fn Reassessing_Outcome(sources: &[SourceFile], selected: &[RuleId], fixture: &mut ReassessingFixture) -> CheckOutcome
{
    let context = Test_Context(Path::new("."), &mut fixture.workspace, &mut fixture.store);
    return Run_Reassessing(sources, context, selected, &mut fixture.reassessment);
}

/// The state a `Run_Reassessing` call reads and writes and that outlives it -- the workspace
/// cache, the fact store and the reassessment cache, all three held across two calls by the
/// test above. Grouped because they travel together through every call that test makes, and
/// because passing them separately is five positional parameters, one past this crate's own
/// `parameter-count` limit.
struct ReassessingFixture
{
    workspace: Option<Workspace>,
    store: MemoryFactStore,
    reassessment: RuleReassessmentCache,
}

fn Test_Reassessing_Fixture() -> ReassessingFixture
{
    return ReassessingFixture { workspace: None, store: MemoryFactStore::New(), reassessment: RuleReassessmentCache::New() };
}

/// The [`RunContext`] every test above builds: this crate's own standard launcher,
/// filesystem and environment, a fresh store, and the test build variant. Extracted rather
/// than written at each call site because a real one is eleven lines, which is more than
/// the assertion it feeds.
fn Test_Context<'a>(root: &'a Path, workspace: &'a mut Option<Workspace>, store: &'a mut MemoryFactStore) -> RunContext<'a, StdProgramLauncher, StdFileSystem, StdEnvironment>
{
    return RunContext {
        variant: Test_Variant(),
        root,
        launcher: &StdProgramLauncher,
        filesystem: &StdFileSystem,
        environment: &StdEnvironment,
        workspace,
        store,
    };
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

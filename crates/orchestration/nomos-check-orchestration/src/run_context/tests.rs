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
use nomos_rules::{COMPLETENESS_MIRROR, FILE_SIZE_JUSTIFICATION_TRIGGER};

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

    let CheckOutcome::Judged { findings, examined, claim } = outcome
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

/// `P40-INCREMENTAL-SKIP-UNCHANGED-RULES`'s own done_when: a second [`Run_Reassessing`]
/// call over a workspace, store and cache all reused from the first, with the same
/// selection and no source changed at all, must not run `COMPLETENESS_MIRROR`'s closure
/// again -- it requires only `nomos.cap.syntax.items`, and nothing in that family moved.
/// `FILE_SIZE_JUSTIFICATION_TRIGGER` requires `nomos.cap.limits.policy`, whose own
/// materializer (`crate::facts::policy_materialization`) has no currency check yet, so
/// it must run again regardless -- proving this is a real skip of a real closure and not
/// an accident of the whole rule table going quiet.
#[test]
fn Test_Run_Reassessing_Should_Skip_A_Syntax_Only_Rule_When_Nothing_Changed_And_Rerun_One_Whose_Family_Has_No_Currency_Check()
{
    let sources = vec![
        SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n"),
        SourceFile::New("b.rs", nomos_model::Subject_Of_Path("b.rs"), "pub fn Also_Ok() {}\n"),
    ];
    let selected = [RuleId::New(COMPLETENESS_MIRROR), RuleId::New(FILE_SIZE_JUSTIFICATION_TRIGGER)];
    let mut fixture = Test_Reassessing_Fixture();

    let first = Reassessing_Outcome(&sources, &selected, &mut fixture);
    assert!(matches!(first, CheckOutcome::Judged { .. }), "a tree the provider can read must be judged");
    let recorded_after_first = fixture.reassessment.Recorded();
    assert_eq!(recorded_after_first, selected.len() as u32, "both selected rules must run their real closure the first time, with nothing yet cached");

    let second = Reassessing_Outcome(&sources, &selected, &mut fixture);
    assert!(matches!(second, CheckOutcome::Judged { .. }), "a tree the provider can read must be judged");
    assert_eq!(
        fixture.reassessment.Recorded(), recorded_after_first + 1,
        "only the limits-policy rule should have run its closure again; the syntax-only rule's prior findings should have been reused"
    );
}

/// `OD-CAPABILITY-009`'s corrected fix, exercised directly: a `.rs` path's enrichment
/// must resolve to `nomos_lang_rust`'s own identity, the same identity
/// [`crate::facts::dependency_materialization::Materialize_Syntax`]'s write side
/// dispatches on, so the two sides agree by construction.
#[test]
fn Test_Recognized_Sources_Should_Populate_Preferred_Syntax_Provider()
{
    let sources = vec![SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n")];

    let recognized = Recognized_Sources(&sources);

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

    let recognized = Recognized_Sources(&sources);

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

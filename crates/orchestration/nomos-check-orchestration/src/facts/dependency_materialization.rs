//! Materializing facts from already-walked source: syntax, reachability and dependency
//! edges.
//!
//! Grouped by what they share -- pure, per-source materialization over a [`Context`] this
//! module does not build -- not by the capability each one answers; see each function's own
//! doc for why the three stayed independent steps rather than one generalization.
//!
//! # One submodule per capability
//!
//! Six public behaviours sharing the `Materialize` prefix named a boundary this module had
//! not drawn: reading source, writing facts and reporting a provider's refusal are three
//! different jobs that happened to share a spelling, not one responsibility. Each
//! capability's own step now sits in the submodule named for it -- `syntax`,
//! `dependencies`, `lint_materialization`, `policy_materialization`,
//! `review_materialization` -- and the one type they all need, `Subprocess`, in
//! `subprocess`. What stays here is the module's own doc, the `DependencyMaterialization`
//! `dependencies` and its callers both name, and the tests.

mod dependencies;
mod lint_materialization;
mod policy_materialization;
mod review_materialization;
mod subprocess;
mod syntax;

pub use dependencies::Materialize_Dependencies;
pub use lint_materialization::{LintMaterialization, Materialize_Lint};
pub use policy_materialization::{Materialize_Policy, PolicyMaterialization};
pub use review_materialization::{Materialize_Review, ReviewMaterialization};
pub use subprocess::Subprocess;
pub use syntax::{Materialize_Reachability, Materialize_Syntax};

use nomos_analysis::MemoryFactStore;
use nomos_contracts::Finding;
use nomos_rules::SourceFile;

/// What materializing `dependency.edges` facts produced: the sources a rule can judge them
/// under, and any finding the materialization itself already raised (a failed `cargo
/// metadata` call, reported rather than judged) -- named rather than left as a positional
/// pair, so a caller reads which is which without re-deriving it from
/// [`Materialize_Dependencies`]'s own body.
pub struct DependencyMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}

/// What [`Materialize_Through`] hands back before a caller renames it as its own answer: the
/// pair of lists every one of the three wrappers carries, named here so the helper returns a
/// type rather than the tuple it started as.
struct Materialized
{
    sources: Vec<SourceFile>,
    findings: Vec<Finding>,
}

/// The one sequence three provider-backed materializations had each spelled out in full:
/// call the capability, turn its refusal into a finding rather than into an empty answer,
/// file what it returned, and hand back the pair a caller reads.
///
/// `dependencies`, `lint_materialization` and `policy_materialization` were the same four
/// statements with every identifier changed. What genuinely differs between them is only the
/// provider, that provider's error type, the rule and wording of the finding a refusal
/// produces, and the type the caller reads the answer as -- so those are the parameters, and
/// the sequence lives here once.
///
/// `sources_of` stays the caller's because the two shapes really are different: `cargo
/// metadata` and `cargo clippy` answer one fact per workspace member, while `cargo deny`
/// answers exactly one for the whole workspace. Both end as a [`SourceFile`] list because
/// `run_context::Judged` calls every rule uniformly over one.
///
/// Private, and deliberately not `pub(super)`: this is the module's own seam between its
/// children, not something a caller outside it should reach.
fn Materialize_Through<Answer, ProviderError, Call, Sources, Unavailable>(
    call: Call,
    store: &mut MemoryFactStore,
    sources_of: Sources,
    unavailable: Unavailable,
) -> Materialized
where
    Call: FnOnce() -> Result<Answer, ProviderError>,
    Sources: FnOnce(Answer, &mut MemoryFactStore) -> Vec<SourceFile>,
    Unavailable: FnOnce(&ProviderError) -> Finding,
{
    let materialized = match call()
    {
        Ok(materialized) => materialized,
        Err(error) => return Materialized { sources: Vec::new(), findings: vec![unavailable(&error)] },
    };

    return Materialized { sources: sources_of(materialized, store), findings: Vec::new() };
}

#[cfg(test)]
mod tests
{
    //! What this module promises, exercised.

    use super::*;
    use nomos_analysis::{Context, MemoryFactStore};
    use nomos_contracts::RuleId;
    use nomos_platform::Command;
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use nomos_platform_std::StdEnvironment;
    use nomos_workspace::BuildVariant;
    use std::path::PathBuf;

    /// How many sources each test below feeds, all of them recognized: a fresh store must
    /// report a current fact for every one, and every one must still have one after a
    /// second call over the same store.
    const RECOGNIZED_SOURCES: usize = 2;

    /// A recognized, parseable source materializes exactly one fact; a path neither
    /// `nomos_lang_rust` nor `nomos_lang_go` recognizes materializes nothing and is not
    /// dropped silently -- the count this function returns is the denominator a caller
    /// reports it against.
    #[test]
    fn Test_Materialize_Syntax_Should_Write_A_Fact_Only_For_A_Recognized_Source()
    {
        for (path, text, expected_written) in Materialize_Syntax_Cases()
        {
            let sources = [SourceFile::New(path, nomos_model::Subject_Of_Path(path), text)];
            let context = Fixture_Context(&sources);
            let mut store = MemoryFactStore::New();

            let written = Materialize_Syntax(&sources, &context, &mut store);

            assert_eq!(written, expected_written, "{path}");
        }
    }

    /// `(path, text, expected_written)` -- a recognized, parseable source against an
    /// unrecognized one, so a case added later (an unparseable-but-recognized path, say)
    /// is one more row rather than one more copy of the test function.
    fn Materialize_Syntax_Cases() -> Vec<(&'static str, &'static str, usize)>
    {
        return vec![
            ("a.rs", "pub fn Ok() {}\n", 1),
            ("readme.md", "# hi\n", 0),
        ];
    }

    /// `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS`'s own done_when: a second `Materialize_
    /// Syntax` call over a reused `Workspace` and `MemoryFactStore` writes nothing new
    /// for a subject whose bytes did not move, and exactly one new fact for the one that
    /// did -- not two, which is what an unconditional re-materialization of both sources
    /// on every call would report instead.
    #[test]
    fn Test_Materialize_Syntax_Should_Skip_An_Unchanged_Subject_On_A_Reused_Store_And_Workspace()
    {
        let unchanged = SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn One() {}\n".to_owned());
        let changed_before = SourceFile::New("b.rs", nomos_model::Subject_Of_Path("b.rs"), "pub fn Two() {}\n".to_owned());
        let changed_after = SourceFile::New("b.rs", nomos_model::Subject_Of_Path("b.rs"), "pub fn Two_Renamed() {}\n".to_owned());

        let mut workspace = None;
        let mut store = MemoryFactStore::New();

        let first_sources = [unchanged.clone(), changed_before];
        let first = Materialized_Syntax_Run(&first_sources, &mut workspace, &mut store);
        assert_eq!(first.current, RECOGNIZED_SOURCES, "a fresh store must report a current fact for both real sources");

        let second_sources = [unchanged, changed_after];
        let second = Materialized_Syntax_Run(&second_sources, &mut workspace, &mut store);

        assert_eq!(second.current, RECOGNIZED_SOURCES, "both sources still have a current fact after the second call, one of them reused rather than rewritten");
        assert_eq!(
            second.materializations, first.materializations.saturating_add(1),
            "only the changed subject's fact should be newly written to the store; the unchanged one must be skipped"
        );
    }

    /// What one [`Materialized_Syntax_Run`] call answered, each half named rather than left
    /// as a positional pair: `current` is how many of the sources that call was given now
    /// have a current fact, and `materializations` is how many facts `store` holds in
    /// total, so a caller comparing two runs cannot read one for the other.
    struct SyntaxRunResult
    {
        current: usize,
        materializations: u32,
    }

    /// One `Materialize_Syntax` call over `sources` on a caller-held `workspace` and
    /// `store`, answering both questions
    /// [`Test_Materialize_Syntax_Should_Skip_An_Unchanged_Subject_On_A_Reused_Store_And_Workspace`]
    /// asks of a call -- how many of `sources` now have a current fact, and how many facts
    /// `store` has materialized in total -- so that test's own body is a sequence of
    /// comparisons rather than a second copy of the call's setup.
    fn Materialized_Syntax_Run(
        sources: &[SourceFile],
        workspace: &mut Option<nomos_workspace::Workspace>,
        store: &mut MemoryFactStore,
    ) -> SyntaxRunResult
    {
        let context = Reused_Fixture_Context(sources, workspace);
        let current = Materialize_Syntax(sources, &context, store);

        return SyntaxRunResult { current, materializations: store.Materializations() };
    }

    /// [`Fixture_Context`]'s own composition, over a `workspace` the caller keeps and
    /// passes again -- what a real second `Run` call reuses, so its own generation
    /// advances only when `sources`' own content actually moved since the last call
    /// ingested it.
    fn Reused_Fixture_Context(sources: &[SourceFile], workspace: &mut Option<nomos_workspace::Workspace>) -> Context
    {
        let registry = crate::composition::Registered().expect("fixture composition");
        return crate::facts::Ingested_Workspace(sources, &registry, Test_Variant(), workspace).expect("the fixture is a valid tree");
    }

    /// The identical shape [`Materialize_Syntax`]'s own first test proves, for
    /// `nomos.cap.controlflow.reachability`: one well-formed Rust source materializes
    /// exactly one fact.
    #[test]
    fn Test_Materialize_Reachability_Should_Write_One_Fact_Per_Source()
    {
        let sources = [SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn One() {}\n")];
        let context = Fixture_Context(&sources);
        let mut store = MemoryFactStore::New();

        let written = Materialize_Reachability(&sources, &context, &mut store);

        assert_eq!(written, 1, "a single well-formed Rust source must materialize exactly one reachability fact");
    }

    /// A launcher that cannot even be run -- no real `cargo` invocation, so this stays
    /// fast and deterministic -- proving the failure path each of the three subprocess
    /// materializations below shares: a failed launch must report a finding rather than
    /// silently read as "zero findings", which is exactly the vacuity
    /// [`Materialize_Syntax`]'s own `NoFacts` case exists to catch one layer over.
    struct RefusingLauncher;

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for RefusingLauncher
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl nomos_platform::ProgramLauncher for RefusingLauncher
    {
        fn Run(&self, _command: &Command) -> Result<nomos_platform::ProgramOutput, String>
        {
            return Err("refused for this test".to_owned());
        }
    }

    #[test]
    fn Test_Materialize_Dependencies_Should_Report_A_Finding_When_The_Launcher_Refuses()
    {
        let RefusedLaunchFixture { context, mut store } = Refused_Launch_Fixture();

        let materialized = Materialize_Dependencies(&PathBuf::from("."), &context, &mut store, Subprocess { launcher: &RefusingLauncher, environment: &StdEnvironment });

        Assert_Refused_Launch_Reported(&materialized.sources, &materialized.findings, nomos_rules::DEPENDENCY_DIRECTION);
    }

    /// The identical claim
    /// [`Test_Materialize_Dependencies_Should_Report_A_Finding_When_The_Launcher_Refuses`]
    /// proves, for `nomos.cap.lint.diagnostics` and `cargo clippy`.
    #[test]
    fn Test_Materialize_Lint_Should_Report_A_Finding_When_The_Launcher_Refuses()
    {
        let RefusedLaunchFixture { context, mut store } = Refused_Launch_Fixture();

        let materialized = Materialize_Lint(&PathBuf::from("."), &context, &mut store, Subprocess { launcher: &RefusingLauncher, environment: &StdEnvironment });

        Assert_Refused_Launch_Reported(&materialized.sources, &materialized.findings, nomos_rules::LINT_DIAGNOSTICS);
    }

    /// The identical claim
    /// [`Test_Materialize_Dependencies_Should_Report_A_Finding_When_The_Launcher_Refuses`]
    /// proves, for `nomos.cap.dependency.policy` and `cargo deny`.
    #[test]
    fn Test_Materialize_Policy_Should_Report_A_Finding_When_The_Launcher_Refuses()
    {
        let RefusedLaunchFixture { context, mut store } = Refused_Launch_Fixture();

        let materialized = Materialize_Policy(&PathBuf::from("."), &context, &mut store, Subprocess { launcher: &RefusingLauncher, environment: &StdEnvironment });

        Assert_Refused_Launch_Reported(&materialized.sources, &materialized.findings, nomos_rules::DEPENDENCY_POLICY);
    }

    /// The whole of what the three tests above each claim: a refused launch reported
    /// exactly one finding, naming `expected_rule`, and no sources at all. Shared because
    /// the three differ in which materialization they drive and which rule they expect --
    /// the setup and the assertions were pure, identical boilerplate, and a divergence
    /// between the three is still caught here, where each supplies its own expectation.
    fn Assert_Refused_Launch_Reported(sources: &[SourceFile], findings: &[Finding], expected_rule: &str)
    {
        assert!(sources.is_empty(), "a refused launch must not report workspace members");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("the assertion above proves one finding was reported").rule, RuleId::New(expected_rule));
    }

    /// [`Refused_Launch_Fixture`]'s two halves, named rather than a tuple: a caller reading
    /// `.context` or `.store` at the point of use does not have to hold the fixture's own
    /// field order in mind.
    struct RefusedLaunchFixture
    {
        context: Context,
        store: MemoryFactStore,
    }

    /// The identical context-and-store fixture each launcher-refusal test above needs: one
    /// recognized source is enough since none of the three cases inspects what was
    /// materialized, only that the refusal itself is reported. Extracted because the three
    /// tests above verify three separate materialization functions against three separate
    /// expected rules -- merging the assertions would mask a real divergence between them,
    /// but the setup itself was pure, identical boilerplate.
    fn Refused_Launch_Fixture() -> RefusedLaunchFixture
    {
        let sources = [SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n")];
        let context = Fixture_Context(&sources);
        let store = MemoryFactStore::New();

        return RefusedLaunchFixture { context, store };
    }

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
    }

    /// What every function above needs and none of them build: a real, ingested
    /// [`Context`] over `sources` -- the identical two-step composition
    /// `src/tests.rs`'s own `Findings_Over` assembles, restated here because a colocated
    /// test cannot reach that file's private helper.
    fn Fixture_Context(sources: &[SourceFile]) -> Context
    {
        let registry = crate::composition::Registered().expect("fixture composition");
        return crate::facts::Ingested_Workspace(sources, &registry, Test_Variant(), &mut None).expect("the fixture is a valid tree");
    }
}

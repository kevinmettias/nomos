//! Materializing facts from already-walked source: syntax, reachability and dependency
//! edges.
//!
//! Grouped by what they share -- pure, per-source materialization over a [`Context`] this
//! module does not build -- not by the capability each one answers; see each function's own
//! doc for why the three stayed independent steps rather than one generalization.

mod lint_materialization;
mod policy_materialization;
mod review_materialization;

pub use lint_materialization::LintMaterialization;
pub use policy_materialization::PolicyMaterialization;
pub use review_materialization::ReviewMaterialization;

use nomos_analysis::{Context, FactKey, FactStore, GuaranteeDigest, InputDigest, MaterializedFact, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, ProviderId, RuleId};
use nomos_lang_rust::{FactContext, Materialization};
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;

use std::path::Path;

/// Produces one syntax fact per source and returns how many sources now have a current
/// one -- whether this call wrote it or [`Already_Current`] found the store already held
/// it. `crate::run_context::Materialized_Syntax_Facts`'s own vacuity gate and
/// `Examined::facts`'s own coverage statistic both read this as "how many of `sources`
/// this run can judge," and neither means "how much work this call actually did" --
/// [`nomos_analysis::MemoryFactStore::Materializations`]'s own delta across two calls is
/// the honest answer to that question, and what `P40-INCREMENTAL-SKIP-UNCHANGED-
/// SUBJECTS`'s own test now reads instead (`P40-INCREMENTAL-SKIP-COVERAGE-REGRESSION`
/// fixed this return value back to the coverage meaning every real caller already
/// depended on, after briefly repurposing it to mean the newly-written count with nothing
/// downstream updated to match).
///
/// A file neither provider recognizes, or one its own recognized provider refuses to
/// parse, materializes nothing and is not dropped silently: the count returned is the
/// denominator the report prints beside the file count, and the rule's own unread-subject
/// finding names each one individually. Two independent readings of one file -- this
/// provider's and `nomos-rules`' own universe parser -- can refuse independently, and the
/// run is entitled to see which.
///
/// Dispatches per [`Recognized_Syntax_Provider`], `OD-CAPABILITY-009`'s write-side half:
/// `crate::run_context`'s own enrichment step narrows the read side to the same provider identity
/// through the identical function, and the two must agree for `Key_From` to ever find what
/// this function wrote.
///
/// # Skipping a subject the store already has current
///
/// `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS`: before parsing, [`Already_Current`] builds
/// the exact [`FactKey`] parsing `source` would produce -- from `source.text`'s own raw
/// digest, never from a parse -- and asks `store` whether it already holds a live fact
/// under it. A caller that reuses the same `store` (and therefore the same, monotonically
/// advancing generation `crate::facts::Ingested_Workspace` reads off a reused `Workspace`)
/// across two calls pays for a real parse only for a subject whose own bytes moved since
/// the store last saw it -- but still counts that subject as covered either way.
pub fn Materialize_Syntax(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> usize
{
    let rust_production = Rust_Production(context);
    let go_production = Go_Production(context);
    let mut current = 0_usize;

    for source in sources
    {
        if Already_Current(source, context, store)
        {
            current = current.saturating_add(1);
            continue;
        }

        let Some(fact) = Materialized_Syntax_Fact(source, rust_production, go_production)
        else
        {
            continue;
        };

        // No dependency edges: a syntax fact is a leaf, read from one file's bytes and
        // from nothing this store holds.
        if store.Materialize(*fact, &[]).is_ok()
        {
            current = current.saturating_add(1);
        }
    }

    return current;
}

/// Whether `store` already holds a live `nomos.cap.syntax.items` fact for `source` at
/// `context.generation` -- `false` for a path neither provider recognizes, so an
/// unrecognized source always falls through to [`Materialized_Syntax_Fact`]'s own,
/// unchanged "not dropped silently" handling.
///
/// Built from the same public pieces `nomos_lang_rust`/`nomos_lang_go`'s own provider
/// combines into a [`FactKey`] internally (`Capability`, `CONTRACT_VERSION`, `PROVIDER`,
/// `Declared_Guarantee`), restated here rather than exposed as a shared constructor: this
/// is the one caller outside either provider that ever needs to know what a fact it did
/// not produce would be keyed under, and [`InputDigest::Of`] over `source.text`'s own
/// bytes is a raw hash, not a parse -- the whole reason this check can run before one.
fn Already_Current(source: &SourceFile, context: &Context, store: &MemoryFactStore) -> bool
{
    use crate::composition::Recognized_Syntax_Provider;

    let Some(provider) = Recognized_Syntax_Provider(&source.path)
    else
    {
        return false;
    };
    let Some(key) = Syntax_Fact_Key(source, &provider, context)
    else
    {
        return false;
    };

    return store.Current(&key.At(context.generation), context.generation).is_some();
}

/// The [`FactKey`] a real materialization of `source` through `provider` would produce --
/// `None` if `provider` is neither of the two this capability has.
fn Syntax_Fact_Key(source: &SourceFile, provider: &ProviderId, context: &Context) -> Option<FactKey>
{
    let guarantee = if *provider == ProviderId::New(nomos_lang_rust::PROVIDER)
    {
        nomos_lang_rust::Declared_Guarantee()
    }
    else if *provider == ProviderId::New(nomos_lang_go::PROVIDER)
    {
        nomos_lang_go::Declared_Guarantee()
    }
    else
    {
        return None;
    };

    return Some(FactKey {
        contract: nomos_cap_syntax::Capability(),
        contract_version: nomos_cap_syntax::CONTRACT_VERSION,
        subject: source.subject,
        semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
        provider: provider.clone(),
        provider_version: nomos_cap_syntax::CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    });
}

/// The reading context as `nomos_lang_go`'s own provider takes it -- the identical fields
/// [`Rust_Production`] already builds for `nomos_lang_rust`'s own distinct `FactContext`
/// type, restated as `nomos_lang_go`'s.
fn Go_Production(context: &Context) -> nomos_lang_go::FactContext
{
    return nomos_lang_go::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// One source's syntax fact, from whichever provider [`Recognized_Syntax_Provider`] says
/// `source.path` belongs to -- `None` for a path neither recognizes or a recognized path
/// its own provider could not parse.
///
/// Dispatches on [`Recognized_Syntax_Provider`] rather than recomputing `Recognition::Of_Path`
/// itself, the identical function `crate::run_context`'s own read-side enrichment calls to populate
/// `nomos_rules::SourceFile::preferred_syntax_provider` -- `OD-CAPABILITY-009`'s corrected
/// fix, one test both sides consult so they cannot independently drift.
fn Materialized_Syntax_Fact(
    source: &SourceFile,
    rust_production: FactContext,
    go_production: nomos_lang_go::FactContext,
) -> Option<Box<MaterializedFact>>
{
    use crate::composition::Recognized_Syntax_Provider;

    let provider = Recognized_Syntax_Provider(&source.path)?;

    if provider == ProviderId::New(nomos_lang_rust::PROVIDER)
    {
        return Rust_Syntax_Fact(source, rust_production);
    }

    if provider == ProviderId::New(nomos_lang_go::PROVIDER)
    {
        return Go_Syntax_Fact(source, go_production);
    }

    return None;
}

/// `source`'s syntax fact from `nomos_lang_rust`'s own provider, or `None` if it refused to
/// parse.
fn Rust_Syntax_Fact(source: &SourceFile, rust_production: FactContext) -> Option<Box<MaterializedFact>>
{
    let Materialization::Materialized(fact) = nomos_lang_rust::Materialize_Syntax_Fact(source.subject, &source.text, rust_production)
    else
    {
        return None;
    };

    return Some(fact);
}

/// `source`'s syntax fact from `nomos_lang_go`'s own provider, or `None` if it refused to
/// parse.
fn Go_Syntax_Fact(source: &SourceFile, go_production: nomos_lang_go::FactContext) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_go::Materialization::Materialized(fact) = nomos_lang_go::Materialize_Syntax_Fact(source.subject, &source.text, go_production)
    else
    {
        return None;
    };

    return Some(fact);
}

/// Produces one `nomos.cap.controlflow.reachability` fact per source and returns how many
/// were written.
///
/// The identical shape [`Materialize_Syntax`] has: pure, per-source, no I/O --
/// `nomos_lang_rust::reachability::Materialize_Reachability_Fact` takes only a subject, source text and this
/// same [`FactContext`], the tier-1 heuristic provider `P13-CONTROLFLOW-REACHABILITY-
/// CAPABILITY` built. Not folded into `Materialize_Syntax` itself: the two are independent
/// capabilities read from the same bytes, and a shared materialization step would hide
/// which one a caller is actually asking about, the same reason [`Materialize_Dependencies`]
/// stayed a second, separate step rather than a generalization of the first.
pub fn Materialize_Reachability(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> usize
{
    let production = Rust_Production(context);
    let mut written = 0_usize;

    for source in sources
    {
        let Materialization::Materialized(fact) =
            nomos_lang_rust::reachability::Materialize_Reachability_Fact(source.subject, &source.text, production)
        else
        {
            continue;
        };

        // No dependency edges: a reachability fact is a leaf, the identical reasoning
        // Materialize_Syntax gives for its own fact.
        if store.Materialize(*fact, &[]).is_ok()
        {
            written = written.saturating_add(1);
        }
    }

    return written;
}

/// Runs `cargo metadata` over `root` through `launcher`, materializes one
/// `nomos.cap.dependency.edges` fact per workspace member, and returns the subjects a rule
/// can judge them under.
///
/// A second, independent materialization step beside [`Materialize_Syntax`] rather than a
/// generalization of it: `dependency.edges` has exactly one consumer today and nothing to
/// share or schedule against `syntax.items`'s own materialization, so composing this as one
/// more hardcoded step is `OD-HOST-004`'s "composition, not choice" again, not a case for
/// the shared demand planner `ARC-ROADMAP-001` still leaves for later.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero dependency findings" either, which is
/// exactly the vacuity [`Materialize_Syntax`]'s own `NoFacts` case exists to catch one layer
/// over. So a failed materialization returns no dependency sources and one synthetic
/// finding reporting why, rather than nothing at all.
pub fn Materialize_Dependencies<Launcher: ProcessLauncher>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &Launcher,
) -> DependencyMaterialization
{
    let production = Cargo_Production(context);

    let facts = match nomos_lang_rust_cargo::Materialize_Workspace(root, production, launcher)
    {
        Ok(facts) => facts,
        Err(error) => return DependencyMaterialization {
            sources: Vec::new(),
            findings: vec![Dependency_Capability_Unavailable(&error)],
        },
    };

    let sources = Materialized_Dependency_Sources(facts, store);

    return DependencyMaterialization { sources, findings: Vec::new() };
}

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

/// The reading context as `nomos_lang_rust_cargo`'s provider takes it.
fn Cargo_Production(context: &Context) -> nomos_lang_rust_cargo::FactContext
{
    return nomos_lang_rust_cargo::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Every `dependency.edges` fact the store accepted, as the source list `Check_Dependency_
/// Direction` can judge -- one per workspace member `store.Materialize` did not refuse.
fn Materialized_Dependency_Sources(
    facts: Vec<nomos_lang_rust_cargo::PackageFact>,
    store: &mut MemoryFactStore,
) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for package in facts
    {
        if store.Materialize(package.fact, &[]).is_ok()
        {
            let source = SourceFile::New(package.path, package.subject, String::new());
            sources.push(source);
        }
    }

    return sources;
}

/// Runs `cargo clippy --workspace --message-format=json` over `root` through `launcher`,
/// materializes one `nomos.cap.lint.diagnostics` fact per workspace member, and returns
/// the subjects a rule can judge them under -- the identical shape [`Materialize_Dependencies`]
/// already has for the identical reason: this workspace's second provider with I/O of its
/// own.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero lint findings" either, the same
/// `NoFacts`-shaped vacuity [`Materialize_Syntax`]'s own case exists to catch one layer
/// over. So a failed materialization returns no lint sources and one synthetic finding
/// reporting why, rather than nothing at all.
pub fn Materialize_Lint<Launcher: ProcessLauncher>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &Launcher,
) -> LintMaterialization
{
    let production = Clippy_Production(context);

    let facts = match nomos_lang_rust_clippy::Materialize_Workspace(root, production, launcher)
    {
        Ok(facts) => facts,
        Err(error) => return LintMaterialization {
            sources: Vec::new(),
            findings: vec![Lint_Capability_Unavailable(&error)],
        },
    };

    let sources = Materialized_Lint_Sources(facts, store);

    return LintMaterialization { sources, findings: Vec::new() };
}

/// The reading context as `nomos_lang_rust_clippy`'s provider takes it.
fn Clippy_Production(context: &Context) -> nomos_lang_rust_clippy::FactContext
{
    return nomos_lang_rust_clippy::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Every `lint.diagnostics` fact the store accepted, as the source list `Check_Lint_
/// Diagnostics` can judge -- one per workspace member `store.Materialize` did not refuse.
fn Materialized_Lint_Sources(
    facts: Vec<nomos_lang_rust_clippy::DiagnosticsFact>,
    store: &mut MemoryFactStore,
) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for member in facts
    {
        if store.Materialize(member.fact, &[]).is_ok()
        {
            let source = SourceFile::New(member.path, member.subject, String::new());
            sources.push(source);
        }
    }

    return sources;
}

/// The one finding a failed [`nomos_lang_rust_clippy::Materialize_Workspace`] call
/// produces -- the identical shape [`Dependency_Capability_Unavailable`] already has for
/// the identical reason: attributed to the whole tree, since a `cargo clippy` failure is
/// not about any subject this run walked.
fn Lint_Capability_Unavailable(error: &nomos_lang_rust_clippy::ClippyError) -> Finding
{
    return Finding {
        rule: RuleId::New(nomos_rules::LINT_DIAGNOSTICS),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the lint-diagnostics capability could not be materialized, so lint \
             diagnostics were not judged for anything in this run: {error}"
        ),
        locations: Vec::new(),
    };
}

/// Runs `cargo deny check bans licenses sources` over `root` through `launcher`,
/// materializes the one `nomos.cap.dependency.policy` fact this capability's
/// `IncrementalGranularity::WholeWorkspace` ceiling allows, and returns the subject a rule
/// can judge it under -- the identical shape [`Materialize_Lint`] already has for the
/// identical reason: this workspace's third provider with I/O of its own.
///
/// A failure here does not abort the run, the identical reasoning [`Materialize_Lint`]'s
/// own doc gives one layer up: a failed materialization returns no policy sources and one
/// synthetic finding reporting why, rather than nothing at all.
pub fn Materialize_Policy<Launcher: ProcessLauncher>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &Launcher,
) -> PolicyMaterialization
{
    let production = Deny_Production(context);

    let fact = match nomos_lang_rust_deny::Materialize_Workspace(root, production, launcher)
    {
        Ok(fact) => fact,
        Err(error) => return PolicyMaterialization {
            sources: Vec::new(),
            findings: vec![Policy_Capability_Unavailable(&error)],
        },
    };

    let sources = Materialized_Policy_Sources(fact, store);

    return PolicyMaterialization { sources, findings: Vec::new() };
}

/// The reading context as `nomos_lang_rust_deny`'s provider takes it.
fn Deny_Production(context: &Context) -> nomos_lang_rust_deny::FactContext
{
    return nomos_lang_rust_deny::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The one `dependency.policy` fact as the (at most one-element) source list `Check_
/// Dependency_Policy` can judge -- empty if `store.Materialize` refused it, one entry
/// otherwise, the same shape [`Materialized_Lint_Sources`] has for a list rather than a
/// single value: `crate::run_context::Judged` calls every rule uniformly over a source list, and a
/// whole-workspace capability's own list just never holds more than one.
fn Materialized_Policy_Sources(fact: nomos_lang_rust_deny::PolicyFact, store: &mut MemoryFactStore) -> Vec<SourceFile>
{
    if store.Materialize(fact.fact, &[]).is_err()
    {
        return Vec::new();
    }

    return vec![SourceFile::New("workspace", fact.subject, String::new())];
}

/// The one finding a failed [`nomos_lang_rust_deny::Materialize_Workspace`] call produces
/// -- the identical shape [`Lint_Capability_Unavailable`] already has for the identical
/// reason: attributed to the whole tree, since a `cargo deny` failure is not about any
/// subject this run walked.
fn Policy_Capability_Unavailable(error: &nomos_lang_rust_deny::DenyError) -> Finding
{
    return Finding {
        rule: RuleId::New(nomos_rules::DEPENDENCY_POLICY),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the dependency-policy capability could not be materialized, so the workspace's \
             own bans/licenses/sources verdict was not judged in this run: {error}"
        ),
        locations: Vec::new(),
    };
}

/// The one finding a failed [`nomos_lang_rust_cargo::Materialize_Workspace`] call produces.
///
/// Attributed to the whole tree (`Subject_Of_Path("")`, the root's own subject per
/// `nomos_model::path`'s convention) rather than to any one file, because a `cargo metadata`
/// failure is not about any subject this run walked -- it is about whether the dependency
/// capability could answer at all. [`Applicability::ProviderUnavailable`] because the
/// provider is registered and offered; it ran and did not answer, which is exactly that
/// variant's own distinction from `MissingCapability`.
fn Dependency_Capability_Unavailable(error: &nomos_lang_rust_cargo::MetadataError) -> Finding
{
    return Finding {
        rule: RuleId::New(nomos_rules::DEPENDENCY_DIRECTION),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the dependency-edges capability could not be materialized, so dependency \
             direction was not judged for anything in this run: {error}"
        ),
        locations: Vec::new(),
    };
}

/// Produces the `nomos.cap.review.finding` sources [`nomos_rules::Check_Review_Findings`]
/// can judge -- today, always none.
///
/// Unlike [`Materialize_Dependencies`], [`Materialize_Lint`] and [`Materialize_Policy`],
/// this capability is not "run one subprocess over the whole workspace and materialize
/// what it finds": `nomos-connector-coderabbit`'s one provider answers about one
/// already-identified external review comment (a repository and a comment id), and
/// nothing in an ordinary `nomos check` run over already-walked source names either --
/// there is no field on [`crate::RunContext`], [`Context`], or any [`SourceFile`] this
/// crate's own composition can read one from, and inventing a discovery mechanism (which
/// pull request, which comment) is exactly the kind of address `ARC-CONNECTOR-001`
/// deliberately declines to decide in advance of a real caller needing one. So this
/// materialization step is honest about producing nothing today: `Check_Review_Findings`
/// is composed, selectable, and will judge real sources the moment a future composition
/// root supplies a repository and comment id to fetch -- which is deliberately not
/// invented here, ahead of a real caller needing one.
#[must_use]
pub fn Materialize_Review() -> ReviewMaterialization
{
    return ReviewMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The reading context as `nomos_lang_rust`'s own provider takes it.
fn Rust_Production(context: &Context) -> FactContext
{
    return FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

#[cfg(test)]
mod tests
{
    //! What this module promises, exercised.

    use super::*;
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use nomos_platform::Command;
    use nomos_workspace::BuildVariant;
    use std::path::PathBuf;

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
        let first_context = Reused_Fixture_Context(&first_sources, &mut workspace);
        let first_current = Materialize_Syntax(&first_sources, &first_context, &mut store);
        assert_eq!(first_current, 2, "a fresh store must report a current fact for both real sources");
        let materializations_after_first = store.Materializations();

        let second_sources = [unchanged, changed_after];
        let second_context = Reused_Fixture_Context(&second_sources, &mut workspace);
        let second_current = Materialize_Syntax(&second_sources, &second_context, &mut store);
        let materializations_after_second = store.Materializations();

        assert_eq!(second_current, 2, "both sources still have a current fact after the second call, one of them reused rather than rewritten");
        assert_eq!(
            materializations_after_second, materializations_after_first.saturating_add(1),
            "only the changed subject's fact should be newly written to the store; the unchanged one must be skipped"
        );
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

    impl nomos_platform::ProcessLauncher for RefusingLauncher
    {
        fn Run(&self, _command: &Command) -> Result<nomos_platform::ProcessOutput, String>
        {
            return Err("refused for this test".to_owned());
        }
    }

    #[test]
    fn Test_Materialize_Dependencies_Should_Report_A_Finding_When_The_Launcher_Refuses()
    {
        let RefusedLaunchFixture { context, mut store } = Refused_Launch_Fixture();

        let result = Materialize_Dependencies(&PathBuf::from("."), &context, &mut store, &RefusingLauncher);

        assert!(result.sources.is_empty(), "a refused launch must not report workspace members");
        assert_eq!(result.findings.len(), 1, "{:?}", result.findings);
        assert_eq!(
            result.findings.first().expect("the assertion above proves one finding was reported").rule,
            RuleId::New(nomos_rules::DEPENDENCY_DIRECTION)
        );
    }

    /// The identical claim
    /// [`Test_Materialize_Dependencies_Should_Report_A_Finding_When_The_Launcher_Refuses`]
    /// proves, for `nomos.cap.lint.diagnostics` and `cargo clippy`.
    #[test]
    fn Test_Materialize_Lint_Should_Report_A_Finding_When_The_Launcher_Refuses()
    {
        let RefusedLaunchFixture { context, mut store } = Refused_Launch_Fixture();

        let result = Materialize_Lint(&PathBuf::from("."), &context, &mut store, &RefusingLauncher);

        assert!(result.sources.is_empty(), "a refused launch must not report workspace members");
        assert_eq!(result.findings.len(), 1, "{:?}", result.findings);
        assert_eq!(
            result.findings.first().expect("the assertion above proves one finding was reported").rule,
            RuleId::New(nomos_rules::LINT_DIAGNOSTICS)
        );
    }

    /// The identical claim
    /// [`Test_Materialize_Dependencies_Should_Report_A_Finding_When_The_Launcher_Refuses`]
    /// proves, for `nomos.cap.dependency.policy` and `cargo deny`.
    #[test]
    fn Test_Materialize_Policy_Should_Report_A_Finding_When_The_Launcher_Refuses()
    {
        let RefusedLaunchFixture { context, mut store } = Refused_Launch_Fixture();

        let result = Materialize_Policy(&PathBuf::from("."), &context, &mut store, &RefusingLauncher);

        assert!(result.sources.is_empty(), "a refused launch must not report the policy fact");
        assert_eq!(result.findings.len(), 1, "{:?}", result.findings);
        assert_eq!(
            result.findings.first().expect("the assertion above proves one finding was reported").rule,
            RuleId::New(nomos_rules::DEPENDENCY_POLICY)
        );
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

    /// [`Fixture_Context`]'s own composition, over a `workspace` the caller keeps and
    /// passes again -- what a real second `Run` call reuses, so its own generation
    /// advances only when `sources`' own content actually moved since the last call
    /// ingested it.
    fn Reused_Fixture_Context(sources: &[SourceFile], workspace: &mut Option<nomos_workspace::Workspace>) -> Context
    {
        let registry = crate::composition::Registered().expect("fixture composition");
        return crate::facts::Ingested_Workspace(sources, &registry, Test_Variant(), workspace).expect("the fixture is a valid tree");
    }
}

//! Running one check over already-walked source, apart from finding that source or
//! rendering what came of it.

use nomos_analysis::{Context, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::{Environment, FileSystem, ProcessLauncher};
use nomos_rules::{
    RequiredFact, SourceFile, DESCRIPTORS,
    // The six rules of `Judged_Sources`' mapping. Every rule identifier this module used to
    // name for the sake of *running* a rule is now read off `DESCRIPTORS` at run time;
    // these six are the residue `OD-RULES-027` measured and licensed.
    DEPENDENCY_COMPLETENESS, DEPENDENCY_DIRECTION, DEPENDENCY_POLICY, LINT_DIAGNOSTICS, REVIEW_FINDING, WRITE_AUTHORITY,
    // A different axis, deliberately untouched: which rules feed a capability, and so which
    // families a selection has to materialize before anything is judged. `OD-GATE-017`
    // accepted that as a fixed hand-written mapping and `OD-RULES-027` says in as many words
    // that it does not disturb it. The `Materialize_*_Section` predicates below are it.
    ABBREVIATIONS, DATA_NAMES_STAY_LOWER_SNAKE, DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS,
    EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, FILE_SIZE_JUSTIFICATION_TRIGGER, FIVE_HUNDRED_LINE_REVIEW_TRIGGER,
    GOALS_AND_PARTS_LINE_UP, GO_HELPERS_PACKAGE_FIVE_INPUTS, NAMING_CONVENTION, ONE_THOUSAND_LINE_HARD_TRIGGER,
    PARAMETER_COUNT, REQUIREMENT_TRACE_STALENESS, TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE,
    UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, UNREAD_REACHES_FINDING,
};
use nomos_workspace::{BuildVariant, Workspace};
use std::path::Path;

use crate::composition::{Recognized_Language, Recognized_Syntax_Provider, Registered};
use crate::facts::{
    Subprocess,
    DependencyMaterialization, Ingested_Workspace, LintMaterialization, Materialize_Dependencies, Materialize_Goals_Policy,
    Materialize_Limits_Policy, Materialize_Lint, Materialize_Naming_Policy, Materialize_Policy, Materialize_Reachability,
    Materialize_Requirement_Trace, Materialize_Review, Materialize_Scripting_Policy, Materialize_Syntax, Materialize_Words_Policy,
    PolicyMaterialization, ReviewMaterialization,
};
use crate::CheckOutcome;

mod rule_reassessment_cache;
pub use rule_reassessment_cache::RuleReassessmentCache;

/// [`Run`]'s build variant, its subprocess root, the launcher those subprocesses run
/// through, the filesystem a repository-declared policy capability (`nomos.cap.naming.
/// policy` and its siblings) is read through, and the workspace and fact store `Run` reads
/// and writes -- grouped into one value so [`Run`] stays within this crate's own
/// parameter-count limit. See [`Run`]'s own documentation for why each is a composition-root
/// value this crate cannot compute for itself.
pub struct RunContext<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    pub variant: BuildVariant,
    pub root: &'a Path,
    pub launcher: &'a Launcher,
    pub filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from, rather than from this
    /// process's own ambient state. `OD-HOST-001`: the composition root chooses it.
    pub environment: &'a Env,
    pub workspace: &'a mut Option<Workspace>,
    pub store: &'a mut MemoryFactStore,
}

/// Composes the capability registry, ingests `sources` into one workspace state, materializes
/// a syntax fact per file, and, for each of the completeness, naming-convention,
/// dependency-direction, lint-diagnostics, dependency-policy and unread-reaches-finding
/// rules `selected` asks for (every one when `selected` is empty, the same "empty is
/// everything" default `nomos_gate_orchestration::RuleSelector::include` already has),
/// materializes the fact that rule needs and runs it over the result.
///
/// `selected` is a fixed, hand-written mapping from [`RuleId`] to the fact(s) it needs, per
/// `OD-GATE-017`: the same "composition, not choice" shape a fourth unconditional rule already
/// used (`OD-HOST-004`), extended to a second axis -- whether a rule's own materialization
/// runs at all, not only whether its finding counts toward a disposition
/// `nomos_gate_orchestration::RuleSelector` narrows after the fact.
///
/// `sources` is the walk, already done -- `nomos-cli::check::sources::Walked_Sources` stayed
/// in the composition root, and its own doc carries the current reason:
/// [`nomos_platform::FileSystem`]'s `Read_Directory` is one level by `OD-PLATFORM-002`'s own
/// floor, not a traversal. `context.variant` is what that root's own binary was
/// compiled as, read through `env!` there because that macro resolves against the
/// *compiling* crate and cannot be read correctly from this one. `context.root` is the
/// tree `sources` was walked from -- carried separately because the dependency-edges,
/// lint-diagnostics and dependency-policy providers each run their own subprocess (`cargo
/// metadata`, `cargo clippy`, `cargo deny`) rather than reading bytes `sources` already
/// holds; every other provider in this workspace is a pure function over bytes a caller
/// already read. `context.launcher` is what those subprocess calls run through -- generic
/// the same way `nomos_work_orchestration::Run` is generic over [`nomos_platform`]'s
/// traits, so this crate depends on `nomos-platform` and not on any concrete implementation
/// of it; the composition root supplies one. `context.workspace` and `context.store` are the
/// caller's, not this function's own -- `OD-ANALYSIS-009`'s own first real increment. Every
/// caller this crate has today passes `&mut None` and a freshly constructed
/// [`MemoryFactStore`], which reproduces exactly what this function used to do
/// unconditionally: build both from nothing, on every call. What changes is that a caller is
/// no longer forced to. Passing the *same* `workspace` and `store` across two calls reuses
/// the workspace's own real diff ([`nomos_workspace::Workspace::Apply`] compares each
/// submitted path's content against what it already holds, so an unmoved file is
/// `Redundant` and the generation only advances when something genuinely did) and the
/// store's own generation-scoped history, instead of starting from an empty tree and an
/// empty store every time. This function still ingests and materializes every source in
/// `sources` on every call -- reuse buys correctness of carrying state across calls, not yet
/// a skipped recomputation for a subject nothing touched. The five are grouped into
/// [`RunContext`] so this function stays within this crate's own parameter-count limit.
///
/// Writes nothing and never exits: [`CheckOutcome`] is the whole answer, the same
/// division `nomos_work_orchestration::Run` draws around [`nomos_work_orchestration`]'s own
/// `WorkOutcome`. An empty `sources` is a composition root's decision
/// (`CheckOutcome::NoSource`) made before this function is ever called, not a case this
/// function classifies.
///
/// Delegates to [`Run_Reassessing`] with a cache built fresh for this call alone and
/// dropped at its end -- every rule this crate composes has never run under that cache, so
/// every one still runs, unconditionally, exactly as this function did before
/// [`RuleReassessmentCache`] existed. A caller wanting the skip keeps its own cache across
/// calls and calls [`Run_Reassessing`] directly instead.
#[must_use]
pub fn Run<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    context: RunContext<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
) -> CheckOutcome
{
    let mut reassessment = RuleReassessmentCache::New();

    return Run_Reassessing(sources, context, selected, &mut reassessment);
}

/// [`Run`], with one addition: a rule already recorded in `reassessment` does not run its
/// closure again this call when none of its own `nomos_rules::RuleDescriptor.requires`
/// names a family that changed since it was recorded -- see `rule_reassessment`'s own
/// module doc for exactly which rules that covers today, and why it stops there.
///
/// `context.store` decides which families this call is even judged against a currency
/// check for: `Materialize_Syntax`'s own well before this crate had a second entry point,
/// so a source whose bytes did not move since `store` last saw it never re-parses, on
/// either function. What only this function adds is skipping the *rule* on top of that,
/// for a rule the syntax family is the only thing it reads.
#[must_use]
pub fn Run_Reassessing<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    context: RunContext<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
    reassessment: &mut RuleReassessmentCache,
) -> CheckOutcome
{
    let RunContext { variant, root, launcher, filesystem, environment, workspace, store } = context;

    let recognized = Recognized_Sources(sources);
    let sources: &[SourceFile] = &recognized;

    let (registry, context) = match Composed_Registry_And_Context(sources, variant, workspace)
    {
        Ok(composed) => composed,
        Err(outcome) => return outcome,
    };

    let materializations_before_syntax = store.Materializations();
    let facts = match Materialized_Syntax_Facts(sources, &context, store)
    {
        Some(facts) => facts,
        None => return CheckOutcome::NoFacts { files: sources.len() },
    };

    let mut changed = Vec::new();
    if store.Materializations() > materializations_before_syntax
    {
        changed.push(RequiredFact::SyntaxItems);
    }

    let run = RunEnvironment { root, launcher, filesystem, environment, registry: &registry, context, selected };
    let mut state = RunState { store, reassessment, changed };
    let findings = Judged_Over(sources, run, &mut state);

    return Outcome_Of(sources.len(), facts, findings);
}

/// The registry composed and `sources` ingested through it, or the [`CheckOutcome`] that
/// already answers the run when either step refuses. `workspace` is threaded through to
/// [`Ingested_Workspace`] rather than built here: reuse across two calls to [`Run`] is a
/// property of the same [`Workspace`] object seeing a second `Workspace::Apply`, not
/// something this function can arrange after the fact.
fn Composed_Registry_And_Context(
    sources: &[SourceFile], variant: BuildVariant, workspace: &mut Option<Workspace>,
) -> Result<(Registry, Context), CheckOutcome>
{
    let registry = Registered().map_err(CheckOutcome::Contradictory)?;
    let context = Ingested_Workspace(sources, &registry, variant, workspace).map_err(|_error| return CheckOutcome::Unreadable)?;

    return Ok((registry, context));
}

/// The syntax facts materialized into `store`, or `None` when there were none to judge --
/// [`Run`]'s own first early exit, given a name so its body reads as one decision per line.
fn Materialized_Syntax_Facts(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> Option<usize>
{
    let facts = Materialize_Syntax(sources, context, store);
    if facts == 0
    {
        return None;
    }

    return Some(facts);
}

/// [`Run`]'s own root, launcher, filesystem, registry, composed context and rule
/// selection -- everything [`Judged_Over`] needs beside the sources and mutable state it is
/// handed separately, grouped so that function's parameter list names one environment
/// instead of six loose values.
struct RunEnvironment<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    root: &'a Path,
    launcher: &'a Launcher,
    filesystem: &'a Fs,
    environment: &'a Env,
    registry: &'a Registry,
    context: Context,
    selected: &'a [RuleId],
}

/// The store, the reassessment cache, and which capability families this call has changed
/// so far -- every value [`Judged_Over`] mutates, grouped into one so that function and
/// [`Run_Reassessing`] both stay within this crate's own parameter-count limit. `changed`
/// starts already carrying whatever [`Run_Reassessing`] found before [`Judged_Over`] is
/// ever called (the syntax family, from its own before/after [`MemoryFactStore::
/// Materializations`] snapshot around [`Materialized_Syntax_Facts`]) and
/// [`Materialize_Capabilities`] appends the other nine to it.
struct RunState<'a>
{
    store: &'a mut MemoryFactStore,
    reassessment: &'a mut RuleReassessmentCache,
    changed: Vec<RequiredFact>,
}

/// Every capability [`Run`] can materialize, judged -- the two steps [`Run`] itself used to
/// inline, composed here so its own body names one step instead of four.
fn Judged_Over<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(sources: &[SourceFile], environment: RunEnvironment<'_, Launcher, Fs, Env>, state: &mut RunState<'_>) -> Vec<Finding>
{
    let mut materialization_environment = MaterializationEnvironment {
        root: environment.root,
        context: &environment.context,
        store: state.store,
        launcher: environment.launcher,
        filesystem: environment.filesystem,
        environment: environment.environment,
    };
    let capabilities = Materialize_Capabilities(sources, &mut materialization_environment, environment.selected, &mut state.changed);

    let judge_environment = JudgeEnvironment { store: state.store, registry: environment.registry, context: environment.context };
    let reassessment = Reassessment { selected: environment.selected, cache: state.reassessment, changed: &state.changed };
    return Judged_Findings(sources, capabilities, judge_environment, reassessment);
}

/// The dependency-edges, lint-diagnostics, dependency-policy, reachability and
/// naming-policy facts, materialized into `store` alongside the syntax facts [`Run`] already
/// wrote -- the capabilities beside `syntax.items` that this crate's registration composes,
/// each with its own materialization step for the reasons [`Materialize_Dependencies`],
/// [`Materialize_Lint`], [`Materialize_Policy`], [`Materialize_Reachability`] and
/// [`Materialize_Naming_Policy`] give.
///
/// Each runs only when `selected` asks for a rule it feeds -- `DEPENDENCY_DIRECTION`,
/// `DEPENDENCY_COMPLETENESS` or `WRITE_AUTHORITY` for the first (all three judge the same
/// `dependencies.sources`), `LINT_DIAGNOSTICS` for the second, `DEPENDENCY_POLICY` for the
/// third,
/// `UNREAD_REACHES_FINDING` for the fourth, any of `NAMING_CONVENTION`, `DATA_NAMES_STAY_
/// LOWER_SNAKE`, `EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE`, `UNEXPORTED_FUNCTIONS_
/// LOWERCASE_ONLY_THE_FIRST_LETTER` or `TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE` for the
/// fifth, per `OD-GATE-017`. Skipping `Materialize_Dependencies`, `Materialize_Lint`,
/// `Materialize_Policy` or `Materialize_Naming_Policy` skips its own subprocess launch or
/// filesystem read entirely, not merely its finding's place in a later disposition.
///
/// Each call also names, into `changed`, the family it just wrote to at all -- every one of
/// them does, unconditionally, whenever `selected` triggers it at all: none of the ten has
/// `Materialize_Syntax`'s own currency check yet. `rule_reassessment::RuleReassessmentCache`
/// reads `changed` to decide which rules are still safe to reuse; a family missing from it
/// because this function forgot to report it would let a stale rule's prior findings stand
/// in for a real one, which is why every section below is wrapped rather than only the ones
/// a caller might expect to benefit.
fn Materialize_Capabilities<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
    changed: &mut Vec<RequiredFact>,
) -> CapabilityMaterialization
{
    let dependencies = Tracking(env, changed, RequiredFact::DependencyEdges, |env| return Materialize_Dependency_Section(env, selected));
    let lint = Tracking(env, changed, RequiredFact::LintDiagnostics, |env| return Materialize_Lint_Section(env, selected));
    let policy = Tracking(env, changed, RequiredFact::DependencyPolicy, |env| return Materialize_Policy_Section(env, selected));
    Tracking(env, changed, RequiredFact::Reachability, |env| Materialize_Reachability_Section(sources, env, selected));
    Tracking(env, changed, RequiredFact::NamingPolicy, |env| Materialize_Naming_Policy_Section(env, selected));
    Tracking(env, changed, RequiredFact::LimitsPolicy, |env| Materialize_Limits_Policy_Section(env, selected));
    Tracking(env, changed, RequiredFact::ScriptingPolicy, |env| Materialize_Scripting_Policy_Section(env, selected));
    Tracking(env, changed, RequiredFact::GoalsPolicy, |env| Materialize_Goals_Policy_Section(env, selected));
    Tracking(env, changed, RequiredFact::WordsPolicy, |env| Materialize_Words_Policy_Section(env, selected));
    Tracking(env, changed, RequiredFact::RequirementTrace, |env| Materialize_Requirement_Trace_Section(env, selected));
    let review = Tracking(env, changed, RequiredFact::ReviewFindings, |_env| return Materialize_Review_Section(selected));

    return Capability_Materialization_Of(dependencies, lint, policy, review);
}

/// Runs `section`, and records `family` into `changed` if `env.store` gained a new
/// materialization while it ran -- the one signal available today for "did this family just
/// change," since a skipped section (its own gating rule not selected) writes nothing and a
/// run one writes unconditionally.
fn Tracking<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment, Answer>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    changed: &mut Vec<RequiredFact>,
    family: RequiredFact,
    section: impl FnOnce(&mut MaterializationEnvironment<'_, Launcher, Fs, Env>) -> Answer,
) -> Answer
{
    let before = env.store.Materializations();
    let result = section(env);
    if env.store.Materializations() > before
    {
        changed.push(family);
    }

    return result;
}

/// The `root`, `context`, `store`, `launcher` and `filesystem` every
/// [`Materialize_Capabilities`] section reads or writes through -- grouped into one value so
/// that function takes those five as one parameter rather than five.
struct MaterializationEnvironment<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    root: &'a Path,
    context: &'a Context,
    store: &'a mut MemoryFactStore,
    launcher: &'a Launcher,
    filesystem: &'a Fs,
    environment: &'a Env,
}

/// The dependency-edges section: [`Materialize_Dependencies`] when `selected` feeds on it,
/// an empty result otherwise.
fn Materialize_Dependency_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
) -> DependencyMaterialization
{
    if Is_Rule_Selected(selected, DEPENDENCY_DIRECTION)
        || Is_Rule_Selected(selected, DEPENDENCY_COMPLETENESS)
        || Is_Rule_Selected(selected, WRITE_AUTHORITY)
    {
        return Materialize_Dependencies(env.root, env.context, env.store, Subprocess { launcher: env.launcher, environment: env.environment });
    }

    return DependencyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The lint-diagnostics section: [`Materialize_Lint`] when `selected` feeds on it, an empty
/// result otherwise.
fn Materialize_Lint_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
) -> LintMaterialization
{
    if Is_Rule_Selected(selected, LINT_DIAGNOSTICS)
    {
        return Materialize_Lint(env.root, env.context, env.store, Subprocess { launcher: env.launcher, environment: env.environment });
    }

    return LintMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The dependency-policy section: [`Materialize_Policy`] when `selected` feeds on it, an
/// empty result otherwise.
fn Materialize_Policy_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
) -> PolicyMaterialization
{
    if Is_Rule_Selected(selected, DEPENDENCY_POLICY)
    {
        return Materialize_Policy(env.root, env.context, env.store, Subprocess { launcher: env.launcher, environment: env.environment });
    }

    return PolicyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The review-finding section: [`Materialize_Review`] when `selected` feeds on it, an
/// empty result otherwise.
///
/// Unlike [`Materialize_Dependency_Section`], [`Materialize_Lint_Section`] and
/// [`Materialize_Policy_Section`], this section takes no [`MaterializationEnvironment`]:
/// [`Materialize_Review`]'s own doc gives the reason -- this connector's one provider
/// answers about one already-identified external review comment, and nothing in this
/// call chain names one yet, so there is nothing here for a root, a launcher or a
/// filesystem to be read through. `Materialize_Capabilities`' own `Tracking` wrapper still
/// calls this the identical way, through a closure that ignores the environment it is
/// handed -- the same `|_reader|` idiom this crate's own text-only `ComposedRule` entries
/// already use for the identical reason.
fn Materialize_Review_Section(selected: &[RuleId]) -> ReviewMaterialization
{
    if Is_Rule_Selected(selected, REVIEW_FINDING)
    {
        return Materialize_Review();
    }

    return ReviewMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The reachability section: [`Materialize_Reachability`] when `selected` feeds on it --
/// writes into `env.store` directly and produces no return value of its own, the same shape
/// the call it wraps already has.
fn Materialize_Reachability_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
)
{
    if Is_Rule_Selected(selected, UNREAD_REACHES_FINDING)
    {
        Materialize_Reachability(sources, env.context, env.store);
    }
}

/// The naming-policy section: [`Materialize_Naming_Policy`] when `selected` feeds on it --
/// writes into `env.store` directly. `Materialize_Naming_Policy` returns how many facts
/// landed; this section does not need that count, the same "written but not captured" shape
/// [`Materialize_Reachability_Section`] already has for the count its own call returns.
///
/// Gated on the five naming-convention rules this crate composes today that read `nomos.cap.
/// naming.policy` through their own `Resolve_Case`; `PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_
/// SNAKE_CASE` reads the identical capability but is not itself composed into
/// [`Rule_Findings`] yet, so gating on it here would materialize a fact for a rule that never
/// runs.
fn Materialize_Naming_Policy_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
)
{
    let feeds_naming_policy = Is_Rule_Selected(selected, NAMING_CONVENTION)
        || Is_Rule_Selected(selected, DATA_NAMES_STAY_LOWER_SNAKE)
        || Is_Rule_Selected(selected, EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE)
        || Is_Rule_Selected(selected, UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER)
        || Is_Rule_Selected(selected, TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE);

    if feeds_naming_policy
    {
        Materialize_Naming_Policy(env.root, env.context, env.store, env.filesystem);
    }
}

/// The limits-policy section: [`Materialize_Limits_Policy`] when `selected` feeds on it.
///
/// Gated on the six composed rules that read `nomos.cap.limits.policy` through their own
/// `Resolve_Limit` -- the four file-size triggers and the two parameter-count caps. Each of
/// them already has a hardcoded fallback equal to what this repository declares, so
/// materializing the fact changes no finding here; it changes which repositories the
/// thresholds belong to.
fn Materialize_Limits_Policy_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
)
{
    let feeds_limits_policy = Is_Rule_Selected(selected, FILE_SIZE_JUSTIFICATION_TRIGGER)
        || Is_Rule_Selected(selected, ONE_THOUSAND_LINE_HARD_TRIGGER)
        || Is_Rule_Selected(selected, FIVE_HUNDRED_LINE_REVIEW_TRIGGER)
        || Is_Rule_Selected(selected, PARAMETER_COUNT)
        || Is_Rule_Selected(selected, GO_HELPERS_PACKAGE_FIVE_INPUTS);

    if feeds_limits_policy
    {
        Materialize_Limits_Policy(env.root, env.context, env.store, env.filesystem);
    }
}

/// The scripting-policy section: [`Materialize_Scripting_Policy`] when `selected` feeds on
/// it.
///
/// One composed rule reads this capability, and unlike every other policy section here its
/// absence is not a fallback: `Check_Declared_Tooling_Language_For_Scripts` reports nothing
/// at all without the fact, so before this call existed the rule ran in every check and could
/// never fire.
fn Materialize_Scripting_Policy_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
)
{
    if Is_Rule_Selected(selected, DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS)
    {
        Materialize_Scripting_Policy(env.root, env.context, env.store, env.filesystem);
    }
}

/// The goals-policy section: [`Materialize_Goals_Policy`] when its one rule is selected.
///
/// The narrowest gate of the four policy sections, because exactly one rule reads this
/// capability -- and the first one whose rule takes no sources at all, so there is nothing
/// here to gate on but the rule's own selection.
fn Materialize_Goals_Policy_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
)
{
    if Is_Rule_Selected(selected, GOALS_AND_PARTS_LINE_UP)
    {
        Materialize_Goals_Policy(env.root, env.context, env.store, env.filesystem);
    }
}

/// The words-policy section: [`Materialize_Words_Policy`] when its one rule is selected.
///
/// One rule reads this capability, so the gate is that rule's own selection -- the same
/// shape [`Materialize_Goals_Policy_Section`] has one function above.
fn Materialize_Words_Policy_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
)
{
    if Is_Rule_Selected(selected, ABBREVIATIONS)
    {
        Materialize_Words_Policy(env.root, env.context, env.store, env.filesystem);
    }
}

/// The requirement-trace section: [`Materialize_Requirement_Trace`] when its one rule is
/// selected.
///
/// One rule reads this capability, so the gate is that rule's own selection -- the same
/// shape [`Materialize_Goals_Policy_Section`] and [`Materialize_Words_Policy_Section`] each
/// have one function above.
fn Materialize_Requirement_Trace_Section<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
)
{
    if Is_Rule_Selected(selected, REQUIREMENT_TRACE_STALENESS)
    {
        Materialize_Requirement_Trace(env.root, env.context, env.store, env.filesystem);
    }
}

/// The assembly section: what the three source-and-finding materializations produced,
/// gathered into one [`CapabilityMaterialization`].
fn Capability_Materialization_Of(
    dependencies: DependencyMaterialization,
    lint: LintMaterialization,
    policy: PolicyMaterialization,
    review: ReviewMaterialization,
) -> CapabilityMaterialization
{
    return CapabilityMaterialization {
        dependency_sources: dependencies.sources,
        dependency_findings: dependencies.findings,
        lint_sources: lint.sources,
        lint_findings: lint.findings,
        policy_sources: policy.sources,
        policy_findings: policy.findings,
        review_sources: review.sources,
        review_findings: review.findings,
    };
}

/// What [`Materialize_Capabilities`] produced: the dependency-edges, lint-diagnostics and
/// dependency-policy sources a rule can judge, and any finding materializing one already
/// raised on its own -- named rather than left as positional pairs, the same reason
/// [`crate::facts::DependencyMaterialization`], [`crate::facts::LintMaterialization`] and
/// [`crate::facts::PolicyMaterialization`] each exist one layer under it.
struct CapabilityMaterialization
{
    dependency_sources: Vec<SourceFile>,
    dependency_findings: Vec<Finding>,
    lint_sources: Vec<SourceFile>,
    lint_findings: Vec<Finding>,
    policy_sources: Vec<SourceFile>,
    policy_findings: Vec<Finding>,
    review_sources: Vec<SourceFile>,
    review_findings: Vec<Finding>,
}

/// Every finding [`Rule_Findings`] produces over `sources` and `capabilities`' own source
/// lists, plus whatever [`Materialize_Capabilities`] already found on its own via
/// [`Capability_Findings`] (a failed dependency or lint materialization, reported rather
/// than judged) -- unconditionally, since each such finding already carries its own rule
/// and a caller that did not select it would never have triggered the materialization
/// that raises it.
fn Judged_Findings(sources: &[SourceFile], capabilities: CapabilityMaterialization, env: JudgeEnvironment<'_>, reassessment: Reassessment<'_>) -> Vec<Finding>
{
    let mut reader = Reader::On(env.store, env.registry, env.context);

    let mut findings = Rule_Findings(sources, &capabilities, &mut reader, reassessment);
    findings.extend(Capability_Findings(capabilities));

    return findings;
}

/// The `store`, `registry` and `context` [`Judged_Findings`] reads the [`Reader`] from --
/// grouped into one value so that function takes those three as one parameter rather than
/// three.
struct JudgeEnvironment<'a>
{
    store: &'a MemoryFactStore,
    registry: &'a Registry,
    context: Context,
}

/// A rule's selection, its reassessment cache, and which capability families this call has
/// changed -- grouped into one value so [`Judged_Findings`], [`Rule_Findings`] and
/// [`Findings_For_Selected_Rules`] each take it as one parameter rather than three.
struct Reassessment<'a>
{
    selected: &'a [RuleId],
    cache: &'a mut RuleReassessmentCache,
    changed: &'a [RequiredFact],
}

/// Every finding the completeness, naming-convention, dependency-direction,
/// dependency-completeness, lint-diagnostics, dependency-policy, unread-reaches-finding,
/// cross-language-correspondence and ten text-only rules `selected` asks for produce over
/// `sources` and `capabilities`' own source lists.
///
/// The ten text-only rules (no-trailing-whitespace through no-mod-rs-files below) take only
/// `sources`, the same shape `Check_Naming_Convention` and every fact-reading rule
/// it does not: a rule implemented in `nomos-rules` but never composed here reports as
/// unenforced when it is not, so wiring one in is this crate's own territory, the same
/// "composed into nomos-check-orchestration::Run" section every one of these rules' own
/// module docs already names. Five siblings this crate also implements --
/// `no-decorative-section-dividers`, `unwrap-expect-discipline`, `panics-are-justified-
/// documented-and-validated`, `boolean-predicates` and `test-functions-use-test-subject-
/// should-behavior` -- are deliberately absent: this repository's own tree currently
/// violates all five (71, 45 and 217 findings for the first, fourth and fifth respectively,
/// checked by running each alone through a real `gate run --root .` first), and wiring a
/// rule this tree fails is a different, larger change than composing one it already
/// satisfies -- the last two are real struct fields without a predicate prefix and real
/// `#[test]` names with no `_Should_`/`_Should_Not_`, not a rule bug, so there is no version
/// of this fix that is merely a defect to correct in the rule itself. `a-disabled-test-
/// states-why`, `inline-always-requires-justification`, `no-wildcard-imports`,
/// `no-single-line-function-bodies` and `single-letter-names`, checked the same way, found
/// next to nothing to violate -- the 49, 288 and 657 findings the last three first measured
/// were, almost entirely, a rule bug each (`use super::*;`'s exemption could not see across
/// a `tests.rs` split from its `#[cfg(test)]` parent, `P27-RULES-NO-WILDCARD-IMPORTS-
/// EXEMPTION`; a bare text scan with no string-literal or comment awareness at all mistook a
/// `"fn a() {}"`-shaped test fixture for real code in 287 of 288 findings, `P27-RULES-NO-
/// SINGLE-LINE-FUNCTION-BODIES-STRING-AWARE`; a use-binding's own glob (`*`) or discard
/// (`_`) token was judged as if it were a declared name in 657 of 657 findings,
/// `P27-RULES-SINGLE-LETTER-NAMES-USE-BINDING-EXEMPTION`) -- not real violations, and the
/// one real `no-single-line-function-bodies` finding left (`tests/corpus/analysis/gamma/
/// broken.rs`, deliberately-invalid corpus content) was fixed directly rather than composed
/// around, since it cost one line. All five are composed below with their two already-wired
/// siblings.
fn Rule_Findings(sources: &[SourceFile], capabilities: &CapabilityMaterialization, reader: &mut Reader<'_, '_>, reassessment: Reassessment<'_>) -> Vec<Finding>
{
    return Findings_For_Selected_Rules(Judged { sources, capabilities }, reader, reassessment);
}

/// What a rule is judged over: the walked sources, and the capability slices the six rules
/// that do not read the walked ones are judged over instead.
///
/// Grouped rather than passed as two parameters so [`Findings_For_Selected_Rules`] stays
/// within this crate's own `parameter-count` limit.
struct Judged<'a>
{
    /// Every source the walk found, already enriched by [`Recognized_Sources`].
    sources: &'a [SourceFile],
    /// What each capability family materialized.
    capabilities: &'a CapabilityMaterialization,
}

/// Runs every `rules` entry `reassessment.selected` names, in table order, and collects
/// what each produces -- except one already recorded in `reassessment.cache` whose own
/// required families are all absent from `reassessment.changed`, whose prior findings are
/// reused instead of running its closure again. See `rule_reassessment`'s own module doc
/// for which rules that is, today.
fn Findings_For_Selected_Rules(judged: Judged<'_>, reader: &mut Reader<'_, '_>, reassessment: Reassessment<'_>) -> Vec<Finding>
{
    let Reassessment { selected, cache, changed } = reassessment;

    let mut findings = Vec::new();
    for descriptor in DESCRIPTORS
    {
        if !Is_Rule_Selected(selected, descriptor.id)
        {
            continue;
        }

        if let Some(reused) = cache.Reusable(descriptor.id, changed)
        {
            findings.extend(reused.clone());
            continue;
        }

        let rule_findings = (descriptor.check)(Judged_Sources(descriptor.id, &judged), reader);
        cache.Record(descriptor.id, rule_findings.clone());
        findings.extend(rule_findings);
    }

    return findings;
}

/// The sources `rule` is judged over: a capability family's own materialized slice for the
/// six rules that read one, and the walked sources for every other rule.
///
/// # Why this mapping is a declaration and not a planner
///
/// `OD-RULES-027` measured the seventy composed entries this function replaced and found
/// that sixty-two closed over the same walked sources; these six are the whole residue.
/// Which slice a rule reads is fixed at the moment that rule is written and is not computed
/// from anything -- not from what was selected, not from what is already materialized, not
/// from what the run has done so far. That makes it the same kind of fixed, hand-written
/// mapping `OD-GATE-017` already accepted for the neighbouring axis of which facts to
/// materialize: composition, not choice.
///
/// The line it must not cross is also that record's: an entry that ever reads "this slice,
/// unless that one is already materialized" has stopped being a declared fact about a rule
/// and become the demand planner `OD-RULES-009` has declined across eight rounds. That
/// change belongs in that record, not in this function.
///
/// It lives here rather than in `nomos_rules::DESCRIPTORS` because a capability slice is an
/// orchestration concept. A descriptor table naming one would be a lower band describing an
/// upper band's shape.
fn Judged_Sources<'a>(rule: &str, judged: &Judged<'a>) -> &'a [SourceFile]
{
    return match rule
    {
        DEPENDENCY_DIRECTION | DEPENDENCY_COMPLETENESS | WRITE_AUTHORITY => &judged.capabilities.dependency_sources,
        LINT_DIAGNOSTICS => &judged.capabilities.lint_sources,
        DEPENDENCY_POLICY => &judged.capabilities.policy_sources,
        REVIEW_FINDING => &judged.capabilities.review_sources,
        _ => judged.sources,
    };
}

/// What [`Materialize_Capabilities`] already found on its own -- a failed dependency, lint
/// or policy materialization, reported rather than judged -- unconditionally, since each
/// such finding already carries its own rule and a caller that did not select it would
/// never have triggered the materialization that raises it.
fn Capability_Findings(capabilities: CapabilityMaterialization) -> Vec<Finding>
{
    let mut findings = capabilities.dependency_findings;
    findings.extend(capabilities.lint_findings);
    findings.extend(capabilities.policy_findings);
    findings.extend(capabilities.review_findings);

    return findings;
}

/// The whole run, once judging is done -- how many files and facts it examined, and the
/// claim its own findings support.
fn Outcome_Of(files: usize, facts: usize, findings: Vec<Finding>) -> CheckOutcome
{
    use crate::examined::{Claim_Of, Examined};

    let examined = Examined { files, facts };
    let claim = Claim_Of(&findings);

    return CheckOutcome::Judged { findings, examined, claim };
}

/// Every rule [`Run`] composes, in the order it runs them.
///
/// This is `nomos_rules::DESCRIPTORS` read for its identifiers, and it is derived rather
/// than written down: [`Findings_For_Selected_Rules`] walks the same list in the same order,
/// so a rule declared there appears here with no second edit and none can be declared
/// without appearing.
///
/// It was not always derived. `OD-GATE-020` measured the gate registry offering eight rules
/// against the fifty-six then composed here, found the declared parity between the two had
/// gone false silently twice, and named why closing it by hand again would not fix the
/// shape. The answer then was one array literal read from two places; the answer now is no
/// array literal at all, which `OD-RULES-027` decided was available once the seventy
/// closures the old table held were measured rather than assumed to be seventy different
/// things.
#[must_use]
pub fn Composed_Rules() -> Vec<RuleId>
{
    return DESCRIPTORS.iter().map(|descriptor| return RuleId::New(descriptor.id)).collect();
}

/// `sources`, each carrying its own resolved [`nomos_rules::SourceFile::preferred_syntax_provider`]
/// -- `OD-CAPABILITY-009`'s corrected fix, computed once here because this composition root
/// is the one place in the call chain allowed to know `nomos_lang_rust` and `nomos_lang_go`
/// by name; `nomos_rules` itself never does. Every rule this crate composes sees only the
/// enriched copy, so a subject's syntax provider identity is settled before any of them run,
/// the same "carried rather than derived" reasoning [`SourceFile::subject`] already states
/// for the field this one sits beside.
///
/// `pub(crate)` rather than private to [`Run`] alone: this crate's own `tests.rs` reaches
/// past `Run` into `crate::composition` and `crate::facts` directly, by design, to prove the
/// split-composition guarantee against the store rather than against `Run`'s one call shape
/// -- and a fixture built that way needs the identical enrichment `Run` gives every other
/// caller, not a second, differently-behaved copy of it.
pub(crate) fn Recognized_Sources(sources: &[SourceFile]) -> Vec<SourceFile>
{
    return sources
        .iter()
        .cloned()
        .map(|mut source| {
            source.preferred_syntax_provider = Recognized_Syntax_Provider(&source.path);
            source.language = Recognized_Language(&source.path);
            return source;
        })
        .collect();
}


/// Whether `rule` is one `selected` asks for -- every rule when `selected` is empty, the same
/// "empty is everything" default `nomos_gate_orchestration::RuleSelector::include` already
/// has.
fn Is_Rule_Selected(selected: &[RuleId], rule: &str) -> bool
{
    return selected.is_empty() || selected.iter().any(|id| return id.As_Str() == rule);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::ProviderId;
    use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
    // Named here rather than beside the module's own imports: production code no longer
    // names this rule, and putting it back up there to satisfy a test would undo exactly
    // what `OD-RULES-027` removed.
    use nomos_rules::COMPLETENESS_MIRROR;

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
        let root = Path::new(".");

        let outcome = Run(
            &sources,
            RunContext {
                variant: Test_Variant(),
                root,
                launcher: &StdProcessLauncher,
                filesystem: &StdFileSystem,
                environment: &StdEnvironment,
                workspace: &mut None,
                store: &mut MemoryFactStore::New(),
            },
            &selected,
        );

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
        let root = Path::new(".");

        let mut workspace = None;
        let mut store = MemoryFactStore::New();
        let mut reassessment = RuleReassessmentCache::New();

        let first = Run_Reassessing(
            &sources,
            RunContext {
                variant: Test_Variant(),
                root,
                launcher: &StdProcessLauncher,
                filesystem: &StdFileSystem,
                environment: &StdEnvironment,
                workspace: &mut workspace,
                store: &mut store,
            },
            &selected,
            &mut reassessment,
        );
        assert!(matches!(first, CheckOutcome::Judged { .. }), "a tree the provider can read must be judged");
        let recorded_after_first = reassessment.Recorded();
        assert_eq!(recorded_after_first, 2, "both selected rules must run their real closure the first time, with nothing yet cached");

        let second = Run_Reassessing(
            &sources,
            RunContext {
                variant: Test_Variant(),
                root,
                launcher: &StdProcessLauncher,
                filesystem: &StdFileSystem,
                environment: &StdEnvironment,
                workspace: &mut workspace,
                store: &mut store,
            },
            &selected,
            &mut reassessment,
        );
        assert!(matches!(second, CheckOutcome::Judged { .. }), "a tree the provider can read must be judged");

        assert_eq!(
            reassessment.Recorded(),
            recorded_after_first + 1,
            "only the limits-policy rule should have run its closure again; the syntax-only rule's prior findings should have been reused"
        );
    }

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
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
}

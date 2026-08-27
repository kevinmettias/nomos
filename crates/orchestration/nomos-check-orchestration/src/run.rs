//! Running one check over already-walked source, apart from finding that source or
//! rendering what came of it.

use nomos_analysis::{Context, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::ProcessLauncher;
use nomos_rules::{
    Check_Completeness_Mirrors, Check_Cross_Language_Correspondence, Check_Dependency_Direction,
    Check_Dependency_Policy, Check_Every_Member_Declares_A_Band, Check_Lint_Diagnostics, Check_Naming_Convention,
    Check_Unread_Reaches_A_Finding, SourceFile, COMPLETENESS_MIRROR, CROSS_LANGUAGE_CORRESPONDENCE,
    DEPENDENCY_COMPLETENESS, DEPENDENCY_DIRECTION, DEPENDENCY_POLICY, LINT_DIAGNOSTICS, NAMING_CONVENTION,
    UNREAD_REACHES_FINDING,
};
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::composition::{Recognized_Syntax_Provider, Registered};
use crate::facts::{
    DependencyMaterialization, Ingested, LintMaterialization, Materialize_Dependencies, Materialize_Lint,
    Materialize_Policy, Materialize_Reachability, Materialize_Syntax, PolicyMaterialization,
};
use crate::outcome::{Claim_Of, Examined};
use crate::CheckOutcome;

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
/// `sources` is the walk, already done -- this crate has no [`nomos_platform::FileSystem`]
/// port to walk a directory through, the same reason `nomos-cli::check::sources::Walked`
/// stayed in the composition root. `variant` is what that root's own binary was compiled
/// as, read through `env!` there because that macro resolves against the *compiling*
/// crate and cannot be read correctly from this one. `root` is the tree `sources` was
/// walked from -- carried separately because the dependency-edges, lint-diagnostics and
/// dependency-policy providers each run their own subprocess (`cargo metadata`, `cargo
/// clippy`, `cargo deny`) rather than reading bytes `sources` already holds; every other
/// provider in this workspace is a
/// pure function over bytes a caller already read. `launcher` is what those subprocess
/// calls run through -- generic the same way `nomos_work_orchestration::Run` is generic over
/// [`nomos_platform`]'s traits, so this crate depends on `nomos-platform` and not on any
/// concrete implementation of it; the composition root supplies one.
///
/// Writes nothing and never exits: [`CheckOutcome`] is the whole answer, the same
/// division `nomos_work_orchestration::Run` draws around [`nomos_work_orchestration`]'s own
/// `WorkOutcome`. An empty `sources` is a composition root's decision
/// (`CheckOutcome::NoSource`) made before this function is ever called, not a case this
/// function classifies.
#[must_use]
pub fn Run<P: ProcessLauncher>(sources: &[SourceFile], variant: BuildVariant, root: &Path, launcher: &P, selected: &[RuleId]) -> CheckOutcome
{
    let recognized = Recognized(sources);
    let sources: &[SourceFile] = &recognized;

    let (registry, context) = match Composed(sources, variant)
    {
        Ok(composed) => composed,
        Err(outcome) => return outcome,
    };

    let mut store = MemoryFactStore::New();
    let facts = match Materialized_Syntax_Facts(sources, &context, &mut store)
    {
        Some(facts) => facts,
        None => return CheckOutcome::NoFacts { files: sources.len() },
    };

    let environment = RunEnvironment { root, launcher, registry: &registry, context };
    let findings = Judged_Over(sources, environment, &mut store, selected);

    return Outcome_Of(sources.len(), facts, findings);
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

/// [`Run`]'s own root, launcher, registry and composed context -- everything [`Judged_Over`]
/// needs beside the sources and store it is handed separately, grouped so that function's
/// parameter list names one environment instead of four loose values.
struct RunEnvironment<'a, P: ProcessLauncher>
{
    root: &'a Path,
    launcher: &'a P,
    registry: &'a Registry,
    context: Context,
}

/// Every capability [`Run`] can materialize, judged -- the two steps [`Run`] itself used to
/// inline, composed here so its own body names one step instead of four.
fn Judged_Over<P: ProcessLauncher>(sources: &[SourceFile], environment: RunEnvironment<'_, P>, store: &mut MemoryFactStore, selected: &[RuleId]) -> Vec<Finding>
{
    let mut materialization_env = MaterializationEnv { root: environment.root, context: &environment.context, store, launcher: environment.launcher };
    let capabilities = Materialize_Capabilities(sources, &mut materialization_env, selected);

    let judge_env = JudgeEnv { store, registry: environment.registry, context: environment.context };
    return Judged(sources, capabilities, judge_env, selected);
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
pub(crate) fn Recognized(sources: &[SourceFile]) -> Vec<SourceFile>
{
    return sources
        .iter()
        .cloned()
        .map(|mut source| {
            source.preferred_syntax_provider = Recognized_Syntax_Provider(&source.path);
            return source;
        })
        .collect();
}

/// Whether `rule` is one `selected` asks for -- every rule when `selected` is empty, the same
/// "empty is everything" default `nomos_gate_orchestration::RuleSelector::include` already
/// has.
fn Wants(selected: &[RuleId], rule: &str) -> bool
{
    return selected.is_empty() || selected.iter().any(|id| return id.As_Str() == rule);
}

/// The registry composed and `sources` ingested through it, or the [`CheckOutcome`] that
/// already answers the run when either step refuses.
fn Composed(sources: &[SourceFile], variant: BuildVariant) -> Result<(Registry, Context), CheckOutcome>
{
    let registry = Registered().map_err(CheckOutcome::Contradictory)?;
    let context = Ingested(sources, &registry, variant).map_err(|_error| return CheckOutcome::Unreadable)?;

    return Ok((registry, context));
}

/// The whole run, once judging is done -- how many files and facts it examined, and the
/// claim its own findings support.
fn Outcome_Of(files: usize, facts: usize, findings: Vec<Finding>) -> CheckOutcome
{
    let examined = Examined { files, facts };
    let claim = Claim_Of(&findings);

    return CheckOutcome::Judged { findings, examined, claim };
}

/// The dependency-edges, lint-diagnostics, dependency-policy and reachability facts,
/// materialized into `store` alongside the syntax facts [`Run`] already wrote -- the
/// capabilities beside `syntax.items` that this crate's registration composes, each with
/// its own materialization step for the reasons [`Materialize_Dependencies`],
/// [`Materialize_Lint`], [`Materialize_Policy`] and [`Materialize_Reachability`] give.
///
/// Each runs only when `selected` asks for a rule it feeds -- `DEPENDENCY_DIRECTION` or
/// `DEPENDENCY_COMPLETENESS` for the first (both judge the same `dependencies.sources`),
/// `LINT_DIAGNOSTICS` for the second, `DEPENDENCY_POLICY` for the third,
/// `UNREAD_REACHES_FINDING` for the fourth, per `OD-GATE-017`. Skipping
/// `Materialize_Dependencies`, `Materialize_Lint` or `Materialize_Policy` skips its own
/// subprocess launch entirely, not merely its finding's place in a later disposition.
fn Materialize_Capabilities<P: ProcessLauncher>(
    sources: &[SourceFile],
    env: &mut MaterializationEnv<'_, P>,
    selected: &[RuleId],
) -> CapabilityMaterialization
{
    let dependencies = Materialize_Dependency_Section(env, selected);
    let lint = Materialize_Lint_Section(env, selected);
    let policy = Materialize_Policy_Section(env, selected);
    Materialize_Reachability_Section(sources, env, selected);

    return Capability_Materialization_Of(dependencies, lint, policy);
}

/// The `root`, `context`, `store` and `launcher` every [`Materialize_Capabilities`] section
/// reads or writes through -- grouped into one value so that function takes those four as
/// one parameter rather than four.
struct MaterializationEnv<'a, P: ProcessLauncher>
{
    root: &'a Path,
    context: &'a Context,
    store: &'a mut MemoryFactStore,
    launcher: &'a P,
}

/// The dependency-edges section: [`Materialize_Dependencies`] when `selected` feeds on it,
/// an empty result otherwise.
fn Materialize_Dependency_Section<P: ProcessLauncher>(
    env: &mut MaterializationEnv<'_, P>,
    selected: &[RuleId],
) -> DependencyMaterialization
{
    if Wants(selected, DEPENDENCY_DIRECTION) || Wants(selected, DEPENDENCY_COMPLETENESS)
    {
        return Materialize_Dependencies(env.root, env.context, env.store, env.launcher);
    }

    return DependencyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The lint-diagnostics section: [`Materialize_Lint`] when `selected` feeds on it, an empty
/// result otherwise.
fn Materialize_Lint_Section<P: ProcessLauncher>(
    env: &mut MaterializationEnv<'_, P>,
    selected: &[RuleId],
) -> LintMaterialization
{
    if Wants(selected, LINT_DIAGNOSTICS)
    {
        return Materialize_Lint(env.root, env.context, env.store, env.launcher);
    }

    return LintMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The dependency-policy section: [`Materialize_Policy`] when `selected` feeds on it, an
/// empty result otherwise.
fn Materialize_Policy_Section<P: ProcessLauncher>(
    env: &mut MaterializationEnv<'_, P>,
    selected: &[RuleId],
) -> PolicyMaterialization
{
    if Wants(selected, DEPENDENCY_POLICY)
    {
        return Materialize_Policy(env.root, env.context, env.store, env.launcher);
    }

    return PolicyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The reachability section: [`Materialize_Reachability`] when `selected` feeds on it --
/// writes into `env.store` directly and produces no return value of its own, the same shape
/// the call it wraps already has.
fn Materialize_Reachability_Section<P: ProcessLauncher>(
    sources: &[SourceFile],
    env: &mut MaterializationEnv<'_, P>,
    selected: &[RuleId],
)
{
    if Wants(selected, UNREAD_REACHES_FINDING)
    {
        Materialize_Reachability(sources, env.context, env.store);
    }
}

/// The assembly section: what the three source-and-finding materializations produced,
/// gathered into one [`CapabilityMaterialization`].
fn Capability_Materialization_Of(
    dependencies: DependencyMaterialization,
    lint: LintMaterialization,
    policy: PolicyMaterialization,
) -> CapabilityMaterialization
{
    return CapabilityMaterialization {
        dependency_sources: dependencies.sources,
        dependency_findings: dependencies.findings,
        lint_sources: lint.sources,
        lint_findings: lint.findings,
        policy_sources: policy.sources,
        policy_findings: policy.findings,
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
}

/// Every finding [`Rule_Findings`] produces over `sources` and `capabilities`' own source
/// lists, plus whatever [`Materialize_Capabilities`] already found on its own via
/// [`Capability_Findings`] (a failed dependency or lint materialization, reported rather
/// than judged) -- unconditionally, since each such finding already carries its own rule
/// and a caller that did not select it would never have triggered the materialization
/// that raises it.
fn Judged(sources: &[SourceFile], capabilities: CapabilityMaterialization, env: JudgeEnv<'_>, selected: &[RuleId]) -> Vec<Finding>
{
    let mut reader = Reader::On(env.store, env.registry, env.context);

    let mut findings = Rule_Findings(sources, &capabilities, &mut reader, selected);
    findings.extend(Capability_Findings(capabilities));

    return findings;
}

/// The `store`, `registry` and `context` [`Judged`] reads the [`Reader`] from -- grouped
/// into one value so that function takes those three as one parameter rather than three.
struct JudgeEnv<'a>
{
    store: &'a MemoryFactStore,
    registry: &'a Registry,
    context: Context,
}

/// Every finding the completeness, naming-convention, dependency-direction,
/// dependency-completeness, lint-diagnostics, dependency-policy, unread-reaches-finding and
/// cross-language-correspondence rules `selected` asks for produce over `sources` and
/// `capabilities`' own source lists.
fn Rule_Findings(
    sources: &[SourceFile],
    capabilities: &CapabilityMaterialization,
    reader: &mut Reader<'_, '_>,
    selected: &[RuleId],
) -> Vec<Finding>
{
    const RULE_COUNT: usize = 8;
    let rules: [(&str, &dyn Fn(&mut Reader<'_, '_>) -> Vec<Finding>); RULE_COUNT] = [
        (COMPLETENESS_MIRROR, &|reader| return Check_Completeness_Mirrors(sources, reader)),
        (NAMING_CONVENTION, &|reader| return Check_Naming_Convention(sources, reader)),
        (DEPENDENCY_DIRECTION, &|reader| return Check_Dependency_Direction(&capabilities.dependency_sources, reader)),
        (DEPENDENCY_COMPLETENESS, &|reader| return Check_Every_Member_Declares_A_Band(&capabilities.dependency_sources, reader)),
        (LINT_DIAGNOSTICS, &|reader| return Check_Lint_Diagnostics(&capabilities.lint_sources, reader)),
        (DEPENDENCY_POLICY, &|reader| return Check_Dependency_Policy(&capabilities.policy_sources, reader)),
        (UNREAD_REACHES_FINDING, &|reader| return Check_Unread_Reaches_A_Finding(sources, reader)),
        (CROSS_LANGUAGE_CORRESPONDENCE, &|reader| return Check_Cross_Language_Correspondence(sources, reader)),
    ];

    let mut findings = Vec::new();
    for (rule, check) in rules
    {
        if Wants(selected, rule)
        {
            let rule_findings = check(reader);
            findings.extend(rule_findings);
        }
    }

    return findings;
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

    return findings;
}

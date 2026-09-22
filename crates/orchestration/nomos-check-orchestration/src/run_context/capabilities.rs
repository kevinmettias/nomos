//! Materializing every capability family a rule selection feeds on, one section per family,
//! and the assembly that gathers what they produced.
//!
//! [`Materialize_Capabilities`] is this module's whole contract; everything below it is a
//! step it names. The sections differ in what they read and what they write, but they share
//! one shape -- run only when a selected rule declares the family, and record into `changed`
//! the family that actually moved -- so each is passed through [`Materialization_Tracking`] rather than
//! repeating that wrapper itself. The seven repository-declared policy families are that same
//! shape over seven different facts, so they are driven from [`Policy_Families`] and
//! dispatched by [`Materialize_Policy_Family`]. Spelling those seven out as seven more
//! statements beside the three that return a value made one run of ten structurally identical
//! statements, which this file's own intrafile-duplication rule reads as one concept written
//! ten times -- which it was.

use nomos_contracts::RuleId;
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use nomos_rules::{RequiredFact, SourceFile};

use crate::composed_providers::WorkspacePolicyProvider;
use crate::facts::{
    DependencyMaterialization, LintMaterialization, Materialize_Dependencies, Materialize_Lint,
    Materialize_Policy, Materialize_Policy_Fact, Materialize_Reachability, Materialize_Review,
    PolicyMaterialization, PolicyReading, ReviewMaterialization, Subprocess, WorkspaceReading,
};

use super::{CapabilityMaterialization, Is_Rule_Selected, MaterializationEnvironment, MaterializedCapability};

/// The dependency-edges, lint-diagnostics, dependency-policy, reachability and
/// repository-declared policy facts, materialized into `store` alongside the syntax facts
/// [`crate::run_context::Run`] already wrote -- the capabilities beside `syntax.items` that
/// this crate's registration composes, each with its own materialization step for the reasons
/// [`Materialize_Dependencies`], [`Materialize_Lint`], [`Materialize_Policy`],
/// [`Materialize_Reachability`] and [`Materialize_Naming_Policy`] give.
///
/// Each runs only when a selected rule declares the family it writes, which
/// [`Demanded_Families`] computes once as the union of `nomos_rules::RuleDescriptor::requires`
/// over the selection. Which rules those are is not restated here and is not written down in
/// this module at all: `DESCRIPTORS` is where a rule declares what it reads, and a second copy
/// of that relation is what `P102` removed after the two had drifted. Skipping
/// [`Materialize_Dependencies`], [`Materialize_Lint`], [`Materialize_Policy`] or
/// [`Materialize_Naming_Policy`] skips its own subprocess launch or filesystem read entirely,
/// not merely its finding's place in a later disposition.
///
/// Each call also names, into `changed`, the family it actually wrote to.
/// `crate::run_context::rule_reassessment_cache::RuleReassessmentCache` reads `changed` to
/// decide which rules are still safe to reuse; a family missing from it because this function
/// forgot to report it would let a stale rule's prior findings stand in for a real one, which
/// is why every section below is wrapped rather than only the ones a caller might expect to
/// benefit.
///
/// # Which families prove their fact is current
///
/// All of them, as of `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY`. Until that item, only
/// `Materialize_Syntax` checked currency and every other section wrote unconditionally
/// whenever `selected` triggered it, so eleven families reported themselves *changed* on
/// every call they were selected for and every rule declaring one was re-judged on every call
/// over a reused store. `P40-INCREMENTAL-DEMAND-DRIVEN-RECOMPUTE-2` was declined on exactly
/// that measurement. Every write below now routes through `crate::facts::currency`, whose own
/// doc carries the argument -- including why a per-subject currency check is not the demand
/// planner `OD-RULES-009` declines and `OD-ROADMAP-003` conditions its lapse on: demand still
/// decides whether a family is produced, and currency decides only whether the write is
/// redundant.
///
/// What is still paid on every demanded call, and what no reading of `changed` should be
/// taken to deny: the provider itself. `Materialize_Syntax` alone skips a subject's *parse*,
/// because a caller can rebuild a syntax key from the source bytes it already holds. Every
/// other provider here files with an empty `semantic_inputs` and states why, so its check can
/// only run once the provider has answered -- the subprocess still launches, the policy file
/// is still read, the reachability parse still happens. Skipping those needs each provider to
/// publish a digest of its own inputs, which is territory in the provider crates and not in
/// this one.
pub(super) fn Materialize_Capabilities<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
    changed: &mut Vec<RequiredFact>,
) -> CapabilityMaterialization
{
    let demanded = Demanded_Families(selected);
    let demanded = demanded.as_slice();

    let dependencies = Materialization_Tracking(env, changed, RequiredFact::DependencyEdges, |env| return Materialize_Dependency_Section(env, demanded));
    let lint = Materialization_Tracking(env, changed, RequiredFact::LintDiagnostics, |env| return Materialize_Lint_Section(env, demanded));
    let policy = Materialization_Tracking(env, changed, RequiredFact::DependencyPolicy, |env| return Materialize_Policy_Section(env, demanded));

    Materialization_Tracking(env, changed, RequiredFact::Reachability, |env| Materialize_Reachability_Section(sources, env, demanded));

    for family in Policy_Families()
    {
        Materialization_Tracking(env, changed, family, |env| Materialize_Policy_Family(env, demanded, family));
    }

    let review = Materialization_Tracking(env, changed, RequiredFact::ReviewFindings, |_env| return Materialize_Review_Section(demanded));

    return Capability_Materialization_Of(dependencies, lint, policy, review);
}

/// The fact families the selected rules declare they need.
///
/// The union of [`nomos_rules::RuleDescriptor::requires`] over the selected rules, and
/// nothing else. This is the one place the rule-to-fact relation is read, rather than the
/// second place it used to be written: each section below used to re-derive "is any rule
/// that feeds on this family selected" from a hand-written list of rule identifiers, which
/// is the same relation `DESCRIPTORS` already declares. Two statements of one relation
/// drift, and these had -- six rules declare `RequiredFact::LimitsPolicy` and the guard
/// named five, so `NESTING_DEPTH` selected without its five siblings ran against
/// `MAX_NESTING_DEPTH`'s built-in default rather than the limit the repository configured,
/// silently. `OD-RULES-027` closed the identical shape between the gate registry and the
/// composed rule list by deriving one from the other; this is that move, one axis over.
///
/// It is a union over declarations and nothing more. It reads no cache state, no provider
/// cost, no dependency structure between families, and never asks whether a fact is already
/// materialized -- the line this module's own documentation draws, and the point past which
/// it would become the demand planner `OD-RULES-009` has declined across eight rounds.
///
/// `crate::facts::currency` does ask the store a question, and this is the function that
/// keeps that from being the same thing. The forbidden sentence `OD-ROADMAP-003` names is
/// *materialize this family unless the store already holds it* -- a **demand** conditioned on
/// store state, which would be a line in this function's own body. There is none: a family
/// any selected descriptor declares is demanded, its section runs, and its provider answers
/// in full. Only the write of an answer identical to one the store is already serving is
/// skipped, one subject at a time, after the production has happened.
fn Demanded_Families(selected: &[RuleId]) -> Vec<RequiredFact>
{
    let mut demanded: Vec<RequiredFact> = Vec::new();

    for descriptor in nomos_rules::DESCRIPTORS
    {
        if !Is_Rule_Selected(selected, descriptor.id)
        {
            continue;
        }

        for family in descriptor.requires
        {
            if !demanded.contains(family)
            {
                demanded.push(*family);
            }
        }
    }

    return demanded;
}

/// Runs `section`, and records `family` into `changed` if `env.store` gained a new
/// materialization while it ran.
///
/// The counter is the signal because it now answers the question: a section skipped for want
/// of a selected rule writes nothing, and a section that ran writes only what
/// `crate::facts::currency` found the store was not already serving. Before
/// `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY` every section but the syntax one wrote
/// unconditionally, so this wrapper reported eleven families as changed on every call they
/// were selected for -- sound, since it can only over-report, and worth nothing to a caller
/// reusing a store.
///
/// It still over-reports in one direction that is worth naming: a family is recorded as
/// changed when *any* of its subjects moved, so one edited workspace member puts every rule
/// reading `nomos.cap.lint.diagnostics` back in play rather than the ones that read the
/// edited member. `changed` is a list of families and not of subjects, and narrowing it is a
/// question about what a rule closure can be given rather than about this wrapper --
/// `P40-INCREMENTAL-DEMAND-DRIVEN-RECOMPUTE-2`'s decline says why: a rule reads the whole
/// source list and returns one `Vec<Finding>`, not a per-subject partial result.
fn Materialization_Tracking<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment, Answer>(
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

/// The dependency-edges section: [`Materialize_Dependencies`] through the composed provider
/// when `selected` feeds on it, an empty result otherwise.
fn Materialize_Dependency_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
) -> DependencyMaterialization
{
    if demanded.contains(&RequiredFact::DependencyEdges)
    {
        let provider = env.providers.dependencies;

        return Materialize_Dependencies(Workspace_Reading(env), provider);
    }

    return DependencyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The lint-diagnostics section: [`Materialize_Lint`] through the composed provider when
/// `selected` feeds on it, an empty result otherwise.
fn Materialize_Lint_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
) -> LintMaterialization
{
    if demanded.contains(&RequiredFact::LintDiagnostics)
    {
        let provider = env.providers.lint;

        return Materialize_Lint(Workspace_Reading(env), provider);
    }

    return LintMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The dependency-policy section: [`Materialize_Policy`] through the composed provider when
/// `selected` feeds on it, an empty result otherwise.
fn Materialize_Policy_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
) -> PolicyMaterialization
{
    if demanded.contains(&RequiredFact::DependencyPolicy)
    {
        let provider = env.providers.dependency_policy;

        return Materialize_Policy(Workspace_Reading(env), provider);
    }

    return PolicyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The root, context, store and the two subprocess ports one subprocess-backed provider is
/// run through, as the three sections above each hand them over.
fn Workspace_Reading<'a, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &'a mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
) -> WorkspaceReading<'a, Launcher, Env>
{
    return WorkspaceReading {
        root: env.root,
        context: env.context,
        store: env.store,
        subprocess: Subprocess { launcher: env.launcher, environment: env.environment },
    };
}

/// The reachability section: [`Materialize_Reachability`] through the composed provider when
/// `selected` feeds on it -- writes into `env.store` directly and produces no return value of
/// its own, the same shape the call it wraps already has.
fn Materialize_Reachability_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::Reachability)
    {
        Materialize_Reachability(sources, env.context, env.store, env.providers.reachability);
    }
}

/// The eight repository-declared policy families, in the order [`Materialize_Capabilities`]
/// runs them -- the one list [`Materialize_Policy_Family`] is a total function over, so a
/// family added to the set has one row here and one arm there rather than a tenth and an
/// eleventh statement above.
fn Policy_Families() -> Vec<RequiredFact>
{
    return vec![
        RequiredFact::NamingPolicy,
        RequiredFact::LimitsPolicy,
        RequiredFact::ArchitectureDeclaration,
        RequiredFact::ScriptingPolicy,
        RequiredFact::GoalsPolicy,
        RequiredFact::WordsPolicy,
        RequiredFact::TestMaterialPolicy,
        RequiredFact::RequirementTrace,
    ];
}

/// The composed provider `family` is answered by, over the whole [`RequiredFact`] set so
/// that a new variant is a compile error here rather than a family that silently
/// materializes nothing.
///
/// This match is the declared table `OD-ROADMAP-003`'s surviving constraint requires a
/// materialization section to be: one row per family, naming a provider the composition
/// supplied, reading no store state, no cost and no prior materialization. It replaced eight
/// section functions that differed only in which provider crate they called, and the
/// difference moved here because here is where it can be checked -- the six variants that
/// are not repository-declared policies are named as answering `None` deliberately.
/// `Reachability` is its own section one function up, because it reads `sources`, which the
/// policy families do not; the remaining five belong to the three sections that return a
/// value, which [`Materialize_Capabilities`] calls directly.
fn Policy_Provider_For<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &MaterializationEnvironment<'_, Launcher, Fs, Env>,
    family: RequiredFact,
) -> Option<WorkspacePolicyProvider<Fs>>
{
    return match family
    {
        RequiredFact::NamingPolicy => Some(env.providers.naming_policy),
        RequiredFact::LimitsPolicy => Some(env.providers.limits_policy),
        RequiredFact::ArchitectureDeclaration => Some(env.providers.architecture),
        RequiredFact::ScriptingPolicy => Some(env.providers.scripting_policy),
        RequiredFact::GoalsPolicy => Some(env.providers.goals_policy),
        RequiredFact::WordsPolicy => Some(env.providers.words_policy),
        RequiredFact::TestMaterialPolicy => Some(env.providers.test_material_policy),
        RequiredFact::RequirementTrace => Some(env.providers.requirement_trace),
        RequiredFact::SyntaxItems
        | RequiredFact::DependencyEdges
        | RequiredFact::LintDiagnostics
        | RequiredFact::DependencyPolicy
        | RequiredFact::Reachability
        | RequiredFact::ReviewFindings => None,
    };
}

/// One repository-declared policy family, materialized through its composed provider when a
/// selected rule declares it.
///
/// Eight families share this one body because, read through a port, they have nothing left
/// to differ in: each reads one declaration from the repository root and answers at most one
/// fact. What differs between them is which provider answers, which
/// [`Policy_Provider_For`] states, and what a rule does when the fact is absent, which is the
/// rule's own business -- `OD-CAPABILITY-004` makes absence a silent "no override" for seven
/// of them, while the three dependency rules reading
/// `nomos.cap.architecture.declaration` report a declaration they could not read rather than
/// judge against a default that cannot exist. Neither difference is visible here, and neither
/// ever was: this step's answer for an absent fact has always been the same.
fn Materialize_Policy_Family<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
    family: RequiredFact,
)
{
    let Some(provider) = Policy_Provider_For(env, family)
    else
    {
        return;
    };

    if !demanded.contains(&family)
    {
        return;
    }

    Materialize_Policy_Fact(
        PolicyReading { root: env.root, context: env.context, store: env.store, filesystem: env.filesystem },
        provider,
    );
}

/// The review-finding section: [`Materialize_Review`] when `selected` feeds on it, an
/// empty result otherwise.
///
/// Unlike [`Materialize_Dependency_Section`], [`Materialize_Lint_Section`] and
/// [`Materialize_Policy_Section`], this section takes no [`MaterializationEnvironment`]:
/// [`Materialize_Review`]'s own doc gives the reason -- this connector's one provider
/// answers about one already-identified external review comment, and nothing in this
/// call chain names one yet, so there is nothing here for a root, a launcher or a
/// filesystem to be read through. [`Materialize_Capabilities`]' own `Materialization_Tracking` wrapper still
/// calls this the identical way, through a closure that ignores the environment it is
/// handed -- the same `|_reader|` idiom this crate's own text-only `ComposedRule` entries
/// already use for the identical reason.
fn Materialize_Review_Section(demanded: &[RequiredFact]) -> ReviewMaterialization
{
    if demanded.contains(&RequiredFact::ReviewFindings)
    {
        return Materialize_Review();
    }

    return ReviewMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The assembly section: what the three source-and-finding materializations produced,
/// gathered into one [`CapabilityMaterialization`], each pair kept together as the
/// [`MaterializedCapability`] it already was one layer down.
fn Capability_Materialization_Of(
    dependencies: DependencyMaterialization,
    lint: LintMaterialization,
    policy: PolicyMaterialization,
    review: ReviewMaterialization,
) -> CapabilityMaterialization
{
    return CapabilityMaterialization {
        dependency: MaterializedCapability { sources: dependencies.sources, findings: dependencies.findings },
        lint: MaterializedCapability { sources: lint.sources, findings: lint.findings },
        policy: MaterializedCapability { sources: policy.sources, findings: policy.findings },
        review: MaterializedCapability { sources: review.sources, findings: review.findings },
    };
}

#[cfg(test)]
mod demand_tests;

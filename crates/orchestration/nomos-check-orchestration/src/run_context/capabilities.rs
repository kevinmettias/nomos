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

use crate::facts::{
    DependencyMaterialization, LintMaterialization, Materialize_Architecture, Materialize_Dependencies,
    Materialize_Goals_Policy, Materialize_Limits_Policy, Materialize_Lint, Materialize_Naming_Policy,
    Materialize_Policy, Materialize_Reachability, Materialize_Requirement_Trace, Materialize_Review,
    Materialize_Scripting_Policy, Materialize_Words_Policy, PolicyMaterialization, ReviewMaterialization,
    Subprocess,
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
/// Each call also names, into `changed`, the family it just wrote to at all -- every one of
/// them does, unconditionally, whenever `selected` triggers it at all: none of them has
/// `Materialize_Syntax`'s own currency check yet.
/// `crate::run_context::rule_reassessment_cache::RuleReassessmentCache` reads `changed` to
/// decide which rules are still safe to reuse; a family missing from it because this function
/// forgot to report it would let a stale rule's prior findings stand in for a real one, which
/// is why every section below is wrapped rather than only the ones a caller might expect to
/// benefit.
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
/// materialization while it ran -- the one signal available today for "did this family just
/// change," since a skipped section (its own gating rule not selected) writes nothing and a
/// run one writes unconditionally.
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

/// The dependency-edges section: [`Materialize_Dependencies`] when `selected` feeds on it,
/// an empty result otherwise.
fn Materialize_Dependency_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
) -> DependencyMaterialization
{
    if demanded.contains(&RequiredFact::DependencyEdges)
    {
        return Materialize_Dependencies(env.root, env.context, env.store, Subprocess { launcher: env.launcher, environment: env.environment });
    }

    return DependencyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The lint-diagnostics section: [`Materialize_Lint`] when `selected` feeds on it, an empty
/// result otherwise.
fn Materialize_Lint_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
) -> LintMaterialization
{
    if demanded.contains(&RequiredFact::LintDiagnostics)
    {
        return Materialize_Lint(env.root, env.context, env.store, Subprocess { launcher: env.launcher, environment: env.environment });
    }

    return LintMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The dependency-policy section: [`Materialize_Policy`] when `selected` feeds on it, an
/// empty result otherwise.
fn Materialize_Policy_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
) -> PolicyMaterialization
{
    if demanded.contains(&RequiredFact::DependencyPolicy)
    {
        return Materialize_Policy(env.root, env.context, env.store, Subprocess { launcher: env.launcher, environment: env.environment });
    }

    return PolicyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The reachability section: [`Materialize_Reachability`] when `selected` feeds on it --
/// writes into `env.store` directly and produces no return value of its own, the same shape
/// the call it wraps already has.
fn Materialize_Reachability_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::Reachability)
    {
        Materialize_Reachability(sources, env.context, env.store);
    }
}

/// The seven repository-declared policy families, in the order [`Materialize_Capabilities`]
/// runs them -- the one list [`Materialize_Policy_Family`] is a total function over, so a
/// family added to the set has one row here and one arm there rather than a ninth and a
/// tenth statement above.
fn Policy_Families() -> Vec<RequiredFact>
{
    return vec![
        RequiredFact::NamingPolicy,
        RequiredFact::LimitsPolicy,
        RequiredFact::ArchitectureDeclaration,
        RequiredFact::ScriptingPolicy,
        RequiredFact::GoalsPolicy,
        RequiredFact::WordsPolicy,
        RequiredFact::RequirementTrace,
    ];
}

/// The one policy section `family` names, over the whole [`RequiredFact`] set so that a new
/// variant is a compile error here rather than a family that silently materializes nothing.
/// The six variants that are not repository-declared policies are named as doing nothing
/// deliberately: `Reachability` is its own section one function up (it reads `sources`,
/// which the other sections do not), and the remaining five belong to the three sections that
/// return a value, which [`Materialize_Capabilities`] calls directly.
fn Materialize_Policy_Family<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
    family: RequiredFact,
)
{
    match family
    {
        RequiredFact::NamingPolicy => Materialize_Naming_Policy_Section(env, demanded),
        RequiredFact::LimitsPolicy => Materialize_Limits_Policy_Section(env, demanded),
        RequiredFact::ArchitectureDeclaration => Materialize_Architecture_Section(env, demanded),
        RequiredFact::ScriptingPolicy => Materialize_Scripting_Policy_Section(env, demanded),
        RequiredFact::GoalsPolicy => Materialize_Goals_Policy_Section(env, demanded),
        RequiredFact::WordsPolicy => Materialize_Words_Policy_Section(env, demanded),
        RequiredFact::RequirementTrace => Materialize_Requirement_Trace_Section(env, demanded),
        RequiredFact::SyntaxItems
        | RequiredFact::DependencyEdges
        | RequiredFact::LintDiagnostics
        | RequiredFact::DependencyPolicy
        | RequiredFact::Reachability
        | RequiredFact::ReviewFindings => {}
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
/// `crate::run_context::judging::Findings_For_Selected_Rules` yet, so gating on it here would
/// materialize a fact for a rule that never runs.
fn Materialize_Naming_Policy_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::NamingPolicy)
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
fn Materialize_Limits_Policy_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::LimitsPolicy)
    {
        Materialize_Limits_Policy(env.root, env.context, env.store, env.filesystem);
    }
}

/// The architecture section: [`Materialize_Architecture`] when a selected rule declares it.
///
/// Unlike every policy section beside it, an absent fact here is not a fallback to a built-in
/// default. The three dependency rules have no default an architecture could have, so skipping
/// this makes them report a declaration they could not read rather than judge against an
/// assumption -- which is the honest answer and the one `OD-RULES-003` asks for.
fn Materialize_Architecture_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::ArchitectureDeclaration)
    {
        Materialize_Architecture(env.root, env.context, env.store, env.filesystem);
    }
}

/// The scripting-policy section: [`Materialize_Scripting_Policy`] when `selected` feeds on
/// it.
///
/// One composed rule reads this capability, and unlike every other policy section here its
/// absence is not a fallback: `Check_Declared_Tooling_Language_For_Scripts` reports nothing
/// at all without the fact, so before this call existed the rule ran in every check and could
/// never fire.
fn Materialize_Scripting_Policy_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::ScriptingPolicy)
    {
        Materialize_Scripting_Policy(env.root, env.context, env.store, env.filesystem);
    }
}

/// The goals-policy section: [`Materialize_Goals_Policy`] when its one rule is selected.
///
/// The narrowest gate of the four policy sections, because exactly one rule reads this
/// capability -- and the first one whose rule takes no sources at all, so there is nothing
/// here to gate on but the rule's own selection.
fn Materialize_Goals_Policy_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::GoalsPolicy)
    {
        Materialize_Goals_Policy(env.root, env.context, env.store, env.filesystem);
    }
}

/// The words-policy section: [`Materialize_Words_Policy`] when its one rule is selected.
///
/// One rule reads this capability, so the gate is that rule's own selection -- the same
/// shape [`Materialize_Goals_Policy_Section`] has one function above.
fn Materialize_Words_Policy_Section<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::WordsPolicy)
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
fn Materialize_Requirement_Trace_Section<
    Launcher: ProgramLauncher,
    Fs: FileSystem,
    Env: Environment,
>(
    env: &mut MaterializationEnvironment<'_, Launcher, Fs, Env>,
    demanded: &[RequiredFact],
)
{
    if demanded.contains(&RequiredFact::RequirementTrace)
    {
        Materialize_Requirement_Trace(env.root, env.context, env.store, env.filesystem);
    }
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

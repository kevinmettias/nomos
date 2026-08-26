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
    let facts = Materialize_Syntax(sources, &context, &mut store);
    if facts == 0
    {
        return CheckOutcome::NoFacts { files: sources.len() };
    }

    let capabilities = Materialize_Capabilities(sources, root, &context, &mut store, launcher, selected);
    let findings = Judged(sources, capabilities, &store, &registry, context, selected);

    return Outcome_Of(sources.len(), facts, findings);
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
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &P,
    selected: &[RuleId],
) -> CapabilityMaterialization
{
    let dependencies = if Wants(selected, DEPENDENCY_DIRECTION) || Wants(selected, DEPENDENCY_COMPLETENESS)
    {
        Materialize_Dependencies(root, context, store, launcher)
    }
    else
    {
        DependencyMaterialization { sources: Vec::new(), findings: Vec::new() }
    };

    let lint = if Wants(selected, LINT_DIAGNOSTICS)
    {
        Materialize_Lint(root, context, store, launcher)
    }
    else
    {
        LintMaterialization { sources: Vec::new(), findings: Vec::new() }
    };

    let policy = if Wants(selected, DEPENDENCY_POLICY)
    {
        Materialize_Policy(root, context, store, launcher)
    }
    else
    {
        PolicyMaterialization { sources: Vec::new(), findings: Vec::new() }
    };

    if Wants(selected, UNREAD_REACHES_FINDING)
    {
        Materialize_Reachability(sources, context, store);
    }

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

/// Every finding the completeness, naming-convention, dependency-direction,
/// dependency-completeness, lint-diagnostics and unread-reaches-finding rules `selected`
/// asks for produce over `sources` and `capabilities`' own source lists, plus whatever
/// [`Materialize_Capabilities`]
/// already found on its own (a failed dependency or lint materialization, reported rather
/// than judged) -- unconditionally, since each such finding already carries its own rule
/// and a caller that did not select it would never have triggered the materialization
/// that raises it.
fn Judged(
    sources: &[SourceFile],
    capabilities: CapabilityMaterialization,
    store: &MemoryFactStore,
    registry: &Registry,
    context: Context,
    selected: &[RuleId],
) -> Vec<Finding>
{
    let mut reader = Reader::On(store, registry, context);
    let mut findings = Vec::new();

    if Wants(selected, COMPLETENESS_MIRROR)
    {
        findings.extend(Check_Completeness_Mirrors(sources, &mut reader));
    }

    if Wants(selected, NAMING_CONVENTION)
    {
        findings.extend(Check_Naming_Convention(sources, &mut reader));
    }

    if Wants(selected, DEPENDENCY_DIRECTION)
    {
        findings.extend(Check_Dependency_Direction(&capabilities.dependency_sources, &mut reader));
    }

    if Wants(selected, DEPENDENCY_COMPLETENESS)
    {
        findings.extend(Check_Every_Member_Declares_A_Band(&capabilities.dependency_sources, &mut reader));
    }

    if Wants(selected, LINT_DIAGNOSTICS)
    {
        findings.extend(Check_Lint_Diagnostics(&capabilities.lint_sources, &mut reader));
    }

    if Wants(selected, DEPENDENCY_POLICY)
    {
        findings.extend(Check_Dependency_Policy(&capabilities.policy_sources, &mut reader));
    }

    if Wants(selected, UNREAD_REACHES_FINDING)
    {
        findings.extend(Check_Unread_Reaches_A_Finding(sources, &mut reader));
    }

    if Wants(selected, CROSS_LANGUAGE_CORRESPONDENCE)
    {
        findings.extend(Check_Cross_Language_Correspondence(sources, &mut reader));
    }

    findings.extend(capabilities.dependency_findings);
    findings.extend(capabilities.lint_findings);
    findings.extend(capabilities.policy_findings);

    return findings;
}

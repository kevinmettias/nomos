//! Running one check over already-walked source, apart from finding that source or
//! rendering what came of it.

use nomos_analysis::{Context, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::ProcessLauncher;
use nomos_rules::{
    Check_Completeness_Mirrors, Check_Dependency_Direction, Check_Naming_Convention,
    Check_Unread_Reaches_A_Finding, SourceFile, COMPLETENESS_MIRROR, DEPENDENCY_DIRECTION,
    NAMING_CONVENTION, UNREAD_REACHES_FINDING,
};
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::composition::Registered;
use crate::facts::{DependencyMaterialization, Ingested, Materialize_Dependencies, Materialize_Reachability, Materialize_Syntax};
use crate::outcome::{Claim_Of, Examined};
use crate::CheckOutcome;

/// Composes the capability registry, ingests `sources` into one workspace state, materializes
/// a syntax fact per file, and, for each of the completeness, naming-convention,
/// dependency-direction and unread-reaches-finding rules `selected` asks for (every one when
/// `selected` is empty, the same "empty is everything" default
/// `nomos_gate_orchestration::RuleSelector::include` already has), materializes the fact that
/// rule needs and runs it over the result.
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
/// walked from -- carried separately because the dependency-edges provider runs `cargo
/// metadata` rather than reading bytes `sources` already holds, the one provider in this
/// workspace with I/O of its own. `launcher` is what that `cargo metadata` call runs
/// through -- generic the same way `nomos_work_orchestration::Run` is generic over
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
    let findings = Judged(sources, &capabilities.sources, capabilities.findings, &store, &registry, context, selected);

    return Outcome_Of(sources.len(), facts, findings);
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

/// The dependency-edges and reachability facts, materialized into `store` alongside the
/// syntax facts [`Run`] already wrote -- the two capabilities beside `syntax.items` that
/// this crate's registration composes, each with its own materialization step for the
/// reasons [`Materialize_Dependencies`] and [`Materialize_Reachability`] give.
///
/// Each runs only when `selected` asks for the rule it alone feeds -- `DEPENDENCY_DIRECTION`
/// for the first, `UNREAD_REACHES_FINDING` for the second, per `OD-GATE-017`. Skipping
/// `Materialize_Dependencies` skips its own `cargo metadata` launch entirely, not merely its
/// finding's place in a later disposition.
fn Materialize_Capabilities<P: ProcessLauncher>(
    sources: &[SourceFile],
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &P,
    selected: &[RuleId],
) -> CapabilityMaterialization
{
    let dependencies = if Wants(selected, DEPENDENCY_DIRECTION)
    {
        Materialize_Dependencies(root, context, store, launcher)
    }
    else
    {
        DependencyMaterialization { sources: Vec::new(), findings: Vec::new() }
    };

    if Wants(selected, UNREAD_REACHES_FINDING)
    {
        Materialize_Reachability(sources, context, store);
    }

    let DependencyMaterialization { sources, findings } = dependencies;

    return CapabilityMaterialization { sources, findings };
}

/// What [`Materialize_Capabilities`] produced: the dependency-edges sources a rule can judge,
/// and any finding materializing them already raised on its own -- named rather than left as
/// a positional pair, the same reason [`crate::facts::DependencyMaterialization`] exists one
/// layer under it.
struct CapabilityMaterialization
{
    sources: Vec<SourceFile>,
    findings: Vec<Finding>,
}

/// Every finding the completeness, naming-convention, dependency-direction and
/// unread-reaches-finding rules `selected` asks for produce over `sources` and
/// `dependency_sources`, plus whatever [`Materialize_Capabilities`] already found on its own
/// (a failed dependency materialization, reported rather than judged) -- unconditionally,
/// since that finding already carries `DEPENDENCY_DIRECTION` as its own rule and a caller
/// that did not select it would never have triggered the materialization that raises it.
fn Judged(
    sources: &[SourceFile],
    dependency_sources: &[SourceFile],
    mut dependency_findings: Vec<Finding>,
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
        findings.extend(Check_Dependency_Direction(dependency_sources, &mut reader));
    }

    if Wants(selected, UNREAD_REACHES_FINDING)
    {
        findings.extend(Check_Unread_Reaches_A_Finding(sources, &mut reader));
    }

    findings.append(&mut dependency_findings);

    return findings;
}

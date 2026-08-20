//! Running one check over already-walked source, apart from finding that source or
//! rendering what came of it.

use nomos_analysis::{Context, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::Finding;
use nomos_platform::ProcessLauncher;
use nomos_rules::{
    Check_Completeness_Mirrors, Check_Dependency_Direction, Check_Naming_Convention,
    Check_Unread_Reaches_A_Finding, SourceFile,
};
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::composition::Registered;
use crate::facts::{Ingested, Materialize_Dependencies, Materialize_Reachability, Materialize_Syntax};
use crate::outcome::{Claim_Of, Examined};
use crate::CheckOutcome;

/// Composes the capability registry, ingests `sources` into one workspace state,
/// materializes a syntax fact per file, a reachability fact per file and a dependency-edges
/// fact per workspace member, and runs the completeness, naming-convention,
/// dependency-direction and unread-reaches-finding rules over the result.
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
pub fn Run<P: ProcessLauncher>(sources: &[SourceFile], variant: BuildVariant, root: &Path, launcher: &P) -> CheckOutcome
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

    let (dependency_sources, dependency_findings) = Materialize_Capabilities(sources, root, &context, &mut store, launcher);
    let findings = Judged(sources, &dependency_sources, dependency_findings, &store, &registry, context);

    return Outcome_Of(sources.len(), facts, findings);
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
fn Materialize_Capabilities<P: ProcessLauncher>(
    sources: &[SourceFile],
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &P,
) -> (Vec<SourceFile>, Vec<Finding>)
{
    let (dependency_sources, dependency_findings) = Materialize_Dependencies(root, context, store, launcher);
    Materialize_Reachability(sources, context, store);

    return (dependency_sources, dependency_findings);
}

/// Every finding the completeness, naming-convention, dependency-direction and
/// unread-reaches-finding rules produce over `sources` and `dependency_sources`, plus
/// whatever [`Materialize_Capabilities`] already found on its own (a failed dependency
/// materialization, reported rather than judged).
fn Judged(
    sources: &[SourceFile],
    dependency_sources: &[SourceFile],
    mut dependency_findings: Vec<Finding>,
    store: &MemoryFactStore,
    registry: &Registry,
    context: Context,
) -> Vec<Finding>
{
    let mut reader = Reader::On(store, registry, context);

    let completeness_findings = Check_Completeness_Mirrors(sources, &mut reader);
    let mut findings = completeness_findings;

    let naming_findings = Check_Naming_Convention(sources, &mut reader);
    findings.extend(naming_findings);

    let dependency_direction_findings = Check_Dependency_Direction(dependency_sources, &mut reader);
    findings.extend(dependency_direction_findings);

    let unread_findings = Check_Unread_Reaches_A_Finding(sources, &mut reader);
    findings.extend(unread_findings);

    findings.append(&mut dependency_findings);

    return findings;
}

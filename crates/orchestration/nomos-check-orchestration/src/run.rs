//! Running one check over already-walked source, apart from finding that source or
//! rendering what came of it.

use nomos_analysis::{MemoryFactStore, Reader};
use nomos_rules::{Check_Completeness_Mirrors, Check_Dependency_Direction, Check_Naming_Convention, SourceFile};
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::composition::Registered;
use crate::facts::{Ingested, Materialize_Dependencies, Materialize_Syntax};
use crate::outcome::{Claim_Of, Examined};
use crate::CheckOutcome;

/// Composes the capability registry, ingests `sources` into one workspace state,
/// materializes a syntax fact per file and a dependency-edges fact per workspace member,
/// and runs the completeness, naming-convention and dependency-direction rules over the
/// result.
///
/// `sources` is the walk, already done -- this crate has no [`nomos_platform::FileSystem`]
/// port to walk a directory through, the same reason `nomos-cli::check::sources::Walked`
/// stayed in the composition root. `variant` is what that root's own binary was compiled
/// as, read through `env!` there because that macro resolves against the *compiling*
/// crate and cannot be read correctly from this one. `root` is the tree `sources` was
/// walked from -- carried separately because the dependency-edges provider runs `cargo
/// metadata` itself rather than reading bytes `sources` already holds, the one provider in
/// this workspace with I/O of its own.
///
/// Writes nothing and never exits: [`CheckOutcome`] is the whole answer, the same
/// division `nomos_work_orchestration::Run` draws around [`nomos_work_orchestration`]'s own
/// `WorkOutcome`. An empty `sources` is a composition root's decision
/// (`CheckOutcome::NoSource`) made before this function is ever called, not a case this
/// function classifies.
#[must_use]
pub fn Run(sources: &[SourceFile], variant: BuildVariant, root: &Path) -> CheckOutcome
{
    let registry = match Registered()
    {
        Ok(registry) => registry,
        Err(error) => return CheckOutcome::Contradictory(error),
    };

    let Ok(context) = Ingested(sources, &registry, variant)
    else
    {
        return CheckOutcome::Unreadable;
    };

    let mut store = MemoryFactStore::New();
    let facts = Materialize_Syntax(sources, &context, &mut store);
    if facts == 0
    {
        return CheckOutcome::NoFacts { files: sources.len() };
    }

    let (dependency_sources, mut dependency_findings) = Materialize_Dependencies(root, &context, &mut store);

    let mut reader = Reader::On(&store, &registry, context);
    let mut findings = Check_Completeness_Mirrors(sources, &mut reader);
    findings.extend(Check_Naming_Convention(sources, &mut reader));
    findings.extend(Check_Dependency_Direction(&dependency_sources, &mut reader));
    findings.append(&mut dependency_findings);

    let examined = Examined { files: sources.len(), facts };
    let claim = Claim_Of(&findings);

    return CheckOutcome::Judged { findings, examined, claim };
}

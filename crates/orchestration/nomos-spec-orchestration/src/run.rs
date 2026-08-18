//! Running one `nomos spec` verb, apart from parsing its arguments or rendering what it
//! found.

use nomos_spec_project::{Catalogue, Profile, ProjectError};
use nomos_spec_store::StoreError;

use crate::command::SpecCommand;
use crate::corpus::{Assemble, Assembly, CorpusRequest};
use crate::outcome::{NotYetMigrated, SourcesAnswer, SpecOutcome};

/// The shipped projection catalogue.
///
/// Never assembles a store: the catalogue is embedded in `nomos-spec-project`, so listing
/// it must not fail on a machine that cannot open a database. Moved verbatim from
/// `nomos-cli::spec::verb::listing::Profiles`, minus the writing.
///
/// # Errors
///
/// Returns [`ProjectError`] when the embedded catalogue itself fails to parse -- a defect
/// in this build, not in anything the caller did.
pub fn Profiles() -> Result<Vec<Profile>, ProjectError>
{
    let catalogue = Catalogue::Shipped()?;

    return Ok(catalogue.Profiles().to_vec());
}

/// What an already-assembled store was built from, and what it is missing.
///
/// Takes `assembly` rather than assembling one itself, the same reason
/// `nomos_check_orchestration::Run` takes an already-walked source list: a second command
/// in the same invocation may already have paid for the assembly, and re-paying it here
/// would read the corpus twice for one answer.
#[must_use]
pub fn Sources(assembly: &Assembly) -> SourcesAnswer
{
    return SourcesAnswer {
        read: assembly.read.clone(),
        absent: assembly.absent.clone(),
    };
}

/// Runs one `nomos spec` verb over a corpus request, and hands back what it found.
///
/// [`SpecCommand::Profiles`] never assembles a store, matching
/// [`Profiles`]'s own guarantee. Every other variant assembles one from `request` first --
/// including the seven not yet migrated, so that a caller asking `nomos-spec-orchestration`
/// for one of them fails on the same store-assembly error a fully migrated verb would,
/// rather than succeeding vacuously before reaching the part that is not built yet.
#[must_use]
pub fn Run(command: &SpecCommand, request: &CorpusRequest) -> SpecOutcome
{
    if let SpecCommand::Profiles = command
    {
        return SpecOutcome::Profiles(Profiles());
    }

    let assembly = match Assemble(request)
    {
        Ok(assembly) => assembly,
        Err(error) => return Outcome_For(command, Err(error)),
    };

    return Outcome_For(command, Ok(assembly));
}

/// Every non-`Profiles` variant's outcome, given the assembly its verb needs (or the error
/// that kept one from being built).
fn Outcome_For(command: &SpecCommand, assembled: Result<Assembly, StoreError>) -> SpecOutcome
{
    return match command
    {
        SpecCommand::Sources => SpecOutcome::Sources(assembled.map(|assembly| return Sources(&assembly))),
        SpecCommand::Record(_) => SpecOutcome::Record(NotYetMigrated),
        SpecCommand::Table(_) => SpecOutcome::Table(NotYetMigrated),
        SpecCommand::Render(_) => SpecOutcome::Render(NotYetMigrated),
        SpecCommand::Freshness(_) => SpecOutcome::Freshness(NotYetMigrated),
        SpecCommand::Markdown(_) => SpecOutcome::Markdown(NotYetMigrated),
        SpecCommand::Preview(_) => SpecOutcome::Preview(NotYetMigrated),
        SpecCommand::Commit(_) => SpecOutcome::Commit(NotYetMigrated),
        SpecCommand::Profiles => SpecOutcome::Profiles(Profiles()),
    };
}

//! Running one `nomos spec` verb, apart from parsing its arguments or rendering what it
//! found.

mod commit;
mod freshness;
mod markdown;
mod preview;
mod record;
mod render;
mod submit;
mod table;

use nomos_platform::FileSystem;
use nomos_spec_project::{Catalogue, Profile, ProjectError};
use nomos_spec_store::StoreError;

use crate::command::SpecCommand;
use crate::corpus::{Assemble, Assembly, CorpusRequest};
use crate::outcome::{CommitRefusal, FreshnessRefusal, PreviewRefusal, RenderRefusal, SourcesAnswer, SpecOutcome};

pub use commit::Commit;
pub use freshness::Freshness;
pub use markdown::Markdown;
pub use preview::Preview;
pub use record::Record;
pub use render::Render;
pub use submit::Submit;
pub use table::Table;

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
/// Generic over [`FileSystem`] for the four verbs that place or read a file at a path this
/// crate does not fully own ([`SpecCommand::Render`], [`SpecCommand::Freshness`],
/// [`SpecCommand::Preview`], [`SpecCommand::Commit`]) -- see `run::render`, `run::freshness`,
/// `run::preview` and `run::commit`'s own documentation for why those four, and not the
/// other five, earned that seam. Every other variant ignores `filesystem` entirely; it is
/// threaded through all nine here regardless, the same way `nomos_work_orchestration::Run`
/// takes `F`, `C`, `L` and `P` for every [`nomos_ledger`] verb even the ones that touch none
/// of them, because one generic `Run` a caller can depend on without also depending on
/// `nomos-platform-std` is worth more than sparing the five that do not need `F` a type
/// parameter. [`Submit`] is generic over [`FileSystem`] the same way, for the same reason
/// [`crate::run::render`] gives -- it just is not one of this function's nine cases, per
/// [`SpecCommand`]'s own module documentation.
///
/// [`SpecCommand::Profiles`] never assembles a store, matching
/// [`Profiles`]'s own guarantee. Every other variant assembles one from `request` first, so
/// a caller asking `nomos-spec-orchestration` for any of them fails on the same
/// store-assembly error every other verb would rather than a bespoke one.
#[must_use]
pub fn Run<F: FileSystem>(command: &SpecCommand, request: &CorpusRequest, filesystem: &F) -> SpecOutcome
{
    if let SpecCommand::Profiles = command
    {
        return SpecOutcome::Profiles(Profiles());
    }

    let assembly = match Assemble(request)
    {
        Ok(assembly) => assembly,
        Err(error) => return Outcome_For(command, Err(error), filesystem),
    };

    return Outcome_For(command, Ok(assembly), filesystem);
}

/// Every non-`Profiles` variant's outcome, given the assembly its verb needs (or the error
/// that kept one from being built).
fn Outcome_For<F: FileSystem>(
    command: &SpecCommand,
    assembled: Result<Assembly, StoreError>,
    filesystem: &F,
) -> SpecOutcome
{
    return match command
    {
        SpecCommand::Sources => SpecOutcome::Sources(assembled.map(|assembly| return Sources(&assembly))),
        SpecCommand::Record(request) => SpecOutcome::Record(
            assembled
                .map_err(crate::outcome::RecordRefusal::Store)
                .and_then(|assembly| return Record(&assembly, request)),
        ),
        SpecCommand::Table(request) => SpecOutcome::Table(
            assembled
                .map_err(crate::outcome::TableRefusal::Store)
                .and_then(|assembly| return Table(&assembly, request)),
        ),
        SpecCommand::Render(request) => SpecOutcome::Render(
            assembled
                .map_err(RenderRefusal::Store)
                .and_then(|assembly| return Render(&assembly, request, filesystem)),
        ),
        SpecCommand::Freshness(request) => SpecOutcome::Freshness(
            assembled
                .map_err(FreshnessRefusal::Store)
                .and_then(|assembly| return Freshness(&assembly, request, filesystem)),
        ),
        SpecCommand::Markdown(request) => SpecOutcome::Markdown(
            assembled
                .map_err(nomos_spec_store::EditError::Store)
                .and_then(|assembly| return Markdown(&assembly, request)),
        ),
        SpecCommand::Preview(request) => SpecOutcome::Preview(
            assembled
                .map_err(nomos_spec_store::EditError::Store)
                .map_err(PreviewRefusal::Edit)
                .and_then(|assembly| return Preview(&assembly, request, filesystem)),
        ),
        SpecCommand::Commit(request) => SpecOutcome::Commit(
            assembled
                .map_err(nomos_spec_store::EditError::Store)
                .map_err(PreviewRefusal::Edit)
                .map_err(CommitRefusal::from)
                .and_then(|mut assembly| return Commit(&mut assembly, request, filesystem)),
        ),
        SpecCommand::Profiles => SpecOutcome::Profiles(Profiles()),
    };
}

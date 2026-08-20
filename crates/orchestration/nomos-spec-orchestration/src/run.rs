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
use crate::outcome::{
    CommitRefusal, FreshnessRefusal, PreviewRefusal, RecordRefusal, RenderRefusal, SourcesAnswer, SpecOutcome,
    TableRefusal,
};
use crate::request::{CommitRequest, EditRequest, FreshnessRequest, RecordRequest, RenderRequest, TableRequest};

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
///
/// One private helper per verb rather than one arm's worth of chained `map_err`/`and_then`
/// inline: each helper is the answer to "what does `nomos spec <verb>` do with an assembly,"
/// nameable on its own, and this function is left as exactly the dispatch its own doc
/// comment already promises -- match the verb, hand the assembly to the verb's own outcome.
fn Outcome_For<F: FileSystem>(
    command: &SpecCommand,
    assembled: Result<Assembly, StoreError>,
    filesystem: &F,
) -> SpecOutcome
{
    return match command
    {
        SpecCommand::Sources => Sources_Outcome(assembled),
        SpecCommand::Record(request) => Record_Outcome(assembled, request),
        SpecCommand::Table(request) => Table_Outcome(assembled, request),
        SpecCommand::Render(request) => Render_Outcome(assembled, request, filesystem),
        SpecCommand::Freshness(request) => Freshness_Outcome(assembled, request, filesystem),
        SpecCommand::Markdown(request) => Markdown_Outcome(assembled, request),
        SpecCommand::Preview(request) => Preview_Outcome(assembled, request, filesystem),
        SpecCommand::Commit(request) => Commit_Outcome(assembled, request, filesystem),
        SpecCommand::Profiles => SpecOutcome::Profiles(Profiles()),
    };
}

fn Sources_Outcome(assembled: Result<Assembly, StoreError>) -> SpecOutcome
{
    return SpecOutcome::Sources(assembled.map(|assembly| return Sources(&assembly)));
}

fn Record_Outcome(assembled: Result<Assembly, StoreError>, request: &RecordRequest) -> SpecOutcome
{
    return SpecOutcome::Record(
        assembled
            .map_err(RecordRefusal::Store)
            .and_then(|assembly| return Record(&assembly, request)),
    );
}

fn Table_Outcome(assembled: Result<Assembly, StoreError>, request: &TableRequest) -> SpecOutcome
{
    return SpecOutcome::Table(
        assembled
            .map_err(TableRefusal::Store)
            .and_then(|assembly| return Table(&assembly, request)),
    );
}

fn Render_Outcome<F: FileSystem>(
    assembled: Result<Assembly, StoreError>,
    request: &RenderRequest,
    filesystem: &F,
) -> SpecOutcome
{
    return SpecOutcome::Render(
        assembled
            .map_err(RenderRefusal::Store)
            .and_then(|assembly| return Render(&assembly, request, filesystem)),
    );
}

fn Freshness_Outcome<F: FileSystem>(
    assembled: Result<Assembly, StoreError>,
    request: &FreshnessRequest,
    filesystem: &F,
) -> SpecOutcome
{
    return SpecOutcome::Freshness(
        assembled
            .map_err(FreshnessRefusal::Store)
            .and_then(|assembly| return Freshness(&assembly, request, filesystem)),
    );
}

fn Markdown_Outcome(assembled: Result<Assembly, StoreError>, request: &RecordRequest) -> SpecOutcome
{
    return SpecOutcome::Markdown(
        assembled
            .map_err(nomos_spec_store::EditError::Store)
            .and_then(|assembly| return Markdown(&assembly, request)),
    );
}

fn Preview_Outcome<F: FileSystem>(
    assembled: Result<Assembly, StoreError>,
    request: &EditRequest,
    filesystem: &F,
) -> SpecOutcome
{
    return SpecOutcome::Preview(
        assembled
            .map_err(nomos_spec_store::EditError::Store)
            .map_err(PreviewRefusal::Edit)
            .and_then(|assembly| return Preview(&assembly, request, filesystem)),
    );
}

fn Commit_Outcome<F: FileSystem>(
    assembled: Result<Assembly, StoreError>,
    request: &CommitRequest,
    filesystem: &F,
) -> SpecOutcome
{
    return SpecOutcome::Commit(
        assembled
            .map_err(nomos_spec_store::EditError::Store)
            .map_err(PreviewRefusal::Edit)
            .map_err(CommitRefusal::from)
            .and_then(|mut assembly| return Commit(&mut assembly, request, filesystem)),
    );
}

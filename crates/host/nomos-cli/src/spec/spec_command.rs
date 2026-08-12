//! Every verb `nomos spec` answers.

use crate::spec::request::CommitRequest;
use crate::spec::request::EditRequest;
use crate::spec::request::FreshnessRequest;
use crate::spec::request::RenderRequest;
use crate::spec::request::TableRequest;
use crate::spec::request::RecordRequest;
/// What to read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SpecCommand
{
    /// Print a record's source, byte for byte.
    Record(RecordRequest),
    /// Print a document's table rows, as authored.
    Table(TableRequest),
    /// Build a projection profile and write it out.
    Render(RenderRequest),
    /// Compare the outputs already on disk against the store and their own stamps.
    Freshness(FreshnessRequest),
    /// Read a record out of the store as markdown, rendered from its rows.
    Markdown(RecordRequest),
    /// Say what committing an edited record would change, and change nothing.
    Preview(EditRequest),
    /// Preview an edited record and then commit it.
    Commit(CommitRequest),
    /// List the shipped projection profiles.
    Profiles,
    /// Say what this store was assembled from, and what was missing.
    Sources,
}

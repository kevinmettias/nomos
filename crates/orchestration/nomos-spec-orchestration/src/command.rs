//! Every verb `nomos spec` answers.
//!
//! Moved verbatim from `nomos-cli::spec::command`, the same way `nomos_check_orchestration
//! ::CheckCommand` moved from `nomos-cli::check::command`: a second adapter wanting this
//! vocabulary previously had to depend on `nomos-cli` itself to get it.
//!
//! `nomos request submit` is deliberately not a tenth variant here. `OD-HOST-005` moved its
//! verb into this crate too, but its own resolution is explicit that "a `nomos request
//! submit` invocation is not a `nomos spec` verb by the CLI's own naming" -- [`crate::Submit`]
//! is a sibling of [`crate::Run`] over this same enum, not a case of it, and [`SpecCommand`]
//! stays exactly the nine verbs `nomos spec` itself answers.

use crate::request::CommitRequest;
use crate::request::EditRequest;
use crate::request::FreshnessRequest;
use crate::request::RenderRequest;
use crate::request::TableRequest;
use crate::request::RecordRequest;
/// What to read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecCommand
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

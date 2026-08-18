//! What a `nomos spec` verb produced.
//!
//! One variant per [`crate::SpecCommand`] variant, the same shape `nomos_check_orchestration
//! ::CheckOutcome` and `nomos_work_orchestration::WorkOutcome` already carry: a caller that
//! wanted `nomos spec profiles`'s answer previously had to depend on `nomos-cli` and call its
//! rendering function, throwing the structure away on the way in and reconstructing it by
//! parsing text on the way out.
//!
//! [`SpecOutcome::Profiles`], [`SpecOutcome::Sources`], [`SpecOutcome::Record`],
//! [`SpecOutcome::Table`], [`SpecOutcome::Markdown`], [`SpecOutcome::Render`] and
//! [`SpecOutcome::Freshness`] carry a real, computed answer. [`SpecOutcome::Markdown`] reuses
//! `nomos-spec-store`'s own [`RecordProjection`] and [`EditError`] rather than a type of this
//! crate's own — `Record_Markdown` already returns exactly that shape, and a second type
//! here would only restate it. The remaining two verbs (`Preview`, `Commit`) still execute
//! entirely inside `nomos-cli::spec`, and [`NotYetMigrated`] marks the outcome vocabulary
//! reserved for them without pretending they have moved.

mod freshness;
mod record;
mod render;
mod sources;
mod table;

pub use freshness::{FreshnessAnswer, FreshnessRefusal, ProfileOutcome, Verdict};
pub use record::{RecordAnswer, RecordRefusal};
pub use render::{RenderAnswer, RenderRefusal};
pub use sources::SourcesAnswer;
pub use table::{TableAnswer, TableRefusal};

use nomos_spec_project::{Profile, ProjectError};
use nomos_spec_store::{EditError, RecordProjection, StoreError};

/// This verb has not moved into `nomos-spec-orchestration` yet.
///
/// `nomos-cli::spec` still parses, assembles a store for, and dispatches this command
/// entirely on its own -- calling [`crate::Run`] with it would be premature, so nothing
/// does. One future increment replaces each marked variant's payload with the outcome its
/// verb actually produces, the same way earlier increments replaced `Profiles`, `Sources`,
/// `Record`, `Table`, `Markdown`, `Render` and `Freshness`'s own markers with real data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotYetMigrated;

/// What a `nomos spec` verb produced.
pub enum SpecOutcome
{
    /// `nomos spec record` -- the one document behind an identifier, or why none answered.
    Record(Result<RecordAnswer, RecordRefusal>),
    /// `nomos spec table` -- the rows a table request selected, or why none answered.
    Table(Result<TableAnswer, TableRefusal>),
    /// `nomos spec render` -- the projection built and placed, or why it was not.
    Render(Result<RenderAnswer, RenderRefusal>),
    /// `nomos spec freshness` -- every profile examined and what was found, or why nothing
    /// was examined at all.
    Freshness(Result<FreshnessAnswer, FreshnessRefusal>),
    /// `nomos spec markdown` -- a record rendered from the store's own rows, or why it could
    /// not be.
    Markdown(Result<RecordProjection, EditError>),
    /// `nomos spec preview` -- not yet migrated.
    Preview(NotYetMigrated),
    /// `nomos spec commit` -- not yet migrated.
    Commit(NotYetMigrated),
    /// The shipped projection catalogue, or why it could not be read.
    ///
    /// Never touches a store -- the catalogue is embedded, so a machine that cannot open a
    /// database can still list what this build can render.
    Profiles(Result<Vec<Profile>, ProjectError>),
    /// What this store was assembled from, or why no store could be assembled at all.
    Sources(Result<SourcesAnswer, StoreError>),
}

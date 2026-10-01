//! What a `nomos spec` verb produced.
//!
//! One variant per [`crate::SpecCommand`] variant, the same shape `nomos_check_orchestration
//! ::CheckOutcome` and `nomos_work_orchestration::WorkOutcome` already carry: a caller that
//! wanted `nomos spec profiles`'s answer previously had to depend on `nomos-cli` and call its
//! rendering function, throwing the structure away on the way in and reconstructing it by
//! parsing text on the way out.
//!
//! Every variant now carries a real, computed answer -- the last two landed in this crate's
//! fourth and final increment. [`SpecOutcome::Preview`] reuses [`EditPreview`] directly, the
//! same reuse [`SpecOutcome::Markdown`] already makes for [`RecordProjection`]: a preview's
//! own answer is already the whole thing an author reads, and a wrapper of this crate's own
//! would only restate it. [`SpecOutcome::Commit`] carries [`crate::spec_outcome::CommitAnswer`], a type
//! of this crate's own, because committing does more than a preview does -- it applies the
//! transaction, writes the bytes, and closes the round trip -- and no single
//! `nomos-spec-store` type carries all of that at once.

mod commit_answer;
mod verdict;
mod preview_refusal;
mod record_answer;
mod render_answer;
mod sources_answer;
mod submit_answer;
mod table_answer;

pub use commit_answer::{
    CommitAnswer, CommitRefusal, CommitRefusalError, CommitRefusalKind, Reproduction, VacateOutcome, Vacated,
};
pub use verdict::{FreshnessAnswer, FreshnessRefusal, ProfileOutcome, Verdict};
pub use preview_refusal::PreviewRefusal;
pub use record_answer::{RecordAnswer, RecordRefusal};
pub use render_answer::{RenderAnswer, RenderRefusal};
pub use sources_answer::SourcesAnswer;
pub use submit_answer::{SubmitAnswer, SubmitRefusal};
pub use table_answer::{TableAnswer, TableRefusal};

use nomos_spec_project::{Profile, ProjectError};
use nomos_spec_store::{EditError, EditPreview, RecordProjection, StoreError};

/// What a `nomos spec` verb produced.
// `Preview` and `Commit` carry an owned `EditPreview` -- the staged markdown and every block,
// identity and relation change computed against it -- because a preview's whole point is that
// an author reads the same value this crate hands back, the same reason
// `RecordRefusal::NotFound` carries an owned `NodeSummary` unboxed in `spec_outcome::record_answer`.
// Boxing either variant would only move the size this lint is measuring, not remove it.
#[allow(clippy::large_enum_variant)]
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
    /// `nomos spec preview` -- what committing the staged edit would change, or why nothing
    /// could be previewed.
    Preview(Result<EditPreview, PreviewRefusal>),
    /// `nomos spec commit` -- what committing the staged edit changed, or why it did not.
    Commit(Result<CommitAnswer, CommitRefusal>),
    /// The shipped projection catalogue, or why it could not be read.
    ///
    /// Never touches a store -- the catalogue is embedded, so a machine that cannot open a
    /// database can still list what this build can render.
    Profiles(Result<Vec<Profile>, ProjectError>),
    /// What this store was assembled from, or why no store could be assembled at all.
    Sources(Result<SourcesAnswer, StoreError>),
}

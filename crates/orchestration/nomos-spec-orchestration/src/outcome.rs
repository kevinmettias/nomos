//! What a `nomos spec` verb produced.
//!
//! One variant per [`crate::SpecCommand`] variant, the same shape `nomos_check_orchestration
//! ::CheckOutcome` and `nomos_work_orchestration::WorkOutcome` already carry: a caller that
//! wanted `nomos spec profiles`'s answer previously had to depend on `nomos-cli` and call its
//! rendering function, throwing the structure away on the way in and reconstructing it by
//! parsing text on the way out.
//!
//! Only [`SpecOutcome::Profiles`] and [`SpecOutcome::Sources`] carry a real, computed
//! answer in this increment -- the other seven verbs still execute entirely inside
//! `nomos-cli::spec`, and [`NotYetMigrated`] marks the outcome vocabulary reserved for them
//! without pretending they have moved.

use crate::corpus::Absence;
use nomos_spec_project::{Profile, ProjectError};
use nomos_spec_store::StoreError;

/// What went into this store, and what did not.
///
/// The rendering-free half of `nomos-cli::spec::verb::listing::Sources`: everything that
/// function used to write directly, kept here as data so a second caller can render it its
/// own way instead of parsing the lines `nomos-cli` happened to print.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcesAnswer
{
    /// One line per input the assembly read, in the order it was read.
    pub read: Vec<String>,
    /// Every input the assembly expected and did not find.
    pub absent: Vec<Absence>,
}

impl SourcesAnswer
{
    /// Whether anything this store was expected to hold is missing.
    #[must_use]
    pub fn Is_Complete(&self) -> bool
    {
        return self.absent.is_empty();
    }

    /// Every absence, one after another.
    #[must_use]
    pub fn Describe_Absences(&self) -> String
    {
        return self
            .absent
            .iter()
            .map(Absence::Describe)
            .collect::<Vec<String>>()
            .join("\n");
    }
}

/// This verb has not moved into `nomos-spec-orchestration` yet.
///
/// `nomos-cli::spec` still parses, assembles a store for, and dispatches this command
/// entirely on its own -- calling [`crate::Run`] with it would be premature, so nothing
/// does. Three future increments replace each marked variant's payload with the outcome its
/// verb actually produces, the same way this increment replaced `Profiles` and `Sources`'
/// own markers with real data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotYetMigrated;

/// What a `nomos spec` verb produced.
pub enum SpecOutcome
{
    /// `nomos spec record` -- not yet migrated.
    Record(NotYetMigrated),
    /// `nomos spec table` -- not yet migrated.
    Table(NotYetMigrated),
    /// `nomos spec render` -- not yet migrated.
    Render(NotYetMigrated),
    /// `nomos spec freshness` -- not yet migrated.
    Freshness(NotYetMigrated),
    /// `nomos spec markdown` -- not yet migrated.
    Markdown(NotYetMigrated),
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

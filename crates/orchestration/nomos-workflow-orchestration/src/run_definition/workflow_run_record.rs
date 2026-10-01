//! One definition run, kept beside the definition it ran under.

use crate::{DefinitionRun, WorkflowDefinition};

/// What one definition run produced, kept beside the definition it actually ran under.
///
/// The record carries the whole definition rather than a reference to one or a digest of
/// one, which is the entire mechanism behind "replayed against the definition it ran under
/// rather than against whatever a caller assembles now". A record holding only an identity
/// and a version would have had to go and find the definition again, and the thing it
/// found would be whatever that identity resolves to today -- which is precisely the
/// substitution a pinned replay exists to prevent.
///
/// Both fields are private and there is no constructor a caller can reach: a record is
/// produced by [`crate::Run_Definition`] and by nothing else. A record whose definition
/// could be swapped after the fact would let a replay be pinned to a definition the run
/// never saw, and would answer the question with the same confidence either way.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowRunRecord
{
    definition: WorkflowDefinition,
    produced: DefinitionRun,
}

impl WorkflowRunRecord
{
    /// The record of `produced`, run under `definition`.
    pub(super) const fn Of(definition: WorkflowDefinition, produced: DefinitionRun) -> Self
    {
        return Self { definition, produced };
    }

    /// The definition this run ran under.
    #[must_use]
    pub const fn Definition(&self) -> &WorkflowDefinition
    {
        return &self.definition;
    }

    /// What the run produced.
    #[must_use]
    pub const fn Produced(&self) -> &DefinitionRun
    {
        return &self.produced;
    }
}

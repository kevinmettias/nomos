//! Why a run wrote nothing, or wrote and undid what it wrote.

use crate::refusal::Refusal;

/// A run that did not complete, and what state the tree was left in.
///
/// The three variants are three different states, deliberately not collapsed: a caller that
/// could not tell a refusal from a rollback would not know whether a retry is safe, and one
/// that could not tell a rollback from a failed rollback would not know whether the tree
/// still describes anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaterializationError
{
    /// Refused before any write. The tree is untouched, because every intent is planned in
    /// full before the first byte is placed.
    Refused
    {
        /// The target whose intent was refused, as the intent spelled it.
        target: String,
        /// Why.
        refusal: Refusal,
    },
    /// A write failed and every write already made was undone. The tree is as it was found.
    RolledBack
    {
        /// The target whose write failed, as the intent spelled it.
        target: String,
        /// What the filesystem said.
        cause: String,
    },
    /// A write failed, and undoing an earlier one failed too. The tree is in neither the
    /// state it started in nor the one the run declared, and this is the one outcome that
    /// needs a person: reported loudly rather than folded into the variant above, which
    /// would claim a restoration that did not happen.
    RollbackFailed
    {
        /// The target whose write failed, as the intent spelled it.
        target: String,
        /// What the filesystem said about that write.
        cause: String,
        /// The earlier target that could not be put back.
        unrestored_target: String,
        /// What the filesystem said about putting it back.
        unrestored_cause: String,
    },
}

impl core::fmt::Display for MaterializationError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Refused { target, refusal } =>
            {
                write!(formatter, "nothing was written: the intent for `{target}` was refused because {refusal}.")
            }
            Self::RolledBack { target, cause } =>
            {
                write!(formatter, "`{target}` did not write ({cause}); every earlier write was undone and the tree is as it was found.")
            }
            Self::RollbackFailed { target, cause, unrestored_target, unrestored_cause } =>
            {
                write!(
                    formatter,
                    "`{target}` did not write ({cause}), and `{unrestored_target}` could not be put back ({unrestored_cause}). \
                     The tree is in neither state and needs a person."
                )
            }
        };
    }
}

impl std::error::Error for MaterializationError
{}

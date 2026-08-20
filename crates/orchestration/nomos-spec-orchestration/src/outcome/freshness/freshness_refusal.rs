//! Why `nomos spec freshness` did not examine anything.

use nomos_spec_project::ProjectError;
use nomos_spec_store::StoreError;

/// Why `nomos spec freshness` did not examine anything.
///
/// A single profile's build failure is not here -- it is [`crate::outcome::Verdict::
/// Compared`]'s own `Err`. These are the refusals that keep the whole run from starting at
/// all.
#[derive(Debug)]
pub enum FreshnessRefusal
{
    /// `--profile` or a `--require` names an identifier the catalogue does not carry.
    NoSuchProfile
    {
        requested: String,
        known: Vec<String>,
    },
    /// `--require` names a profile that `--profile` narrowed this run away from.
    ///
    /// Reported before any disk is read: answering it would mean reporting success over a
    /// requirement nothing checked.
    RequirementUnexamined
    {
        requested: String,
        only: Option<String>,
    },
    /// The embedded catalogue itself failed to parse -- a defect in this build.
    Project(ProjectError),
    /// The store could not be assembled at all.
    Store(StoreError),
}

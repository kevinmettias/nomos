//! What `nomos spec freshness` produced, or why it did not.

use nomos_spec_project::{Freshness, Profile, ProjectError};
use nomos_spec_store::StoreError;

/// One profile's answer, once its build root has been read.
///
/// Moved from `nomos-cli::spec::verb::freshness`'s own `Verdict`: which of these happened is
/// a fact about the filesystem and the store, not about how a terminal reports it. The
/// half-present cases ([`Verdict::Unstamped`], [`Verdict::Unbodied`]) are told apart from
/// [`Verdict::Absent`] deliberately -- `OD-GATE-005` treats a body with no sidecar as a
/// failure rather than something skipped, because otherwise deleting the sidecar is how an
/// edit stops being caught.
#[derive(Debug)]
pub enum Verdict
{
    /// Neither the body nor its sidecar is on disk.
    Absent,
    /// A body is there and no sidecar beside it.
    Unstamped,
    /// A sidecar is there and no body beside it.
    Unbodied,
    /// Both halves are there, compared against what the store would produce now.
    ///
    /// `Err` when rebuilding the profile to compare against failed -- one profile's build
    /// failure does not stop the rest of a run from being checked, so it is carried per
    /// profile rather than aborting [`crate::run::Freshness`] outright.
    Compared(Result<Freshness, ProjectError>),
}

/// One profile a `freshness` run examined, and what it found.
#[derive(Debug)]
pub struct ProfileOutcome
{
    /// The profile examined, resolved and (if subject-addressed) already narrowed.
    pub profile: Profile,
    /// What was found for it.
    pub verdict: Verdict,
}

/// What a `freshness` run examined, and what it was promised.
#[derive(Debug)]
pub struct FreshnessAnswer
{
    /// Every profile this run looked at, in catalogue order.
    pub examined: Vec<ProfileOutcome>,
    /// The profiles this run was told must be present, by identifier.
    pub required: Vec<String>,
}

/// Why `nomos spec freshness` did not examine anything.
///
/// A single profile's build failure is not here -- it is [`Verdict::Compared`]'s own `Err`.
/// These are the refusals that keep the whole run from starting at all.
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

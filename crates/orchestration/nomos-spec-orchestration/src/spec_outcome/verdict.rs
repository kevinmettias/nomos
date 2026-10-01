//! What `nomos spec freshness` produced, or why it did not.

mod freshness_answer;
mod freshness_refusal;
mod profile_outcome;

pub use freshness_answer::FreshnessAnswer;
pub use freshness_refusal::FreshnessRefusal;
pub use profile_outcome::ProfileOutcome;

use nomos_spec_project::{Freshness, ProjectError};

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

//! What `nomos request submit` was asked to accept.

use nomos_spec_model::{DecisionGap, SubmissionKind, SubmissionState};
use std::path::PathBuf;

/// One submission, as a transport read it off its own surface.
///
/// Moved verbatim from `nomos-cli::request`'s own `SubmitRequest`, alongside the
/// [`Constructed_Submission`] conversion `run::submit` now carries -- see that module for why
/// `state` and `contract_version` are facts about this run rather than about the submission,
/// and why every `field` value carries origin `submitted` unconditionally.
///
/// [`Constructed_Submission`]: crate::run::submit::Constructed_Submission
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmitRequest
{
    /// Which kind of submission this is.
    pub kind: SubmissionKind,
    /// The subject identifier this submission is filed under.
    pub id: String,
    /// Who submitted it.
    pub by: String,
    /// The state a caller declared for this submission, defaulted before this type is built.
    pub state: SubmissionState,
    /// The form contract version a caller declared, defaulted before this type is built.
    pub contract_version: u32,
    /// Every `name=value` pair, in the order they were given.
    pub fields: Vec<(String, String)>,
    /// Every decision gap the submitter declared against it.
    pub gaps: Vec<DecisionGap>,
    /// Which transport constructed this submission -- `"cli"` for `nomos-cli::request`, and
    /// whatever a second adapter calling this crate directly names itself.
    ///
    /// This crate does not default or guess it: a value moved here from a single adapter's
    /// own hard-coded `"cli"` would silently mislabel every submission a second adapter
    /// accepts through the same door.
    pub submitted_through: String,
    /// Where the accepted submission's `subject-dossier` projection is placed, if at all.
    pub into: Option<PathBuf>,
}

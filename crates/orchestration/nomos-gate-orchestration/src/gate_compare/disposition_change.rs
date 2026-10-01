//! One finding occurrence whose gate treatment differs between two compared runs.

use crate::{FindingDisposition, SuppressionReason};
use nomos_contracts::{RuleId, SubjectId};

/// One finding occurrence present in both runs [`crate::Compare_Gate_Runs`] compared, but whose
/// gate treatment or the reason producing it differs between them.
///
/// Identified across the two runs by [`nomos_model::FindingOccurrenceId`], so two violations one
/// rule makes about one subject are two changes here rather than one. `rule`, `subject` and
/// `subject_name` are carried for a reader; they no longer identify the entry on their own, and
/// `locations` is what tells two entries at one subject apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispositionChange
{
    pub rule: RuleId,
    pub subject: SubjectId,
    pub subject_name: String,
    /// Where this occurrence is, carried so two changes at one subject are distinguishable.
    ///
    /// Geometry, not identity: the occurrence identity stays internal to this module, and
    /// `Finding::locations`' own rule -- a location is never what a claim is attributed to --
    /// is unaffected by showing a reader where to look.
    pub locations: Vec<String>,
    pub before: FindingDisposition,
    pub after: FindingDisposition,
    /// Why the finding was in `before`'s bucket, when a disposition named it.
    ///
    /// Carried so a change *within* one bucket is reportable. A `FalsePositiveDisposition`
    /// becoming a `FormalRiskAcceptance` leaves `before` and `after` equal and is an opposite
    /// engineering claim; a lapsed waiver leaves the suppressed bucket entirely, and a reader
    /// needs to see that a tolerance came due rather than that a violation appeared.
    pub before_reason: Option<SuppressionReason>,
    /// Why the finding is in `after`'s bucket, when a disposition names it.
    pub after_reason: Option<SuppressionReason>,
}

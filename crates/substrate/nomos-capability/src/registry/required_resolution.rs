//! The answer to a requirement whose named provider is not negotiable.

use crate::Selection;
use nomos_contracts::CapabilityId;

use super::required_unmet::RequiredUnmet;

/// The answer to a requirement whose named provider is not negotiable.
///
/// Deliberately shaped like [`crate::Resolution`] — satisfied or not, never a bare no —
/// because it answers the same question at a different strength. It is a distinct type
/// rather than a third [`nomos_contracts::Applicability`] or a flag beside one, because
/// nothing short of a different return type stops a caller from reading
/// `Applicability::SupportedWithFallback` as good enough when it required the naming rather
/// than merely preferring it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequiredResolution
{
    /// The required provider answered. There is exactly one way to reach this: nothing else
    /// is reported, because a required naming that is honoured is indistinguishable from an
    /// ordinary answer.
    Satisfied
    {
        selection: Selection,
    },
    /// The required provider did not answer — whether because nobody could, or because
    /// somebody else did.
    Unsatisfied
    {
        capability: CapabilityId,
        reason: RequiredUnmet,
    },
}

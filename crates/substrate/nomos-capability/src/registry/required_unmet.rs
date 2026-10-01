//! Why a *required* naming was not honoured.

use crate::Unmet;
use nomos_contracts::ProviderId;

/// Why a *required* naming was not honoured.
///
/// [`Unmet`] already has the vocabulary for "nobody usable answered at all", and that half
/// is reused unchanged — a required naming that nobody could have answered fails for the
/// same reason a preferred one would have. The half `Unmet` cannot say is that somebody
/// usable *did* answer and it was not who was named; [`crate::Registry::Resolve`] calls that
/// [`nomos_contracts::Applicability::SupportedWithFallback`] and returns
/// [`crate::Resolution::Satisfied`]. [`crate::Registry::Resolve_Requiring`] does not, because
/// the caller said which provider, not merely which guarantee — `docs/records/OD-CAPABILITY-005`
/// is why that difference is enough to refuse rather than merely to flag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequiredUnmet
{
    /// No usable offer existed at all, whoever was required. Carries the same reason
    /// [`crate::Registry::Resolve`] would have reported.
    Unavailable(Unmet),
    /// A usable offer existed and it was not the required provider.
    AnsweredByOther
    {
        required: ProviderId,
        answered: ProviderId,
    },
}

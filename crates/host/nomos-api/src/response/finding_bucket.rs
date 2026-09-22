//! [`FindingBucket`], the bucket a finding fell into, as a wire caller reads it.

use nomos_gate_orchestration::FindingDisposition;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::FindingDisposition`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason `crate::response`'s own doc gives, and because `OD-GATE-022-A` refuses to
/// give it one on a caller's behalf. Kept to the same variants, in the same order, so a
/// mismatch is a compile error in [`FindingBucket::From`] rather than a silent divergence --
/// the discipline [`super::Disposition`] already uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingBucket
{
    /// The finding failed the build.
    Blocking,
    /// An `AdoptionPolicy` calibration kept it from blocking.
    Calibrated,
    /// A `Suppression` kept it from blocking.
    Suppressed,
    /// A `BaselineDebt` entry kept it from blocking.
    Baselined,
    /// A `BaselineDebt` entry matched, and its scope held more occurrences than it accepted,
    /// so nothing kept it from blocking.
    ///
    /// Kept apart from [`Self::Blocking`] so a caller can see a tolerance running out of room
    /// as the distinct event it is -- a finding moving between the two is a real transition,
    /// and collapsing them would report a repository's debt growing past what it adopted as
    /// though a rule had simply started failing.
    BaselineExceeded,
    /// The gate's declared evidence floor is above the class this finding's evidence carries,
    /// so it could not block.
    ///
    /// Its own variant rather than one of the four above, because `OD-GATE-034` keeps "this
    /// rule is advisory" and "this finding's evidence was too weak under this gate" apart: the
    /// other four each say a person authored something about this finding, and this one says
    /// nobody did.
    BelowEvidenceFloor,
}

impl FindingBucket
{
    pub(crate) fn From(disposition: FindingDisposition) -> Self
    {
        return match disposition
        {
            FindingDisposition::Blocking => Self::Blocking,
            FindingDisposition::Calibrated => Self::Calibrated,
            FindingDisposition::Suppressed => Self::Suppressed,
            FindingDisposition::Baselined => Self::Baselined,
            FindingDisposition::BaselineExceeded => Self::BaselineExceeded,
            FindingDisposition::BelowEvidenceFloor => Self::BelowEvidenceFloor,
        };
    }
}

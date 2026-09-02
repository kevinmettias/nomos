//! Whether a rule's violations may be mechanically corrected or explicitly suppressed.

/// `ARCH-002`'s "correction and suppression contracts" for one rule.
///
/// No shipped rule declares one yet — `OD-PACKAGE-008`'s four-rule measurement found
/// none of the four has a violation with a safe, judgment-free fix today, and
/// `OD-CORRECTIONS-001` independently confirmed the same absence from
/// `nomos-corrections`' side. `mechanical_correction` names the real, tested type that
/// would carry a rule's fix once one exists —
/// [`nomos_corrections::CorrectionCandidate`](../nomos_corrections/struct.CorrectionCandidate.html)
/// is the nearest real analog, cited here rather than duplicated because this crate does
/// not depend on `nomos-corrections` (band 27, above this crate's band 26) and a
/// manifest field only needs to say a rule's violations *are* mechanically correctable,
/// not carry the correction logic itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionAndSuppressionContract
{
    /// Whether this rule's violations have a mechanical, judgment-free correction.
    pub mechanical_correction_available: bool,
    /// Whether a finding from this rule may be explicitly suppressed with a rationale,
    /// the same shape `nomos_gate_orchestration::SuppressionPolicy` already checks
    /// findings against.
    pub suppression_supported: bool,
}

impl CorrectionAndSuppressionContract
{
    /// Constructs a correction-and-suppression contract.
    #[must_use]
    pub const fn New(mechanical_correction_available: bool, suppression_supported: bool) -> Self
    {
        return Self { mechanical_correction_available, suppression_supported };
    }
}

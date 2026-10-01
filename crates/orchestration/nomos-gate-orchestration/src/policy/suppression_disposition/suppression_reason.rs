//! Why a suppressed finding is suppressed, as a run recorded it.

use super::{SuppressionDisposition, SuppressionStatus};

/// Why a finding did not block, recorded by the run that decided it.
///
/// # Why the run records this rather than a reader re-deriving it
///
/// Because the only other way to know is to evaluate the policy again, and a second evaluator
/// is a second reading of one relation. `Compare_Gate_Runs` is the consumer, and it compares
/// two runs that may have been judged under different policies and at different moments —
/// re-deriving would answer with *today's* policy about *yesterday's* run, which is the
/// opposite of what a historical comparison is for. Recording it at the moment of the decision
/// is the only reading that stays true afterwards.
///
/// It also keeps `gate_compare` from becoming a place that understands suppression evaluation.
/// `P102` removed one duplicated authority of exactly that shape between rule descriptors and
/// materialization; this declines to add another.
///
/// # Why both fields
///
/// The disposition says what kind of decision was made; the status says whether it still
/// applied when the run was judged. They move independently and each move means something a
/// reader would act on differently. A `FalsePositiveDisposition` becoming a
/// `FormalRiskAcceptance` is somebody deciding the rule was right after all and accepting the
/// risk — the same bucket, an opposite engineering claim. A `TemporaryWaiver` going `Active` to
/// `Expired` is a tolerance coming due, which is the visibility `P103` built into the run model
/// and which would otherwise be lost again in comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SuppressionReason
{
    /// Which of the six dispositions matched.
    pub disposition: SuppressionDisposition,
    /// Whether it still applied at the moment the run was judged.
    pub status: SuppressionStatus,
}

#[cfg(test)]
mod tests
{
    use super::{SuppressionDisposition, SuppressionReason, SuppressionStatus};

    /// Two reasons are equal only when both halves are.
    ///
    /// The whole point of carrying two fields. If equality ignored either, the comparison this
    /// type exists to enable would go back to reporting no change for the transition that
    /// motivated it.
    #[test]
    fn Test_A_Reason_Should_Differ_When_Either_Half_Differs()
    {
        let waived = SuppressionReason {
            disposition: SuppressionDisposition::TemporaryWaiver,
            status: SuppressionStatus::Active,
        };

        let lapsed = SuppressionReason { status: SuppressionStatus::Expired, ..waived };
        let accepted = SuppressionReason { disposition: SuppressionDisposition::FormalRiskAcceptance, ..waived };

        assert_ne!(waived, lapsed, "a waiver that lapsed must not equal one that still applies");
        assert_ne!(waived, accepted, "a waiver must not equal a formal risk acceptance");
        assert_eq!(waived, SuppressionReason { ..waived }, "a reason must equal itself");
    }
}

//! What is kept on record when a disqualified candidate transitions to a fallback.

use serde::{Deserialize, Serialize};

use crate::RuntimeCandidateDisqualification;

/// `MODEL-ROUTE-035`: "A runtime-disqualified candidate may transition to a fallback
/// only when the fallback edge is reachable, the disqualification is retryable or
/// substitution-eligible, and every changed dimension is permitted by
/// `FallbackAdmissibility`. The trace shall retain the original candidate,
/// disqualification evidence, fallback decision, changed guarantees, and discarded
/// assembly or request cost."
///
/// Five fields, each a direct transcription of one named retained item.
/// `original_candidate` stays a raw identifier -- no `CandidateId` type exists
/// anywhere in this workspace to reuse. `disqualification` reuses
/// [`RuntimeCandidateDisqualification`] (`MODEL-ROUTE-034`) directly rather than
/// re-deriving "disqualification evidence" as a new shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackTransitionTrace
{
    pub original_candidate: String,
    pub disqualification: RuntimeCandidateDisqualification,
    pub fallback_decision: String,
    pub changed_guarantees: Vec<String>,
    pub discarded_cost: String,
}

/// `MODEL-ROUTE-035`'s own precondition for a transition, not one of the trace's five
/// retained fields -- "may transition ... only when ... the disqualification is
/// retryable or substitution-eligible" is a gate a caller checks before producing a
/// [`FallbackTransitionTrace`], kept as its own standalone type rather than folded
/// into the trace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisqualificationEligibility
{
    Retryable,
    SubstitutionEligible,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::DisqualificationReason;

    #[test]
    fn Test_A_Trace_Carries_Exactly_What_It_Was_Given()
    {
        let trace = FallbackTransitionTrace {
            original_candidate: "gpt-5-mini".to_owned(),
            disqualification: RuntimeCandidateDisqualification {
                reason: DisqualificationReason::ProjectedBudgetBreach,
            },
            fallback_decision: "route to claude-haiku-4-5".to_owned(),
            changed_guarantees: vec!["cost_or_latency_class".to_owned()],
            discarded_cost: "1200 assembled tokens".to_owned(),
        };

        assert_eq!(trace.original_candidate, "gpt-5-mini");
        assert_eq!(trace.disqualification.reason, DisqualificationReason::ProjectedBudgetBreach);
        assert_eq!(trace.changed_guarantees, ["cost_or_latency_class"]);
    }

    #[test]
    fn Test_Eligibility_Has_Two_Distinct_States()
    {
        assert_ne!(DisqualificationEligibility::Retryable, DisqualificationEligibility::SubstitutionEligible);
    }
}

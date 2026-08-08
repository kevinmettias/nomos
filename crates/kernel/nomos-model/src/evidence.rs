//! What supports a claim, and what was never looked at.

use nomos_contracts::{Applicability, EvidenceClass, ProviderId, SubjectId};
use serde::{Deserialize, Serialize};

/// A pointer to something that supports a claim.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceRef
{
    /// What kind of thing is being pointed at — a run record, a capture, a test result,
    /// a source range.
    pub kind: String,
    /// How to find it.
    pub locator: String,
}

/// A claim together with how it was come by and who produced it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence
{
    /// How this was come by.
    pub class: EvidenceClass,
    /// Who produced it.
    pub producer: ProviderId,
    /// What supports it.
    pub supporting: Vec<EvidenceRef>,
}

impl Evidence
{
    /// Whether this evidence may be reported as a mechanical result.
    ///
    /// Delegates to [`EvidenceClass::Is_Mechanical`] rather than restating the rule, so
    /// there is one place that decides what counts as a machine having checked
    /// something.
    #[must_use]
    pub const fn Is_Mechanical(&self) -> bool
    {
        return self.class.Is_Mechanical();
    }
}

/// Something a run did not look at, and why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageGap
{
    /// What was not covered.
    pub subject: SubjectId,
    /// Why not.
    pub reason: Applicability,
}

/// What a run examined, and what it did not.
///
/// The second half is the part that matters and the part every tool forgets. A run that
/// evaluated four subjects out of nine hundred and reported no findings has said almost
/// nothing, and without this record it looks exactly like a run that evaluated all nine
/// hundred.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage
{
    /// How many subjects were actually judged.
    pub evaluated: u64,
    /// How many were deliberately out of scope — the rule does not bind them, or policy
    /// switched it off.
    pub excluded: u64,
    /// What was in scope and not examined.
    pub gaps: Vec<CoverageGap>,
}

impl Coverage
{
    /// Whether this run examined anything at all.
    ///
    /// A run with no effective coverage must not be able to report success. This
    /// predicate is what a gate consults before it is permitted to construct a pass,
    /// and it is deliberately about *evaluation*, not about the absence of findings.
    #[must_use]
    pub const fn Has_Effective_Coverage(&self) -> bool
    {
        return self.evaluated > 0;
    }

    /// The gaps that represent analysis that was wanted and did not happen.
    ///
    /// Excludes deliberate absences, which are decisions rather than debt.
    pub fn Debt(&self) -> impl Iterator<Item = &CoverageGap>
    {
        return self.gaps.iter().filter(|gap| gap.reason.Is_Coverage_Debt());
    }

    /// Whether every in-scope subject was examined.
    #[must_use]
    pub fn Is_Complete(&self) -> bool
    {
        return self.Has_Effective_Coverage() && self.Debt().next().is_none();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::digest::Content_Digest;

    fn Subject(name: &str) -> SubjectId
    {
        return SubjectId::From_Digest(Content_Digest(name.as_bytes()));
    }

    /// The load-bearing case. A run that judged nothing must not be able to say it
    /// found nothing wrong.
    #[test]
    fn Test_A_Run_That_Evaluated_Nothing_Should_Have_No_Effective_Coverage()
    {
        assert!(!Coverage::default().Has_Effective_Coverage());
        assert!(!Coverage::default().Is_Complete());
    }

    #[test]
    fn Test_Debt_Should_Exclude_Deliberate_Absences()
    {
        let coverage = Coverage {
            evaluated: 3,
            excluded: 2,
            gaps: vec![
                CoverageGap {
                    subject: Subject("a.rs"),
                    reason: Applicability::NotApplicable,
                },
                CoverageGap {
                    subject: Subject("b.rs"),
                    reason: Applicability::ConfigurationDisabled,
                },
                CoverageGap {
                    subject: Subject("c.rs"),
                    reason: Applicability::MissingCapability,
                },
            ],
        };

        let debt: Vec<&CoverageGap> = coverage.Debt().collect();

        assert_eq!(debt.len(), 1);
        assert_eq!(
            debt.first().map(|gap| gap.reason),
            Some(Applicability::MissingCapability)
        );
        assert!(!coverage.Is_Complete());
    }

    /// A run with only deliberate absences really is complete — otherwise every honest
    /// configuration reads as incomplete and the signal is worthless.
    #[test]
    fn Test_Deliberate_Absences_Alone_Should_Still_Be_Complete()
    {
        let coverage = Coverage {
            evaluated: 3,
            excluded: 1,
            gaps: vec![CoverageGap {
                subject: Subject("a.rs"),
                reason: Applicability::NotApplicable,
            }],
        };

        assert!(coverage.Is_Complete());
    }

    #[test]
    fn Test_Agent_Evidence_Should_Not_Be_Mechanical()
    {
        let agent_claim = Evidence {
            class: EvidenceClass::AgentJudged,
            producer: ProviderId::New("some-executor"),
            supporting: Vec::new(),
        };

        assert!(!agent_claim.Is_Mechanical());
    }
}

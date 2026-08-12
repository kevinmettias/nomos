use serde::{Deserialize, Serialize};

use super::CoverageGap;

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
    /// Excludes deliberate absences, which are decisions rather than debt. Also excludes
    /// [`Coverage::Agent_Required`], which is wanted and undone but is not debt: nothing
    /// is missing, and telling a reader to install a provider would be false.
    pub fn Debt(&self) -> impl Iterator<Item = &CoverageGap>
    {
        return self.gaps.iter().filter(|gap| gap.reason.Is_Coverage_Debt());
    }

    /// The gaps a model could close, and no provider will.
    ///
    /// `CHK-003`'s seventh reporting category, kept out of both `evaluated` and
    /// [`Coverage::Debt`] because it is neither. Before `OD-CONTRACTS-002` these subjects
    /// were filed as `MissingCapability` or left out of `gaps` altogether, so a run either
    /// overstated a capability gap or said nothing.
    pub fn Agent_Required(&self) -> impl Iterator<Item = &CoverageGap>
    {
        return self.gaps.iter().filter(|gap| gap.reason.Requires_Agent());
    }

    /// Whether every in-scope subject was examined.
    ///
    /// Agent-required subjects were not examined, so they hold this false. Their being
    /// outside [`Coverage::Debt`] is about what the remedy is, not about whether the run
    /// finished — collapsing those two questions is what made a subject nobody judged
    /// indistinguishable from one nothing binds.
    #[must_use]
    pub fn Is_Complete(&self) -> bool
    {
        return self.Has_Effective_Coverage()
            && self.Debt().next().is_none()
            && self.Agent_Required().next().is_none();
    }
}

#[cfg(test)]
mod tests
{
    use nomos_contracts::{Applicability, SubjectId};

    use super::*;
    use crate::Content_Digest;

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
                Gap("a.rs", Applicability::NotApplicable),
                Gap("b.rs", Applicability::ConfigurationDisabled),
                Gap("c.rs", Applicability::MissingCapability),
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

    /// One subject the run did not evaluate, and why.
    fn Gap(subject: &str, reason: Applicability) -> CoverageGap
    {
        return CoverageGap {
            subject: Subject(subject),
            reason,
        };
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

    /// `CHK-003`'s seventh category, and the two answers it replaces. An agent-required
    /// subject counted as debt sends the reader to install a provider that does not
    /// exist; one counted as evaluated or dropped from `gaps` says a run judged it.
    #[test]
    fn Test_Agent_Required_Should_Be_A_Gap_That_Is_Not_Debt()
    {
        let coverage = Coverage {
            evaluated: 3,
            excluded: 1,
            gaps: vec![CoverageGap {
                subject: Subject("a.rs"),
                reason: Applicability::AgentRequired,
            }],
        };

        assert_eq!(coverage.Debt().count(), 0, "no provider is missing");
        assert_eq!(coverage.Agent_Required().count(), 1);
        assert_eq!(coverage.evaluated, 3, "a model has not judged it");
        assert!(
            !coverage.Is_Complete(),
            "a subject nobody judged must not read as examined"
        );
    }

    /// The three buckets are disjoint, so a subject lands in exactly one of them. Debt
    /// and agent-required overlapping is how the same gap gets both remedies, and neither
    /// gets acted on.
    #[test]
    fn Test_The_Buckets_Should_Not_Overlap()
    {
        let coverage = Coverage {
            evaluated: 1,
            excluded: 0,
            gaps: vec![
                CoverageGap {
                    subject: Subject("a.rs"),
                    reason: Applicability::MissingCapability,
                },
                CoverageGap {
                    subject: Subject("b.rs"),
                    reason: Applicability::AgentRequired,
                },
                CoverageGap {
                    subject: Subject("c.rs"),
                    reason: Applicability::NotApplicable,
                },
            ],
        };

        assert_eq!(coverage.Debt().count(), 1);
        assert_eq!(coverage.Agent_Required().count(), 1);
        assert!(coverage.Debt().all(|gap| !gap.reason.Requires_Agent()));
        assert!(coverage
            .Agent_Required()
            .all(|gap| !gap.reason.Is_Coverage_Debt()));
    }
}

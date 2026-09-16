//! What one family reconciliation found.

use core::fmt::Write as _;
use crate::Family;

/// How many absent identifiers a summary line names before it falls back to the count alone.
/// Naming a few is what makes a count checkable by eye; naming all of them makes the summary
/// into the report it is supposed to introduce.
const NAMED_IN_A_SUMMARY: usize = 5;

use crate::Disposition;
use crate::IdentifierOutcome;
/// One row per v14 identifier, never a count.
///
/// The plan requires differences reported per identifier and never summarised, so the
/// report holds every outcome and the counts are queries over it. A report that stored
/// totals could not answer "which ones", which is the only question worth asking.
#[derive(Clone, Debug, Default)]
pub struct ReconciliationReport
{
    pub outcomes: Vec<IdentifierOutcome>,
}

impl ReconciliationReport
{
    #[must_use]
    pub fn Absent(&self) -> Vec<&IdentifierOutcome>
    {
        return self
            .outcomes
            .iter()
            .filter(|outcome| outcome.disposition == Disposition::Absent)
            .collect();
    }

    #[must_use]
    pub fn Reworded(&self) -> Vec<&IdentifierOutcome>
    {
        return self
            .outcomes
            .iter()
            .filter(|outcome| matches!(outcome.disposition, Disposition::Reworded { .. }))
            .collect();
    }

    #[must_use]
    pub fn Preserved_In(&self, family: Family) -> usize
    {
        return self
            .outcomes
            .iter()
            .filter(|outcome| outcome.family == family && outcome.disposition == Disposition::Preserved)
            .count();
    }

    #[must_use]
    pub fn Declared_In(&self, family: Family) -> usize
    {
        return self.outcomes.iter().filter(|outcome| outcome.family == family).count();
    }

    #[must_use]
    pub fn Absent_In(&self, family: Family) -> Vec<&str>
    {
        return self
            .Absent()
            .into_iter()
            .filter(|outcome| outcome.family == family)
            .map(|outcome| outcome.id.as_str())
            .collect();
    }

    /// Names families and identifiers, never a bare total.
    #[must_use]
    pub fn Summary(&self) -> String
    {
        let mut lines = Vec::new();
        for family in Family::All()
        {
            lines.push(self.Family_Line(*family));
        }

        return lines.join("\n");
    }

    /// One family's count, and the identifiers behind it when any are missing.
    ///
    /// Only the first five are named. The list is there so the reader can go and look at
    /// one, and a hundred identifiers on a summary line is not a list anybody looks at.
    fn Family_Line(&self, family: Family) -> String
    {
        let absent = self.Absent_In(family);
        let mut line = format!(
            "{}: {} of {} preserved",
            family.Label(),
            self.Preserved_In(family),
            self.Declared_In(family)
        );
        if !absent.is_empty()
        {
            let named: Vec<&str> = absent.iter().take(NAMED_IN_A_SUMMARY).copied().collect();
            let _ = write!(
                line,
                ", {} absent ({}{})",
                absent.len(),
                named.join(", "),
                if absent.len() > named.len() { ", …" } else { "" }
            );
        }

        return line;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_model::ContentHash;

    /// How many `Requirement` outcomes the `Declared_In` fixture below carries — two, against
    /// one `Story`, so a count that ignored the family filter would answer three.
    const REQUIREMENT_OUTCOMES: usize = 2;

    fn Outcome(id: &str, family: Family, disposition: Disposition) -> IdentifierOutcome
    {
        return IdentifierOutcome { id: id.to_owned(), family, disposition };
    }

    #[test]
    fn Test_Absent_Should_List_Outcomes_Reported_As_Absent()
    {
        let report = ReconciliationReport {
            outcomes: vec![
                Outcome("MODEL-001", Family::Requirement, Disposition::Preserved),
                Outcome("MODEL-002", Family::Requirement, Disposition::Absent),
            ],
        };

        let absent = report.Absent();

        assert_eq!(absent.len(), 1);
        assert_eq!(absent.first().map(|outcome| outcome.id.as_str()), Some("MODEL-002"));
    }

    #[test]
    fn Test_Reworded_Should_List_Outcomes_Whose_Statement_Changed()
    {
        let report = ReconciliationReport {
            outcomes: vec![
                Outcome("MODEL-001", Family::Requirement, Disposition::Preserved),
                Outcome(
                    "MODEL-002",
                    Family::Requirement,
                    Disposition::Reworded {
                        v14: ContentHash::Of_Normalized("a"),
                        v15: ContentHash::Of_Normalized("b"),
                    },
                ),
            ],
        };

        let reworded = report.Reworded();

        assert_eq!(reworded.len(), 1);
        assert_eq!(reworded.first().map(|outcome| outcome.id.as_str()), Some("MODEL-002"));
    }

    #[test]
    fn Test_Preserved_In_Should_Count_One_Family_Preserved_Outcomes()
    {
        let report = ReconciliationReport {
            outcomes: vec![
                Outcome("MODEL-001", Family::Requirement, Disposition::Preserved),
                Outcome("US-A-001", Family::Story, Disposition::Preserved),
            ],
        };

        assert_eq!(report.Preserved_In(Family::Requirement), 1);
        assert_eq!(report.Preserved_In(Family::Acceptance), 0);
    }

    #[test]
    fn Test_Declared_In_Should_Count_Every_Outcome_For_One_Family()
    {
        let report = ReconciliationReport {
            outcomes: vec![
                Outcome("MODEL-001", Family::Requirement, Disposition::Preserved),
                Outcome("MODEL-002", Family::Requirement, Disposition::Absent),
                Outcome("US-A-001", Family::Story, Disposition::Preserved),
            ],
        };

        assert_eq!(report.Declared_In(Family::Requirement), REQUIREMENT_OUTCOMES);
        assert_eq!(report.Declared_In(Family::Story), 1);
    }

    #[test]
    fn Test_Absent_In_Should_Name_Absent_Identifiers_For_One_Family()
    {
        let report = ReconciliationReport {
            outcomes: vec![
                Outcome("MODEL-001", Family::Requirement, Disposition::Absent),
                Outcome("US-A-001", Family::Story, Disposition::Absent),
            ],
        };

        assert_eq!(report.Absent_In(Family::Requirement), vec!["MODEL-001"]);
    }

    #[test]
    fn Test_Summary_Should_List_Every_Family_With_Its_Preserved_And_Declared_Counts()
    {
        let report = ReconciliationReport {
            outcomes: vec![Outcome("MODEL-001", Family::Requirement, Disposition::Preserved)],
        };

        let summary = report.Summary();

        assert!(summary.contains("requirement: 1 of 1 preserved"));
        assert!(summary.contains("story: 0 of 0 preserved"));
    }
}

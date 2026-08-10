//! What one family reconciliation found.

use core::fmt::Write as _;
use crate::family::Family;
use crate::disposition::Disposition;
use crate::identifier_outcome::IdentifierOutcome;
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
            let named: Vec<&str> = absent.iter().take(5).copied().collect();
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

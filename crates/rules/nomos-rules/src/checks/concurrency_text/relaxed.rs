//! `concurrency_text`'s `Relaxed`-specific rule, in its own file so the family's three rules
//! no longer share one `Check_` prefix.

use super::{Findings_For, RELAXED_NOT_USED_WHEN_ORDERING_MATTERS};
use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

/// Reports a `Relaxed` ordering with no adjacent `atomic-ordering: allow` reason -- the "name why
/// no ordering guarantee is needed" obligation.
#[must_use]
pub fn Check_Relaxed_Not_Used_When_Ordering_Matters(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    return Findings_For(sources, facts, RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, |variant| return variant == "Relaxed");
}

#[cfg(test)]
mod tests
{
    use super::super::tests::{Check, Source_File, SourceText};
    use super::*;
    use nomos_contracts::RuleId;

    #[test]
    fn Test_Check_Relaxed_Not_Used_When_Ordering_Matters_Should_Report_An_Unexplained_Relaxed()
    {
        let source = Source_File(SourceText { path: "src/counter.rs", text: "counter.fetch_add(1, Ordering::Relaxed);\n" });
        let findings = Check(Check_Relaxed_Not_Used_When_Ordering_Matters, source);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(RELAXED_NOT_USED_WHEN_ORDERING_MATTERS));
    }

    #[test]
    fn Test_Check_Relaxed_Not_Used_When_Ordering_Matters_Should_Accept_A_Marker_Reason()
    {
        let source = Source_File(SourceText { path: "src/counter.rs", text: "counter.fetch_add(1, Ordering::Relaxed); // atomic-ordering: allow: statistics-only, nothing else reads this\n" });
        let findings = Check(Check_Relaxed_Not_Used_When_Ordering_Matters, source);
        assert!(findings.is_empty(), "{findings:?}");
    }
}

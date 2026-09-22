//! [`SarifLevel`], the `result.level` a finding projects to.

use nomos_contracts::{Finding, GateCategory};
use serde::Serialize;

/// SARIF 2.1.0 §3.27.10's three reportable levels, in the order the specification lists them.
///
/// `none` is deliberately absent. The specification reserves it for a result whose `kind` is
/// not `fail`, and every result this crate emits is a `fail` -- a finding is a rule's judgment
/// that something is wrong, and what a policy did about it afterwards is carried by
/// [`super::sarif_suppression::SarifSuppression`], never by downgrading the level. A consumer
/// that reads `level` alone therefore sees what the finding *deserves*; one that also reads
/// `suppressions` sees what the run *did*, which is the two-halves split
/// `nomos_contracts::GateCategory`'s own doc says a reader must be able to keep apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SarifLevel
{
    /// The finding can fail a build.
    Error,
    /// The finding is reported at the reader and cannot fail a build.
    Warning,
    /// The finding is information about the rule or the analysis, not a violation a build
    /// would act on.
    Note,
}

impl SarifLevel
{
    /// The level `finding` deserves, from its gate category and its applicability together.
    ///
    /// `Finding::Can_Fail_A_Build` decides `error`, and it is consulted rather than the gate
    /// category alone for the reason that function's own doc gives: a rule that never reached
    /// its subject has reported on the analysis, not on the code, and a consumer told it was
    /// an error would fail a build over a provider that could not run. Such a finding is a
    /// `note` whatever its gate category says. Of the evaluated remainder, an advisory gate is
    /// a `warning` -- the wiring reports it and discards the result -- and a `Review` or
    /// `Unreachable` gate is a `note`, because nothing mechanical stands behind either.
    pub(crate) fn Of(finding: &Finding) -> Self
    {
        if finding.Can_Fail_A_Build()
        {
            return Self::Error;
        }

        if finding.applicability.Is_Evaluated() && finding.gate == GateCategory::Advisory
        {
            return Self::Warning;
        }

        return Self::Note;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::Finding_At;
    use nomos_contracts::Applicability;

    #[test]
    fn Test_Of_Should_Be_Error_For_An_Evaluated_Blocking_Finding()
    {
        assert_eq!(SarifLevel::Of(&Finding_At("a-rule", "a.rs:1")), SarifLevel::Error);
    }

    #[test]
    fn Test_Of_Should_Be_Warning_For_An_Evaluated_Advisory_Finding()
    {
        let mut advisory = Finding_At("a-rule", "a.rs:1");
        advisory.gate = GateCategory::Advisory;

        assert_eq!(SarifLevel::Of(&advisory), SarifLevel::Warning);
    }

    /// Nothing mechanical stands behind either category, so neither is a warning a reader
    /// should act on as though a gate had spoken.
    #[test]
    fn Test_Of_Should_Be_Note_For_A_Review_Or_Unreachable_Gate()
    {
        for toothless in [GateCategory::Review, GateCategory::Unreachable]
        {
            let mut finding = Finding_At("a-rule", "a.rs:1");
            finding.gate = toothless;

            assert_eq!(SarifLevel::Of(&finding), SarifLevel::Note, "{toothless}");
        }
    }

    /// The falsifier for the applicability half of the derivation: with the guard removed a
    /// blocking-category finding from a rule that could not run would project as an error,
    /// and a consumer would fail a build over a provider being absent.
    #[test]
    fn Test_Of_Should_Be_Note_For_A_Blocking_Finding_Whose_Rule_Never_Reached_Its_Subject()
    {
        let mut unreached = Finding_At("a-rule", "a.rs:1");
        unreached.applicability = Applicability::ProviderUnavailable;

        assert_eq!(unreached.gate, GateCategory::Blocking, "the fixture must be blocking for this to discriminate");
        assert_eq!(SarifLevel::Of(&unreached), SarifLevel::Note);
    }

    /// An advisory finding from a rule that never ran is not even a warning: the guard reads
    /// applicability before it reads the category.
    #[test]
    fn Test_Of_Should_Be_Note_For_An_Advisory_Finding_Whose_Rule_Never_Reached_Its_Subject()
    {
        let mut unreached = Finding_At("a-rule", "a.rs:1");
        unreached.gate = GateCategory::Advisory;
        unreached.applicability = Applicability::Unparseable;

        assert_eq!(SarifLevel::Of(&unreached), SarifLevel::Note);
    }

    #[test]
    fn Test_Every_Level_Should_Serialize_As_Its_Lowercase_Specification_Spelling()
    {
        let rendered = serde_json::to_string(&[SarifLevel::Error, SarifLevel::Warning, SarifLevel::Note])
            .expect("a derived Serialize over three unit variants has nothing to refuse");

        assert_eq!(rendered, r#"["error","warning","note"]"#);
    }
}

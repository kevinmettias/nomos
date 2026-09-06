//! Whether a real correction family exists for a finding's rule -- answered by asking
//! `nomos-correction-orchestration`, not by declaring it here.
//!
//! [`nomos_correction_orchestration::CorrectionFamily`] is that crate's own declaration of
//! which rules it composes a correction for, and [`AvailableCorrection::Of`] is a lookup
//! against it. This module previously carried a literal array of the same two rule ids,
//! because that crate exported no membership query at all -- real, declared knowledge,
//! since both were `nomos_rules`' own public constants rather than guessed strings, but a
//! copy all the same, and one a third family composed over there would have left stale
//! here without this crate's build failing to notice. The old doc named exporting that
//! membership as the real fix; `P73-LSP-CORRECTION-FAMILY-DUPLICATED-2` did it, so the
//! list now lives once, in the crate whose pipeline honours it.
//!
//! Reading a declaration is not the same as re-running the pipeline, and this still never
//! invokes `Run_Correction`. Doing that per diagnostic would re-judge the whole tree to
//! get a coarse, first-claim-wins answer -- `run::Claimed_Fix`'s own doc calls its
//! priority order an arbitrary tie-break rather than a per-finding correspondence -- which
//! is a worse answer than the question actually asked here, and a far slower one.

use nomos_contracts::RuleId;
use nomos_correction_orchestration::CorrectionFamily;
use serde::Serialize;

/// The correction family a finding's rule participates in, when this build knows of one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AvailableCorrection
{
    /// The rule id this correction family fixes -- always `finding.rule`'s own value when
    /// this type is constructed at all, carried explicitly rather than left for a caller to
    /// re-derive from context.
    pub family: String,
}

impl AvailableCorrection
{
    /// Whether `nomos-correction-orchestration` composes a real correction family for
    /// `rule` today, as that crate itself declares it.
    #[must_use]
    pub(crate) fn Of(rule: &RuleId) -> Option<Self>
    {
        if CorrectionFamily::Of(rule).is_none()
        {
            return None;
        }

        return Some(Self { family: rule.As_Str().to_owned() });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The drift guard this module exists to make possible: every family that crate
    /// declares must be answerable here.
    ///
    /// It iterates the export rather than a copy of it, so a third family composed over
    /// there is covered on the day it lands -- and reintroducing a literal array here that
    /// omits one fails loudly instead of quietly reporting no correction available.
    #[test]
    fn Test_Of_Should_Answer_For_Every_Family_Correction_Orchestration_Declares()
    {
        for family in CorrectionFamily::ALL
        {
            let rule = family.Rule();

            let correction = AvailableCorrection::Of(&rule).unwrap_or_else(|| panic!("{family:?} is a declared correction family"));

            assert_eq!(correction.family, rule.As_Str());
        }
    }

    #[test]
    fn Test_Of_Should_Recognize_The_Completeness_Mirror_Family()
    {
        let rule = RuleId::New(nomos_rules::COMPLETENESS_MIRROR);

        let correction = AvailableCorrection::Of(&rule).expect("completeness-mirror has a real correction family");

        assert_eq!(correction.family, nomos_rules::COMPLETENESS_MIRROR);
    }

    #[test]
    fn Test_Of_Should_Recognize_The_Trailing_Whitespace_Family()
    {
        let rule = RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE);

        assert!(AvailableCorrection::Of(&rule).is_some());
    }

    #[test]
    fn Test_Of_Should_Be_None_For_A_Rule_With_No_Known_Correction_Family()
    {
        let rule = RuleId::New(nomos_rules::NAMING_CONVENTION);

        assert!(AvailableCorrection::Of(&rule).is_none());
    }
}

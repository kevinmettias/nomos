//! Whether a real correction family exists for a finding's rule -- the narrow, honest
//! answer `docs/records/OD-HOST-010-*.md` names in full.
//!
//! `nomos-correction-orchestration::run::Judged` composes exactly two rules today --
//! `nomos_rules::COMPLETENESS_MIRROR` and `nomos_rules::NO_TRAILING_WHITESPACE` -- into a
//! real `CorrectionCandidate` through its own private claim recognizers
//! (`phantom_mirror::Phantom_Claim`, `trailing_whitespace::Trailing_Whitespace_Claim`).
//! Neither the rule list nor a "does a family exist for this rule" query is exported from
//! that crate: its own `lib.rs` names only `CorrectionCommand`, `CorrectionOutcome`,
//! `CorrectionEnvironment` and `Run_Correction`. This type declares the two rule ids
//! directly rather than inventing an export that crate does not have, which is the
//! judgment call this crate's own report names explicitly: it is real, declared knowledge
//! (these two constants are `nomos_rules`' own public identifiers, not guessed strings),
//! but it duplicates a private list `nomos-correction-orchestration` could add a third
//! entry to without this crate's own build failing to notice. A real fix is exporting that
//! membership from `nomos-correction-orchestration` itself, not invented here.
//!
//! This never invokes `Run_Correction`. Doing so per diagnostic would re-judge the whole
//! tree for a coarse, first-claim-wins answer (`Claimed_Fix`'s own doc names the priority
//! order as "this pipeline's own arbitrary tie-break", not a per-finding correspondence),
//! which is a worse and slower answer than declaring the two rule ids a correction family
//! is known to exist for.

use nomos_contracts::RuleId;
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
    /// `rule` today.
    #[must_use]
    pub(crate) fn Of(rule: &RuleId) -> Option<Self>
    {
        let known = [nomos_rules::COMPLETENESS_MIRROR, nomos_rules::NO_TRAILING_WHITESPACE];
        if !known.contains(&rule.As_Str())
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

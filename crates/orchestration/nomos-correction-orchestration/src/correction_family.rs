//! Which rules this crate composes a real correction family for -- the one declaration
//! that [`crate::run`]'s own judgment and every host asking "is a correction available for
//! this finding" both read.
//!
//! Before this type the membership existed twice: as a private local array inside
//! [`crate::run`]'s own `Judged`, and again as a literal array inside `nomos-lsp`'s
//! `walk_outward::available_correction`, which had no export to call because this crate
//! published only [`crate::CorrectionCommand`], [`crate::CorrectionOutcome`],
//! [`crate::CorrectionEnvironment`] and [`crate::Run_Correction`]. Both arrays named the
//! same two real `nomos_rules` constants, so both compiled and both stayed green -- and a
//! third family added to [`crate::run`] alone would have left that host silently reporting
//! no correction available for it, with nothing anywhere failing to say so.
//!
//! Enumerating the families rather than listing their rule ids is what makes that drift
//! mechanical rather than remembered. [`crate::run`]'s `Claimed_Fix` dispatches over
//! [`CorrectionFamily::ALL`] with an exhaustive `match`, so a variant added here does not
//! compile until it is wired to a real claim recognizer, and its `Judged` derives the
//! rules it selects from the same list rather than repeating them. The host's own answer
//! is [`CorrectionFamily::Of`], which is a lookup rather than a second list.
//!
//! This module names rules and nothing else. Which recognizer a family uses, and what its
//! fix looks like, stay in [`crate::phantom_mirror`] and [`crate::trailing_whitespace`]
//! where they already were -- neither family's own module knows the other exists, and
//! this one is not the place that changes.

use nomos_contracts::RuleId;

/// One correction family this crate composes: a rule whose blocking findings it
/// recognizes, and the fix shape it builds for them.
///
/// Ordered as [`Self::ALL`] declares it, which is the priority order [`crate::run`]'s own
/// `Claimed_Fix` tries them in. That crate module's doc says what the order is and is not
/// -- an arbitrary tie-break for a tree presenting both at once, never a ranking policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectionFamily
{
    /// [`crate::phantom_mirror`]: one blocking `completeness-mirror` finding, struck as a
    /// single doc-comment line.
    PhantomMirror,
    /// [`crate::trailing_whitespace`]: every blocking `no-trailing-whitespace` finding in
    /// one file, batched into one candidate.
    TrailingWhitespace,
}

impl CorrectionFamily
{
    /// Every family this crate composes, in the priority order [`crate::run`]'s own
    /// `Claimed_Fix` tries them.
    pub const ALL: [Self; 2] = [Self::PhantomMirror, Self::TrailingWhitespace];

    /// The rule whose blocking findings this family corrects.
    #[must_use]
    pub fn Rule(self) -> RuleId
    {
        return match self
        {
            Self::PhantomMirror => RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
            Self::TrailingWhitespace => RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE),
        };
    }

    /// The family this crate composes for `rule`, or `None` when it composes none.
    ///
    /// The question a host asks per diagnostic, answered from the declaration rather than
    /// by running the pipeline: [`crate::Run_Correction`] proposes one candidate per run
    /// under `Claimed_Fix`'s own tie-break, which is a coarser answer than "does a family
    /// exist for this rule" and a far more expensive one to ask per finding.
    #[must_use]
    pub fn Of(rule: &RuleId) -> Option<Self>
    {
        return Self::ALL.into_iter().find(|family| return family.Rule() == *rule);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A family naming a rule this build describes no descriptor for could never match a
    /// finding, because no run would ever produce one carrying that id -- a family retired
    /// or renamed out from under this list, silently doing nothing.
    #[test]
    fn Test_Every_Family_Should_Name_A_Rule_Nomos_Rules_Describes()
    {
        for family in CorrectionFamily::ALL
        {
            let rule = family.Rule();

            assert!(
                nomos_rules::DESCRIPTORS.iter().any(|descriptor| return descriptor.Rule() == rule),
                "{family:?} corrects `{rule}`, which this build's nomos-rules describes no rule for"
            );
        }
    }

    /// Two families claiming one rule would make [`CorrectionFamily::Of`] answer with
    /// whichever [`CorrectionFamily::ALL`] happens to list first -- a coin toss wearing a
    /// decision's clothes.
    #[test]
    fn Test_All_Should_Name_Each_Rule_At_Most_Once()
    {
        let mut rules: Vec<String> = CorrectionFamily::ALL.iter().map(|family| return family.Rule().As_Str().to_owned()).collect();
        let declared = rules.len();
        rules.sort_unstable();
        rules.dedup();

        assert_eq!(rules.len(), declared, "two correction families name the same rule");
    }

    #[test]
    fn Test_Of_Should_Recognize_Every_Declared_Family()
    {
        for family in CorrectionFamily::ALL
        {
            assert_eq!(CorrectionFamily::Of(&family.Rule()), Some(family));
        }
    }

    #[test]
    fn Test_Of_Should_Refuse_A_Rule_No_Family_Corrects()
    {
        assert_eq!(CorrectionFamily::Of(&RuleId::New(nomos_rules::NAMING_CONVENTION)), None);
    }
}

//! What the run that produced a finding can honestly say about the facts behind it.

use crate::walk_outward::FactRead;
use nomos_check_orchestration::{SupportingFactTrail, SupportingFacts as Answer};
use nomos_contracts::RuleId;
use serde::Serialize;

/// `OD-HOST-016`'s four-shape answer for a finding's rule, as an editor receives it.
///
/// Four words and a list, rather than a list and an absence. The hazard the vocabulary exists
/// for is that an absent trail reads as a judgment made without evidence, and that is false in
/// three distinct ways: a rule whose descriptor is `SubjectKind::SourceText` could never have
/// read a fact at all, a finding raised while a capability was materialized never passed
/// through a rule's judgment, and a run that kept no trail simply did not record one. Reduced
/// to one empty list those become the same silence, which would report a structural absence as
/// a gap -- the direction `OD-COMPLETENESS-001` warns about.
///
/// So [`Self::answer`] is what a reader branches on and [`Self::reads`] is never that branch.
/// An empty `reads` under `"Read"` is a rule that asked and got nothing back, which is a
/// judgment made with evidence about an absence and is not the same claim as having asked
/// nothing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SupportingFacts
{
    /// Which of the four shapes this is: `"NotFactBacked"`, `"Read"`,
    /// `"RaisedByMaterialization"` or `"Unrecorded"` -- `OD-HOST-016`'s own four words, never
    /// a label invented here.
    pub answer: &'static str,
    /// The distinct provenance tuples the rule's reads reduced to, populated only under
    /// `"Read"` and empty under it too when the rule read nothing.
    ///
    /// What this may honestly claim is exactly what that record allows: the rule that produced
    /// this finding, in the call that produced it, read these capabilities from these providers
    /// at these guarantees, and these reads missed. It may not claim that any particular one of
    /// them backs this particular finding -- the grain is the rule, because a per-finding
    /// answer would need a join on subject equality that is false for twelve of the fifty-six
    /// sites constructing a finding in `nomos-rules`.
    pub reads: Vec<FactRead>,
}

impl SupportingFacts
{
    /// What `trail` says about `rule`, which is one of four answers and never a silence.
    ///
    /// Read out of the trail the run carried back on its own outcome, never recomputed here:
    /// this crate holds no fact store, no generation and no key, and `OD-HOST-016`'s sixth
    /// decision refused reconstructing one from outside a rule. The descriptor-derived shape
    /// is answered by the trail itself, so a rule this build never described falls through to
    /// whatever was actually observed rather than being guessed at from here.
    #[must_use]
    pub(crate) fn Of(trail: &SupportingFactTrail, rule: &RuleId) -> Self
    {
        return Self::Projected(&trail.Facts_For(rule.As_Str()));
    }

    /// `shape` in the shape an editor receives, one word per shape and the tuples only where
    /// there are any.
    fn Projected(shape: &Answer) -> Self
    {
        return match *shape
        {
            Answer::NotFactBacked => Self { answer: "NotFactBacked", reads: Vec::new() },
            Answer::Read(ref reads) => Self { answer: "Read", reads: reads.iter().map(FactRead::Of).collect() },
            Answer::RaisedByMaterialization => Self { answer: "RaisedByMaterialization", reads: Vec::new() },
            Answer::Unrecorded => Self { answer: "Unrecorded", reads: Vec::new() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The capability the one fixture tuple below is addressed to -- a placeholder, never a
    /// capability this workspace offers, for the reason
    /// `crate::walk_outward::fact_read`'s own fixture constant states.
    const FIXTURE_CAPABILITY: &str = "nomos.cap.test.shapes";

    /// The falsifier for this whole module: the three answers that carry no tuples must not
    /// arrive as one answer that carries no tuples.
    ///
    /// Every one of them projects to an empty `reads`, so a projection that dropped the word
    /// -- or spelled two of the four the same -- would still produce a payload an editor could
    /// read, and a reader would see a structural absence, an unrecorded run and a rule that
    /// read nothing as the same claim. Nothing else in this crate would go red for it.
    #[test]
    fn Test_Projected_Should_Give_Each_Of_The_Four_Shapes_Its_Own_Word()
    {
        let structural = SupportingFacts::Projected(&Answer::NotFactBacked);
        let read_nothing = SupportingFacts::Projected(&Answer::Read(Vec::new()));
        let materialized = SupportingFacts::Projected(&Answer::RaisedByMaterialization);
        let unrecorded = SupportingFacts::Projected(&Answer::Unrecorded);

        let words = [structural.answer, read_nothing.answer, materialized.answer, unrecorded.answer];
        let distinct: std::collections::BTreeSet<&str> = words.iter().copied().collect();

        assert_eq!(distinct.len(), words.len(), "two of the four shapes share a word: {words:?}");
        assert_eq!(structural.answer, "NotFactBacked");
        assert_eq!(read_nothing.answer, "Read");
        assert_eq!(materialized.answer, "RaisedByMaterialization");
        assert_eq!(unrecorded.answer, "Unrecorded");
        assert!(read_nothing.reads.is_empty(), "a rule that asked and got nothing carries no tuples");
    }

    /// A real trail this crate never wrote answers the two shapes an empty one can reach, and
    /// answers them differently for two real rules -- which is what says [`SupportingFacts::Of`]
    /// consults the trail and the descriptor behind it rather than defaulting to one word.
    #[test]
    fn Test_Of_Should_Separate_A_Source_Text_Rule_From_A_Rule_Whose_Trail_Was_Not_Recorded()
    {
        let empty = SupportingFactTrail::New();

        let source_text = SupportingFacts::Of(&empty, &RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE));
        let fact_backed = SupportingFacts::Of(&empty, &RuleId::New(nomos_rules::COMPLETENESS_MIRROR));

        assert_eq!(source_text.answer, "NotFactBacked", "no-trailing-whitespace judges source text; no fact could have been involved");
        assert_eq!(fact_backed.answer, "Unrecorded", "completeness-mirror reads facts, and this trail recorded none for it");
    }

    #[test]
    fn Test_Of_Should_Carry_The_Tuples_A_Trail_Holds_For_A_Rule_That_Read()
    {
        let projected = SupportingFacts::Projected(&Answer::Read(vec![Fixture_Read()]));

        assert_eq!(projected.answer, "Read");
        assert_eq!(projected.reads.len(), 1, "{:?}", projected.reads);
        let only = projected.reads.first().expect("asserted one tuple above");
        assert_eq!(only.capability, FIXTURE_CAPABILITY);
    }

    /// One answered read, enough to show a tuple survives the projection.
    fn Fixture_Read() -> nomos_check_orchestration::FactRead
    {
        return nomos_check_orchestration::FactRead {
            capability: nomos_contracts::CapabilityId::New(FIXTURE_CAPABILITY),
            provider: nomos_contracts::ProviderId::New("nomos.provider.test.projection"),
            provider_version: nomos_contracts::ContractVersion::New(1, 0),
            outcome: nomos_analysis::ReadOutcome::Materialized,
            guarantee: None,
            evidence: None,
            reads: 1,
        };
    }
}

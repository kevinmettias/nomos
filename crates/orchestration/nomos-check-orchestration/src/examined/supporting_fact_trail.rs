//! What each rule read in the call that produced its findings.

use std::collections::{BTreeMap, BTreeSet};

use nomos_contracts::{Finding, RuleId};
use nomos_rules::{SubjectKind, DESCRIPTORS};

use crate::examined::{FactRead, SupportingFacts};

/// The reduced read trail of one `nomos check` run, held per rule.
///
/// `OD-HOST-016` decided this is carried on [`crate::CheckOutcome::Judged`] rather than
/// returned beside it, for `OD-HOST-002`'s reason: two return shapes make the richer one the
/// only complete one, and every caller taking the thinner one silently sees a subset.
///
/// It is a value and not a view onto the store. The reduction resolved every answered read
/// against `nomos_analysis::MemoryFactStore` inside the run, while the generation was
/// unambiguous and the store was still in hand, so what leaves carries
/// `nomos_contracts::Guarantee` and `nomos_contracts::EvidenceClass` by value and no caller
/// ever needs a key or a store. That is the whole of why a `FactKey`-reconstruction
/// capability was refused.
///
/// The grain is the rule. [`Self::Facts_For`] takes a rule identifier and nothing else,
/// because a per-finding answer would need a join on subject equality that is false for
/// twelve of the fifty-six sites constructing a `nomos_contracts::Finding` in `nomos-rules`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SupportingFactTrail
{
    reads_by_rule: BTreeMap<RuleId, Vec<FactRead>>,
    raised_by_materialization: BTreeSet<RuleId>,
}

impl SupportingFactTrail
{
    /// A trail before any rule has run.
    #[must_use]
    pub fn New() -> Self
    {
        return Self { reads_by_rule: BTreeMap::new(), raised_by_materialization: BTreeSet::new() };
    }

    /// Which facts backed `rule`'s judgment, in the four shapes `OD-HOST-016` decided.
    ///
    /// [`SupportingFacts::NotFactBacked`] is answered first and from the descriptor alone, so
    /// a rule that structurally cannot read a fact never reports as a rule that read none --
    /// the hazard this vocabulary exists for, since an absent trail otherwise reads as a
    /// judgment made without evidence. A rule this trail holds reads for answers
    /// [`SupportingFacts::Read`] even when those reads are empty or are all misses, because
    /// "asked and got nothing" is a judgment made with evidence about an absence and is not
    /// the same claim as having asked nothing. Only then does a rule that raised a finding
    /// while a capability was materialized answer [`SupportingFacts::RaisedByMaterialization`],
    /// which is what stops it from falling into [`SupportingFacts::Unrecorded`].
    #[must_use]
    pub fn Facts_For(&self, rule: &str) -> SupportingFacts
    {
        if Judges_Source_Text(rule)
        {
            return SupportingFacts::NotFactBacked;
        }

        if let Some(reads) = self.reads_by_rule.get(&RuleId::New(rule))
        {
            return SupportingFacts::Read(reads.clone());
        }

        if self.raised_by_materialization.contains(&RuleId::New(rule))
        {
            return SupportingFacts::RaisedByMaterialization;
        }

        return SupportingFacts::Unrecorded;
    }

    /// Records the reduced trail one real invocation of `rule` produced.
    pub(crate) fn Record(&mut self, rule: &str, reads: Vec<FactRead>)
    {
        self.reads_by_rule.insert(RuleId::New(rule), reads);
    }

    /// Records which rules raised a finding while a capability was materialized in this call.
    ///
    /// Replaced rather than accumulated: materialization runs on every call, so this is a
    /// fact about the call in hand and not something a later call inherits.
    pub(crate) fn Note_Materialization_Raised(&mut self, findings: &[Finding])
    {
        self.raised_by_materialization = findings.iter().map(|finding| return finding.rule.clone()).collect();
    }

    /// The reduced reads this trail holds for `rule`, whatever its descriptor says.
    ///
    /// Test-only beside [`Self::Facts_For`] because the two answer different questions: that
    /// one answers what may honestly be said about a finding, and this one answers what was
    /// raw-observed, which is what a check of the declaration against reality has to read and
    /// what nothing else has any use for. Publishing both would let the observation be
    /// mistaken for the honest claim.
    #[cfg(test)]
    pub(crate) fn Observed_Reads(&self, rule: &str) -> Option<&[FactRead]>
    {
        return self.reads_by_rule.get(&RuleId::New(rule)).map(Vec::as_slice);
    }
}

/// Whether `rule`'s descriptor declares it judges source text alone, so no fact was ever
/// involved in what it says.
///
/// Derived from [`DESCRIPTORS`] rather than from the rule having read nothing, which is
/// `OD-HOST-016`'s own distinction: a rule that read nothing because its family was absent
/// and a rule that reads nothing because it judges text are the same silence and different
/// claims. `nomos_rules::rule_descriptor`'s own
/// `Test_A_Source_Text_Rule_Should_Require_No_Fact` is what keeps the correspondence true, so
/// this is a derivation rather than a second declaration. A rule no descriptor describes is
/// neither, and answers `false` so it falls through to whatever was actually observed.
fn Judges_Source_Text(rule: &str) -> bool
{
    return DESCRIPTORS
        .iter()
        .find(|descriptor| return descriptor.id == rule)
        .is_some_and(|descriptor| return matches!(descriptor.subject, SubjectKind::SourceText));
}

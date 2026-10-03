//! [`Populations`], how many sources each rule a run selected was judged over.
//!
//! `OD-ANALYSIS-012` version 2's second decision. A rule whose population was empty judged
//! nothing, and that is a fact apart from a rule that judged real subjects and found them clean:
//! the two produce the same zero findings, and only this tells them apart. It is reported beside
//! the claim and never in it. [`crate::Claim_Of`] reads findings alone, so a repository holding no
//! file of a rule's language reaches the same claim it would reach without this, and nothing a gate
//! derives from the claim moves. It carries no `GateCategory` and is not a finding: no subject has
//! it.

use nomos_contracts::RuleId;

/// For each rule a run selected, in the order the run judged them, how many sources it was judged
/// over: the population its descriptor declares, counted over the sources it was handed -- the walk,
/// or the capability family's slice a rule reads instead -- and one for a rule whose subject is the
/// workspace itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Populations
{
    judged: Vec<(RuleId, usize)>,
}

impl Populations
{
    /// No rule judged yet.
    #[must_use]
    pub fn New() -> Self
    {
        return Self::default();
    }

    /// Records that `rule` was judged over `size` sources.
    pub fn Note(&mut self, rule: RuleId, size: usize)
    {
        self.judged.push((rule, size));
    }

    /// How many sources `rule` was judged over, or `None` when the run did not select it.
    #[must_use]
    pub fn Of(&self, rule: &RuleId) -> Option<usize>
    {
        return self.judged.iter().find(|(judged, _)| return judged == rule).map(|(_, size)| return *size);
    }

    /// Every rule the run selected whose population was empty, in the order the run judged them.
    #[must_use]
    pub fn Empty(&self) -> Vec<&RuleId>
    {
        return self.judged.iter().filter(|(_, size)| return *size == 0).map(|(rule, _)| return rule).collect();
    }

    /// Every rule the run selected and the size of its population, in the order the run judged
    /// them.
    #[must_use]
    pub fn Judged(&self) -> &[(RuleId, usize)]
    {
        return &self.judged;
    }
}

use super::IdentityTransitionKind;

/// How many states a record must hold before it can say anything about a lineage's continuity.
///
/// Two, and the reason is that one is *vacuous*. A record holding a single state, which is the
/// state the run asking about it just contributed, has seen the lineage exactly once and has no
/// interval to say it survived: "observed in every state on record" is true of it and means
/// nothing. A claim made from one state would tell a repository that the occurrence it adopted
/// has persisted, on the strength of having looked once.
const STATES_A_CLAIM_NEEDS: u32 = 2;

/// How many of the states some record kept for one scope observed one occurrence lineage, and
/// which of them.
///
/// `OD-GATE-030` names three parts that would make a baseline's continuity provable and calls the
/// third *a history between them*: without a record of the states in between, an identity that
/// survives a revision establishes that an occurrence matching adoption's is present now, and
/// still not that it never left. This type is what a caller holding such a record asks, so that
/// the answer comes back in the vocabulary
/// [`IdentityTransitionKind`](super::IdentityTransitionKind) already publishes rather than in a
/// second one minted beside it.
///
/// # States are numbered from one, and zero means never
///
/// `recorded_states` is how many states the record holds for the scope; the most recent is
/// numbered by it. `first_state` and `last_state` are the earliest and latest that observed this
/// lineage, and both are zero when none did — which is a real case, because a lineage first seen
/// today has a record behind it that never saw it.
///
/// # Why the tally is the whole question
///
/// Whether an occurrence persisted or came back is decided by *when it was not there*, and three
/// counts plus two state numbers say that exactly: seen in every state is continuity, seen and
/// then missed and then seen again is a recreation, and anything the counts cannot separate is
/// neither. Handing this type the states themselves would let a caller pass a list that
/// disagrees with its own totals; handing it the totals cannot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OccurrenceTally
{
    /// How many states the record holds for the scope. Zero when it holds none, which is the
    /// state a record is in the first time a scope is censused.
    pub recorded_states: u32,
    /// The earliest state that observed the lineage, or zero when none did.
    pub first_state: u32,
    /// The latest state that observed the lineage, or zero when none did.
    pub last_state: u32,
    /// How many states observed the lineage.
    pub observed_states: u32,
}

impl OccurrenceTally
{
    /// What these counts establish about the lineage's continuity, or `None` when they establish
    /// nothing.
    ///
    /// Three answers and a refusal, and the refusal is the one that matters most:
    ///
    /// - [`IdentityTransitionKind::ExactContinuity`] when every recorded state observed the
    ///   lineage. There is no state in the record where it was not there.
    /// - [`IdentityTransitionKind::Recreated`] when a state *after* the lineage first appeared
    ///   did not observe it. Something with this identity was there, went away, and is being
    ///   asked about again — which is that variant's own wording, and
    ///   [`IdentityTransitionKind::Is_History_Preserving`] already decides that a tolerance must
    ///   not survive it.
    /// - `None` when the record holds fewer than [`STATES_A_CLAIM_NEEDS`] states for the scope,
    ///   or observed the lineage in none of them, or observed it from some state onward and
    ///   never missed it since. Each of those is a different way of having no evidence: too few
    ///   states to have an interval at all, nothing seen, or a lineage that arrived inside the
    ///   window, which the record cannot tell from the one adopted.
    ///
    /// `None` is deliberately not [`IdentityTransitionKind::Unresolved`]. A caller reporting to a
    /// reader should say `Unresolved`, and that is `OD-GATE-030`'s own floor — *a finding whose
    /// history cannot be established is reported as undetermined rather than as persistent.* But
    /// a caller *deciding* whether a tolerance survives must be able to tell "this link is
    /// unresolved" from "there is no link to resolve", because
    /// [`IdentityTransitionKind::Is_History_Preserving`] answers `false` for `Unresolved` and
    /// answering it for the absence of evidence would decide, by silence, the policy question
    /// `OD-GATE-030` explicitly leaves to a repository.
    ///
    /// # What it never answers
    ///
    /// A recreation is claimed only on an observed return. A lineage the record has never seen
    /// gets `None` and not `Recreated`, even though it is present now and demonstrably was not
    /// at the states behind it — because the identity may have moved rather than the code, which
    /// [`OccurrenceLineageId`](super::OccurrenceLineageId)'s own measurement shows is reachable
    /// for the rules that spell a position into their summary. Missing a real reintroduction is
    /// reported as undetermined; announcing one that did not happen would be a false regression
    /// nobody can act on, and between the two only the first keeps the record worth reading.
    #[must_use]
    pub const fn Established_Continuity(&self) -> Option<IdentityTransitionKind>
    {
        if self.recorded_states < STATES_A_CLAIM_NEEDS || self.observed_states == 0
        {
            return None;
        }

        if self.observed_states == self.recorded_states
        {
            return Some(IdentityTransitionKind::ExactContinuity);
        }

        if self.Was_Missed_After_Arriving()
        {
            return Some(IdentityTransitionKind::Recreated);
        }

        return None;
    }

    /// Whether some state at or after the one that first observed this lineage did not observe
    /// it.
    ///
    /// Two ways that happens and both count: a gap inside the span the lineage was seen across,
    /// and a tail of states after the last that saw it. Checked by arithmetic on the totals
    /// rather than by re-walking states the record has already rolled up.
    const fn Was_Missed_After_Arriving(&self) -> bool
    {
        let span = self.last_state.saturating_sub(self.first_state).saturating_add(1);

        return self.observed_states < span || self.last_state < self.recorded_states;
    }
}

#[cfg(test)]
mod tests
{
    use super::OccurrenceTally;
    use crate::IdentityTransitionKind;

    /// How many states the records below hold, where the number itself decides nothing.
    const A_FEW_STATES: u32 = 4;

    /// An occurrence every state saw is the one case that is continuity.
    #[test]
    fn Test_A_Lineage_Every_State_Saw_Should_Be_Exact_Continuity()
    {
        let seen_throughout =
            OccurrenceTally { recorded_states: A_FEW_STATES, first_state: 1, last_state: A_FEW_STATES, observed_states: A_FEW_STATES };

        assert_eq!(seen_throughout.Established_Continuity(), Some(IdentityTransitionKind::ExactContinuity));
    }

    /// Seen, then missed, then present again: the case a baseline must not go on tolerating.
    #[test]
    fn Test_A_Lineage_Missed_After_Its_Last_Sighting_Should_Be_Recreated()
    {
        let fixed_then_back = OccurrenceTally { recorded_states: A_FEW_STATES, first_state: 1, last_state: 2, observed_states: 2 };

        assert_eq!(fixed_then_back.Established_Continuity(), Some(IdentityTransitionKind::Recreated));
    }

    /// A gap in the middle is a recreation too, and the totals say so without the states.
    #[test]
    fn Test_A_Lineage_With_A_Gap_Inside_Its_Span_Should_Be_Recreated()
    {
        let gapped =
            OccurrenceTally { recorded_states: A_FEW_STATES, first_state: 1, last_state: A_FEW_STATES, observed_states: 3 };

        assert_eq!(gapped.Established_Continuity(), Some(IdentityTransitionKind::Recreated));
    }

    /// A record holding no state for the scope establishes nothing, which is where every scope
    /// starts.
    #[test]
    fn Test_A_Scope_With_No_Recorded_State_Should_Establish_Nothing()
    {
        let unrecorded = OccurrenceTally::default();

        assert_eq!(unrecorded.Established_Continuity(), None);
    }

    /// One state is one sighting and no interval, so "seen in every state on record" is true of
    /// it and says nothing. This is the claim a record makes the very first time a scope is
    /// censused, and it must not be continuity.
    #[test]
    fn Test_A_Single_State_Should_Establish_Nothing_Even_Though_It_Saw_The_Lineage()
    {
        let seen_once = OccurrenceTally { recorded_states: 1, first_state: 1, last_state: 1, observed_states: 1 };

        assert_eq!(
            seen_once.Established_Continuity(),
            None,
            "observed in all one of the states on record is not evidence that anything persisted"
        );
    }

    /// Two states is the least that can carry a claim, and the claim it carries is real: one
    /// interval, crossed without the lineage going away.
    #[test]
    fn Test_Two_States_That_Both_Saw_The_Lineage_Should_Be_Exact_Continuity()
    {
        let across_one_interval = OccurrenceTally { recorded_states: 2, first_state: 1, last_state: 2, observed_states: 2 };

        assert_eq!(across_one_interval.Established_Continuity(), Some(IdentityTransitionKind::ExactContinuity));
    }

    /// A lineage the record has never seen establishes nothing, and specifically is not a
    /// recreation. The identity may have moved rather than the code.
    #[test]
    fn Test_A_Lineage_No_State_Saw_Should_Establish_Nothing()
    {
        let never_seen = OccurrenceTally { recorded_states: A_FEW_STATES, first_state: 0, last_state: 0, observed_states: 0 };

        assert_eq!(never_seen.Established_Continuity(), None, "a first sighting is not a return");
    }

    /// A lineage that arrived inside the window and has been there since establishes nothing
    /// either: it was not there at the start, and nothing says whether it is what was adopted.
    #[test]
    fn Test_A_Lineage_That_Arrived_Inside_The_Window_Should_Establish_Nothing()
    {
        let arrived_late =
            OccurrenceTally { recorded_states: A_FEW_STATES, first_state: 2, last_state: A_FEW_STATES, observed_states: 3 };

        assert_eq!(arrived_late.Established_Continuity(), None);
    }

    /// The two answers this type gives are exactly the two a tolerance is decided by, and they
    /// disagree — which is what makes the decision mean anything.
    #[test]
    fn Test_Continuity_Should_Preserve_History_And_Recreation_Should_Not()
    {
        assert!(IdentityTransitionKind::ExactContinuity.Is_History_Preserving());
        assert!(!IdentityTransitionKind::Recreated.Is_History_Preserving());
    }
}

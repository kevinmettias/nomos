//! Every state one correction run has observed its affected scope in, and whether it has
//! just arrived somewhere it has already been.

use crate::StateSignature;

/// The states an affected scope has been observed in during one correction run, in the
/// order they were observed, and the one question that ordering answers: is the state
/// just observed a state this run has already been in?
///
/// `COR-006`: "The engine shall detect oscillation, divergence, stalls, and repeated
/// states and shall never continue blindly." All four of those are one observation here.
/// Two corrections that undo each other return the scope to a state it was in before, an
/// earlier round holds it, and the run stops. A correction that changes nothing returns
/// the scope to the state of the round immediately before, which is the same answer with
/// a nearer round. Divergence is the case this deliberately does *not* stop: a run that
/// keeps reaching states it has never been in is making progress by the only evidence
/// available here, and stopping it would be a budget rather than a detection.
///
/// **Not an attempt count.** A trail that keeps seeing new states never refuses, however
/// many rounds it runs for; a trail that sees a state twice refuses on the second
/// sighting, however early. The stopping condition is a property of what was observed and
/// nothing else, which is what separates it from a retry limit wearing a detector's
/// clothes -- and a retry limit could not tell a run that is converging slowly from one
/// that is oscillating, which is the distinction that matters.
///
/// What it cannot see is deliberately left visible: it compares whole-scope signatures,
/// so it detects a return to a previous state and not which correction caused it. Naming
/// the responsible pair needs the per-correction attribution `COR-002`'s interaction
/// graph is for, and inventing one here from a digest would be a guess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConvergenceTrail
{
    observed: Vec<StateSignature>,
}

impl ConvergenceTrail
{
    /// A trail that has observed nothing yet.
    #[must_use]
    pub const fn New() -> Self
    {
        return Self {
            observed: Vec::new(),
        };
    }

    /// Records `signature` as this run's next observed state, and reports the earlier
    /// round that already held it -- `None` when this state is new.
    ///
    /// A caller that gets `Some(round)` must stop rather than run another correction:
    /// continuing would carry the scope onward from a state it has already carried it
    /// onward from, and whatever it did the first time is what it would do again.
    ///
    /// The repeated observation is recorded like any other, so [`Self::Rounds`] counts
    /// what was really observed rather than what was acted on.
    pub fn Observe(&mut self, signature: StateSignature) -> Option<usize>
    {
        let earlier = self.observed.iter().position(|seen| return *seen == signature);
        self.observed.push(signature);

        return earlier;
    }

    /// How many states this trail has observed.
    #[must_use]
    pub fn Rounds(&self) -> usize
    {
        return self.observed.len();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const FIRST_ROUND: usize = 0;
    const SECOND_ROUND: usize = 1;
    const ROUNDS_AFTER_TWO_OBSERVATIONS: usize = 2;
    const ROUNDS_AFTER_THREE_OBSERVATIONS: usize = 3;

    fn Signature(observations: &[&str]) -> StateSignature
    {
        let owned: Vec<String> = observations.iter().map(|observation| return (*observation).to_owned()).collect();

        return StateSignature::Of(&owned);
    }

    #[test]
    fn Test_New_Should_Have_Observed_Nothing()
    {
        assert_eq!(ConvergenceTrail::New().Rounds(), 0);
    }

    #[test]
    fn Test_Observe_Should_Report_No_Earlier_Round_For_A_State_It_Has_Not_Seen()
    {
        let mut trail = ConvergenceTrail::New();

        assert_eq!(trail.Observe(Signature(&["a"])), None);
        assert_eq!(trail.Observe(Signature(&["b"])), None);
    }

    /// Oscillation: two corrections that undo each other put the scope back in a state an
    /// earlier round held, and the trail names that round rather than the previous one.
    #[test]
    fn Test_Observe_Should_Name_The_Round_That_Already_Held_The_State()
    {
        let mut trail = ConvergenceTrail::New();
        let _first = trail.Observe(Signature(&["a"]));
        let _second = trail.Observe(Signature(&["b"]));

        assert_eq!(trail.Observe(Signature(&["a"])), Some(FIRST_ROUND));
    }

    /// A stall is the same answer with a nearer round: the state did not move, so the
    /// round immediately before already held it.
    #[test]
    fn Test_Observe_Should_Name_The_Previous_Round_When_Nothing_Changed()
    {
        let mut trail = ConvergenceTrail::New();
        let _first = trail.Observe(Signature(&["a"]));
        let _second = trail.Observe(Signature(&["b"]));

        assert_eq!(trail.Observe(Signature(&["b"])), Some(SECOND_ROUND));
    }

    /// Divergence is not stopped here. Three states, none repeated, and the trail refuses
    /// nothing -- a count-based limit would have refused the third for a reason that is
    /// not about what it observed.
    #[test]
    fn Test_Observe_Should_Not_Refuse_A_Run_That_Keeps_Reaching_New_States()
    {
        let mut trail = ConvergenceTrail::New();

        let answers = [trail.Observe(Signature(&["a"])), trail.Observe(Signature(&["b"])), trail.Observe(Signature(&["c"]))];

        assert_eq!(answers, [None, None, None]);
        assert_eq!(trail.Rounds(), ROUNDS_AFTER_THREE_OBSERVATIONS);
    }

    #[test]
    fn Test_Rounds_Should_Count_A_Repeated_Observation_Too()
    {
        let mut trail = ConvergenceTrail::New();
        let _first = trail.Observe(Signature(&["a"]));
        let _repeat = trail.Observe(Signature(&["a"]));

        assert_eq!(trail.Rounds(), ROUNDS_AFTER_TWO_OBSERVATIONS);
    }
}

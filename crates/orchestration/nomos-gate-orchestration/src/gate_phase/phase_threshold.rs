//! A phase's own numeric threshold -- how many blocking findings a phase tolerates before it
//! fails, apart from the phase that declares it.

/// A phase's own numeric threshold -- `WF-001`'s "thresholds" clause, narrower than a whole
/// run's: a phase judges only the blocking findings its own [`crate::GatePhase::rules`] admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseThreshold
{
    /// Any blocking finding fails this phase -- [`crate::Disposition_Of_Findings`]'s own
    /// rule, narrowed to one phase's own findings rather than a whole run's.
    AnyBlockingFinding,
    /// This phase tolerates up to `max` blocking findings before it fails; `max` itself is
    /// still tolerated.
    MaxBlockingFindings
    {
        max: usize,
    },
}

impl PhaseThreshold
{
    /// Whether `count` blocking findings exceed this threshold.
    ///
    /// Crate-visible rather than private because the phase that owns this threshold is
    /// declared a module away, in [`crate::GatePhase`], and the comparison belongs with the
    /// values it compares rather than beside its caller.
    #[must_use]
    pub(crate) const fn Exceeded_By(&self, count: usize) -> bool
    {
        return match *self
        {
            Self::AnyBlockingFinding => count > 0,
            Self::MaxBlockingFindings { max } => count > max,
        };
    }
}

//! The trait by which a strategy declares what it promises.

use super::{DeterminismStrength, ReproducibilityScope, TraceEquivalence};

/// Every swappable implementation declares its reproducibility properties here.
///
/// All three constants are required. Rust enforces that at compile time — omitting one
/// is an error, not a default — and that is the point: a strategy cannot decline to say
/// what it promises, so there is no silent middle state between "claims determinism"
/// and "does not".
///
/// A declaration is a **testable claim**, not documentation. The verification a
/// strategy owes follows mechanically from what it declared:
/// [`ReproducibilityScope::CrossRun`] owes a two-process comparison,
/// [`ReproducibilityScope::CrossPlatform`] owes reference capture on one platform and
/// comparison from the others, and a declared/observed mismatch is a build failure
/// resolved by either fixing the implementation or lowering the declaration — never by
/// marking the test flaky.
pub trait Strategy
{
    /// What guarantee the strategy makes about reproducing its outputs.
    const STRENGTH: DeterminismStrength;
    /// Across what environment the determinism guarantee is valid.
    const SCOPE: ReproducibilityScope;
    /// What constitutes "the same trace" for this strategy.
    const TRACE: TraceEquivalence;
}

/// Whether a declaration triple is internally coherent.
///
/// One cross-axis rule binds the three constants: a trace claim and a strength claim
/// must agree about whether reproducibility is being promised at all.
/// [`TraceEquivalence::NotApplicable`] is honest only when the strength is
/// [`DeterminismStrength::None`], and a strategy claiming `State` or better must say
/// what "the same" means or its claim cannot be checked.
///
/// This is a free function rather than a trait method so it can be applied to a triple
/// read off the wire — a peer's declaration, or one loaded from a package manifest —
/// and not only to one a Rust type declared.
#[must_use]
pub const fn Declaration_Is_Coherent(
    strength: DeterminismStrength,
    trace: TraceEquivalence,
) -> bool
{
    return match trace
    {
        TraceEquivalence::NotApplicable => !strength.Can_Claim_Reproducibility(),
        TraceEquivalence::BehaviorallyEquivalent | TraceEquivalence::BitIdentical =>
        {
            strength.Can_Claim_Reproducibility()
        }
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The analysis kernel's own declaration, as a compile-time example and a
    /// reminder of what the strongest row in the domain table costs.
    struct AnalysisKernel;

    impl Strategy for AnalysisKernel
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    /// The agent host's declaration. Honest, and deliberately the weakest possible.
    struct AgentHost;

    impl Strategy for AgentHost
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::None;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
    }

    #[test]
    fn Test_Declaration_Is_Coherent_Should_Accept_The_Reference_Declarations()
    {
        assert!(Declaration_Is_Coherent(
            AnalysisKernel::STRENGTH,
            AnalysisKernel::TRACE
        ));
        assert!(Declaration_Is_Coherent(
            AgentHost::STRENGTH,
            AgentHost::TRACE
        ));
    }

    /// A strategy that promises reproducible state but declines to define sameness has
    /// made an unverifiable claim, which is the shape of claim this whole triple exists
    /// to refuse.
    #[test]
    fn Test_Declaration_Is_Coherent_Should_Require_A_Trace_Definition_For_Reproducible_Strength()
    {
        assert!(!Declaration_Is_Coherent(
            DeterminismStrength::State,
            TraceEquivalence::NotApplicable
        ));
        assert!(!Declaration_Is_Coherent(
            DeterminismStrength::StateTemporal,
            TraceEquivalence::NotApplicable
        ));
    }

    /// The inverse, and the more tempting mistake: claiming bit-identical output while
    /// declaring no determinism at all. It reads as rigour and means nothing.
    #[test]
    fn Test_Declaration_Is_Coherent_Should_Refuse_A_Trace_Claim_With_No_Strength()
    {
        assert!(!Declaration_Is_Coherent(
            DeterminismStrength::None,
            TraceEquivalence::BitIdentical
        ));
        assert!(!Declaration_Is_Coherent(
            DeterminismStrength::None,
            TraceEquivalence::BehaviorallyEquivalent
        ));
    }
}

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

/// A reference to a strategy promises exactly what the strategy promises.
///
/// Without this, a type implementing a port for `&Self` — which several of this
/// workspace's test doubles do, because the port is taken by shared reference and the
/// double holds interior state — would have to restate all three constants, and the
/// restatement is a second place for them to drift. There is no judgment in the
/// forwarding: indirection is not a property a determinism claim is about.
impl<Referent: Strategy + ?Sized> Strategy for &Referent
{
    const STRENGTH: DeterminismStrength = Referent::STRENGTH;
    const SCOPE: ReproducibilityScope = Referent::SCOPE;
    const TRACE: TraceEquivalence = Referent::TRACE;
}

/// Whether a declaration triple is internally coherent.
///
/// One cross-axis rule binds all three constants: they must agree about whether
/// reproducibility is being promised at all. [`DeterminismStrength::None`] *is* the
/// declaration that no claim is made, and the other two axes describe a claim, so when
/// there is none they have nothing to describe.
///
/// - [`DeterminismStrength::None`] requires [`ReproducibilityScope::SingleRun`] and
///   [`TraceEquivalence::NotApplicable`].
/// - `State` or better requires a real trace claim, or the claim cannot be checked. Any
///   scope is admissible: a guarantee holding only within one run is narrow, not
///   incoherent.
///
/// # Why the scope axis is part of this
///
/// It was not, and the omission had a measurable cost. The rule bound strength to trace
/// and left scope alone, so a declaration naming an environment for a guarantee it had
/// just declined to make passed. A scan of the sibling XVPE workspace's 742 strategy
/// declarations found that shape three times, in declarations that were also wrong on
/// the trace axis: the narrower rule would have caught those three by accident, and a
/// scope-only violation not at all.
///
/// This is a free function rather than a trait method so it can be applied to a triple
/// read off the wire — a peer's declaration, or one loaded from a package manifest —
/// and not only to one a Rust type declared.
#[must_use]
pub const fn Declaration_Is_Coherent(
    strength: DeterminismStrength,
    scope: ReproducibilityScope,
    trace: TraceEquivalence,
) -> bool
{
    if !strength.Can_Claim_Reproducibility()
    {
        return matches!(scope, ReproducibilityScope::SingleRun)
            && matches!(trace, TraceEquivalence::NotApplicable);
    }

    return !matches!(trace, TraceEquivalence::NotApplicable);
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
            AnalysisKernel::SCOPE,
            AnalysisKernel::TRACE
        ));
        assert!(Declaration_Is_Coherent(
            AgentHost::STRENGTH,
            AgentHost::SCOPE,
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
            ReproducibilityScope::CrossRun,
            TraceEquivalence::NotApplicable
        ));
        assert!(!Declaration_Is_Coherent(
            DeterminismStrength::StateTemporal,
            ReproducibilityScope::CrossPlatform,
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
            ReproducibilityScope::SingleRun,
            TraceEquivalence::BitIdentical
        ));
        assert!(!Declaration_Is_Coherent(
            DeterminismStrength::None,
            ReproducibilityScope::SingleRun,
            TraceEquivalence::BehaviorallyEquivalent
        ));
    }

    /// The axis the rule did not used to cover: naming an environment for a guarantee
    /// that was never given. Found three times in the sibling workspace, which is why
    /// the rule grew rather than this case being hypothetical.
    #[test]
    fn Test_Declaration_Is_Coherent_Should_Refuse_A_Scope_With_No_Strength()
    {
        assert!(!Declaration_Is_Coherent(
            DeterminismStrength::None,
            ReproducibilityScope::CrossPlatform,
            TraceEquivalence::NotApplicable
        ));
        assert!(!Declaration_Is_Coherent(
            DeterminismStrength::None,
            ReproducibilityScope::CrossBinary,
            TraceEquivalence::NotApplicable
        ));
    }
}

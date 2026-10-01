//! The determinism vocabulary is defined twice on purpose. This is what keeps the two
//! definitions from meaning different things.
//!
//! `nomos-contracts` may depend on `serde` and nothing else — `boundaries/graph.rs`
//! asserts it — because every type in that crate is reimplemented by peers that will
//! never compile it. So `DeterminismStrength`, `ReproducibilityScope`, `TraceEquivalence`
//! and `Declaration_Is_Coherent` cannot simply be re-exported from
//! `xvpe_primitives::strategy`, where the same four things already live. The duplication
//! is the architecture working, not a defect.
//!
//! What *is* a defect is two copies that drift, and they had already begun to. Measured
//! 2026-09-11:
//!
//! - `TraceEquivalence` is declared in the **opposite order** in the two workspaces.
//!   Nomos derives `Ord` and takes its meaning of "stronger" from declaration order;
//!   XVPE derives no `Ord` and takes it from an explicit `Rank()`. The two agree today
//!   only because `Rank()` inverts XVPE's own declaration order. Nothing on either side
//!   would have noticed if it stopped agreeing.
//! - `Declaration_Is_Coherent` existed only here, and bound only two of the three axes.
//!   A scan of XVPE's 742 strategy declarations found three that named an environment
//!   for a guarantee they had declined to make — the axis this rule did not cover, and
//!   which nothing over there covered either. Both rules now bind it. They still differ
//!   on the trace axis, deliberately; see
//!   [`Test_Nomos_Coherence_Should_Be_A_Strict_Refinement_Of_Xvpe_Coherence`].
//!
//! The module doc on `nomos_contracts::determinism` says a divergent second spelling
//! would be worse than no spelling at all. This file is what makes that sentence
//! enforceable rather than aspirational.
//!
//! # Why the mappings are exhaustive matches
//!
//! Each direction is a `match` over every variant of one side, returning the other's.
//! A variant added to either enum, in either workspace, stops this file compiling —
//! which is the only guard that does not need someone to remember this file exists.

use nomos_contracts::{
    Declaration_Is_Coherent, DeterminismStrength, ReproducibilityScope, TraceEquivalence,
};

/// Every strength this workspace declares. Extended beside [`Xvpe_Strength`], which
/// stops compiling when a variant is added.
const ALL_STRENGTHS: &[DeterminismStrength] = &[
    DeterminismStrength::None,
    DeterminismStrength::State,
    DeterminismStrength::StateTemporal,
];

const ALL_SCOPES: &[ReproducibilityScope] = &[
    ReproducibilityScope::SingleRun,
    ReproducibilityScope::CrossRun,
    ReproducibilityScope::CrossPlatform,
    ReproducibilityScope::CrossBinary,
];

const ALL_TRACES: &[TraceEquivalence] = &[
    TraceEquivalence::NotApplicable,
    TraceEquivalence::BehaviorallyEquivalent,
    TraceEquivalence::BitIdentical,
];

fn Xvpe_Strength(strength: DeterminismStrength) -> xvpe_primitives::DeterminismStrength
{
    return match strength
    {
        DeterminismStrength::None => xvpe_primitives::DeterminismStrength::None,
        DeterminismStrength::State => xvpe_primitives::DeterminismStrength::State,
        DeterminismStrength::StateTemporal => xvpe_primitives::DeterminismStrength::StateTemporal,
    };
}

fn Nomos_Strength(strength: xvpe_primitives::DeterminismStrength) -> DeterminismStrength
{
    return match strength
    {
        xvpe_primitives::DeterminismStrength::None => DeterminismStrength::None,
        xvpe_primitives::DeterminismStrength::State => DeterminismStrength::State,
        xvpe_primitives::DeterminismStrength::StateTemporal => DeterminismStrength::StateTemporal,
    };
}

fn Xvpe_Scope(scope: ReproducibilityScope) -> xvpe_primitives::ReproducibilityScope
{
    return match scope
    {
        ReproducibilityScope::SingleRun => xvpe_primitives::ReproducibilityScope::SingleRun,
        ReproducibilityScope::CrossRun => xvpe_primitives::ReproducibilityScope::CrossRun,
        ReproducibilityScope::CrossPlatform => xvpe_primitives::ReproducibilityScope::CrossPlatform,
        ReproducibilityScope::CrossBinary => xvpe_primitives::ReproducibilityScope::CrossBinary,
    };
}

fn Nomos_Scope(scope: xvpe_primitives::ReproducibilityScope) -> ReproducibilityScope
{
    return match scope
    {
        xvpe_primitives::ReproducibilityScope::SingleRun => ReproducibilityScope::SingleRun,
        xvpe_primitives::ReproducibilityScope::CrossRun => ReproducibilityScope::CrossRun,
        xvpe_primitives::ReproducibilityScope::CrossPlatform => ReproducibilityScope::CrossPlatform,
        xvpe_primitives::ReproducibilityScope::CrossBinary => ReproducibilityScope::CrossBinary,
    };
}

fn Xvpe_Trace(trace: TraceEquivalence) -> xvpe_primitives::TraceEquivalence
{
    return match trace
    {
        TraceEquivalence::NotApplicable => xvpe_primitives::TraceEquivalence::NotApplicable,
        TraceEquivalence::BehaviorallyEquivalent =>
        {
            xvpe_primitives::TraceEquivalence::BehaviorallyEquivalent
        }
        TraceEquivalence::BitIdentical => xvpe_primitives::TraceEquivalence::BitIdentical,
    };
}

fn Nomos_Trace(trace: xvpe_primitives::TraceEquivalence) -> TraceEquivalence
{
    return match trace
    {
        xvpe_primitives::TraceEquivalence::NotApplicable => TraceEquivalence::NotApplicable,
        xvpe_primitives::TraceEquivalence::BehaviorallyEquivalent =>
        {
            TraceEquivalence::BehaviorallyEquivalent
        }
        xvpe_primitives::TraceEquivalence::BitIdentical => TraceEquivalence::BitIdentical,
    };
}

/// The guard in front of every other assertion here.
///
/// Each list below is walked by the tests that follow. A list that lost an entry would
/// leave them passing over a smaller universe, which is the failure this whole suite
/// exists to refuse elsewhere. The counts are literal because a variant added on either
/// side already breaks the mappings above; this catches the case where the mapping was
/// extended and the list beside it was not.
#[test]
fn Test_Every_Variant_Should_Be_Under_Test()
{
    assert_eq!(ALL_STRENGTHS.len(), 3, "a strength variant was added; extend ALL_STRENGTHS");
    assert_eq!(ALL_SCOPES.len(), 4, "a scope variant was added; extend ALL_SCOPES");
    assert_eq!(ALL_TRACES.len(), 3, "a trace variant was added; extend ALL_TRACES");

    for strength in ALL_STRENGTHS
    {
        assert_eq!(Nomos_Strength(Xvpe_Strength(*strength)), *strength);
    }
    for scope in ALL_SCOPES
    {
        assert_eq!(Nomos_Scope(Xvpe_Scope(*scope)), *scope);
    }
    for trace in ALL_TRACES
    {
        assert_eq!(Nomos_Trace(Xvpe_Trace(*trace)), *trace);
    }
}

/// The spelling a peer reads.
///
/// These labels reach a peer as the variant names in the JSON Schema generated from
/// these declarations, and they reach an XVPE diagnostic through `Display`. A rename on
/// one side alone would be a silent protocol break in one direction and a confusing
/// log line in the other.
#[test]
fn Test_Both_Workspaces_Should_Spell_Every_Variant_Identically()
{
    for strength in ALL_STRENGTHS
    {
        assert_eq!(strength.Label(), Xvpe_Strength(*strength).Label());
    }
    for scope in ALL_SCOPES
    {
        assert_eq!(scope.Label(), Xvpe_Scope(*scope).Label());
    }
    for trace in ALL_TRACES
    {
        assert_eq!(trace.Label(), Xvpe_Trace(*trace).Label());
    }
}

/// The one that would have caught the drift already present.
///
/// Nomos orders these by declaration order through a derived `Ord`; XVPE orders them by
/// an explicit `Rank()` that its own declaration order does not follow. For
/// `TraceEquivalence` the two declaration orders are literal opposites, so this holding
/// is a property of `Rank()` and not of either enum's shape. Reordering a variant here,
/// or renumbering one there, is exactly the edit that looks harmless and is not.
#[test]
fn Test_Both_Workspaces_Should_Order_Every_Axis_Identically()
{
    for left in ALL_STRENGTHS
    {
        for right in ALL_STRENGTHS
        {
            assert_eq!(
                left >= right,
                Xvpe_Strength(*left).Meets(Xvpe_Strength(*right)),
                "strength order disagrees at {left:?} vs {right:?}"
            );
        }
    }

    for left in ALL_SCOPES
    {
        for right in ALL_SCOPES
        {
            assert_eq!(
                left >= right,
                Xvpe_Scope(*left).Meets(Xvpe_Scope(*right)),
                "scope order disagrees at {left:?} vs {right:?}"
            );
        }
    }

    for left in ALL_TRACES
    {
        for right in ALL_TRACES
        {
            assert_eq!(
                left >= right,
                Xvpe_Trace(*left).Meets(Xvpe_Trace(*right)),
                "trace order disagrees at {left:?} vs {right:?}"
            );
        }
    }
}

/// Every declaration triple the three axes can spell, flattened.
///
/// The comparison below is about one triple at a time, and writing it as three nested
/// loops put its body four levels deep — far enough that `nesting-depth` reports it, and
/// it is right to: the assertion is the point, and the iteration is bookkeeping.
fn Every_Triple() -> Vec<(DeterminismStrength, ReproducibilityScope, TraceEquivalence)>
{
    let mut triples = Vec::new();

    for strength in ALL_STRENGTHS
    {
        for scope in ALL_SCOPES
        {
            for trace in ALL_TRACES
            {
                triples.push((*strength, *scope, *trace));
            }
        }
    }

    return triples;
}

/// The rule itself, over its whole domain — including where the two deliberately differ.
///
/// Thirty-six triples is the entire input space, so this is exhaustive rather than a
/// sample.
///
/// The two rules are **not** the same rule, and pretending otherwise would be the more
/// comfortable lie. Both refuse a scope without a guarantee to scope, and both refuse a
/// guarantee with no notion of sameness. Nomos additionally refuses a trace claim from a
/// declaration that promises no reproducibility; XVPE admits it.
///
/// That divergence is correct in both directions and follows from what a trace claim is
/// about in each place. XVPE's `TRACE` names how two strategies *behind one surface* are
/// differentially compared, and a member can reproduce nothing while still owing its
/// sibling a comparison — a GPU pass against its CPU reference, a pooled executor against
/// a synchronous one. A [`nomos_contracts::WorkflowStep`] has no sibling: the only thing
/// its trace claim can be about is the step compared with another run of itself, and a
/// step that reproduces nothing has nothing to compare.
///
/// So Nomos's rule is a strict refinement of XVPE's, and this pins exactly that: every
/// declaration Nomos admits, XVPE admits, and the two shapes XVPE admits beyond it are
/// named. A change to either rule moves that set and fails here.
#[test]
fn Test_Nomos_Coherence_Should_Be_A_Strict_Refinement_Of_Xvpe_Coherence()
{
    let mut here_admits = 0_usize;
    let mut there_admits = 0_usize;
    let mut admitted_only_by_xvpe = Vec::new();

    for (strength, scope, trace) in Every_Triple()
    {
        let here = Declaration_Is_Coherent(strength, scope, trace);
        let there = xvpe_primitives::Declaration_Is_Coherent(
            Xvpe_Strength(strength),
            Xvpe_Scope(scope),
            Xvpe_Trace(trace),
        );

        assert!(
            !here || there,
            "nomos admits {strength:?} / {scope:?} / {trace:?} and xvpe refuses it. Nomos is              meant to be the stricter of the two; a declaration accepted here and rejected              there is a divergence in the other direction, which nothing has reasoned about."
        );

        here_admits += usize::from(here);
        there_admits += usize::from(there);

        if there && !here
        {
            admitted_only_by_xvpe.push((strength, scope, trace));
        }
    }

    // Both rules have to bite, or agreement is agreement about nothing.
    assert_eq!(here_admits, 17, "nomos admitted {here_admits} of 36 triples");
    assert_eq!(there_admits, 19, "xvpe admitted {there_admits} of 36 triples");

    // A member that reproduces nothing, still naming how it compares with its siblings.
    assert_eq!(
        admitted_only_by_xvpe,
        vec![
            (
                DeterminismStrength::None,
                ReproducibilityScope::SingleRun,
                TraceEquivalence::BehaviorallyEquivalent
            ),
            (
                DeterminismStrength::None,
                ReproducibilityScope::SingleRun,
                TraceEquivalence::BitIdentical
            ),
        ],
        "the two rules now differ on a different set than the one that was reasoned about"
    );
}

/// The serialized spelling, which is the only form a peer ever sees.
///
/// Nomos derives `Serialize` on these enums and XVPE does not — it is a `no_std`,
/// dependency-free crate by charter. So the wire form is this workspace's own, and what
/// keeps it tied to the shared vocabulary is that it must equal the label both sides
/// agree on above.
#[test]
fn Test_The_Serialized_Form_Should_Be_The_Shared_Label()
{
    for strength in ALL_STRENGTHS
    {
        let written = serde_json::to_string(strength).expect("a strength serializes");
        assert_eq!(written, format!("\"{}\"", strength.Label()));
    }
    for scope in ALL_SCOPES
    {
        let written = serde_json::to_string(scope).expect("a scope serializes");
        assert_eq!(written, format!("\"{}\"", scope.Label()));
    }
    for trace in ALL_TRACES
    {
        let written = serde_json::to_string(trace).expect("a trace serializes");
        assert_eq!(written, format!("\"{}\"", trace.Label()));
    }
}

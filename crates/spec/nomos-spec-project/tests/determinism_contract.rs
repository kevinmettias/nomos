//! The seam between `nomos_spec_project` and `nomos_contracts`, driven through this crate's
//! own public API.
//!
//! `ProjectionOutput` is this crate's declaration of which row of
//! `nomos_contracts::Strategy`'s domain table the projection engine occupies. Nothing under
//! this crate's own `tests/` checked that declaration against the contract itself before
//! this file — `tests/integration/tests/determinism/declarations.rs` at the workspace root
//! exercises it too, alongside every other domain in the tree, but that is a different
//! crate and does not stand in for this one certifying its own contract.

use nomos_contracts::{Declaration_Is_Coherent, DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_spec_project::ProjectionOutput;

/// The happy path: the declared triple is exactly the row this crate's own module doc
/// commits to — `State` strength, `CrossPlatform` scope, `BitIdentical` trace.
#[test]
fn Test_Projection_Output_Should_Declare_The_Cross_Platform_State_Row()
{
    assert_eq!(ProjectionOutput::STRENGTH, DeterminismStrength::State);
    assert_eq!(ProjectionOutput::SCOPE, ReproducibilityScope::CrossPlatform);
    assert_eq!(ProjectionOutput::TRACE, TraceEquivalence::BitIdentical);
}

/// The cross-axis rule `nomos_contracts::Strategy`'s own module names: a strategy claiming
/// `State` or better must define what "the same" means, or the claim cannot be checked.
/// Checked here through `nomos_contracts`'s own coherence rule rather than reimplemented by
/// hand, so a future change to that rule is caught here too.
#[test]
fn Test_The_Declared_Triple_Should_Be_Coherent_Under_Nomos_Contracts_Own_Rule()
{
    assert!(Declaration_Is_Coherent(
        ProjectionOutput::STRENGTH,
        ProjectionOutput::TRACE
    ));
}

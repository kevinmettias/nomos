//! The two controls, which are what make every assertion in [`domains`] worth anything.
//!
//! A suite that only ever watches correct code pass proves the code is a function of its
//! input and proves nothing about the instrument. These run the instrument over something
//! that is *not*, and assert it fails.
//!
//! [`domains`]: crate::domains

use crate::spec_productions::{Bundle_Bytes, Order};
use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_integration_tests::{Production, Verify};

/// A declaration with the bundle row's triple, over a domain that does not hold it.
///
/// Held here rather than in a crate because it is not a domain: it is the negative control
/// for the harness, and a strategy declared in a `src/` directory would be found by
/// `Test_Every_Declaration_Should_Be_Held_To_It_By_The_Harness` and correctly reported as a
/// promise with nothing behind it.
struct Wobbly;

impl Strategy for Wobbly
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossBinary;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

/// The control that says the harness would catch a real violation.
///
/// Without it every assertion in [`crate::domains`] is consistent with a [`Verify`] that
/// compares nothing: a domain repeated eight times and found to agree proves the domain is a
/// function of its input, and proves nothing whatever about the instrument. So this runs the
/// instrument over a production that is *not* a function of its input and asserts it fails,
/// with the message it fails by, at the declared strength.
///
/// The variation is a counter rather than a hash seed or a clock, because a control has to
/// fail on every machine on every run — a control that is itself flaky is a control nobody
/// believes when it goes green.
#[test]
fn Test_A_Domain_That_Does_Not_Repeat_Itself_Should_Fail_The_Harness()
{
    use std::cell::Cell;

    let call = Cell::new(0_u32);
    let wobbles = || {
        let seen = call.get();
        call.set(seen.saturating_add(1));

        return format!("line\tone\nline\t{seen}\n").into_bytes();
    };
    let refusal = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        return Verify::<Wobbly>("wobbly", &wobbles);
    }))
    .expect_err("a production that changes between repetitions must fail the harness");
    let said = refusal
        .downcast_ref::<String>()
        .map_or_else(String::new, Clone::clone);
    assert!(
        said.contains("produced a different set"),
        "the harness failed for the wrong reason: {said}"
    );
    assert!(
        call.get() > 1,
        "the control never reached a second repetition, so it proved nothing"
    );
}

/// The control for the golden, which is the half of a scope claim a repetition cannot make.
///
/// A committed digest only catches a changed encoding if it is a function of the bytes.
/// Both spec domains declare `State`, and a `State` digest is taken over the *sorted* line
/// set — so the reasonable worry is that it is insensitive to something it ought to catch.
/// This shows it is not: one altered byte anywhere in a real bundle moves the digest the
/// golden is compared against.
#[test]
fn Test_An_Altered_Byte_Should_Move_The_Digest_The_Golden_Pins()
{
    use crate::goldens::BUNDLE_GOLDEN;

    let honest = Production {
        trace: Bundle_Bytes(Order::Forwards),
    };
    let mut altered = honest.trace.clone();
    let last = altered
        .iter()
        .rposition(|byte| return byte.is_ascii_lowercase())
        .expect("a bundle carries lower-case text");
    let target = altered
        .get_mut(last)
        .expect("the position just found is in range");
    *target = target.to_ascii_uppercase();
    let tampered = Production { trace: altered };

    assert_ne!(
        tampered.trace, honest.trace,
        "the control altered nothing, so the comparison below cannot fail"
    );
    assert_ne!(
        tampered.Digest_At(DeterminismStrength::State),
        honest.Digest_At(DeterminismStrength::State),
        "a changed byte left the digest where it was, so the golden pins nothing"
    );
    assert_eq!(
        honest.Digest_At(DeterminismStrength::State),
        BUNDLE_GOLDEN,
        "the honest half of this control must be the bundle the golden was captured from"
    );
}

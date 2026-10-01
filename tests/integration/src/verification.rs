//! Checking a declared [`Strategy`] against what the declaring domain actually does.
//!
//! The mapping from declaration to obligation is code rather than prose, so a domain gets
//! the checks its own declaration implies rather than the checks its author remembered.
//! `crate::determinism` carries the table of which triple obliges what.

// What a strength claim owes before it can be stood behind: a floor the provider must
// clear, what the domain actually produced, and what the claim owes beyond one process.
mod cross_environment;
mod floors;
mod production;

pub use cross_environment::{CrossEnvironment, Cross_Environment_Owed};
pub use floors::{Approximate_Floor, Parsed_Floor};
pub use production::Production;

use nomos_contracts::{
    Declaration_Is_Coherent, DeterminismStrength, Strategy, TraceEquivalence,
};

/// How many times a production is repeated in one process.
///
/// Small on purpose. Repetition in a single process catches an unordered collection
/// reaching the output and little else, because everything that varies per *process* —
/// hash seeds, address layout, environment — is fixed for the whole of it. The scope axis
/// is what buys the rest, and it is the axis worth spending time on.
const REPEATS: usize = 8;

/// What a declaration obliged, and what was observed.
///
/// Returned rather than only asserted so a caller can print it. A run that says which
/// obligations it discharged is the difference between this and a green tick over
/// checks that were never selected.
#[derive(Clone, Debug)]
pub struct Verification
{
    /// The domain's name, for diagnostics.
    pub domain: String,
    /// The obligations this declaration implied, in the order checked.
    pub discharged: Vec<String>,
    /// The digest every repetition agreed on.
    pub digest: String,
}

/// Checks a domain against its own declaration, in one process.
///
/// The cross-environment half of a scope claim cannot be discharged from inside one
/// process and is not attempted here; [`Verification::digest`] is what the caller compares
/// against a child process and against a committed golden. [`crate::Cross_Environment_Owed`]
/// says which of those two the declaration requires, so that a caller cannot quietly
/// discharge fewer obligations than the declaration named.
///
/// # Panics
///
/// Panics when the declaration is incoherent, or when repeated production disagrees at the
/// declared strength. Both are the failure this exists to produce: a declared/observed
/// mismatch is a build failure resolved by fixing the implementation or lowering the
/// declaration, never by marking the test flaky.
#[must_use]
// `Fn` is the minimal bound here and not merely the habitual one (waived in suppressions.json,
// since check-closure-bounds is not one of the safety-critical checks an in-code marker
// still argues under this repository's safety-only policy). The producer is invoked once per
// repetition through a shared reference, so `FnOnce` cannot express it; `FnMut` could, and is
// refused deliberately, because a producer carrying mutable state between repetitions is
// exactly the thing this verifier exists to rule out.
pub fn Verify<S: Strategy>(domain: &str, produce: &dyn Fn() -> Vec<u8>) -> Verification
{
    Assert_The_Declaration_Is_Coherent::<S>(domain);

    let first = Production { trace: produce() };

    // The vacuity guard. A production that yields nothing agrees with itself every time,
    // and every assertion below would hold over it — which is the shape of check this
    // workspace exists to refuse.
    assert!(
        !first.trace.is_empty(),
        "{domain} produced no bytes, so repeating it asserts nothing"
    );
    Assert_Repeats_Agree::<S>(domain, &first, produce);

    return Verification {
        domain: domain.to_owned(),
        discharged: Discharged::<S>(),
        digest: first.Digest_At(S::STRENGTH),
    };
}

/// A triple whose three axes cannot all be true is a declaration nothing can discharge.
fn Assert_The_Declaration_Is_Coherent<S: Strategy>(domain: &str)
{
    assert!(
        Declaration_Is_Coherent(S::STRENGTH, S::SCOPE, S::TRACE),
        "{domain} declares {} across {} with a trace of {}, which is not a coherent claim",
        S::STRENGTH,
        S::SCOPE,
        S::TRACE
    );
}

/// Every repetition, compared against the first production at the declared strength.
fn Assert_Repeats_Agree<S: Strategy>(
    domain: &str,
    first: &Production,
    // Already erased at `Verify`'s boundary, so there is no concrete closure type left to
    // name here. A type parameter would be instantiated at `dyn Fn` and buy nothing, and the
    // one indirect call it would remove sits in front of a whole production.
    produce: &dyn Fn() -> Vec<u8>,
)
{
    for repeat in 1..REPEATS
    {
        let again = Production { trace: produce() };

        Assert_Agrees::<S>(domain, first, &again, repeat);
    }
}

/// What the run discharged, which is what it is entitled to report having checked.
///
/// `BitIdentical` says what "the same" means when two runs are compared: equality, with no
/// tolerance to define. It does not add a comparison of its own — the strength axis decides
/// *what* is compared and this axis decides *how*, which is why a domain can declare `State`
/// and `BitIdentical` together without contradiction. Recorded rather than checked, and said
/// out loud because the tempting misreading is that `BitIdentical` alone obliges byte-stable
/// repetition.
fn Discharged<S: Strategy>() -> Vec<String>
{
    let mut discharged = vec!["coherent".to_owned()];
    if S::STRENGTH != DeterminismStrength::None
    {
        discharged.push(format!("repeated x{REPEATS} at {}", S::STRENGTH));
    }
    if S::TRACE == TraceEquivalence::BitIdentical
    {
        discharged.push(format!("trace {}", TraceEquivalence::BitIdentical));
    }

    return discharged;
}

/// Two productions compared at whatever the declared strength says "the same" means.
///
/// `None` promises nothing, so nothing is checked. Coherence has already refused the case
/// where it pairs with a trace claim.
fn Assert_Agrees<S: Strategy>(domain: &str, first: &Production, again: &Production, repeat: usize)
{
    match S::STRENGTH
    {
        DeterminismStrength::StateTemporal => assert!(
            again.trace == first.trace,
            "{domain} declares {} and repetition {repeat} produced different bytes",
            DeterminismStrength::StateTemporal
        ),
        DeterminismStrength::State => assert!(
            again.State() == first.State(),
            "{domain} declares {} and repetition {repeat} produced a different set",
            DeterminismStrength::State
        ),
        DeterminismStrength::None =>
        {}
    }
}

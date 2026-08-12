//! Checking a declared [`Strategy`] against what the declaring domain actually does.
//!
//! # Why this is a harness and not four hand-written tests
//!
//! A declaration is a testable claim, and the verification it owes follows mechanically
//! from the triple it declared — `nomos_contracts::determinism` says so in prose, and
//! saying it in prose is what left four domains declaring nothing for as long as they
//! did. So the mapping from declaration to obligation is code here, and a domain gets the
//! checks its own declaration implies rather than the checks its author remembered:
//!
//! | Declared | Obligation discharged here |
//! |---|---|
//! | any triple | the triple is internally coherent |
//! | `STRENGTH: StateTemporal` | repeated production is byte-identical, order included |
//! | `STRENGTH: State` | repeated production agrees as a set, order not promised |
//! | `STRENGTH: None` | nothing — and `TRACE` must then be `NotApplicable` |
//! | `SCOPE` above `SingleRun` | a second process reaches the same bytes |
//! | `SCOPE` at `CrossPlatform` or above | the bytes match a digest committed to the tree |
//!
//! Raising a declaration therefore raises what is checked, with no second edit. Lowering
//! one is the sanctioned way to make a failing domain honest, and it is visible in a diff
//! as a weakened promise rather than as a deleted test.
//!
//! # Why the fixtures are in this repository
//!
//! `nomos-lang-rust/tests/corpus.rs` already asserts reading determinism over 7,500 real
//! files, and `nomos-workspace/tests/portable.rs` asserts snapshot identity over a hundred
//! ingestion orders of that corpus. Both are stronger instruments than anything here and
//! both are corpus-gated, so on every machine without `NOMOS_RUST_CORPUS` they report `ok`
//! having read nothing — which is every machine the gate runs on. `docs/records/OD-GATE-001`
//! measures that hole at 68 assertions.
//!
//! A declaration whose only proof is inside that hole is a declaration CI has never seen
//! tested. So these fixtures are small, sufficient and here.

use nomos_contracts::{
    Declaration_Is_Coherent, DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence,
};
use nomos_model::{Content_Digest, Digest_Of_Parts};

/// How many times a production is repeated in one process.
///
/// Small on purpose. Repetition in a single process catches an unordered collection
/// reaching the output and little else, because everything that varies per *process* —
/// hash seeds, address layout, environment — is fixed for the whole of it. The scope axis
/// is what buys the rest, and it is the axis worth spending time on.
const REPEATS: usize = 8;

/// The environment variable by which a parent hands a child the domain to report on.
///
/// Assembled rather than written, for the reason `tests/contract/src/gates.rs` assembles
/// the corpus variable names: a file that contains the spelling of a variable it also
/// scans for is a file that finds itself.
#[must_use]
pub fn Child_Variable() -> String
{
    return concat!("NOMOS_", "DETERMINISM_", "CHILD").to_owned();
}

/// What one domain produced, in the two projections a strength claim can be about.
///
/// Every payload this workspace produces is line-oriented, and deliberately so — the
/// encodings are hand-written precisely so that a derive cannot re-address them. That
/// makes the set projection well defined without any domain having to supply one: the
/// lines, sorted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Production
{
    /// The bytes as produced, in the order produced.
    pub trace: Vec<u8>,
}

impl Production
{
    /// The bytes, with the sequence discarded.
    ///
    /// What `DeterminismStrength::State` promises and `StateTemporal` promises on top of.
    #[must_use]
    pub fn State(&self) -> Vec<Vec<u8>>
    {
        let mut lines: Vec<Vec<u8>> = self
            .trace
            .split(|byte| return *byte == b'\n')
            .map(<[u8]>::to_vec)
            .collect();
        lines.sort();
        return lines;
    }

    /// The digest of what was produced, taken at the strength that was declared.
    ///
    /// The unit a cross-process and cross-platform comparison travels in: a child process
    /// prints this, and a committed golden is one of these.
    ///
    /// # Why the strength has to reach this
    ///
    /// A cross-run comparison of raw bytes would hold a `State` domain to byte-stable
    /// ordering across processes — which is `StateTemporal`, one step above what it
    /// declared. The check would pass today and would fail the first time a `State`
    /// domain legitimately reordered its output, and the failure would name a promise
    /// nobody made. Taking the digest at the declared strength is what keeps the scope
    /// axis and the strength axis independent, which is the entire reason the triple has
    /// three axes instead of one bit.
    #[must_use]
    pub fn Digest_At(&self, strength: DeterminismStrength) -> String
    {
        return match strength
        {
            DeterminismStrength::StateTemporal | DeterminismStrength::None =>
            {
                Content_Digest(&self.trace).to_string()
            }
            DeterminismStrength::State =>
            {
                let lines = self.State();
                let parts: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
                // Length-framed by `Digest_Of_Parts`, so that a set of lines cannot
                // collide with a differently-split set of the same bytes.
                Digest_Of_Parts(&parts).to_string()
            }
        };
    }
}

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
/// against a child process and against a committed golden. [`Cross_Environment_Owed`]
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
// rust-closure: allow: `Fn` is the minimal bound here and not merely the habitual one. The
// producer is invoked once per repetition through a shared reference, so `FnOnce` cannot
// express it; `FnMut` could, and is refused deliberately, because a producer carrying
// mutable state between repetitions is exactly the thing this verifier exists to rule out.
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

/// A strength and a trace that cannot both be true is a declaration nothing can discharge.
fn Assert_The_Declaration_Is_Coherent<S: Strategy>(domain: &str)
{
    assert!(
        Declaration_Is_Coherent(S::STRENGTH, S::TRACE),
        "{domain} declares {} with a trace of {}, which is not a coherent claim",
        S::STRENGTH,
        S::TRACE
    );
}

/// Every repetition, compared against the first production at the declared strength.
fn Assert_Repeats_Agree<S: Strategy>(
    domain: &str,
    first: &Production,
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

/// What a scope claim owes beyond one process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossEnvironment
{
    /// `SingleRun`. Nothing outside this process was promised.
    Nothing,
    /// `CrossRun`. A second process must reach the same bytes.
    SecondProcess,
    /// `CrossPlatform` or `CrossBinary`. A second process, and agreement with a digest
    /// captured on another platform or another build — which in a repository is a
    /// committed constant, because the other platform is not here to ask.
    SecondProcessAndGolden,
}

/// The obligation a scope declaration carries beyond one process.
///
/// A function of the declaration alone, so a caller cannot discharge the obligations of a
/// weaker scope than the one its domain declared.
#[must_use]
pub const fn Cross_Environment_Owed(scope: ReproducibilityScope) -> CrossEnvironment
{
    return match scope
    {
        ReproducibilityScope::SingleRun => CrossEnvironment::Nothing,
        ReproducibilityScope::CrossRun => CrossEnvironment::SecondProcess,
        ReproducibilityScope::CrossPlatform | ReproducibilityScope::CrossBinary =>
        {
            CrossEnvironment::SecondProcessAndGolden
        }
    };
}

/// The line a child process prints so its parent can read one domain's digest.
#[must_use]
pub fn Report_Line(domain: &str, digest: &str) -> String
{
    return format!("nomos-determinism\t{domain}\t{digest}");
}

/// Reads a digest for one domain out of a child process's output.
#[must_use]
pub fn Digest_In(output: &str, domain: &str) -> Option<String>
{
    for line in output.lines()
    {
        let mut fields = line.trim().split('\t');
        if fields.next() != Some("nomos-determinism")
        {
            continue;
        }
        if fields.next() != Some(domain)
        {
            continue;
        }
        if let Some(digest) = fields.next()
        {
            return Some(digest.to_owned());
        }
    }

    return None;
}

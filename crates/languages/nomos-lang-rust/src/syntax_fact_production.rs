//! What this provider promises about repeating itself, stated as a declaration and
//! checked against what it actually does.
//!
//! [`crate::Declared_Guarantee`] says how good this provider's answer is. This says
//! whether asking twice yields the same answer, which is a different question and was
//! until now an unstated one: the reproduction property held, and nothing claimed it.
//!
//! The verification owed follows mechanically from the triple below and is discharged in
//! `tests/integration/tests/determinism.rs`, over fixtures held in this repository —
//! deliberately not over the corpus. `tests/corpus.rs` already asserts this property at
//! far greater scale, and reports `ok` having read nothing on every machine that lacks
//! `NOMOS_RUST_CORPUS`, which is every machine CI runs on. A declaration proven only
//! there is proven nowhere the gate can see. See `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing facts in this crate: the `nomos.cap.syntax.items` fact for one file by
/// parsing it, and the `nomos.cap.module.index` fact for a module by rolling those up.
///
/// The analysis-kernel row of the domain table in [`nomos_contracts::Strategy`]'s module,
/// and this crate is the first thing in the workspace that occupies it against input it
/// did not write.
///
/// # Why one declaration covers two producers
///
/// A `Strategy` is a claim about an *execution domain*, and the domain table has six rows
/// for the whole system. Two producers in one crate that occupy one row and hold one triple
/// are one declaration honestly; splitting them would produce two promises with identical
/// content and one more thing to keep in step.
///
/// The name says `Syntax` because it was written when this crate had one producer. It is
/// now narrower than what it names, which is a wart and not a defect — the type is exported
/// from `lib.rs` and read by name in the integration harness, so renaming it is a
/// public-surface change with two call sites and no property riding on it. What the name
/// cannot be allowed to do is mislead about coverage, which is what this section is for.
///
/// What makes that sound rather than convenient is the harness. Each producer has a
/// production of its own in `tests/integration/tests/determinism.rs` — `syntax-fact-
/// production` and `module-index-rollup` — so the declaration is discharged twice, once
/// against each thing it covers. A restatement that added a sentence here and no production
/// there would be widening a promise to cover code nothing measures, which is the defect
/// this crate's guard exists to catch, committed deliberately.
///
/// # What the rollup's reproducibility rests on, and where it differs
///
/// Not the same argument as the parser's, and that is why it is written out. The parser is
/// a function of one file's bytes. The rollup is a function of *facts already in a store*,
/// read through a registry, so three things could vary that cannot vary for a parse.
///
/// **The member order is the provider's, not the caller's.** `rollup::Materialize_Index`
/// sorts members by subject digest and deduplicates before it reads anything, so a caller
/// that assembled a module in a different order — from a directory walk, a hash map, a
/// changed file list — reaches the same bytes. Without that the payload would be ordered by
/// whatever the caller happened to hold, which is the classic unordered-collection leak
/// wearing a caller's clothes.
///
/// **Every collection it reads from is ordered.** `MemoryFactStore` keys its entries and its
/// dependents index on `BTreeMap` and `BTreeSet`, and the registry hands back its candidates
/// in the ranking it computed. Nothing on the path iterates a hash map.
///
/// **The dependency edges are in read order**, which is the canonical member order above,
/// so the edge list a caller stores is as reproducible as the payload. That matters more
/// than it looks: the edges decide what a later change invalidates, so an edge list that
/// varied between runs would make invalidation itself non-reproducible while every payload
/// digest still agreed.
///
/// The three axes below hold for both producers, and each holds for the rollup for the
/// reason its own paragraph gives — `StateTemporal` because the payload is ordinal-bearing
/// and order-sensitive, `CrossPlatform` because nothing in the path touches a clock, a path
/// separator or a locale, and `BitIdentical` because the output is bytes.
///
/// # The grain of the guard, answered
///
/// `tests/contract/tests/determinism_declarations.rs` quantifies per crate: it asks whether
/// a crate that constructs facts has *a* declaration. So a second producer arriving in a
/// declaring crate is invisible to it, which is how the rollup arrived covered by a sentence
/// about parsing.
///
/// That grain should not be changed to count producers. "Produces facts" is recognised by
/// scanning source text for construction of the fact type; "which producers" is not a
/// property that scan can recover, and the only way to enumerate them is a list somebody
/// maintains — which `OD-COMPLETENESS-001` refuses, and rightly, because the list is where
/// the next omission would hide. The completeness that can be held mechanically is on the
/// other side: every declaration must be discharged by the harness, and a producer with no
/// production there is a producer nothing measures. That is the obligation this doc now
/// carries and the reason each producer got its own production rather than a shared one.
pub struct SyntaxFactProduction;

impl Strategy for SyntaxFactProduction
{
    /// `StateTemporal`, and the temporal half is the load-bearing one.
    ///
    /// The payload encodes each item with its ordinal, so two runs that found the same
    /// items in a different order are not merely differently ordered — they produce
    /// different bytes and therefore a different [`nomos_analysis::FactPayload::Digest`],
    /// which is a different fact about the same file. `State` alone would be a claim this
    /// provider's own encoding cannot express.
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;

    /// `CrossPlatform`, because the fact this produces is keyed by a digest of its own
    /// bytes and cached under that key.
    ///
    /// A fact cache shared between a Linux runner and a Windows workstation — which is
    /// what a fact cache is for — is wrong rather than slow if the same file digests
    /// differently on the two. The claim costs nothing to hold here because nothing in
    /// the path touches a clock, a path separator, an environment variable or an
    /// unordered collection: the walk is `syn`'s, in source order, and the digest is
    /// blake3.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`. There is no tolerance to define: the output is bytes, and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

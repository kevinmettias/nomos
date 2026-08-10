//! What spec-bundle serialization promises about repeating itself.
//!
//! The second row of the domain table in [`nomos_contracts::Strategy`]'s module reads
//! "snapshot and spec-bundle serialization". `nomos-workspace` declared the snapshot half
//! and said in its own doc comment that this half was undeclared, because the spec bundle
//! is a different crate in a band it does not reach. This is that half.
//!
//! # Why this row is the expensive one
//!
//! A bundle is the authority committed to git. The database is the working copy; the
//! bundle is what a build that did not write it reads back — a different checkout, a
//! different machine, and in the general case a different binary, because nothing in the
//! bundle names the build that produced it beyond a format number and a schema version.
//! That is the definition of [`ReproducibilityScope::CrossBinary`], and it is why the row
//! claims it rather than the cheaper `CrossRun` that would have been easy to hold.
//!
//! Verified in `tests/integration/tests/determinism.rs`, and what that verification is
//! worth without a corpus is written down in `docs/records/OD-DETERMINISM-002`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Writing a whole [`crate::Bundle`] out as text.
///
/// The domain is [`crate::Export`] followed by [`crate::Bundle::Write`], taken together:
/// the exporter decides which rows travel and in what order, and the writer decides how
/// each one is spelled. Splitting them would leave a declaration on each half and a
/// promise on neither, because the property that matters — the same corpus yields the
/// same text — is a property of the pair.
pub struct BundleSerialization;

impl Strategy for BundleSerialization
{
    /// `State`, and the word is chosen against the temptation to claim more.
    ///
    /// Every ordering in the exporter is by natural key rather than by the `uid` a row
    /// happened to be assigned, so two stores holding the same corpus written in different
    /// orders export the same text — `Test_Two_Stores_Of_The_Same_Corpus_Should_Export_Identically`
    /// asserts exactly that. The set of lines is therefore a function of the corpus alone,
    /// which is what `State` says.
    ///
    /// The stronger claim would be that the *sequence* of lines is also a function of the
    /// corpus, which is true today. It is not declared, because the manifest digest covers
    /// the header and every record line in order and so already refuses a reordering:
    /// a bundle whose lines moved carries a different manifest line and therefore a
    /// different set. The sequence is pinned by the content rather than by a promise, and
    /// a promise that restates what the content already forces is a promise that will one
    /// day be relied on for the part it does not cover.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossBinary`. A bundle is read by a build that did not write it.
    ///
    /// The manifest carries a digest over the header and every record line, and
    /// [`crate::Bundle::Parse`] refuses a bundle whose digest disagrees, whose counts
    /// disagree, or whose lines are not the canonical serialization of themselves. Every
    /// one of those refusals is a comparison between what one build wrote and what another
    /// build computes, so a recompile that changed the spelling of a single field would
    /// turn every committed bundle into a file this build refuses to read. That is not a
    /// performance regression that looks like slowness; it is the authority becoming
    /// unreadable.
    ///
    /// What makes it holdable is that the encoding names nothing about the build: the
    /// lines are `serde_json` over owned data with `BTreeMap` for every map, there is no
    /// `HashMap` iteration and no `#[derive(Hash)]` in the path, and the only numbers in
    /// the file are counts and a schema version the store itself declares.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossBinary;

    /// `BitIdentical`. The bundle's whole value is that it diffs, and a tolerance on a
    /// diff is a diff nobody can read. The manifest digest also leaves no room for one:
    /// two texts that are "equivalent within a tolerance" hash differently and the second
    /// is refused.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

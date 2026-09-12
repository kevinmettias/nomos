//! The three axes of a reproducibility claim.
//!
//! Ported from the sibling xvpe workspace's `xvpe-primitives::strategy`, deliberately
//! and with its vocabulary intact, because the two products need to mean the same thing
//! by the word "deterministic" and a divergent second spelling would be worse than no
//! spelling at all.
//!
//! # What keeps the two spellings the same
//!
//! Not a dependency. This crate may name `serde` and nothing else -- the allowlist in
//! `tests/contract/tests/boundaries/graph.rs` is what makes "reimplemented by peers that
//! never compile it" true rather than aspirational -- so re-exporting XVPE's copy is
//! refused here for the same reason it would be refused for any other crate.
//!
//! `tests/contract/tests/determinism_vocabulary.rs` closes the gap that leaves. It is
//! the one place both definitions are in scope at once, and it asserts they agree on
//! every variant's spelling, on the order of all three axes, and on
//! [`Declaration_Is_Coherent`]'s verdict for all thirty-six triples -- the whole input
//! space, not a sample. Adding a variant on either side stops that file compiling.
//!
//! It was written after the copies were measured apart. `TraceEquivalence` is declared
//! in the opposite order in the two workspaces; they agree on which end is stronger only
//! because XVPE derives no `Ord` and ranks its variants explicitly. That is the kind of
//! divergence this module's own first paragraph warns about, and nothing had been
//! watching for it.
//!
//! # Why three axes and not one bit
//!
//! "Is it deterministic?" is three questions wearing a trench coat:
//!
//! - [`DeterminismStrength`] — *what kind* of sameness is promised: the same end state,
//!   or the same end state **and** the same sequence of observable events?
//! - [`ReproducibilityScope`] — *across what environment* the promise holds: one
//!   process, one machine, every supported platform, or every compiler version?
//! - [`TraceEquivalence`] — *what "the same" means* when comparing: byte-for-byte, or
//!   observably equivalent within a documented tolerance?
//!
//! Collapsing them loses the distinctions that matter in practice. A fact cache that is
//! reproducible across runs on one machine is genuinely useful and genuinely not
//! portable. A stored baseline that must stay comparable after the analyzer is
//! recompiled needs [`ReproducibilityScope::CrossBinary`], which is a much stronger and
//! much more expensive promise than the one an incremental cache needs — and before
//! this vocabulary existed there was no way to say so.
//!
//! # Where Nomos makes these claims
//!
//! Classification attaches to the **execution domain**, not to the crate that happens
//! to contain the code. One crate routinely holds analysis code, serialization code and
//! progress reporting, and each answers to a different row:
//!
//! | Domain | Strength | Scope | Trace |
//! |---|---|---|---|
//! | Analysis kernel — facts, checks, findings | `StateTemporal` | `CrossPlatform` | `BitIdentical` |
//! | Snapshot and spec-bundle serialization | `State` | `CrossBinary` | `BitIdentical` |
//! | Fact cache and incremental reuse | `State` | `CrossRun` | `BitIdentical` |
//! | Projection engine output | `State` | `CrossPlatform` | `BitIdentical` |
//! | Correction planning and staging | `StateTemporal` | `CrossRun` | `BitIdentical` |
//! | Progress UI, logs, telemetry, agent execution | `None` | `SingleRun` | `NotApplicable` |
//!
//! The last row is the one that keeps this affordable. Determinism is enforced by
//! substituting strategies at boundaries, not by constraining the whole workspace, so
//! the CLI, the reporting layer and the agent host pay nothing for it.

mod reproducibility_scope;
mod strategy;
mod strength;
mod trace_equivalence;

pub use reproducibility_scope::ReproducibilityScope;
pub use strength::Strength as DeterminismStrength;
pub use strategy::{Declaration_Is_Coherent, Strategy};
pub use trace_equivalence::TraceEquivalence;

//! Band 41 — the seam for correction planning and lifecycle both hosts call.
//!
//! `OD-CORRECTIONS-001` found `nomos-corrections` real (a tested `Preview -> Stage ->
//! Validate -> Commit -> Rollback` lifecycle over `nomos_workspace::Workspace`'s one door)
//! and its one real caller was `nomos-cli`'s own `correct.rs` — a CLI module, not a seam,
//! so `nomos-api` had no way to reach it at all. Every later consumer this gap would have
//! blocked — an agent-proposed validated fix, an API transport, an MCP surface — needs a
//! correction seam that is not a CLI module.
//!
//! This crate is that seam, the same shape `nomos-gate-orchestration` already is for
//! `Gate`: [`Run_Correction`] composes an already-walked tree, a real
//! `nomos-check-orchestration` judgment, and `nomos-corrections`' own lifecycle into a
//! [`CorrectionOutcome`] a caller renders or serializes -- apart from choosing a platform,
//! walking a tree, or rendering the answer, exactly the three things a composition root
//! still has to assemble. Band 41, one band above `nomos-check-orchestration` (40) and
//! `nomos-corrections` (35), the same reason `nomos-gate-orchestration` sits at 41 rather
//! than beside either: a band may not depend on its own band.
//!
//! [`phantom_mirror`] is `nomos-cli`'s own former `correct/candidate.rs`, moved here
//! unchanged in substance: turning one blocking Phantom finding from
//! `Check_Completeness_Mirrors` into a real `CorrectionCandidate` is composition-root
//! wiring between one specific rule and the correction lifecycle, not a fact either
//! `nomos-corrections` (which must stay generic over what a correction is) or
//! `nomos-check-orchestration` (which must stay generic over what a rule judges) should
//! have to know about the other to express.
//!
//! [`trailing_whitespace`] is this crate's second correction family, over a different
//! rule and a different fix shape -- `P40-CORRECTIONS-SECOND-FAMILY-3`'s own evidence that
//! `phantom_mirror` was one instance of a pattern and not the pattern itself. See
//! [`run`]'s own module doc for what trying two families through one shared pipeline
//! proves about batching, and what it still leaves undecided about ranking.
//!
//! `nomos-cli`'s `correct` module and `nomos-api`'s own correction surface both call
//! [`Run_Correction`] now; neither owns the composition any more.
//!
//! [`CorrectionFamily`] is where the rules this crate corrects are declared, and the only
//! place they are. [`Run_Correction`] is the whole pipeline and can only answer by running
//! it; a host needing to know whether a correction family exists for one finding --
//! `nomos-lsp` asks exactly that, per diagnostic -- reads that list rather than keeping a
//! copy of it that this crate's own build would never notice going stale.

#![forbid(unsafe_code)]

mod correction_command;
mod correction_family;
mod correction_outcome;
mod phantom_mirror;
mod run;
mod trailing_whitespace;

pub use correction_command::CorrectionCommand;
pub use correction_family::CorrectionFamily;
pub use correction_outcome::CorrectionOutcome;
pub use run::{CorrectionEnvironment, Run_Correction};

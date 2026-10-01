//! Zone: Repo Tooling — the work ledger.
//!
//! Territory-based mutual exclusion, so that several agents can work one repository
//! without silently overwriting each other.
//!
//! # The three rules that matter
//!
//! **Territory is the unit of exclusion, not the item.** Two items may be claimed
//! concurrently exactly when their territories are provably disjoint. "Provably" is
//! doing work in that sentence: an unanswerable overlap question refuses the claim
//! rather than granting it, because unknown independence is not safe parallelism.
//!
//! **A claim is a lease, not a lock.** It lapses. An agent that dies holding one stops
//! excluding others when the lease runs out, and the lapsed claim stays visible so a
//! person can see the work was abandoned rather than never started. Giving a claim up
//! deliberately is recorded too, with the reason its holder was required to give — see
//! [`Abandonment`]. Until that existed only the lapse left a trace, so an agent that died
//! was legible afterwards and one that stopped on purpose was not.
//!
//! A lapsed item is recovered by a verb of its own, [`FileLedger::Take_Over`], and never by
//! `Claim`. The claim it displaces moves to [`LedgerItem::displaced`] rather than being
//! overwritten, because that claim is the only thing recording that the work was ever
//! started — so the item returns to the pool without anybody editing the file by hand, and
//! who held it, when they took it and when the lease ran out all survive the takeover.
//! `OD-LEDGER-012`.
//!
//! **Finishing runs a predicate.** `done_when` is prose for a human;
//! [`VerificationPredicate`] is an argument vector that gets executed, and an item
//! cannot report itself done because somebody typed that it was.
//!
//! An item that turns out not to be work is ended by a third verb, [`FileLedger::Decline`],
//! and not by either of those two. Abandoning ends a *claim* and returns the item to the
//! board because the work is still wanted; declining ends the *item*. Reaching
//! [`ItemState::Declined`] took a hand edit until `OD-LEDGER-019`, so a superseded item went
//! back to `Ready` and was offered again — measured once at the cost of a whole session's
//! run. [`Declination`] gives who ended it and when the home on the item that
//! [`Abandonment`] gives the other transition.
//!
//! # Scope
//!
//! This crate is the durable, git-committed instance of the exclusion model. The
//! run-scoped reservations a correction scheduler will need, and the session-scoped
//! leases delegated agents will hold, are different *instances* with different
//! lifetimes and authority — they share [`ExclusionLedger`], [`SubjectSet`] and the
//! overlap predicate, and they do not share persistence.
//!
//! [`SubjectSet`]: nomos_model::SubjectSet
//!
//! [`Territory`] and [`VerificationPredicate`] are not declared here. `OD-LEDGER-037` found
//! both ledger-agnostic in their real field shapes, and `nomos-scope-verification` is where
//! they now live; this crate re-exports them, unchanged, for its own claim/overlap and
//! finish logic -- a behavior-preserving move, not a redesign.

#![forbid(unsafe_code)]

mod claim;
#[path = "exclusion_ledger.rs"]
mod exclusion;
mod finish;
#[path = "gate_unknown.rs"]
mod gate;
#[path = "holder.rs"]
mod item;
mod ledger_error;
#[path = "file_ledger.rs"]
mod store;
mod verification;

pub use claim::{Claim, ClaimRefusal, RefusalLayer};
pub use exclusion::{Blocker, ExclusionLedger, Reservation};
pub use finish::{Abandonment, Declination, Finish_Item, FinishRefusal, Finishing};
pub use finish::release_outcome::ReleaseOutcome;
pub use gate::{Derive_Step, Derive_Steps, DerivedStep, GATE_WORKFLOW, GateOutcome, GateUnknown, LINT_STEP, LocalGateRun, Run_Gate_Locally, StepExecution, StepGuard, StepName, Workflow_Path, WorkflowText};
pub use item::{DEFAULT_LEASE, DeclineReason, Holder, ItemId, ItemKind, ItemOrigin, ItemState, LedgerItem, MAXIMUM_LEASE, Widening};
pub use ledger_error::LedgerError;
pub use nomos_scope_verification::{Normalize_Path, Territory};
pub use store::{AddRefusal, Claim_Refusal, Eligible_Items, FileLedger, LOCK_STALE_AFTER, LOCK_WAIT_LIMIT, LedgerDocument, SCHEMA_VERSION, Validate_Document};
pub use store::{BoardFiles, Board_In, DOCUMENT_FILENAME, LOCK_FILENAME};
pub use verification::{VerificationPredicate, VerificationRecord};

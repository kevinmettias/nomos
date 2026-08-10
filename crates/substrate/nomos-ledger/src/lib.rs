//! Band 2 — the work ledger.
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
//! **Finishing runs a predicate.** `done_when` is prose for a human;
//! [`VerificationPredicate`] is an argument vector that gets executed, and an item
//! cannot report itself done because somebody typed that it was.
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

#![forbid(unsafe_code)]

mod exclusion;
mod finish;
mod gate;
mod item;
mod store;
mod territory;

pub use exclusion::{
    Check_Lease, ClaimRefusal, ExclusionLedger, Refusal_From, ReleaseOutcome, Reservation,
};
pub use finish::{Finish, FinishRefusal};
pub use gate::{Derive_Step, GATE_WORKFLOW, GateUnknown, LINT_STEP, Workflow_Path};
pub use item::{
    Abandonment, Blocker, Claim, DEFAULT_LEASE, GateOutcome, ItemId, ItemState, LedgerItem,
    MAXIMUM_LEASE, VerificationPredicate, VerificationRecord,
};
pub use store::{
    Claim_Refusal, FileLedger, LOCK_STALE_AFTER, LOCK_WAIT_LIMIT, LedgerDocument, LedgerError,
    SCHEMA_VERSION, Validate,
};
pub use territory::{Normalize_Path, Subject_Of, Territory};

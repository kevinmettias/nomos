//! What a unit of work is, and what it means to have finished one.
//!
//! # Why every container here refuses a key it does not declare
//!
//! Each type below carries `#[serde(deny_unknown_fields)]`, and the reason is stated once
//! here rather than eight times below. Serde's default is to ignore an unrecognized key, so
//! a binary built before a field existed read the current ledger, dropped that field, wrote
//! the document back and exited 0 — the state change surviving and the data not. Refusing
//! the parse is what makes such a writer stop at the door instead of succeeding quietly.
//! `OD-LEDGER-008` records the decision and what it does not reach.
//!
//! The refusal is asymmetric on purpose: a new build still reads an old file, because every
//! added field carries `#[serde(default)]`, and an old build no longer reads a new one.
//! Forward compatibility for this file was only ever buying the ability to lose it.
//!
//! Nothing enumerates these attributes, because a hand-written list of types is only as
//! complete as the hand — `OD-COMPLETENESS-001`. What holds them in place is
//! `Test_Every_Object_In_A_Ledger_Should_Refuse_An_Undeclared_Key`, which walks a fully
//! populated document and probes every object node it finds.

use crate::territory::Territory;
use nomos_platform::Timestamp;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The lease a claim gets when the holder does not ask for a specific one.
///
/// Long enough that an agent working a real item is not interrupted; short enough that
/// an agent which died at lunchtime does not hold territory until tomorrow.
pub const DEFAULT_LEASE: Duration = Duration::from_secs(2 * 60 * 60);

/// The longest lease anyone may hold.
///
/// A ceiling, not a suggestion. Without one, "just set it to a year" is the obvious
/// workaround for an agent that keeps getting interrupted, and the ledger stops
/// excluding anything.
pub const MAXIMUM_LEASE: Duration = Duration::from_secs(14 * 24 * 60 * 60);

/// A ledger item's stable identifier.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemId(String);

impl ItemId
{
    /// Wraps an authored identifier.
    #[must_use]
    pub fn New(value: impl Into<String>) -> Self
    {
        return Self(value.into());
    }

    /// The identifier as authored.
    #[must_use]
    pub fn As_Str(&self) -> &str
    {
        return &self.0;
    }
}

impl core::fmt::Display for ItemId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        // `pad`, not `write_str`. Writing to the formatter directly discards the width
        // and alignment the caller asked for, so `{:<10}` silently does nothing and a
        // listing that was supposed to be columns comes out ragged.
        return formatter.pad(&self.0);
    }
}

/// Why an item cannot be worked on.
///
/// A typed reason rather than free prose, so a query can answer "what is blocked on a
/// decision?" without matching strings. The prototype stored this as a sentence, and
/// the result was that nobody could tell how much of the backlog was waiting on a
/// person versus waiting on a dependency.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Blocker
{
    /// Waiting on other items.
    Dependency
    {
        /// The items that must finish first.
        items: Vec<ItemId>,
    },
    /// Waiting on a decision somebody has to make.
    Decision
    {
        /// What has to be decided.
        question: String,
    },
    /// The item's territory does not match what the work actually touches.
    TerritoryMismatch
    {
        /// What is wrong with it.
        detail: String,
    },
    /// Waiting on something outside this repository.
    ExternalResource
    {
        /// What is being waited for.
        resource: String,
    },
    /// Too large to claim as one unit.
    NeedsSplit,
    /// Something else, stated.
    Other
    {
        /// What.
        detail: String,
    },
}

/// What state a ledger item is in.
///
/// A closed set. The compiler refuses a sixth, which is the point: a state that exists
/// only in one tool's imagination is a state no query can filter on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ItemState
{
    /// Available to claim.
    Ready,
    /// Someone is working on it.
    Claimed,
    /// Cannot proceed.
    Blocked,
    /// Finished, with its verification predicate having passed.
    Done,
    /// Deliberately not going to be done.
    Declined
    {
        /// Why not. Carried in the variant so an item cannot be declined reasonlessly.
        reason: String,
    },
}

impl ItemState
{
    /// Whether an item in this state may be claimed.
    #[must_use]
    pub const fn Is_Claimable(&self) -> bool
    {
        return matches!(self, Self::Ready);
    }

    /// Whether this state is terminal.
    #[must_use]
    pub const fn Is_Finished(&self) -> bool
    {
        return matches!(self, Self::Done | Self::Declined { .. });
    }
}

/// A held claim on an item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim
{
    /// Who holds it.
    pub holder: String,
    /// When they took it.
    pub acquired_at: Timestamp,
    /// When it lapses if not renewed.
    pub lease_expires_at: Timestamp,
}

impl Claim
{
    /// Whether this claim has lapsed as of `now`.
    ///
    /// A lapsed claim does not release itself. It stops excluding — every *other* item is
    /// claimable again, which is what `MAXIMUM_LEASE` exists for — and it stays visible so
    /// that a person can see the work was abandoned rather than never started.
    ///
    /// It does not let the next agent take *this* item, and this comment said it did until
    /// `P10-LAPSE-BRICKS` measured it. The item stays `Claimed`, and `Claim_Refusal` rejects
    /// anything that is not `Ready` before it ever reaches the lease. That is deliberate as
    /// of `OD-LEDGER-009` rather than merely true: a claim overwrites `claim`, and `claim`
    /// is the only thing recording that the work was ever started, which `OD-LEDGER-006`
    /// decided must survive. Taking a lapsed item over is a different operation from
    /// claiming a free one and is not one yet.
    #[must_use]
    pub fn Has_Lapsed(&self, now: Timestamp) -> bool
    {
        return now > self.lease_expires_at;
    }
}

/// A claim somebody gave up on purpose, and why.
///
/// Kept on the item rather than in the transition that produced it, which is the whole
/// point. [`ItemState::Declined`] carries its reason and survives for exactly one reason:
/// it is part of a *state*, and states are what get written down. An abandonment is a
/// *transition*, and a transition leaves nothing behind unless something on the item is
/// given the job of holding it. This is that job.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Abandonment
{
    /// Who gave it up.
    pub holder: String,
    /// Why, in the words they gave.
    pub reason: String,
    /// When they gave it up.
    pub abandoned_at: Timestamp,
}

/// A command that decides whether an item is actually finished.
///
/// An argument vector, never a command string. The prototype stored a string and split
/// it on whitespace with quote handling, which put a bespoke parser between what an
/// author wrote and what ran — in a file several agents write concurrently.
///
/// It is also a *predicate*, not a description. `done_when` on the item is prose for a
/// human; this is the thing that gets run, and an item cannot report itself finished
/// because somebody typed that it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationPredicate
{
    /// The program and its arguments.
    pub argv: Vec<String>,
    /// How long to allow before giving up.
    pub timeout_seconds: u64,
}

impl VerificationPredicate
{
    /// A predicate running the given argument vector.
    #[must_use]
    pub fn New(argv: Vec<String>) -> Self
    {
        return Self {
            argv,
            timeout_seconds: 600,
        };
    }

    /// Whether this predicate could actually be run.
    ///
    /// An empty argument vector is not a predicate; it is a field somebody filled in
    /// to satisfy a schema.
    #[must_use]
    pub fn Is_Runnable(&self) -> bool
    {
        return self.argv.first().is_some_and(|program| !program.is_empty());
    }
}

/// What the derived gate step did, alongside the item's own predicate.
///
/// Recorded rather than merely run. Without it a reader cannot tell an item finished
/// under the gate from one finished before the gate was derived at all, and every
/// `verified` block written earlier would silently read as though it had been checked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateOutcome
{
    /// What was run, as derived from the workflow.
    pub argv: Vec<String>,
    /// What it exited with.
    pub exit_code: i32,
}

/// What happened when the predicate was run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationRecord
{
    /// What was run.
    pub argv: Vec<String>,
    /// What it exited with.
    pub exit_code: i32,
    /// The tail of its output, for a human reading the ledger later.
    pub output_tail: String,
    /// When it ran.
    pub verified_at: Timestamp,
    /// The gate step that ran first, when one could be derived.
    ///
    /// `None` on every record written before the gate was part of finishing. That is a
    /// fact about those records and is left visible rather than backfilled.
    #[serde(default)]
    pub gate: Option<GateOutcome>,
}

/// One unit of work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerItem
{
    /// Stable identifier.
    pub id: ItemId,
    /// What it is, in one line.
    pub title: String,
    /// Why it is worth doing. Prose, for a human deciding what to pick up.
    pub why: String,
    /// What "finished" means, in prose. Paired with — never a substitute for —
    /// [`LedgerItem::verification`].
    pub done_when: String,
    /// What the work touches. The basis for mutual exclusion.
    pub territory: Territory,
    /// Current state.
    pub state: ItemState,
    /// Items that must finish first.
    #[serde(default)]
    pub depends_on: Vec<ItemId>,
    /// Why it cannot proceed, when blocked.
    #[serde(default)]
    pub blocked: Option<Blocker>,
    /// The active claim, if any.
    #[serde(default)]
    pub claim: Option<Claim>,
    /// The predicate that decides whether it is done.
    #[serde(default)]
    pub verification: Option<VerificationPredicate>,
    /// The recorded result of running that predicate.
    #[serde(default)]
    pub verified: Option<VerificationRecord>,
    /// Every claim on this item that was given up deliberately, oldest first.
    ///
    /// A list, not the most recent one. An item abandoned twice was abandoned twice, and
    /// keeping only the latest discards the earlier reason — which is the loss this field
    /// exists to stop, one scale down.
    ///
    /// `#[serde(default)]` because every item written before this field existed has none,
    /// and that is a fact about those items rather than something to backfill.
    #[serde(default)]
    pub abandoned: Vec<Abandonment>,
}

impl LedgerItem
{
    /// Whether this item's claim is currently excluding others.
    ///
    /// A lapsed claim is not active. This is the difference between an item somebody is
    /// working on and an item somebody walked away from four hours ago.
    #[must_use]
    pub fn Has_Active_Claim(&self, now: Timestamp) -> bool
    {
        return self
            .claim
            .as_ref()
            .is_some_and(|claim| !claim.Has_Lapsed(now));
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Item(id: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: "an item".to_owned(),
            why: "because".to_owned(),
            done_when: "when it is done".to_owned(),
            territory: Territory::Empty(),
            state: ItemState::Ready,
            depends_on: Vec::new(),
            blocked: None,
            claim: None,
            verification: None,
            verified: None,
            abandoned: Vec::new(),
        };
    }

    fn At(seconds: i64) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(seconds);
    }

    #[test]
    fn Test_Only_Ready_Items_Should_Be_Claimable()
    {
        assert!(ItemState::Ready.Is_Claimable());
        assert!(!ItemState::Claimed.Is_Claimable());
        assert!(!ItemState::Blocked.Is_Claimable());
        assert!(!ItemState::Done.Is_Claimable());
        assert!(
            !ItemState::Declined {
                reason: "superseded".to_owned()
            }
            .Is_Claimable()
        );
    }

    /// A lapsed claim must stop excluding, or one crashed agent holds territory until
    /// somebody notices and edits the file by hand.
    #[test]
    fn Test_A_Lapsed_Claim_Should_Stop_Excluding()
    {
        let mut item = Item("T-1");
        item.claim = Some(Claim {
            holder: "agent-a".to_owned(),
            acquired_at: At(1_000),
            lease_expires_at: At(2_000),
        });

        assert!(item.Has_Active_Claim(At(1_999)));
        assert!(!item.Has_Active_Claim(At(2_001)));
    }

    /// The boundary. A lease expiring exactly now has not yet lapsed — otherwise a
    /// holder renewing at the moment of expiry races against being displaced.
    #[test]
    fn Test_A_Claim_Should_Not_Lapse_On_Its_Expiry_Second()
    {
        let claim = Claim {
            holder: "agent-a".to_owned(),
            acquired_at: At(1_000),
            lease_expires_at: At(2_000),
        };

        assert!(!claim.Has_Lapsed(At(2_000)));
        assert!(claim.Has_Lapsed(At(2_001)));
    }

    /// An empty argv is a field somebody filled in, not a predicate. Accepting it would
    /// let an item claim verified completion having run nothing.
    #[test]
    fn Test_An_Empty_Predicate_Should_Not_Be_Runnable()
    {
        assert!(!VerificationPredicate::New(Vec::new()).Is_Runnable());
        assert!(!VerificationPredicate::New(vec![String::new()]).Is_Runnable());
        assert!(VerificationPredicate::New(vec!["cargo".to_owned(), "test".to_owned()]).Is_Runnable());
    }

    /// A listing is columns, and columns need the width the caller asked for. A `Display`
    /// that writes straight to the formatter drops it without any error.
    #[test]
    fn Test_An_Item_Id_Should_Honor_Format_Width()
    {
        assert_eq!(format!("{:<10}|", ItemId::New("P1-MODEL")), "P1-MODEL  |");
        assert_eq!(format!("{}", ItemId::New("P1-MODEL")), "P1-MODEL");
    }

    /// A field added here without the schema version moving produces a refusal that misstates
    /// why.
    ///
    /// This protects the *message*, never the data. Under `OD-LEDGER-008` the guarantee is
    /// `deny_unknown_fields`, which is mechanical: a build meeting a field it does not know
    /// refuses whatever the version says. What a forgotten bump costs is that the refusal comes
    /// out as [`crate::LedgerError::Malformed`] instead of `Unrecognized`, so the operator is
    /// sent to repair a file that is correct rather than to rebuild a binary that is old. A
    /// stale explanation is the class of defect that record exists to fix, one layer along, so
    /// the count is asserted here rather than trusted.
    ///
    /// Nothing about this test makes the version a guard. It makes forgetting it visible.
    #[test]
    fn Test_A_Field_Added_To_An_Item_Should_Raise_The_Schema_Version()
    {
        let serialized = serde_json::to_value(Item("T-1")).expect("an item serializes");
        let fields = serialized
            .as_object()
            .expect("an item serializes as an object");

        assert_eq!(
            fields.len(),
            12,
            "a field was added to `LedgerItem`. Raise `SCHEMA_VERSION` in `store.rs` and this \
             count together, or a build that predates the field will be told the ledger is \
             malformed instead of being told it is old"
        );
    }

    #[test]
    fn Test_Terminal_States_Should_Be_Recognized()
    {
        assert!(ItemState::Done.Is_Finished());
        assert!(
            ItemState::Declined {
                reason: "not worth it".to_owned()
            }
            .Is_Finished()
        );
        assert!(!ItemState::Ready.Is_Finished());
        assert!(!ItemState::Claimed.Is_Finished());
    }
}

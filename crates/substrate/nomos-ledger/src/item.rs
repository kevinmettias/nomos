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

// An item's identity, the state it is in, what kind of work it is, and where it came from.
mod id;
mod kind;
mod origin;
mod state;

pub use id::ItemId;
pub use kind::ItemKind;
pub use origin::ItemOrigin;
pub use state::ItemState;

#[cfg(test)]
mod tests;

use crate::Declination;
use crate::Abandonment;
use crate::VerificationRecord;
use crate::VerificationPredicate;
use crate::Claim;
use crate::Blocker;
use crate::Territory;
use nomos_platform::Timestamp;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Who is declining an item, distinguished from [`DeclineReason`] purely by type.
///
/// The two travel as adjacent parameters through every layer of `Decline` —
/// [`LedgerItem::Decline`], [`crate::FileLedger::Decline`], and the verb body beneath it —
/// and two same-typed strings at any of those layers would let a caller swap who is
/// declining for why and have the compiler accept it. `From<&str>` and `From<&String>` both
/// convert into one, so no existing call site needs to change shape to adopt it: every one
/// already passes a borrowed string.
#[derive(Clone, Copy, Debug)]
pub struct Holder<'a>(&'a str);

impl<'a> From<&'a str> for Holder<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Holder(value);
    }
}

impl<'a> From<&'a String> for Holder<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Holder(value.as_str());
    }
}

impl<'a> Holder<'a>
{
    /// The holder's identifier as a plain string.
    #[must_use]
    pub fn As_Str(&self) -> &'a str
    {
        return self.0;
    }
}

/// Why an item is being declined, distinguished from [`Holder`] for the reason given there.
#[derive(Clone, Copy, Debug)]
pub struct DeclineReason<'a>(&'a str);

impl<'a> From<&'a str> for DeclineReason<'a>
{
    fn from(value: &'a str) -> Self
    {
        return DeclineReason(value);
    }
}

impl<'a> From<&'a String> for DeclineReason<'a>
{
    fn from(value: &'a String) -> Self
    {
        return DeclineReason(value.as_str());
    }
}

impl<'a> DeclineReason<'a>
{
    /// The reason as a plain string.
    #[must_use]
    pub fn As_Str(&self) -> &'a str
    {
        return self.0;
    }
}

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

/// One unit of work.
///
/// `WORK-LEDGER-001` names the things an item shall be addressable by, and this type carries
/// all of them except **`priority`**. `OD-LEDGER-017` defers that rather than adding a field:
/// the corpus wants `priority` as the first of three ranking keys, and the other two — how many
/// descendants an item unblocks, and conflict risk — are already derivable from
/// [`LedgerItem::depends_on`] and [`LedgerItem::territory`]. A bare integer would satisfy the
/// letter of the requirement while buying the one key of the three that ages worst. The
/// follow-on is ranking, not a field.
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
    /// What kind of work this is, from a closed set. `OD-LEDGER-024`.
    ///
    /// No `#[serde(default)]`, deliberately: every item written before this field existed
    /// was migrated in the same commit that added it, because `OD-LEDGER-024`'s `done_when`
    /// refuses "a field empty on a hundred rows and set on the next" — an absence a reader
    /// could not tell apart from an oversight. A build that meets a row without one refuses
    /// the document rather than guessing, the same way it already refuses an unrecognized
    /// key.
    pub kind: ItemKind,
    /// Where this item came from: a person, or a session that proposed it. `OD-LEDGER-024`.
    ///
    /// No `#[serde(default)]`, for the reason [`LedgerItem::kind`] carries none.
    pub origin: ItemOrigin,
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
    /// Every claim on this item that lapsed and was displaced by a takeover, oldest first.
    ///
    /// The claim itself, kept exactly as it stood, rather than a summary of it. A summary is
    /// a second shape that can drift from [`Claim`]; the claim cannot drift from itself. It
    /// carries no record of who displaced it or when, because neither is new knowledge —
    /// both are the `holder` and `acquired_at` of the claim that *replaced* it, which is
    /// [`LedgerItem::claim`] for the most recent entry and `displaced[i + 1]` for any
    /// earlier one. `OD-LEDGER-012` states what that indirection costs.
    ///
    /// A list for the reason [`LedgerItem::abandoned`] is a list: an item taken over twice
    /// was taken over twice, and keeping only the most recent discards the earlier holder —
    /// this same loss one scale down.
    ///
    /// `#[serde(default)]` because every item written before this field existed has none,
    /// and that is a fact about those items rather than something to backfill. It carries no
    /// `skip_serializing_if`, deliberately: `abandoned` has none either, and a field whose
    /// presence in the file depends on its content cannot be counted. `grep -c '"abandoned"'
    /// work/ledger.json` equalling the item count is this repository's standing check that
    /// no stale writer has been through the file, and it only works while the shape of the
    /// document is independent of what is in it.
    #[serde(default)]
    pub displaced: Vec<Claim>,
    /// Who declined this item and when, if it was declined through the verb.
    ///
    /// Not a list, unlike its two neighbours, because declining is terminal: a second decline
    /// is refused rather than appended, so there is never a second entry to lose. That is the
    /// same argument [`LedgerItem::abandoned`] makes, reaching the opposite shape from the
    /// opposite fact.
    ///
    /// `#[serde(default)]` for the reason the fields above carry it, and with one instance
    /// already on the board: `P3-RESTORE` was declined by hand before any verb could, so it
    /// reads as declined with nobody named. That is exactly true and is left visible rather
    /// than backfilled with an agent identifier nothing recorded.
    #[serde(default)]
    pub declined: Option<Declination>,
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

    /// Replaces a lapsed claim with a new one, keeping the claim it replaced.
    ///
    /// Returns `false` and changes nothing when there is no claim, or when the claim has not
    /// lapsed. A live holder is never displaced by this, and an item recording no claim is
    /// never given one — installing a claim over a hole would produce exactly the item
    /// [`crate::Validate`] calls a corruption, and would destroy the fact that the record
    /// was already missing.
    ///
    /// The move and the install are **one operation**, for the reason
    /// [`crate::ReleaseOutcome::Record_On`] is one: spelled out at a call site, an
    /// implementation is free to install the new claim and not keep the old one, and that is
    /// the entire defect this exists to prevent. There is no ordering of the statements below
    /// in which `replacement` lands and `previous` is not kept, and nothing else writes
    /// `claim` during a takeover. That is where the guarantee lives — the tests are checks on
    /// it rather than the thing providing it.
    ///
    /// The lapse is re-checked here rather than trusted from the caller, so the method is
    /// still safe standing alone if a second caller ever appears.
    #[must_use]
    pub fn Replace_Lapsed_Claim(&mut self, replacement: Claim, now: Timestamp) -> bool
    {
        let Some(previous) = self.claim.as_ref()
        else
        {
            return false;
        };

        if !previous.Has_Lapsed(now)
        {
            return false;
        }

        self.displaced.push(previous.clone());
        self.claim = Some(replacement);

        return true;
    }

    /// Ends this item, keeping who ended it and when.
    ///
    /// One operation, for the reason [`LedgerItem::Replace_Lapsed_Claim`] and
    /// [`crate::ReleaseOutcome::Record_On`] are each one: spelled out at a call site, an
    /// implementation is free to write the state and not the [`Declination`], and that is
    /// precisely the half this whole change exists to stop being lost. There is no ordering
    /// of the statements below in which the state lands and the declination does not.
    ///
    /// It performs no checks. Whether this item may be declined at all is
    /// `Decline_Refusal`'s question, asked once under the lock, and asking it twice is how
    /// two answers come to disagree.
    pub fn Decline<'a>(&mut self, reason: impl Into<DeclineReason<'a>>, holder: impl Into<Holder<'a>>, at: Timestamp)
    {
        let reason = reason.into();
        let holder = holder.into();

        self.state = ItemState::Declined {
            reason: reason.As_Str().to_owned(),
        };
        self.declined = Some(Declination {
            holder: holder.As_Str().to_owned(),
            declined_at: at,
        });
    }
}

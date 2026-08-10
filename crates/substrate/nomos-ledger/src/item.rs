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
///
/// # Six of `WORK-LEDGER-005`'s seven causes, and why the seventh is not here
///
/// This enum is derived from `WORK-LEDGER-005`, a normative accepted corpus requirement, and
/// declares six of the seven causes it names, in its order. The absent one is
/// `stale probe artifact`. `OD-LEDGER-017` decided it is **declined rather than missing**: the
/// seven are an inventory of what one earlier ledger's prose `blocked` field was observed to
/// contain, this build has no probe artifact for an item to wait on, and the requirement's own
/// list ends "or other typed cause" — which [`Blocker::Other`] is. The variant becomes owed the
/// day an item here waits on *somebody else* regenerating a derived artifact; a stale
/// projection is not that, because re-rendering it needs no second party.
///
/// The comparison is transcribed as `CORPUS_CAUSES` in this module's test module rather than
/// left in the record, so the next reader does not have to re-derive it against a corpus that
/// is not on their machine — `D-134`'s reason for putting a universe beside its mirror.
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
    /// It does not let the next agent *claim* this item, and this comment said it did until
    /// `P10-LAPSE-BRICKS` measured it. The item stays `Claimed`, and `Claim_Refusal` refuses
    /// a plain claim on it — with [`crate::ClaimRefusal::Lapsed`], which names the holder
    /// whose lease ran out and the remedy. That refusal is deliberate as of `OD-LEDGER-009`
    /// rather than merely true: a claim overwrites `claim`, and `claim` is the only thing
    /// recording that the work was ever started, which `OD-LEDGER-006` decided must survive.
    ///
    /// Taking a lapsed item over is therefore a different operation from claiming a free
    /// one, and `OD-LEDGER-012` is where it became one: [`crate::FileLedger::Take_Over`]
    /// installs the new claim and [`LedgerItem::Replace_Lapsed_Claim`] moves the claim it
    /// replaced onto [`LedgerItem::displaced`], so what a lapse recorded survives the thing
    /// that ends it.
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
            displaced: Vec::new(),
        };
    }

    fn At(seconds: i64) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(seconds);
    }

    /// The seven typed blocker causes `WORK-LEDGER-005` names, in the order it names them,
    /// each paired with the [`Blocker`] variant that carries it.
    ///
    /// A transcription of the requirement's statement, quoted in full from
    /// `01_authoring/artifacts/requirements/WORK-LEDGER-005.md` (`status: normative`,
    /// `authority: canonical-normative-record`, `maturity: accepted`):
    ///
    /// > WORK-LEDGER-005 Blocked work shall classify the blocker as dependency, decision,
    /// > territory mismatch, external resource, needs-split, stale probe artifact, or other
    /// > typed cause rather than relying only on prose notes.
    ///
    /// So a row is a claim about the corpus rather than a local preference, and the corpus is
    /// not on the machine that runs this test — which is the reason the comparison is here
    /// rather than only in a record.
    ///
    /// The `None` row is `stale probe artifact`, and it is empty **deliberately**.
    /// `OD-LEDGER-017` records why, what would reverse it, and why declining it does not
    /// violate the requirement.
    const CORPUS_CAUSES: [(&str, Option<&str>); 7] = [
        ("dependency", Some("Dependency")),
        ("decision", Some("Decision")),
        ("territory mismatch", Some("TerritoryMismatch")),
        ("external resource", Some("ExternalResource")),
        ("needs-split", Some("NeedsSplit")),
        ("stale probe artifact", None),
        ("other", Some("Other")),
    ];

    /// Where a cause sits in [`CORPUS_CAUSES`].
    ///
    /// The match is exhaustive on purpose, and that is the whole mechanism for membership: a
    /// seventh variant makes it non-exhaustive, so this module stops compiling and whoever
    /// added the cause has to arrive here, beside the table and the record it cites, rather
    /// than adding one these assertions would never visit.
    const fn Corpus_Position(blocker: &Blocker) -> usize
    {
        return match blocker
        {
            Blocker::Dependency { .. } => 0,
            Blocker::Decision { .. } => 1,
            Blocker::TerritoryMismatch { .. } => 2,
            Blocker::ExternalResource { .. } => 3,
            Blocker::NeedsSplit => 4,
            Blocker::Other { .. } => 6,
        };
    }

    /// The externally tagged variant name a blocker serializes as.
    ///
    /// Read back out of `serde` rather than from a `Debug` string, because the serialized name
    /// is what a reader of `work/ledger.json` sees and what a stale writer's
    /// `deny_unknown_fields` refusal turns on. A unit variant serializes as a bare string and a
    /// struct variant as a one-key object, so both shapes are handled and anything else is a
    /// panic rather than a silent miss.
    fn Serialized_Tag(blocker: &Blocker) -> String
    {
        return match serde_json::to_value(blocker).expect("a blocker serializes")
        {
            serde_json::Value::String(name) => name,
            serde_json::Value::Object(fields) => fields
                .keys()
                .next()
                .cloned()
                .expect("an externally tagged struct variant carries one key"),
            other => panic!("a blocker serialized as neither a string nor an object: {other:?}"),
        };
    }

    /// One sample of every cause this enum declares.
    ///
    /// Built here rather than inside a test so both assertions below read the same six, and so
    /// the compiler's exhaustiveness check on [`Corpus_Position`] is the only thing deciding
    /// what "every declared cause" means.
    fn Declared_Causes() -> Vec<Blocker>
    {
        return vec![
            Blocker::Dependency {
                items: vec![ItemId::New("T-1")],
            },
            Blocker::Decision {
                question: "which substrate is canonical".to_owned(),
            },
            Blocker::TerritoryMismatch {
                detail: "the work reaches a crate the item does not reserve".to_owned(),
            },
            Blocker::ExternalResource {
                resource: "a corpus that is not on this machine".to_owned(),
            },
            Blocker::NeedsSplit,
            Blocker::Other {
                detail: "stated".to_owned(),
            },
        ];
    }

    /// `WORK-LEDGER-005` is the corpus requirement this enum is derived from, and six of its
    /// seven causes in its order is the entire evidence for that derivation. Asserted rather
    /// than trusted: the resemblance is what makes the absent seventh a decision instead of an
    /// accident, and a relabelling or a reordering would destroy the evidence while leaving
    /// every other test in this file green.
    #[test]
    fn Test_The_Declared_Causes_Should_Be_The_Corpus_Causes_Minus_The_Declined_One()
    {
        let mut occupied: Vec<usize> = Vec::new();

        for blocker in Declared_Causes()
        {
            let position = Corpus_Position(&blocker);
            let (cause, transcribed) = *CORPUS_CAUSES
                .get(position)
                .expect("Corpus_Position returns an index into CORPUS_CAUSES");

            assert_eq!(
                transcribed,
                Some(Serialized_Tag(&blocker).as_str()),
                "the variant declared for WORK-LEDGER-005's `{cause}` does not serialize as the \
                 name CORPUS_CAUSES transcribes for it"
            );

            occupied.push(position);
        }

        occupied.sort_unstable();

        let transcribed: Vec<usize> = CORPUS_CAUSES
            .iter()
            .enumerate()
            .filter(|(_, entry)| return entry.1.is_some())
            .map(|(position, _)| return position)
            .collect();

        assert_eq!(
            occupied, transcribed,
            "`Blocker` no longer covers exactly the causes CORPUS_CAUSES says it covers. \
             OD-LEDGER-017 decided which of WORK-LEDGER-005's seven this enum declares and which \
             it declines, so changing the membership means amending that record and this table \
             together"
        );
    }

    /// Which row is empty is the decision, not merely how many are.
    ///
    /// Without this, `OD-LEDGER-017`'s answer could be relocated to a different cause while the
    /// count stayed at six and the test above stayed green — a decline of `needs-split` reading
    /// as the decline of `stale probe artifact` that record actually argued for.
    #[test]
    fn Test_The_One_Declined_Cause_Should_Be_The_Stale_Probe_Artifact()
    {
        let declined: Vec<&str> = CORPUS_CAUSES
            .iter()
            .filter(|entry| return entry.1.is_none())
            .map(|entry| return entry.0)
            .collect();

        assert_eq!(
            declined,
            vec!["stale probe artifact"],
            "OD-LEDGER-017 declines exactly one of WORK-LEDGER-005's seven causes and names which \
             one. A different row going empty is a different decision and needs its own record"
        );
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

    fn Claimed_By(holder: &str, expires: i64) -> Claim
    {
        return Claim {
            holder: holder.to_owned(),
            acquired_at: At(1_000),
            lease_expires_at: At(expires),
        };
    }

    /// The guarantee the whole of `OD-LEDGER-012` rests on, at the unit that provides it.
    ///
    /// A takeover must not be able to erase the previous holder. Asserted here as well as
    /// over the store, because this method is where the property is structural: the store
    /// tests would still pass if the push moved to a caller, and the next caller would then
    /// be free to omit it.
    #[test]
    fn Test_Replacing_A_Lapsed_Claim_Should_Keep_The_Claim_It_Replaced()
    {
        let mut item = Item("T-1");
        item.state = ItemState::Claimed;
        item.claim = Some(Claimed_By("dead-agent", 2_000));

        assert!(item.Replace_Lapsed_Claim(Claimed_By("agent-b", 9_000), At(2_001)));

        assert_eq!(
            item.claim.as_ref().map(|claim| return claim.holder.clone()),
            Some("agent-b".to_owned())
        );
        assert_eq!(
            item.displaced
                .iter()
                .map(|claim| return claim.holder.clone())
                .collect::<Vec<String>>(),
            vec!["dead-agent".to_owned()],
            "the claim the takeover replaced was dropped, so nothing says whose work this was"
        );
    }

    /// The two refusals, which are what keep the method safe standing alone.
    ///
    /// A live holder is not displaced — otherwise `takeover` is a way to steal work in
    /// progress, which is worse than the defect it fixes. And an item recording no claim is
    /// not given one: writing a claim over a hole would destroy the evidence that the record
    /// was already missing, which is [`crate::Validate`]'s one remaining corruption.
    #[test]
    fn Test_Replacing_Should_Refuse_A_Live_Claim_And_An_Absent_One()
    {
        let mut live = Item("T-1");
        live.state = ItemState::Claimed;
        live.claim = Some(Claimed_By("agent-a", 2_000));

        assert!(!live.Replace_Lapsed_Claim(Claimed_By("agent-b", 9_000), At(1_999)));
        assert_eq!(
            live.claim.as_ref().map(|claim| return claim.holder.clone()),
            Some("agent-a".to_owned()),
            "a live holder was displaced"
        );
        assert!(live.displaced.is_empty());

        let mut hollow = Item("T-2");
        hollow.state = ItemState::Claimed;

        assert!(!hollow.Replace_Lapsed_Claim(Claimed_By("agent-b", 9_000), At(2_001)));
        assert!(
            hollow.claim.is_none(),
            "a claim was written over an item that recorded none"
        );
        assert!(hollow.displaced.is_empty());
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
            13,
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

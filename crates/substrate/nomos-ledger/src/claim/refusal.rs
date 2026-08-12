//! Every way a claim is refused, in the words the holder gets.

use std::time::Duration;
use nomos_model::UnknownReason;
use crate::item::ItemId;
use nomos_platform::Timestamp;
/// Why a claim was refused.
///
/// Every variant tells the caller something different about what to do next, which is
/// why this is not a `bool` or a string. An agent that is told "held by agent-b until
/// 14:30" waits or picks something else; an agent told "independence could not be
/// established" has found a modelling gap somebody needs to close.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimRefusal
{
    /// Somebody else holds overlapping territory.
    HeldBy
    {
        /// Who holds it.
        holder: String,
        /// When their lease lapses.
        until: Timestamp,
        /// The item they hold.
        item: ItemId,
    },
    /// Whether the two territories overlap could not be established.
    ///
    /// **Not** a permission to proceed. This is the arm that keeps "we could not tell"
    /// from becoming "go ahead".
    UnknownIndependence
    {
        /// The item whose territory could not be compared.
        against: ItemId,
        /// Why the comparison failed.
        reason: UnknownReason,
    },
    /// The requested lease exceeds [`MAXIMUM_LEASE`].
    LeaseTooLong
    {
        /// What was asked for.
        requested: Duration,
        /// The ceiling.
        maximum: Duration,
    },
    /// The item's holder is gone: the lease ran out and nobody renewed it.
    ///
    /// Distinct from [`ClaimRefusal::NotClaimable`] because the remedy is distinct and
    /// nameable. A `Done` item is a dead end; a lapsed one is takeable by anybody willing to
    /// say so, and a refusal that does not say which of the two it is sends the caller to
    /// read the JSON. `OD-LEDGER-005` is this repository's record of what a state word costs
    /// when it means something other than what a reader takes it to mean, and `claimed`
    /// covering both of these was the second instance.
    ///
    /// Not retryable, and that is the arm's point rather than an oversight: no amount of
    /// waiting turns a dead holder into a live one. Somebody has to decide to take the work.
    Lapsed
    {
        /// The item.
        item: ItemId,
        /// Who held it when the lease ran out.
        holder: String,
        /// When it ran out.
        since: Timestamp,
    },
    /// The item itself is held, and the operation would end somebody's live work.
    ///
    /// Distinct from [`ClaimRefusal::HeldBy`], which is about a *different* item whose
    /// territory overlaps the one asked for. Here the subject and the blocker are the same
    /// item, and a sentence about overlapping territory would be false about it — the
    /// mis-subject `OD-LEDGER-014` measured, in the one place reusing that arm would have
    /// reintroduced it.
    ///
    /// Retryable, and for a stronger reason than most: a claim is a lease, so it is released
    /// or it lapses. The remedy is named in the sentence because it is two commands and the
    /// first of them is one only the holder can run. `OD-LEDGER-001` is why it has to be
    /// theirs — territory is declared and not enforced, so the holder is the only party who
    /// knows whether the work is still running.
    StillHeld
    {
        /// The item.
        item: ItemId,
        /// Who holds it.
        holder: String,
        /// When their lease lapses.
        until: Timestamp,
    },
    /// The item is not in a state that can be claimed.
    NotClaimable
    {
        /// The item.
        item: ItemId,
        /// What state it is in.
        state: String,
    },
    /// Something this item depends on is not finished.
    ///
    /// A dependency edge that only `validate` reads is a comment. This is the arm that
    /// makes it a constraint, and it is retryable because finishing the dependency is
    /// what resolves it.
    DependencyUnmet
    {
        /// The item that was refused.
        item: ItemId,
        /// The dependency that is not done.
        dependency: ItemId,
        /// What state that dependency is in.
        state: String,
    },
    /// No such item.
    NoSuchItem
    {
        /// The identifier that matched nothing.
        item: ItemId,
    },
    /// The ledger itself could not be read or written.
    ///
    /// Nothing to do with the item, and that is why it is its own arm. Every load and save
    /// failure used to be reported as [`ClaimRefusal::NoSuchItem`], so an unreadable file,
    /// a parse error and an invalid document all told the operator their identifier was
    /// wrong — sending them to check a spelling while the ledger was broken. `OD-LEDGER-009`
    /// records it as the third instance of a reason not surviving the failure it explains.
    LedgerUnusable
    {
        /// What the store said, verbatim.
        cause: String,
    },
}

impl ClaimRefusal
{
    /// A one-line explanation a person or an agent can act on.
    ///
    /// # The refused item is never the grammatical subject here
    ///
    /// This is the form for a caller that has **not** already named the item it asked about,
    /// and every arm is phrased so that it composes under one that has. The distinction is
    /// not cosmetic: several arms carry the identifier of a *different* item — the blocker in
    /// [`ClaimRefusal::HeldBy`], the unfinished dependency in
    /// [`ClaimRefusal::DependencyUnmet`] — and a sentence that opens with one of those names
    /// reads as a statement about the wrong item.
    ///
    /// `OD-LEDGER-014` records the measurement. The held arm used to read
    /// `{blocker} overlaps territory held by {holder}`, which is true standing alone and says
    /// the reverse of what happened the moment a caller prints it beneath the subject's own
    /// identifier: `work audit` emitted `P1-MODEL: P9-AUTHORING overlaps territory held by …`
    /// for forty-four items, and no reader could tell which of the two names was refused.
    ///
    /// So a caller that has named its subject prints this straight after it, and one that has
    /// not still gets a sentence that claims nothing false. Anything needing a different
    /// phrasing reads the identifiers off the variant rather than writing a second rendering
    /// — that second rendering is what this record removed.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::HeldBy {
                holder,
                until,
                item,
            } => Held_By(item, holder, *until),
            Self::UnknownIndependence { against, reason } => Unknown_Independence(against, reason),
            Self::LeaseTooLong { requested, maximum } =>
            {
                format!("a lease of {requested:?} exceeds the {maximum:?} ceiling")
            }
            Self::Lapsed {
                item,
                holder,
                since,
            } => Lapsed(item, holder, *since),
            Self::StillHeld {
                item,
                holder,
                until,
            } => Still_Held(item, holder, *until),
            Self::NotClaimable { item, state } =>
            {
                format!("{item} is {state}, so the operation was refused")
            }
            Self::DependencyUnmet {
                item,
                dependency,
                state,
            } => format!("{item} depends on {dependency}, which is {state}"),
            Self::NoSuchItem { item } => format!("no item named {item}"),
            Self::LedgerUnusable { cause } =>
            {
                format!("the ledger could not be used: {cause}")
            }
        };
    }

    /// Whether retrying later might succeed.
    ///
    /// Distinguishes a queue from a dead end, which is what lets an agent decide
    /// between waiting and finding other work.
    #[must_use]
    pub const fn Is_Retryable(&self) -> bool
    {
        return matches!(
            self,
            Self::HeldBy { .. } | Self::DependencyUnmet { .. } | Self::StillHeld { .. }
        );
    }
}

/// Territory another live claim already covers.
fn Held_By(item: &ItemId, holder: &str, until: Timestamp) -> String
{
    return format!(
        "territory overlaps {item}, held by {holder} until unix {}",
        until.Unix_Seconds()
    );
}

/// Two territories the ledger cannot prove disjoint.
///
/// Refused rather than granted: unknown independence is not safe parallelism, and a claim
/// granted on a maybe is two sessions writing one path believing they are alone.
fn Unknown_Independence(against: &ItemId, reason: &UnknownReason) -> String
{
    return format!(
        "cannot establish independence from {against}: {}. Unknown independence is not safe \
         parallelism, so this claim is refused rather than granted",
        reason.Describe()
    );
}

/// A claim whose lease has run out, and what replaces it.
fn Lapsed(item: &ItemId, holder: &str, since: Timestamp) -> String
{
    return format!(
        "{item} was held by {holder} and the lease ran out at unix {}; `nomos work takeover` \
         replaces it and keeps {holder}'s claim on the item",
        since.Unix_Seconds()
    );
}

/// A live claim somebody else holds, and whose call it is to end it.
fn Still_Held(item: &ItemId, holder: &str, until: Timestamp) -> String
{
    return format!(
        "{item} is held by {holder} until unix {}; ending it is {holder}'s call — `nomos work \
         abandon` releases it and `nomos work decline` then ends it",
        until.Unix_Seconds()
    );
}

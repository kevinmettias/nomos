//! The one-line renderings behind [`super::ClaimRefusal::Describe`].
//!
//! Split out because none of this is part of the crate's public surface: every function
//! here is a private helper reached only through `Describe`, which keeps its own
//! documentation and signature in `refusal.rs` — the one file the crate's public-surface
//! reader resolves `ClaimRefusal` against.

use std::time::Duration;
use nomos_model::UnknownReason;
use crate::ItemId;
use nomos_platform::Timestamp;

/// Territory another live claim already covers.
pub(super) fn Held_By(item: &ItemId, holder: &str, until: Timestamp) -> String
{
    return format!(
        "territory overlaps {item}, held by {holder} until unix {}",
        until.Unix_Seconds()
    );
}

/// A dependency that will never finish, and the remedy that is not "wait".
pub(super) fn Dependency_Declined(item: &ItemId, dependency: &ItemId, state: &str) -> String
{
    return format!(
        "{item} depends on {dependency}, which is {state}; a declined dependency never becomes \
         Done, so retrying will not resolve this — {item} must be closed with `nomos work \
         decline` and re-authored against a dependency that can still finish"
    );
}

/// A requested lease past the ceiling coordination allows.
pub(super) fn Lease_Too_Long(requested: Duration, maximum: Duration) -> String
{
    return format!("a lease of {requested:?} exceeds the {maximum:?} ceiling");
}

/// An item whose current state refuses the operation outright.
pub(super) fn Not_Claimable(item: &ItemId, state: &str) -> String
{
    return format!("{item} is {state}, so the operation was refused");
}

/// A dependency that has not finished yet, and may still.
pub(super) fn Dependency_Unmet(item: &ItemId, dependency: &ItemId, state: &str) -> String
{
    return format!("{item} depends on {dependency}, which is {state}");
}

/// An identifier that matched nothing on the board.
pub(super) fn No_Such_Item(item: &ItemId) -> String
{
    return format!("no item named {item}");
}

/// The ledger's own store could not be read or written.
pub(super) fn Ledger_Unusable(cause: &str) -> String
{
    return format!("the ledger could not be used: {cause}");
}

/// Two territories the ledger cannot prove disjoint.
///
/// Refused rather than granted: unknown independence is not safe parallelism, and a claim
/// granted on a maybe is two sessions writing one path believing they are alone.
pub(super) fn Unknown_Independence(against: &ItemId, reason: &UnknownReason) -> String
{
    return format!(
        "cannot establish independence from {against}: {}. Unknown independence is not safe \
         parallelism, so this claim is refused rather than granted",
        reason.Describe()
    );
}

/// A claim whose lease has run out, and what replaces it.
pub(super) fn Lapsed_Claim(item: &ItemId, holder: &str, since: Timestamp) -> String
{
    return format!(
        "{item} was held by {holder} and the lease ran out at unix {}; `nomos work takeover` \
         replaces it and keeps {holder}'s claim on the item",
        since.Unix_Seconds()
    );
}

/// A live claim somebody else holds, and whose call it is to end it.
pub(super) fn Still_Held(item: &ItemId, holder: &str, until: Timestamp) -> String
{
    return format!(
        "{item} is held by {holder} until unix {}; ending it is {holder}'s call — `nomos work \
         abandon` releases it and `nomos work decline` then ends it",
        until.Unix_Seconds()
    );
}

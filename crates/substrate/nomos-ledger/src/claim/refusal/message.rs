//! The one-line renderings behind [`super::Refusal::Describe`].
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Held_By_Should_Name_The_Item_The_Holder_And_The_Lease()
    {
        let said = Held_By(&ItemId::New("T-1"), "agent-a", Timestamp::From_Unix_Seconds(2_000));

        assert!(said.contains("T-1"), "{said}");
        assert!(said.contains("agent-a"), "{said}");
        assert!(said.contains("2000"), "{said}");
    }

    #[test]
    fn Test_Dependency_Declined_Should_Name_The_Dependency_And_The_Remedy()
    {
        let said = Dependency_Declined(&ItemId::New("T-1"), &ItemId::New("T-2"), "declined");

        assert!(said.contains("T-1"), "{said}");
        assert!(said.contains("T-2"), "{said}");
        assert!(said.contains("nomos work decline"), "the remedy must be named: {said}");
    }

    #[test]
    fn Test_Lease_Too_Long_Should_Name_Both_The_Request_And_The_Ceiling()
    {
        let requested = Duration::from_secs(9_999_999);
        let maximum = Duration::from_secs(3_600);

        let said = Lease_Too_Long(requested, maximum);

        assert!(said.contains(&format!("{requested:?}")), "{said}");
        assert!(said.contains(&format!("{maximum:?}")), "{said}");
    }

    #[test]
    fn Test_Not_Claimable_Should_Name_The_Item_And_Its_State()
    {
        let said = Not_Claimable(&ItemId::New("T-1"), "Done");

        assert!(said.contains("T-1"), "{said}");
        assert!(said.contains("Done"), "{said}");
    }

    #[test]
    fn Test_Dependency_Unmet_Should_Name_The_Item_The_Dependency_And_Its_State()
    {
        let said = Dependency_Unmet(&ItemId::New("T-1"), &ItemId::New("T-2"), "Ready");

        assert!(said.contains("T-1"), "{said}");
        assert!(said.contains("T-2"), "{said}");
        assert!(said.contains("Ready"), "{said}");
    }

    #[test]
    fn Test_No_Such_Item_Should_Name_The_Identifier_That_Matched_Nothing()
    {
        let said = No_Such_Item(&ItemId::New("T-1"));

        assert!(said.contains("T-1"), "{said}");
    }

    #[test]
    fn Test_Ledger_Unusable_Should_Carry_The_Stores_Own_Cause()
    {
        let said = Ledger_Unusable("disk full");

        assert!(said.contains("disk full"), "{said}");
    }

    #[test]
    fn Test_Unknown_Independence_Should_Name_The_Item_And_Refuse_Rather_Than_Grant()
    {
        let said = Unknown_Independence(&ItemId::New("T-1"), &UnknownReason::IncomparableSnapshots);

        assert!(said.contains("T-1"), "{said}");
        assert!(said.contains("refused"), "must not read as a grant: {said}");
    }

    #[test]
    fn Test_Lapsed_Claim_Should_Name_The_Holder_And_The_Takeover_Remedy()
    {
        let said = Lapsed_Claim(&ItemId::New("T-1"), "dead-agent", Timestamp::From_Unix_Seconds(2_000));

        assert!(said.contains("T-1"), "{said}");
        assert!(said.contains("dead-agent"), "{said}");
        assert!(said.contains("takeover"), "the remedy must be named: {said}");
    }

    #[test]
    fn Test_Still_Held_Should_Name_The_Holder_And_Whose_Call_It_Is_To_End_It()
    {
        let said = Still_Held(&ItemId::New("T-1"), "agent-a", Timestamp::From_Unix_Seconds(2_000));

        assert!(said.contains("T-1"), "{said}");
        assert!(said.contains("agent-a"), "{said}");
        assert!(said.contains("abandon"), "the remedy must be named: {said}");
    }
}

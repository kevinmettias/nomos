//! What this module promises, exercised.

use super::*;

use crate::Blocker;
use crate::Claim;
use crate::Territory;
use crate::VerificationPredicate;
use nomos_platform::Timestamp;

fn Item(id: &str) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: "an item".to_owned(),
        why: "because".to_owned(),
        done_when: "when it is done".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: Territory::Empty(),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        declined: None,
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
        occupied.push(Assert_It_Serializes_As_Transcribed(&blocker));
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

/// The row of the corpus table one declared cause occupies, having checked that it serializes
/// as the name that table transcribes for it.
fn Assert_It_Serializes_As_Transcribed(blocker: &Blocker) -> usize
{
    let position = Corpus_Position(blocker);
    let (cause, transcribed) = *CORPUS_CAUSES
        .get(position)
        .expect("Corpus_Position returns an index into CORPUS_CAUSES");

    assert_eq!(
        transcribed,
        Some(Serialized_Tag(blocker).as_str()),
        "the variant declared for WORK-LEDGER-005's `{cause}` does not serialize as the \
         name CORPUS_CAUSES transcribes for it"
    );

    return position;
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

    assert!(item.Try_Replace_Lapsed_Claim(Claimed_By("agent-b", 9_000), At(2_001)));

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

    assert!(!live.Try_Replace_Lapsed_Claim(Claimed_By("agent-b", 9_000), At(1_999)));
    assert_eq!(
        live.claim.as_ref().map(|claim| return claim.holder.clone()),
        Some("agent-a".to_owned()),
        "a live holder was displaced"
    );
    assert!(live.displaced.is_empty());

    let mut hollow = Item("T-2");
    hollow.state = ItemState::Claimed;

    assert!(!hollow.Try_Replace_Lapsed_Claim(Claimed_By("agent-b", 9_000), At(2_001)));
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
    assert!(!VerificationPredicate::From_String_Arguments(Vec::new()).Is_Runnable());
    assert!(!VerificationPredicate::From_String_Arguments(vec![String::new()]).Is_Runnable());
    assert!(VerificationPredicate::From_String_Arguments(vec!["cargo".to_owned(), "test".to_owned()]).Is_Runnable());
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
        16,
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

/// Every kind `ItemKind` declares.
///
/// A named provider rather than an inline literal, so a sixth kind is a value added here
/// rather than a change to the loop that reads them.
fn Every_Declared_Item_Kind() -> [ItemKind; 5]
{
    return [
        ItemKind::Capability,
        ItemKind::Decision,
        ItemKind::Validation,
        ItemKind::Correction,
        ItemKind::Cleanup,
    ];
}

/// Every origin `ItemOrigin` declares.
///
/// A named provider rather than an inline literal, for the reason [`Every_Declared_Item_Kind`]
/// is one.
fn Every_Declared_Item_Origin() -> [ItemOrigin; 2]
{
    return [ItemOrigin::Required, ItemOrigin::Proposed];
}

/// `OD-LEDGER-024`'s closed set: a kind or an origin outside the five and two named
/// variants is refused rather than accepted and ignored, the same guarantee
/// `#[serde(deny_unknown_fields)]` gives an unrecognized *key* — this is the same promise
/// for an unrecognized *value*.
#[test]
fn Test_An_Unrecognized_Kind_Or_Origin_Should_Be_Refused()
{
    assert!(serde_json::from_value::<ItemKind>(serde_json::json!("Feature")).is_err());
    assert!(serde_json::from_value::<ItemOrigin>(serde_json::json!("Discovered")).is_err());

    // And every declared variant round-trips, or the refusal above would be trivially true
    // of a type nothing can construct either.
    for kind in Every_Declared_Item_Kind()
    {
        let value = serde_json::to_value(kind).expect("a kind serializes");
        assert_eq!(
            serde_json::from_value::<ItemKind>(value).expect("a kind round-trips"),
            kind
        );
    }
    for origin in Every_Declared_Item_Origin()
    {
        let value = serde_json::to_value(origin).expect("an origin serializes");
        assert_eq!(
            serde_json::from_value::<ItemOrigin>(value).expect("an origin round-trips"),
            origin
        );
    }
}

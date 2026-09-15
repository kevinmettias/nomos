use super::*;
use crate::{ItemKind, ItemOrigin, Territory};

/// The instant these tests work at, named so the number is a decision rather than a
/// value that happens to be spelled the same way in every fixture here.
const NOW_SECONDS: i64 = 1_000;

/// Independent defects the collecting fixture is built to carry: one duplicated
/// identifier and one workable item reserving nothing.
///
/// Named rather than written at the assertion so that the number is the fixture's own
/// count and not a second place the test has to be kept in step with it.
const DEFECTS_IN_THIS_FIXTURE: usize = 2;

#[test]
fn Test_Validate_Document_Should_Collect_Every_Violation_Not_Just_The_First()
{
    let now = Timestamp::From_Unix_Seconds(NOW_SECONDS);
    let duplicate_id = ItemId::New("DUP-1");
    let first = Workable_Item(duplicate_id.clone());
    let mut second = Workable_Item(duplicate_id);
    second.territory = Territory::Of_Files(["src/other.rs"]);
    let mut reserves_nothing = Workable_Item(ItemId::New("EMPTY-1"));
    reserves_nothing.territory = Territory::Empty();

    let document = LedgerDocument {
        schema_version: crate::SCHEMA_VERSION,
        items: vec![first, second, reserves_nothing],
    };

    let violations = Validate_Document(&document, now);

    assert!(violations.iter().any(|line| line.contains("more than once")), "{violations:?}");
    assert!(violations.iter().any(|line| line.contains("reserves nothing")), "{violations:?}");
    assert_eq!(violations.len(), DEFECTS_IN_THIS_FIXTURE, "exactly these two violations for this fixture, no more, no fewer: {violations:?}");
}

/// Two items naming each other is a ring nothing in it can leave.
///
/// Reported once for the ring, not once per member: a caller has one thing to break, and
/// two violations saying the same thing would read as two defects.
#[test]
fn Test_Two_Items_Depending_On_Each_Other_Should_Be_One_Violation()
{
    let document = Document(vec![Depending("A-1", &["B-1"]), Depending("B-1", &["A-1"])]);

    let cycles = Cycles_Among(&document);

    assert_eq!(cycles.len(), 1, "{cycles:?}");
    let ring = cycles.first().expect("asserted len 1 above");
    assert!(ring.contains("A-1") && ring.contains("B-1"), "{ring}");
}

/// A ring longer than two, which a check comparing pairs would miss entirely.
#[test]
fn Test_A_Longer_Ring_Should_Be_One_Violation_Naming_Every_Member()
{
    let document = Document(vec![
        Depending("C-1", &["D-1"]),
        Depending("D-1", &["E-1"]),
        Depending("E-1", &["C-1"]),
    ]);

    let cycles = Cycles_Among(&document);

    assert_eq!(cycles.len(), 1, "{cycles:?}");
    let ring = cycles.first().expect("asserted len 1 above");
    for member in ["C-1", "D-1", "E-1"]
    {
        assert!(ring.contains(member), "{member} missing from {ring}");
    }
}

/// An item naming itself.
///
/// Measured not to be covered by `Check_Dependencies`, which asks only whether the named
/// item is one the ledger holds -- and it is, it is this one. So it is a ring of one and
/// is reported as such, rather than left to a check that does not reach it.
#[test]
fn Test_An_Item_Depending_On_Itself_Should_Be_Reported_As_A_Ring_Of_One()
{
    let document = Document(vec![Depending("F-1", &["F-1"])]);

    let cycles = Cycles_Among(&document);

    assert_eq!(cycles.len(), 1, "{cycles:?}");
    let ring = cycles.first().expect("asserted len 1 above");
    assert!(ring.contains("F-1") && ring.contains("depends on itself"), "{ring}");
}

/// The negative control: two paths from one item down to another is not a ring.
///
/// A diamond arrives at `J-1` twice, so any check written on "have I been here before"
/// rather than on "do these reach each other" reports it. This is the shape that tells
/// those two apart, and a real board is full of it.
#[test]
fn Test_An_Acyclic_Diamond_Should_Not_Be_Reported()
{
    let document = Document(vec![
        Depending("G-1", &["H-1", "I-1"]),
        Depending("H-1", &["J-1"]),
        Depending("I-1", &["J-1"]),
        Depending("J-1", &[]),
    ]);

    let cycles = Cycles_Among(&document);

    assert!(cycles.is_empty(), "a diamond is not a cycle: {cycles:?}");
}

/// A widening row that names a path the territory does not reserve is a corruption.
///
/// Unreachable through `Widen`, which grows the territory and records the growth in one
/// operation. Reachable by a hand edit and by any future writer that does the two as two
/// statements, which is exactly the shape `OD-LEDGER-039` keeps these rows to avoid.
///
/// The falsifier for `Check_Widenings`. Without it the check is a guard nobody has watched
/// fail, which this repository counts as no guard at all.
#[test]
fn Test_A_Widening_Naming_A_Path_The_Territory_Lost_Should_Be_Reported()
{
    let mut item = Workable_Item(ItemId::New("K-1"));
    item.widened.push(crate::Widening {
        holder: "agent-a".to_owned(),
        added: vec!["src/b.rs".to_owned()],
        widened_at: Timestamp::From_Unix_Seconds(NOW_SECONDS),
    });

    let violations = Validate_Document(&Document(vec![item]), Timestamp::From_Unix_Seconds(NOW_SECONDS));

    assert!(
        violations.iter().any(|line| return line.contains("src/b.rs") && line.contains("come apart")),
        "a widening naming ground the territory does not hold must be reported: {violations:?}"
    );
}

/// The control: the same widening, on a territory that does reserve what it added.
///
/// Without this the check above is satisfied by a rule that reports every widening, which
/// would make a correctly widened board invalid -- and the board this lands on has one.
#[test]
fn Test_A_Widening_Whose_Paths_The_Territory_Reserves_Should_Be_Accepted()
{
    let mut item = Workable_Item(ItemId::New("K-1"));
    item.territory.paths.push("src/b.rs".to_owned());
    item.widened.push(crate::Widening {
        holder: "agent-a".to_owned(),
        added: vec!["src/b.rs".to_owned()],
        widened_at: Timestamp::From_Unix_Seconds(NOW_SECONDS),
    });

    let violations = Validate_Document(&Document(vec![item]), Timestamp::From_Unix_Seconds(NOW_SECONDS));

    assert!(
        violations.is_empty(),
        "a widening whose paths the territory holds is the ordinary case: {violations:?}"
    );
}

/// Every cycle violation `Validate_Document` reports over `document`, and nothing else.
fn Cycles_Among(document: &LedgerDocument) -> Vec<String>
{
    let now = Timestamp::From_Unix_Seconds(NOW_SECONDS);

    return Validate_Document(document, now)
        .into_iter()
        .filter(|line| return line.contains("cycle"))
        .collect();
}

fn Document(items: Vec<LedgerItem>) -> LedgerDocument
{
    return LedgerDocument { schema_version: crate::SCHEMA_VERSION, items };
}

/// A workable item that depends on the identifiers named.
fn Depending(id: &str, dependencies: &[&str]) -> LedgerItem
{
    let mut item = Workable_Item(ItemId::New(id));
    item.depends_on = dependencies.iter().map(|named| return ItemId::New(*named)).collect();

    return item;
}

fn Workable_Item(id: ItemId) -> LedgerItem
{
    return LedgerItem {
        id,
        title: "an item".to_owned(),
        why: "because".to_owned(),
        done_when: "when it is done".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: Territory::Of_Files(["src/a.rs"]),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
        declined: None,
    };
}
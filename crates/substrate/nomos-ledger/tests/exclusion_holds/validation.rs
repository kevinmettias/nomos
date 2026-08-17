//! What the ledger refuses to call a valid document.
//!
//! Every violation at once rather than the first one found: an author fixing a board wants
//! the list, and a validator that stops at the first fault turns one edit into several.

use crate::board::*;

#[test]
fn Test_Validate_Should_Refuse_Two_Active_Claims_On_Overlapping_Territory()
{
    let document = Document(vec![
        Held_By(Item("T-1", &["src/a.rs", "src/b.rs"]), "agent-a", NOW + 3_600),
        Held_By(Item("T-2", &["src/b.rs", "src/c.rs"]), "agent-b", NOW + 3_600),
    ]);

    let violations = Validate(&document, At(NOW));

    assert!(
        violations.iter().any(|violation| violation.contains("overlapping")),
        "overlapping active claims must be a violation, got: {violations:?}"
    );
}

/// The negative control. Remove the overlap and the same two claims must be fine — if
/// this fails, the rule above is firing on everything and proves nothing.
#[test]
fn Test_Validate_Should_Accept_Two_Active_Claims_On_Disjoint_Territory()
{
    let document = Document(vec![
        Held_By(Item("T-1", &["src/a.rs"]), "agent-a", NOW + 3_600),
        Held_By(Item("T-2", &["src/c.rs"]), "agent-b", NOW + 3_600),
    ]);

    assert_eq!(
        Validate(&document, At(NOW)),
        Vec::<String>::new(),
        "disjoint territory must not be reported as a conflict"
    );
}

/// A lapsed claim stops excluding. Two overlapping claims are fine when one of them
/// belongs to an agent that went away hours ago.
#[test]
fn Test_A_Lapsed_Claim_Should_Not_Conflict()
{
    let document = Document(vec![
        Held_By(Item("T-1", &["src/b.rs"]), "agent-a", NOW - 1),
        Held_By(Item("T-2", &["src/b.rs"]), "agent-b", NOW + 3_600),
    ]);

    let violations = Validate(&document, At(NOW));

    assert!(
        !violations.iter().any(|violation| violation.contains("overlapping")),
        "a lapsed claim must stop excluding, got: {violations:?}"
    );
}

/// Territory that cannot be compared must be reported, not passed over. This is the
/// arm that keeps an unanswered question from reading as an answer.
#[test]
fn Test_Incomparable_Territory_Should_Be_Reported_Not_Ignored()
{
    let mut second = Item("T-2", &["src/b.rs"]);
    second.territory.resolution = SetResolution::Symbol;

    let document = Document(vec![
        Held_By(Item("T-1", &["src/b.rs"]), "agent-a", NOW + 3_600),
        Held_By(second, "agent-b", NOW + 3_600),
    ]);

    let violations = Validate(&document, At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("cannot be shown independent")),
        "incomparable territory must be reported, got: {violations:?}"
    );
}

// ---------------------------------------------------------------------------
// Acceptance 2 — an item cannot be done without recorded verification.
// ---------------------------------------------------------------------------

#[test]
fn Test_A_Done_Item_Should_Require_Recorded_Verification()
{
    let mut finished = Item("T-1", &["src/a.rs"]);
    finished.state = ItemState::Done;

    let violations = Validate(&Document(vec![finished]), At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("no recorded verification")),
        "prose is not a predicate, got: {violations:?}"
    );
}

/// The negative control: with a verification record, the same item is fine.
#[test]
fn Test_A_Verified_Done_Item_Should_Be_Accepted()
{
    let mut finished = Item("T-1", &["src/a.rs"]);
    finished.state = ItemState::Done;
    finished.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned(), "test".to_owned()],
        exit_code: 0,
        output_tail: "test result: ok".to_owned(),
        verified_at: At(NOW),
        gate: None,
        revision: None,
    });

    assert_eq!(Validate(&Document(vec![finished]), At(NOW)), Vec::<String>::new());
}

#[test]
fn Test_An_Unrunnable_Predicate_Should_Be_Refused()
{
    let mut item = Item("T-1", &["src/a.rs"]);
    item.verification = Some(VerificationPredicate::New(Vec::new()));

    let violations = Validate(&Document(vec![item]), At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("cannot be run")),
        "an empty argv is a filled-in field, not a predicate, got: {violations:?}"
    );
}

#[test]
fn Test_A_Blocked_Item_Should_Say_Why()
{
    let mut item = Item("T-1", &["src/a.rs"]);
    item.state = ItemState::Blocked;

    let violations = Validate(&Document(vec![item]), At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("without saying why"))
    );
}

/// Every violation must be reported, not just the first. An author who has to re-run to
/// discover the next problem is an author who stops re-running.
#[test]
fn Test_Validation_Should_Report_Every_Violation_At_Once()
{
    let mut blocked = Item("T-1", &["src/a.rs"]);
    blocked.state = ItemState::Blocked;
    let mut done = Item("T-2", &["src/b.rs"]);
    done.state = ItemState::Done;
    let mut dangling = Item("T-3", &["src/c.rs"]);
    dangling.depends_on = vec![ItemId::New("T-99")];

    let violations = Validate(&Document(vec![blocked, done, dangling]), At(NOW));

    assert!(
        violations.len() >= 3,
        "expected every violation, got: {violations:?}"
    );
}

// ---------------------------------------------------------------------------
// Acceptance 3 — claiming through the ledger honours the same rules.
// ---------------------------------------------------------------------------

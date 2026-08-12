//! Taking an item, and the territory that taking it reserves.
//!
//! Exclusion is the whole point of a claim. A pattern reserves what it matches, a directory
//! reserves its subtree, and a refusal names what it collided with rather than who holds
//! it — because a board is shared and a refusal is not an accusation.

use crate::common::*;

#[test]
fn Test_Claiming_Overlapping_Territory_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At("claim-overlap", vec![
        Item("T-1", &["src/a.rs", "src/shared.rs"]),
        Item("T-2", &["src/shared.rs", "src/b.rs"]),
    ]);

    Take(&mut ledger, "T-1", "agent-a");

    let refusal = Refused(&mut ledger, "T-2", "agent-b");

    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }));
    assert!(refusal.Is_Retryable(), "a held item is a queue, not a wall");
    assert!(refusal.Describe().contains("agent-a"));
}

/// The negative control: disjoint territory claims concurrently, which is the entire
/// point of doing any of this.
#[test]
fn Test_Claiming_Disjoint_Territory_Should_Succeed_Concurrently()
{
    let (_directory, mut ledger) = Board_At("claim-disjoint", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["src/b.rs"]),
    ]);

    Take(&mut ledger, "T-1", "agent-a");
    Take(&mut ledger, "T-2", "agent-b");

    ledger.Validate_Current().expect("both claims are legitimate");
}

/// What an unexpanded pattern does to the board, pinned as measured rather than as argued.
///
/// `P10-PATTERN-BRICK` found this by reading `territory.rs` rather than from an incident: a
/// single entry in `patterns` short-circuits `Territory::Intersect` to `Unknown` before a
/// single path is compared, and `Unknown` refuses non-retryably.
///
/// # The item's own description of this was one clause too strong, and the correction matters
///
/// `P10-PATTERN-BRICK` says the item "can never be claimed by anyone, its own holder
/// included". Measured here, that is not what happens, because `Conflicts` compares only
/// against items holding an **active claim**. So a pattern item on a quiet board claims
/// perfectly normally — asserted below, because it is the step that makes the rest possible.
///
/// The real shape is worse than an item nobody can take, and this is the finding:
///
/// 1. the pattern item is claimable exactly when the board is quiet, so nothing warns the
///    agent who takes it;
/// 2. from that moment every other claim is refused against it, including territory sharing
///    no path with it at all;
/// 3. and the refusal is the non-retryable one, which by `README.md`'s exit-code contract
///    tells each refused agent to stop and fetch a person rather than pick up another item.
///
/// So one agent quietly acquires the power to stop every other session, and learns nothing
/// about having done so. `OD-LEDGER-013` withdrew `--territory-pattern` on the strength of
/// this. The state stays reachable by hand-editing the document, which is why this test can
/// still construct it, and why the `Unknown` in `Territory::Intersect` is kept rather than
/// relaxed: withdrawing the flag removes the way in, not the guard.
#[test]
fn Test_A_Held_Pattern_Should_Refuse_Every_Other_Claim_On_The_Board()
{
    let (_directory, mut ledger) = Board_At("pattern-brick", vec![
        Patterned("T-1", &["src/a.rs"], "crates/spec/**"),
        Item("T-2", &["docs/unrelated.md"]),
        Item("T-3", &["tests/also-unrelated.rs"]),
    ]);
    // 1. It claims without complaint. Nothing is held yet, so nothing is compared, so the
    //    pattern is never consulted. This is the step the item's description missed.
    Take(&mut ledger, "T-1", "agent-a");

    // 2. And now the board is shut. `docs/unrelated.md` shares nothing with `src/a.rs` or
    //    with `crates/spec/**`, and is refused anyway — the short-circuit runs before any
    //    path is looked at, so being unrelated is no defence.
    for (item, holder) in [("T-2", "agent-b"), ("T-3", "agent-c")]
    {
        let collateral = Refused(&mut ledger, item, holder);
        Is_Collateral_Damage(item, &collateral);
    }
}

/// A claim refused for no reason of its own: unanswerable rather than contended, and
/// non-retryable, which is what tells the agent to stop and fetch a person. One held pattern
/// therefore reads to every other session as a broken ledger.
fn Is_Collateral_Damage(item: &str, refusal: &ClaimRefusal)
{
    assert!(
        matches!(refusal, ClaimRefusal::UnknownIndependence { .. }),
        "{item}: {refusal:?}"
    );
    assert!(!refusal.Is_Retryable(), "{item}: {}", refusal.Describe());
}

/// And it shuts in the other direction too, once anything at all is held.
///
/// The complement of the test above, and together they are why the state has no safe
/// ordering: claim the pattern first and it stops everyone else; claim anything else first
/// and the pattern item can never be taken. There is no sequence in which the board both
/// carries a pattern and keeps working.
#[test]
fn Test_A_Pattern_Item_Should_Be_Unclaimable_Once_Anything_Is_Held()
{
    let (_directory, mut ledger) = Board_At("pattern-brick-reverse", vec![
        Item("T-1", &["docs/unrelated.md"]),
        Patterned("T-2", &["src/b.rs"], "crates/spec/**"),
    ]);
    Take(&mut ledger, "T-1", "agent-a");

    let refused = Refused(&mut ledger, "T-2", "agent-b");

    assert!(matches!(refused, ClaimRefusal::UnknownIndependence { .. }), "{refused:?}");
    assert!(
        !refused.Is_Retryable(),
        "and waiting will not help: `docs/unrelated.md` is disjoint from `src/b.rs`, so the \
         refusal is not contention and no lease expiring resolves it"
    );
}

/// A refusal printed under the refused item's own identifier must not read as a statement
/// about the blocker.
///
/// `P10-AUDIT-STATE` measured the failure: `work audit` prints one line per blocked item, the
/// identifier first, and the held arm of `Describe` used to open with the *blocker's* name.
/// Forty-four lines read `P1-MODEL: P9-AUTHORING overlaps territory held by …`, in which the
/// only thing a reader can be sure of is that one of the two names was refused, and nothing
/// says which. `OD-LEDGER-014` moved the phrasing into the library.
///
/// The assertion is positional rather than a substring search, because a substring search is
/// what a wrong sentence also passes: both spellings contain both identifiers, and only the
/// order distinguishes them.
#[test]
fn Test_A_Refusal_Should_Not_Open_With_The_Blockers_Name()
{
    let (_directory, mut ledger) = Board_At("refusal-subject", vec![
        Item("T-BLOCKER", &["src/shared.rs"]),
        Item("T-REFUSED", &["src/shared.rs"]),
    ]);

    Take(&mut ledger, "T-BLOCKER", "agent-a");

    Reads_As_A_Statement_About_The_Refused_Item(
        &Refused(&mut ledger, "T-REFUSED", "agent-b").Describe(),
    );
}

/// The three things the sentence has to do, and the line `work audit` composes from it.
fn Reads_As_A_Statement_About_The_Refused_Item(sentence: &str)
{
    // The whole defect in one assertion: the blocker's name must not be the first thing the
    // sentence says. Restoring `{item} overlaps territory held by {holder} …` makes this red
    // and leaves every other assertion in this file green, which is what makes it the control
    // for this arm rather than a restatement of the ones above.
    assert!(
        !sentence.starts_with("T-BLOCKER"),
        "the refusal opens with the blocker's name, so printed under the refused item's own \
         identifier it says the reverse of what happened: {sentence}"
    );

    // And it still has to say who is in the way, or the fix would have been to delete the
    // information rather than to place it. The composed line is what `work audit` prints:
    // read as English its subject is `T-REFUSED`, and `T-BLOCKER` is what the territory runs
    // into.
    let line = format!("{:<13} {:<9} {sentence}", "T-REFUSED", "held");
    assert!(sentence.contains("T-BLOCKER"), "must still name the blocker: {sentence}");
    assert!(sentence.contains("agent-a"), "and who holds it: {sentence}");
    assert!(
        line.starts_with("T-REFUSED"),
        "the caller names the subject and the description follows it: {line}"
    );
}

/// One rendering, not two.
///
/// `work audit` carried its own copy of the held arm while the library's was wrong. Both are
/// now the library's, and this pins the sentence the CLI composes so the workaround cannot
/// quietly come back as a local `format!` that drifts from this one.
#[test]
fn Test_The_Held_Arm_Should_Have_Exactly_One_Rendering()
{
    let refusal = ClaimRefusal::HeldBy {
        holder: "agent-a".to_owned(),
        until: At(NOW + 3_600),
        item: ItemId::New("T-BLOCKER"),
    };

    assert_eq!(
        refusal.Describe(),
        format!("territory overlaps T-BLOCKER, held by agent-a until unix {}", NOW + 3_600)
    );
}

/// The negative control for the test above: the same two items, minus the pattern.
///
/// Without this, `Test_An_Unexpanded_Pattern_Should_Brick_Every_Claim_On_The_Board` would
/// pass just as happily if claiming were broken for some entirely unrelated reason, and the
/// record would be citing a measurement of nothing. One line differs between the two.
#[test]
fn Test_The_Same_Board_Without_The_Pattern_Should_Claim_Freely()
{
    let (_directory, mut ledger) = Board_At("pattern-brick-control", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["docs/unrelated.md"]),
    ]);

    Take(&mut ledger, "T-1", "agent-a");
    Take(&mut ledger, "T-2", "agent-b");
}

/// A directory reserves what is beneath it, which is what the withdrawn flag was for.
///
/// This is the load-bearing half of `OD-LEDGER-013`: withdrawing `--territory-pattern` is
/// only a narrowing if it removed something an author could otherwise say. It did not.
/// "Everything under `crates/spec`" is an ordinary territory entry, it is decided from the
/// text with no filesystem access, and — unlike the pattern — it answers.
#[test]
fn Test_A_Directory_Should_Reserve_Its_Subtree_Without_A_Pattern()
{
    let (_directory, mut ledger) = Board_At("subtree-without-pattern", vec![
        Item("T-1", &["crates/spec"]),
        Item("T-2", &["crates/spec/nomos-spec-model/src/lib.rs"]),
        Item("T-3", &["crates/host/nomos-cli/src/work.rs"]),
    ]);
    Take(&mut ledger, "T-1", "agent-a");

    let refusal = Refused(&mut ledger, "T-2", "agent-b");
    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "the subtree is *held*, not unanswerable — the distinction is the whole record: \
         {refusal:?}"
    );
    assert!(
        refusal.Is_Retryable(),
        "and it is a queue rather than a wall, which the pattern never was"
    );
    // While genuinely unrelated territory is still free, so the directory entry reserves a
    // subtree rather than the repository.
    Take(&mut ledger, "T-3", "agent-c");
}

// ---------------------------------------------------------------------------
// Acceptance 5 — finishing runs the predicate, and believes it.
// ---------------------------------------------------------------------------



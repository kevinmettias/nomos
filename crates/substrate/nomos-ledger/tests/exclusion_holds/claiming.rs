//! Taking an item, and the territory that taking it reserves.
//!
//! Exclusion is the whole point of a claim. A pattern reserves what it matches, a directory
//! reserves its subtree, and a refusal names what it collided with rather than who holds
//! it — because a board is shared and a refusal is not an accusation.

use crate::board::{
    At, Board, Board_At, Board_Written_By_Hand, ClaimRefusal, Claimant, FileLedger, FileLock, Item, ItemId,
    LEASE_ENDS_AT, Patterned, Refused, StdFileSystem, Take,
};

#[test]
fn Test_Claiming_Overlapping_Territory_Should_Be_Refused()
{
    let Board { directory: _directory, mut ledger } = Board_At("claim-overlap", vec![
        Item("T-1", &["src/a.rs", "src/shared.rs"]),
        Item("T-2", &["src/shared.rs", "src/b.rs"]),
    ]);

    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let refusal = Refused(&mut ledger, "T-2", Claimant("agent-b"));

    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }));
    assert!(refusal.Is_Retryable(), "a held item is a queue, not a wall");
    assert!(refusal.Describe().contains("agent-a"));
}

/// The negative control: disjoint territory claims concurrently, which is the entire
/// point of doing any of this.
#[test]
fn Test_Claiming_Disjoint_Territory_Should_Succeed_Concurrently()
{
    let Board { directory: _directory, mut ledger } = Board_At("claim-disjoint", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["src/b.rs"]),
    ]);

    Take(&mut ledger, "T-1", Claimant("agent-a"));
    Take(&mut ledger, "T-2", Claimant("agent-b"));

    ledger.Validate_Current().expect("both claims are legitimate");
}

/// What an unexpanded pattern does to the board, pinned as measured rather than as argued.
///
/// `P10-PATTERN-BRICK` found this by reading `territory.rs` rather than from an incident: a
/// single entry in `patterns` short-circuits `Territory::Intersect` to `Unknown` before a
/// single path is compared, and `Unknown` refuses non-retryably. At the time that record was
/// written, `Validate` did not look at `patterns` at all, so a pattern item on a quiet board
/// claimed perfectly normally, and only *then* did every other claim start failing — one
/// agent quietly acquired the power to stop every other session, and learned nothing about
/// having done so. `OD-LEDGER-013` named that gap and left it for `Validate` to close.
///
/// `P13-VALIDATE-PATTERN-REFUSAL` closes it: `Validate` now refuses any document whose
/// territory carries a pattern, and every verb ends by saving the whole document, so the
/// quiet first claim is gone. The board is `LedgerUnusable` from the moment the pattern
/// lands, not from the moment somebody happens to claim it — no ordering makes it safe, and
/// now no ordering makes it *quiet* either. The state stays reachable by hand-editing the
/// document, which is why this test still constructs it directly rather than through `Save`.
/// The three items on the pattern-bricked board, and who tries to claim each.
///
/// A named provider rather than an inline literal, so a fourth claimant is a value added
/// here rather than a change to the loop that reads them.
const PATTERN_BRICKED_BOARD_CLAIMANTS: usize = 3;

fn Claimants_Of_The_Pattern_Bricked_Board() -> [(&'static str, &'static str); PATTERN_BRICKED_BOARD_CLAIMANTS]
{
    return [("T-1", "agent-a"), ("T-2", "agent-b"), ("T-3", "agent-c")];
}

#[test]
fn Test_A_Pattern_Anywhere_On_The_Board_Should_Refuse_Every_Claim_As_Ledger_Unusable()
{
    let Board { directory: _directory, mut ledger } = Board_Written_By_Hand("pattern-brick", vec![
        Patterned("T-1", &["src/a.rs"], "crates/spec/**"),
        Item("T-2", &["docs/unrelated.md"]),
        Item("T-3", &["tests/also-unrelated.rs"]),
    ]);

    // Every item is refused the same way, including the pattern item itself and territory
    // sharing no path with it at all: the refusal is about the document, not about what any
    // one claim would have compared against. A claim refused for no reason of its own is
    // non-retryable, which is what tells the agent to stop and fetch a person rather than
    // wait.
    for (item, holder) in Claimants_Of_The_Pattern_Bricked_Board()
    {
        let refusal = Refused(&mut ledger, item, Claimant(holder));
        assert!(
            matches!(refusal, ClaimRefusal::LedgerUnusable { .. }),
            "{item}: {refusal:?}"
        );
        assert!(!refusal.Is_Retryable(), "{item}: {}", refusal.Describe());
    }
}

/// The same finding regardless of where the pattern item sits in the document, so the
/// refusal is provably about the pattern and not about position.
#[test]
fn Test_A_Pattern_Item_Should_Refuse_Every_Claim_Wherever_It_Sits()
{
    let Board { directory: _directory, mut ledger } = Board_Written_By_Hand("pattern-brick-reverse", vec![
        Item("T-1", &["docs/unrelated.md"]),
        Patterned("T-2", &["src/b.rs"], "crates/spec/**"),
    ]);

    let refused = Refused(&mut ledger, "T-1", Claimant("agent-a"));

    assert!(matches!(refused, ClaimRefusal::LedgerUnusable { .. }), "{refused:?}");
    assert!(
        !refused.Is_Retryable(),
        "and waiting will not help: no lease expiring repairs an invalid document"
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
    let Board { directory: _directory, mut ledger } = Board_At("refusal-subject", vec![
        Item("T-BLOCKER", &["src/shared.rs"]),
        Item("T-REFUSED", &["src/shared.rs"]),
    ]);

    Take(&mut ledger, "T-BLOCKER", Claimant("agent-a"));

    let sentence = Refused(&mut ledger, "T-REFUSED", Claimant("agent-b")).Describe();

    // The whole defect in one assertion: the blocker's name must not be the first thing the
    // sentence says. Restoring `{item} overlaps territory held by {holder} …` makes this red
    // and leaves every other assertion in this file green, which is what makes it the control
    // for this arm rather than a restatement of the ones above.
    assert!(
        !sentence.starts_with("T-BLOCKER"),
        "the refusal opens with the blocker's name, so printed under the refused item's own \
         identifier it says the reverse of what happened: {sentence}"
    );
    Names_The_Blocker_After_The_Subject(&sentence);
}

/// It still has to say who is in the way, or the fix would have been to delete the
/// information rather than to place it. The composed line is what `work audit` prints:
/// read as English its subject is `T-REFUSED`, and `T-BLOCKER` is what the territory runs
/// into.
fn Names_The_Blocker_After_The_Subject(sentence: &str)
{
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
        until: At(LEASE_ENDS_AT),
        item: ItemId::New("T-BLOCKER"),
    };

    assert_eq!(
        refusal.Describe(),
        format!("territory overlaps T-BLOCKER, held by agent-a until unix {}", LEASE_ENDS_AT)
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
    let Board { directory: _directory, mut ledger } = Board_At("pattern-brick-control", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["docs/unrelated.md"]),
    ]);

    Take(&mut ledger, "T-1", Claimant("agent-a"));
    Take(&mut ledger, "T-2", Claimant("agent-b"));

    ledger
        .Validate_Current()
        .expect("neither claim collided, so the document stays valid without the pattern");

    Both_Claims_Are_Recorded(&ledger);
}

/// Both claims must have actually landed, or this is not the control it claims to be.
fn Both_Claims_Are_Recorded<Clock: nomos_platform::Clock>(
    ledger: &FileLedger<StdFileSystem, Clock, FileLock>,
)
{
    let document = ledger.Load().expect("the ledger holding both accepted claims reads back");
    let claimed: Vec<&str> = document
        .items
        .iter()
        .filter(|item| return item.claim.is_some())
        .map(|item| return item.id.As_Text())
        .collect();
    assert_eq!(
        claimed,
        vec!["T-1", "T-2"],
        "both claims must have actually landed, or this is not the control it claims to be"
    );
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
    let Board { directory: _directory, mut ledger } = Board_At("subtree-without-pattern", vec![
        Item("T-1", &["crates/spec"]),
        Item("T-2", &["crates/spec/nomos-spec-model/src/lib.rs"]),
        Item("T-3", &["crates/host/nomos-cli/src/work.rs"]),
    ]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let refusal = Refused(&mut ledger, "T-2", Claimant("agent-b"));
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
    Take(&mut ledger, "T-3", Claimant("agent-c"));
}

// ---------------------------------------------------------------------------
// Acceptance 5 — finishing runs the predicate, and believes it.
// ---------------------------------------------------------------------------



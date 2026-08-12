//! `work list` as an agent meets it: the real binary, a real ledger, the real column.
//!
//! P9-DEPENDS-ON is about one word. `ready` meant "nobody holds this" and was read as "I
//! can take this", and the two came apart whenever a dependency was unfinished or somebody
//! held overlapping ground. Measured on 2026-08-09: the column said `ready` for eight items
//! that a single held claim refused, three separate times.
//!
//! These tests run the program rather than calling a function, because the defect was never
//! in the exclusion logic — `claim` always refused correctly. It was in what the listing
//! *said* about what claiming would do, so the assertion has to be made on the output a
//! person and an agent actually read.

use std::path::PathBuf;
use std::process::Command;

const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// Far enough ahead that a lease written here is live whenever the suite runs.
const FOREVER: i64 = 4_102_444_800;

struct Board
{
    root: PathBuf,
}

impl Board
{
    fn New(name: &str, items: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-list-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        std::fs::write(
            root.join("ledger.json"),
            format!("{{\n  \"schema_version\": 1,\n  \"items\": [{items}]\n}}\n"),
        )
        .expect("a scratch ledger");
        return Self { root };
    }

    /// Runs `nomos work list`, returning stdout.
    fn List(&self, filter: Option<&str>) -> String
    {
        let mut command = Command::new(NOMOS);
        command.arg("work").arg("list");
        if let Some(state) = filter
        {
            command.arg("--state").arg(state);
        }
        let output = command
            .env("NOMOS_WORK_DIR", &self.root)
            .output()
            .expect("the binary runs");
        return String::from_utf8_lossy(&output.stdout).into_owned();
    }

    /// The label `work list` prints for one item.
    fn Label_Of(&self, item: &str) -> String
    {
        let listing = self.List(None);
        let line = listing
            .lines()
            .find(|line| line.starts_with(item))
            .unwrap_or_else(|| panic!("{item} must appear in the listing:\n{listing}"));
        return line
            .split_whitespace()
            .nth(1)
            .unwrap_or_default()
            .to_owned();
    }

    /// Runs `nomos work audit`, returning stdout.
    fn Audit(&self) -> String
    {
        let output = Command::new(NOMOS)
            .arg("work")
            .arg("audit")
            .env("NOMOS_WORK_DIR", &self.root)
            .output()
            .expect("the binary runs");
        return String::from_utf8_lossy(&output.stdout).into_owned();
    }
}

/// The audit's line for one item, or `None` if it did not answer for that item.
fn Audit_Line<'a>(audit: &'a str, item: &str) -> Option<&'a str>
{
    return audit
        .lines()
        .find(|line| line.split_whitespace().next() == Some(item));
}

impl Drop for Board
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// Where an item stands: what state it is in, what it waits on, and the claim and
/// verification fields that close it out.
#[derive(Clone, Copy)]
struct Standing<'a>
{
    state: &'a str,
    depends_on: &'a str,
    tail: &'a str,
}

/// One item, authored inline so each test's board is readable in the test.
fn Item(id: &str, paths: &str, standing: Standing<'_>) -> String
{
    let Standing { state, depends_on, tail } = standing;

    return format!(
        "{{\"id\":\"{id}\",\"title\":\"item {id}\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"territory\":{{\"resolution\":\"File\",\"paths\":[{paths}],\"patterns\":[]}},\
         \"state\":\"{state}\",\"depends_on\":[{depends_on}],\"blocked\":null,{tail}}}"
    );
}

const NO_CLAIM: &str = "\"claim\":null,\"verification\":null,\"verified\":null";

/// A `Done` item must carry its verification or the ledger is invalid.
const FINISHED: &str = "\"claim\":null,\"verification\":null,\"verified\":{\"argv\":[\"cargo\",\
                        \"test\"],\"exit_code\":0,\"output_tail\":\"ok\",\"verified_at\":1000000,\
                        \"gate\":null}";

// ---------------------------------------------------------------------------
// The defect: an unfinished dependency is not readiness.
// ---------------------------------------------------------------------------

#[test]
fn Test_An_Item_With_An_Unfinished_Dependency_Should_Not_Be_Listed_Ready()
{
    let board = Board::New(
        "unfinished-dependency",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-1\"",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(
        board.Label_Of("T-2"),
        "waiting",
        "T-2 depends on an unfinished T-1, so nothing can claim it:\n{}",
        board.List(None)
    );
}

/// The negative control that stops the new label being printed for everything.
///
/// Same board, same shape, no dependency. If this said `waiting` too, the test above would
/// pass while the column had simply stopped saying `ready` at all — which is the same defect
/// with the sign flipped.
#[test]
fn Test_An_Item_With_No_Dependency_Should_Still_Be_Listed_Ready()
{
    let board = Board::New(
        "no-dependency",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(board.Label_Of("T-2"), "ready");
    assert_eq!(board.Label_Of("T-1"), "ready");
}

/// The second control, and the sharper one. The dependency is *present* and *satisfied*, so
/// a label driven by "does this item have a `depends_on` entry" would get it wrong while
/// passing both tests above.
#[test]
fn Test_A_Satisfied_Dependency_Should_Leave_An_Item_Ready()
{
    let board = Board::New(
        "satisfied-dependency",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Done",
                depends_on: "",
                tail: FINISHED,
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-1\"",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(
        board.Label_Of("T-2"),
        "ready",
        "T-1 is Done, so T-2 is genuinely claimable:\n{}",
        board.List(None)
    );
}

/// Filtering has to agree with the column, or an agent asking for work still gets items it
/// cannot take. This is the query the defect was actually costing round trips on.
#[test]
fn Test_Filtering_For_Ready_Should_Not_Return_An_Item_Nothing_Can_Claim()
{
    let board = A_Ready_Item_Behind_An_Unfinished_One();

    let ready = board.List(Some("ready"));

    assert!(ready.contains("T-1"), "T-1 is claimable:\n{ready}");
    assert!(
        !ready.contains("T-2"),
        "T-2 cannot be claimed, so it must not answer a request for ready work:\n{ready}"
    );
}

/// Two `Ready` items, the second of which depends on the first and so cannot be taken.
fn A_Ready_Item_Behind_An_Unfinished_One() -> Board
{
    return Board::New(
        "filter-ready",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-1\"",
                tail: NO_CLAIM,
            })
        ),
    );
}

// ---------------------------------------------------------------------------
// The same word was lying about territory too.
// ---------------------------------------------------------------------------

/// Held ground is the other way `ready` was false, and it is the one that was measured
/// widest: eight items, one claim. It costs nothing extra to report, because it comes from
/// the same refusal the dependency case does.
#[test]
fn Test_An_Item_On_Held_Ground_Should_Not_Be_Listed_Ready()
{
    let board = Board::New(
        "held-ground",
        &format!(
            "{},{}",
            Item("T-1", "\"src/shared.rs\"", Standing {
                state: "Claimed",
                depends_on: "",
                tail: &Held_By("agent-a"),
            }),
            Item("T-2", "\"src/shared.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(
        board.Label_Of("T-2"),
        "held",
        "agent-a holds overlapping ground:\n{}",
        board.List(None)
    );
}

/// The control for the one above. Move the territory apart and the same board reports the
/// same item as claimable.
#[test]
fn Test_An_Item_On_Free_Ground_Should_Still_Be_Listed_Ready()
{
    let board = Board::New(
        "free-ground",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Claimed",
                depends_on: "",
                tail: &Held_By("agent-a"),
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            })
        ),
    );

    assert_eq!(board.Label_Of("T-2"), "ready");
}

// ---------------------------------------------------------------------------
// P10-AUDIT-STATE: `audit` answers the same question, so it must give the same answer.
//
// `list` and `audit` are two readings of one rule — what would refuse a claim right now.
// `audit` had its own implementation of it, `ExclusionLedger::Conflicts`, which compares
// territory and knows nothing about state or `depends_on`. It therefore answered for
// finished work nobody is queued behind, and said nothing at all about the one refusal an
// agent looking for work most needs to hear. These tests hold the two readings together:
// the board below carries a terminal item, a dependency-blocked item and a
// territory-blocked item at once, which is the only shape in which both halves of the
// disagreement are visible in a single command.
// ---------------------------------------------------------------------------

/// A live claim, held far enough ahead that it is live whenever the suite runs.
fn Held_By(holder: &str) -> String
{
    return format!(
        "\"claim\":{{\"holder\":\"{holder}\",\"acquired_at\":1000000,\
         \"lease_expires_at\":{FOREVER}}},\"verification\":null,\"verified\":null"
    );
}

/// The board both halves of the defect are visible on.
///
/// * `T-1` holds `src/shared.rs`, so `T-2` is refused for territory.
/// * `T-3` depends on an unfinished `T-4`, so it is refused for a dependency — the refusal
///   `Conflicts` cannot express.
/// * `T-4` is claimable, and is the control against a report that prints everything.
/// * `T-5` is `Done` on the *same* held ground as `T-2`. Territory alone cannot tell them
///   apart, which is exactly how the old audit came to describe finished work as blocked.
fn Mixed_Board(name: &str) -> Board
{
    return Board::New(
        name,
        &format!(
            "{},{},{},{},{}",
            Item("T-1", "\"src/shared.rs\"", Standing {
                state: "Claimed",
                depends_on: "",
                tail: &Held_By("agent-a"),
            }),
            Item("T-2", "\"src/shared.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-3", "\"src/c.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-4\"",
                tail: NO_CLAIM,
            }),
            Item("T-4", "\"src/d.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-5", "\"src/shared.rs\"", Standing {
                state: "Done",
                depends_on: "",
                tail: FINISHED,
            })
        ),
    );
}

/// The pin. One board, one command, both halves of the disagreement asserted at once.
///
/// It goes red if a terminal item comes back into the report, and it goes red if a
/// dependency refusal drops out of it. Those are the two failures the old implementation
/// had simultaneously, and either one alone is the defect returning.
#[test]
fn Test_Audit_Should_Report_Every_Live_Refusal_And_Only_Those()
{
    let board = Mixed_Board("audit-mixed");

    let audit = board.Audit();

    Assert_Both_Live_Refusals_Are_Reported(&audit);

    assert!(
        Audit_Line(&audit, "T-5").is_none(),
        "T-5 is done; nobody is queued behind it, so it must be absent rather than \
         sorted below:\n{audit}"
    );
    assert!(
        Audit_Line(&audit, "T-4").is_none(),
        "T-4 is claimable, so the audit has nothing to say about it:\n{audit}"
    );
    assert!(
        Audit_Line(&audit, "T-1").is_none(),
        "T-1 is the holder, not a blocked item:\n{audit}"
    );
}

/// Territory and dependency are two refusals the board has simultaneously, and either one
/// alone is the defect returning.
fn Assert_Both_Live_Refusals_Are_Reported(audit: &str)
{
    let held = Audit_Line(audit, "T-2")
        .unwrap_or_else(|| panic!("T-2 is on ground agent-a holds:\n{audit}"));
    let waiting = Audit_Line(audit, "T-3").unwrap_or_else(|| {
        panic!(
            "T-3 is refused for an unfinished dependency, which the audit used to be \
             silent about:\n{audit}"
        )
    });

    assert!(
        held.contains("held") && held.contains("T-1") && held.contains("agent-a"),
        "the territory refusal must name what holds the ground and who:\n{held}"
    );
    assert!(
        waiting.contains("waiting") && waiting.contains("T-4"),
        "the dependency refusal must name the dependency:\n{waiting}"
    );
}

/// The audit and the listing must use the same word for the same item.
///
/// The line count was only how the disagreement was noticed. This is the disagreement
/// itself: two guards for one rule, asserted against each other rather than against a
/// number. Any item the audit answers for is one the listing must already call unclaimable,
/// with the same label.
#[test]
fn Test_Audit_Should_Agree_With_The_Listing_Item_For_Item()
{
    let board = Mixed_Board("audit-agrees");

    let audit = board.Audit();

    let mut checked = 0_u32;
    for line in audit.lines()
    {
        let named = Assert_The_Listing_Agrees(&board, line, &audit);
        checked = checked.saturating_add(named);
    }

    assert_eq!(
        checked, 2,
        "the board has exactly two blocked items; a different count means the audit is \
         answering for something else:\n{audit}"
    );
}

/// One audit line against what the listing calls the same item. Answers 1 where a line named
/// an item at all, which is what the count above measures.
fn Assert_The_Listing_Agrees(board: &Board, line: &str, audit: &str) -> u32
{
    let mut fields = line.split_whitespace();
    let (Some(item), Some(label)) = (fields.next(), fields.next())
    else
    {
        return 0;
    };

    assert_eq!(
        board.Label_Of(item),
        label,
        "`audit` and `list` disagree about {item}:\n{audit}"
    );

    return 1;
}

/// The control against a repair that simply prints less.
///
/// Every assertion above is satisfied by an `audit` that has stopped reporting anything at
/// all except two hardcoded lines. Move the ground apart and finish the dependency, and the
/// same command must fall silent — and say so, because an empty report and a report that
/// cannot express the blockage look identical otherwise.
#[test]
fn Test_Audit_Should_Say_So_When_Nothing_Is_Blocked()
{
    let board = A_Board_Where_Nothing_Refuses();

    let audit = board.Audit();

    assert!(
        Audit_Line(&audit, "T-2").is_none() && Audit_Line(&audit, "T-3").is_none(),
        "nothing on this board refuses a claim:\n{audit}"
    );
    assert!(
        audit.contains("nothing"),
        "an empty report must say it found nothing rather than print nothing:\n{audit}"
    );
}

/// The same four items as the mixed board, with the ground moved apart and the dependency
/// finished, so that nothing on it refuses a claim.
fn A_Board_Where_Nothing_Refuses() -> Board
{
    return Board::New(
        "audit-clear",
        &format!(
            "{},{},{},{}",
            Item("T-1", "\"src/a.rs\"", Standing {
                state: "Claimed",
                depends_on: "",
                tail: &Held_By("agent-a"),
            }),
            Item("T-2", "\"src/b.rs\"", Standing {
                state: "Ready",
                depends_on: "",
                tail: NO_CLAIM,
            }),
            Item("T-3", "\"src/c.rs\"", Standing {
                state: "Ready",
                depends_on: "\"T-4\"",
                tail: NO_CLAIM,
            }),
            Item("T-4", "\"src/d.rs\"", Standing {
                state: "Done",
                depends_on: "",
                tail: FINISHED,
            })
        ),
    );
}

//! A lapsed item is taken over, and what the takeover displaced is readable afterwards.
//!
//! `P10-LAPSE-TAKEOVER`, decided in `OD-LEDGER-012`. The unit tests in `nomos-ledger` assert
//! that the displaced claim lands on the item; these assert the half that makes it worth
//! landing. `OD-LEDGER-006`'s rule is that a kept record must be *reported* — a record only
//! somebody willing to read the JSON can find is most of the way back to not keeping it — so
//! `work show` printing the predecessor is part of the decision rather than a convenience.
//!
//! Run against the real binary through `CARGO_BIN_EXE_nomos`, never by shelling out to `cargo`:
//! a nested `cargo test` deadlocks on the target lock, which `OD-GATE-002` records.
//!
//! The lapse is reached by writing a lease that is already in the past rather than by waiting.
//! These tests drive the CLI, which reads the wall clock, so an expiry expressed in absolute
//! unix seconds is the only way to hold the condition still — and a suite that slept two hours
//! to reach it would be a suite nobody runs.

use nomos_ledger::SCHEMA_VERSION;
use std::path::PathBuf;
use std::process::Command;

const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// Well before any wall clock this will run against, so `T-1`'s lease has run out.
const HELD_FROM: i64 = 1_000_000;
/// One hour after [`HELD_FROM`], and still in the past.
const LAPSED_AT: i64 = 1_003_600;
/// Far enough ahead that `T-3`'s claim is live whenever this suite runs. Unix seconds in 2096.
const STILL_LIVE: i64 = 4_000_000_000;

/// What one `nomos work` run said, and what it exited with.
///
/// Named rather than a pair. The same two values were spelled `(String, i32)` in one of
/// these files and `(i32, String)` in the next, which is exactly the swap a name makes
/// impossible and a type does not.
struct Ran
{
    said: String,
    code: i32,
}

struct Board
{
    root: PathBuf,
}

impl Board
{
    /// A scratch board with one lapsed item, one free item and one somebody is still holding.
    ///
    /// Written as JSON rather than built with `work add` and `work claim`, because the claim
    /// this needs is one no honest `claim` would ever write: its lease ran out hours ago.
    ///
    /// No `.github/workflows/gate.yml` beside it. `finish` derives the gate's lint step from
    /// that file and refuses when it cannot, and nothing here finishes anything.
    fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-takeover-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        std::fs::write(
            root.join("ledger.json"),
            format!(
                "{{\n  \"schema_version\": {SCHEMA_VERSION},\n  \"items\": [\
                 {{\"id\":\"T-1\",\"title\":\"the item its holder died on\",\
                 \"why\":\"because\",\"done_when\":\"the tests pass\",\
                 \"territory\":{{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\
                 \"patterns\":[]}},\
                 \"state\":\"Claimed\",\"depends_on\":[],\"blocked\":null,\
                 \"claim\":{{\"holder\":\"dead-agent\",\"acquired_at\":{HELD_FROM},\
                 \"lease_expires_at\":{LAPSED_AT}}},\
                 \"verification\":null,\"verified\":null,\"abandoned\":[]}},\
                 {{\"id\":\"T-2\",\"title\":\"an item nobody has taken\",\
                 \"why\":\"because\",\"done_when\":\"the tests pass\",\
                 \"territory\":{{\"resolution\":\"File\",\"paths\":[\"src/b.rs\"],\
                 \"patterns\":[]}},\
                 \"state\":\"Ready\",\"depends_on\":[],\"blocked\":null,\"claim\":null,\
                 \"verification\":null,\"verified\":null,\"abandoned\":[]}},\
                 {{\"id\":\"T-3\",\"title\":\"an item somebody is still working\",\
                 \"why\":\"because\",\"done_when\":\"the tests pass\",\
                 \"territory\":{{\"resolution\":\"File\",\"paths\":[\"src/c.rs\"],\
                 \"patterns\":[]}},\
                 \"state\":\"Claimed\",\"depends_on\":[],\"blocked\":null,\
                 \"claim\":{{\"holder\":\"agent-a\",\"acquired_at\":{HELD_FROM},\
                 \"lease_expires_at\":{STILL_LIVE}}},\
                 \"verification\":null,\"verified\":null,\"abandoned\":[]}}\
                 ]\n}}\n"
            ),
        )
        .expect("a scratch ledger");
        return Self { root };
    }

    /// Runs `nomos work …`, returning what it said and what it exited with.
    fn Work(&self, arguments: &[&str]) -> Ran
    {
        let output = Command::new(NOMOS)
            .arg("work")
            .args(arguments)
            .env("NOMOS_WORK_DIR", &self.root)
            .output()
            .expect("the binary runs");

        return Ran {
            said: String::from_utf8_lossy(&output.stdout).into_owned(),
            code: output.status.code().unwrap_or(-1),
        };
    }

    /// The ledger file as text, for asserting on what was written rather than on what was said.
    fn Written(&self) -> String
    {
        return std::fs::read_to_string(self.root.join("ledger.json")).expect("readable");
    }
}

impl Drop for Board
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// The subject, end to end: the verb takes the item and the file says whom it displaced.
///
/// Asserted on the file and not only on the exit code, for the reason `OD-LEDGER-008`'s tests
/// are: a takeover that returned 0 while dropping the predecessor would satisfy every assertion
/// about output and would be the defect intact.
#[test]
fn Test_Taking_Over_A_Lapsed_Item_Should_Record_The_Claim_It_Displaced()
{
    let board = Board::New("records-what-it-displaced");

    let Ran { said, code } = board.Work(&["takeover", "--item", "T-1", "--holder", "agent-b"]);

    let written = board.Written();

    assert_eq!(code, 0, "a lapsed item must be takeable: {said}");
    assert!(said.contains("agent-b"), "{said}");
    Assert_The_Displaced_Claim_Survived(&written);
}

/// Who was displaced and when their lease ran out both have to survive the takeover, or
/// nothing says whose work this was.
///
/// The field is written on every item, empty or not. That is what makes `grep -c
/// '"displaced"'` against the item count a usable check that no stale writer has been through
/// the file, and a field that appeared only when populated could not be counted.
fn Assert_The_Displaced_Claim_Survived(written: &str)
{
    assert!(
        written.contains("dead-agent"),
        "the displaced holder is not in the file, so nothing says whose work this was:\n{written}"
    );
    assert!(
        written.contains(&format!("\"lease_expires_at\": {LAPSED_AT}")),
        "when the lease ran out must survive the takeover:\n{written}"
    );
    assert_eq!(
        written.matches("\"displaced\"").count(),
        3,
        "`displaced` must be emitted for every item, not only the one that has any:\n{written}"
    );
}

/// The record is reported, not merely stored.
///
/// `list` is a column per item and cannot carry prose, which is why `show` exists and why
/// `OD-LEDGER-006` treats reporting as part of keeping. Both timestamps are asserted because
/// the holder alone does not say what they held: an agent reading this has to be able to tell a
/// predecessor who worked for an hour from one who claimed and vanished.
#[test]
fn Test_Show_Should_Report_The_Claim_A_Takeover_Displaced()
{
    let board = Board::New("show-reports-it");

    let Ran { code, .. } = board.Work(&["takeover", "--item", "T-1", "--holder", "agent-b"]);
    assert_eq!(code, 0);

    let Ran { said, code } = board.Work(&["show", "--item", "T-1"]);

    assert_eq!(code, 0, "{said}");
    assert!(
        said.contains(&format!(
            "taken over from dead-agent, who held it from unix {HELD_FROM} until unix {LAPSED_AT}"
        )),
        "`show` must report the claim the takeover displaced:\n{said}"
    );
    assert!(
        said.contains("held by agent-b"),
        "`show` must still report the live claim:\n{said}"
    );
}

/// The listing and the verb tell the same truth, which is the pairing rather than either half.
///
/// `OD-LEDGER-005`'s defect was a column saying one thing while claiming did another, and
/// `lapsed` is the first word added to that column since. It now comes from `Claim_Refusal` —
/// the same function the takeover's own refusal comes from — so an item the listing calls
/// `lapsed` is exactly an item `takeover` accepts.
#[test]
fn Test_A_Lapsed_Item_Should_List_As_Lapsed_And_Be_Takeable()
{
    let board = Board::New("lists-lapsed-and-takes");

    let Ran { said: listed, code } = board.Work(&["list", "--state", "lapsed"]);

    assert_eq!(code, 0, "{listed}");
    assert!(
        listed.contains("T-1"),
        "an item whose holder's lease ran out must list as `lapsed`:\n{listed}"
    );
    assert!(
        !listed.contains("T-2") && !listed.contains("T-3"),
        "a free item and a held item are not lapsed:\n{listed}"
    );

    let Ran { said, code } = board.Work(&["takeover", "--item", "T-1", "--holder", "agent-b"]);
    assert_eq!(
        code, 0,
        "the listing said `lapsed` and the takeover refused it, so the two disagree: {said}"
    );
}

/// `claim` never silently becomes a takeover, and says so in a way the caller can act on.
///
/// Silence here is the defect `OD-LEDGER-009` guarded against: the record would exist and
/// nothing would make the agent creating it notice. The refusal has to name both the holder it
/// would displace and the verb that does it, because `NotClaimable`'s "T-1 is Claimed" was true
/// and left the operator with no next step.
#[test]
fn Test_Claiming_A_Lapsed_Item_Should_Refuse_And_Name_The_Takeover()
{
    let board = Board::New("claim-refuses-lapsed");

    let Ran { said, code } = board.Work(&["claim", "--item", "T-1", "--holder", "agent-b"]);

    assert_eq!(
        code, 4,
        "a lapsed item is a conflict a person decides, not a queue to retry: {said}"
    );
    assert!(said.contains("dead-agent"), "{said}");
    assert!(
        said.contains("takeover"),
        "the refusal must name the remedy:\n{said}"
    );

    assert!(
        !board.Written().contains("agent-b"),
        "a refused claim wrote a holder onto the board"
    );
}

/// The two exit codes a caller branches on, pinned.
///
/// `3` is retryable and means wait — the lease running out is what resolves it, so an agent that
/// tried to take live work should try again later or pick something else. `4` means a person
/// reads it: the item is not lapsed at all and no amount of waiting makes it so. No new code was
/// added for this verb, which is why the README's table does not move.
#[test]
fn Test_Takeover_Should_Exit_Three_When_The_Lease_Is_Live_And_Four_When_It_Is_Not_Held()
{
    let board = Board::New("exit-codes");

    let Ran { said: live, code } = board.Work(&["takeover", "--item", "T-3", "--holder", "agent-b"]);
    assert_eq!(
        code, 3,
        "taking over a live claim is a queue, not a conflict: {live}"
    );
    assert!(live.contains("agent-a"), "{live}");

    let Ran { said: free, code } = board.Work(&["takeover", "--item", "T-2", "--holder", "agent-b"]);
    assert_eq!(
        code, 4,
        "taking over an item nobody holds is the wrong verb, and waiting will not fix it: {free}"
    );

    let Ran { said: missing, code } = board.Work(&["takeover", "--item", "T-9", "--holder", "agent-b"]);
    assert_eq!(code, 4, "{missing}");

    assert!(
        !board.Written().contains("agent-b"),
        "a refused takeover wrote to the board"
    );
}

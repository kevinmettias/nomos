//! What `work add` actually guarantees, asserted rather than described.
//!
//! `P10-ADD-PROMISE` is about one sentence. The doc comment on `Add` said it refused an item
//! "whose territory is already spoken for", and it never did: the territory comparison in
//! `Validate` runs only between items holding an active claim, and a brand new item holds
//! none. `OD-LEDGER-010` decided the sentence was wrong rather than the behaviour, and these
//! are the assertions that hold the corrected sentence in place — one per clause of it.
//!
//! They run the binary rather than calling `Add`, for the reason `list_tells_the_truth.rs`
//! gives: the defect was never in the logic, it was in what the surface *said* about the
//! logic, and a caller who is misled reaches this program through its command line.
//!
//! # The one thing here that is not a test
//!
//! Nothing below can go red because the comment drifts back. A doc comment is prose, and the
//! only mechanism in this workspace that judges prose against behaviour is
//! `nomos_rules::mirror`, which reads declared *universes* and not free-standing functions.
//! What these buy is that the corrected sentence is true and stays true, and that a reader
//! who distrusts it finds the answer in the same crate. `OD-LEDGER-010` says so in terms
//! rather than letting this file imply more than it does.

use std::path::PathBuf;
use std::process::Command;

const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// Far enough ahead that a lease written here is live whenever the suite runs.
const FOREVER: i64 = 4_102_444_800;

/// A ledger of this test's own, under the target directory rather than the repository.
/// What one `nomos work` run exited with, and what it said.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
struct Ran
{
    code: i32,
    said: String,
}

struct Board
{
    root: PathBuf,
}

impl Board
{
    fn New(name: &str, items: &str) -> Self
    {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("add-{name}"));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        std::fs::write(
            root.join("ledger.json"),
            format!("{{\n  \"schema_version\": 1,\n  \"items\": [{items}]\n}}\n"),
        )
        .expect("a scratch ledger");
        return Self { root };
    }

    /// Runs `nomos work <arguments>`, returning what it exited with and what it said.
    fn Work(&self, arguments: &[&str]) -> Ran
    {
        let output = Command::new(NOMOS)
            .arg("work")
            .args(arguments)
            .env("NOMOS_WORK_DIR", &self.root)
            .output()
            .expect("the binary runs");

        return Ran {
            code: output.status.code().unwrap_or(-1),
            said: String::from_utf8_lossy(&output.stdout).into_owned(),
        };
    }

    /// The ledger file exactly as it sits on disk.
    ///
    /// Compared byte for byte rather than by re-reading the items, because the claim under
    /// test is that a refused `add` writes *nothing* — a rewrite that happens to preserve
    /// the item list is still the write `Save` is documented not to have performed.
    fn On_Disk(&self) -> String
    {
        return std::fs::read_to_string(self.root.join("ledger.json")).expect("reads the ledger");
    }

    /// The label `work list` prints for one item, or `None` if it is not listed at all.
    fn Label_Of(&self, item: &str) -> Option<String>
    {
        let Ran { said: listing, .. } = self.Work(&["list"]);

        return listing
            .lines()
            .find(|line| line.starts_with(item))
            .and_then(|line| line.split_whitespace().nth(1))
            .map(str::to_owned);
    }
}

impl Drop for Board
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// One item, authored inline so each test's board is readable in the test.
fn Item(id: &str, paths: &str, state: &str, tail: &str) -> String
{
    return format!(
        "{{\"id\":\"{id}\",\"title\":\"item {id}\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"territory\":{{\"resolution\":\"File\",\"paths\":[{paths}],\"patterns\":[]}},\
         \"state\":\"{state}\",\"depends_on\":[],\"blocked\":null,{tail}}}"
    );
}

/// A live claim, held far enough ahead that it is live whenever the suite runs.
fn Held_By(holder: &str) -> String
{
    return format!(
        "\"claim\":{{\"holder\":\"{holder}\",\"acquired_at\":1000000,\
         \"lease_expires_at\":{FOREVER}}},\"verification\":null,\"verified\":null"
    );
}

const NO_CLAIM: &str = "\"claim\":null,\"verification\":null,\"verified\":null";

/// The arguments that add one item, less its territory and dependencies.
fn Add_Arguments(id: &str) -> Vec<String>
{
    return vec![
        "add".to_owned(),
        "--item".to_owned(),
        id.to_owned(),
        "--title".to_owned(),
        format!("item {id}"),
        "--why".to_owned(),
        "because".to_owned(),
        "--done-when".to_owned(),
        "the tests pass".to_owned(),
    ];
}

/// `nomos work add` for `id`, with whatever trailing arguments the test needs.
fn Add(board: &Board, id: &str, rest: &[&str]) -> Ran
{
    let mut arguments = Add_Arguments(id);
    arguments.extend(rest.iter().map(|argument| return (*argument).to_owned()));

    let borrowed: Vec<&str> = arguments
        .iter()
        .map(std::string::String::as_str)
        .collect();

    return board.Work(&borrowed);
}

/// A board with `T-1` held by `agent-a` on ground a second item would want.
fn Held_Ground(name: &str) -> Board
{
    let claim = Held_By("agent-a");
    let item = Item("T-1", "\"src/shared.rs\"", "Claimed", &claim);

    return Board::New(name, &item);
}

// ---------------------------------------------------------------------------
// The clause the old sentence got wrong.
// ---------------------------------------------------------------------------

/// Held ground does not refuse an `add`, and it is not supposed to.
///
/// This is the behaviour the old comment denied, asserted directly. Both items name
/// `src/shared.rs`, `agent-a` holds the first under a live lease, and the second lands.
#[test]
fn Test_Adding_An_Item_On_Held_Ground_Should_Be_Accepted()
{
    let board = Held_Ground("held-ground");

    let Ran { code, said: message } = Add(&board, "T-2", &["--territory", "src/shared.rs"]);

    assert_eq!(
        code, 0,
        "agent-a holds src/shared.rs and T-2 reserves it; opening the item is still \
         legitimate — OD-LEDGER-010:\n{message}"
    );
    assert!(
        board.On_Disk().contains("T-2"),
        "the command reported success and wrote nothing:\n{}",
        board.On_Disk()
    );
}

/// The other half of that decision, and the reason it is a decision rather than a hole.
///
/// The exclusion is not absent, it is deferred to the verb that needs it. Without this, the
/// test above reads as a ledger that waves two agents onto one file — which is the reading
/// the old sentence was written to prevent, and the reason deleting it alone would not have
/// been enough.
#[test]
fn Test_The_Item_Added_On_Held_Ground_Should_Still_Be_Refused_A_Claim()
{
    let board = Held_Ground("deferred-exclusion");

    assert_eq!(Add(&board, "T-2", &["--territory", "src/shared.rs"]).code, 0);

    let Ran { code, said: message } = board.Work(&["claim", "--item", "T-2", "--holder", "agent-b"]);

    assert_eq!(
        code, 3,
        "agent-b must be refused, and told to try another item rather than fetch a \
         human:\n{message}"
    );
    assert!(
        message.contains("T-1") && message.contains("agent-a"),
        "the refusal must name what holds the ground and who:\n{message}"
    );
    assert_eq!(
        board.Label_Of("T-2").as_deref(),
        Some("held"),
        "an item nothing can claim must not be listed as claimable"
    );
}

/// The control against reading the two tests above as "claiming is broken too".
///
/// Same board, same `add`, territory moved apart — and the added item claims. If this were
/// red, the pair above would be satisfied by a ledger that refuses every claim.
#[test]
fn Test_An_Item_Added_On_Free_Ground_Should_Claim()
{
    let board = Held_Ground("free-ground");

    assert_eq!(Add(&board, "T-2", &["--territory", "src/other.rs"]).code, 0);

    assert_eq!(
        board.Label_Of("T-2").as_deref(),
        Some("ready"),
        "T-2 shares no ground with T-1"
    );
    assert_eq!(
        board
            .Work(&["claim", "--item", "T-2", "--holder", "agent-b"])
            .code,
        0
    );
}

// ---------------------------------------------------------------------------
// The two clauses the corrected sentence does promise.
// ---------------------------------------------------------------------------

/// A duplicate identifier is refused, and refused as a conflict rather than as a retry.
///
/// Exit 4 and not 3: a second item under a name already taken is not a race somebody wins
/// by waiting, and the exit code contract is what agents branch on.
#[test]
fn Test_A_Duplicate_Identifier_Should_Be_Refused()
{
    let item = Item("T-1", "\"src/a.rs\"", "Ready", NO_CLAIM);
    let board = Board::New("duplicate-id", &item);
    let before = board.On_Disk();

    let Ran { code, said: message } = Add(&board, "T-1", &["--territory", "src/b.rs"]);

    assert_eq!(code, 4, "a taken identifier is a conflict, not a retryable one:\n{message}");
    assert!(message.contains("T-1"), "the refusal must name the identifier:\n{message}");
    assert_eq!(before, board.On_Disk(), "a refused add rewrote the ledger");
}

/// An item that would break an invariant never lands, and nothing is written when it is
/// refused.
///
/// `--depends-on` naming an item that is not on the ledger is the invariant reachable from
/// the command line without inventing one: `Validate` reports it, `Save` refuses, and the
/// file is untouched. That last part is the clause about validating *before* the write, and
/// it is asserted on the bytes because an item list that happens to match is not evidence
/// that no write occurred.
#[test]
fn Test_An_Item_That_Would_Invalidate_The_Document_Should_Not_Land()
{
    let item = Item("T-1", "\"src/a.rs\"", "Ready", NO_CLAIM);
    let board = Board::New("invalid-dependency", &item);
    let before = board.On_Disk();

    let Ran { code, said: message } = Add(
        &board,
        "T-2",
        &["--territory", "src/b.rs", "--depends-on", "T-9"],
    );

    assert_eq!(code, 1, "a document that would not validate is a validation error:\n{message}");
    assert!(
        message.contains("T-9"),
        "the violation must name what is missing, or the author cannot act on it:\n{message}"
    );
    assert_eq!(
        before,
        board.On_Disk(),
        "the item was written and found invalid afterwards, which leaves every agent \
         reading a ledger the system says is wrong"
    );
}

/// The same guarantee over a violation the item itself carries rather than a reference.
///
/// Two spellings of one path is `Territory::Ambiguous_Paths` — an author who believed they
/// were reserving two things. It is here because the dependency case above could be
/// satisfied by a check on `depends_on` alone, and the promise is about the document
/// validating, not about one field.
#[test]
fn Test_An_Item_Reserving_One_Subject_Twice_Should_Not_Land()
{
    let item = Item("T-1", "\"src/a.rs\"", "Ready", NO_CLAIM);
    let board = Board::New("ambiguous-territory", &item);
    let before = board.On_Disk();

    let Ran { code, said: message } = Add(
        &board,
        "T-2",
        &["--territory", "src/b.rs", "--territory", "SRC/B.rs"],
    );

    assert_eq!(code, 1, "two spellings of one subject is a violation:\n{message}");
    assert!(
        message.contains("src/b.rs") && message.contains("SRC/B.rs"),
        "the violation must name both spellings, or this passed on some other \
         violation:\n{message}"
    );
    assert_eq!(before, board.On_Disk(), "a refused add rewrote the ledger");
}

/// The control for both refusals above: the same board and the same item, well formed.
///
/// Without it, every assertion in this section is satisfied by an `add` that has stopped
/// accepting anything at all.
#[test]
fn Test_A_Well_Formed_Item_Should_Land()
{
    let item = Item("T-1", "\"src/a.rs\"", "Ready", NO_CLAIM);
    let board = Board::New("well-formed", &item);

    let Ran { code, said: message } = Add(
        &board,
        "T-2",
        &["--territory", "src/b.rs", "--depends-on", "T-1"],
    );

    assert_eq!(code, 0, "{message}");
    assert!(message.contains("T-2"), "the acceptance must name what it added:\n{message}");
    assert_eq!(board.Label_Of("T-2").as_deref(), Some("waiting"));
}

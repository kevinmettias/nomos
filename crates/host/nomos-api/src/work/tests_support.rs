//! Scratch-board fixtures shared by more than one `Handle_Work_*` test module. Declared
//! `#[cfg(test)]` by `crate::work`'s own `mod tests_support;`, so nothing here compiles into
//! a real build.
//!
//! Every fixture hands its failure back rather than unwrapping its own steps. A fixture is
//! not the code under test: a scratch board the machine will not let it write says nothing
//! about the `Handle_Work_*` verb that later reads it, and only the caller knows which claim
//! it was proving when that write failed.

use nomos_ledger::ItemId;
use nomos_work_orchestration::{ClaimRequest, EndingRequest};

/// The ledger schema version every fixture board here is written at, and the version a test
/// reading one of them back asserts against.
const LEDGER_SCHEMA_VERSION: u32 = 6;

/// The one-hour lease every `Claim`/`Renew`/`TakeOver` fixture reaches for -- the fixture
/// literal `check-interfile-duplication` flagged as structurally repeated across `claim.rs`,
/// `renew.rs` and `take_over.rs` once `Handle_Work_Claim`/`Handle_Work_Renew`/
/// `Handle_Work_TakeOver` themselves stopped duplicating their own ledger/territory/launcher
/// wiring.
const CLAIM_LEASE: std::time::Duration = std::time::Duration::from_secs(3600);

/// The area every scratch directory in this module is named under, the same way
/// `crate::test_support`'s own callers name theirs.
const WORK_AREA: crate::test_support::Area = crate::test_support::Area("work");

/// A `ClaimRequest` for `item`, held by `holder`, with [`CLAIM_LEASE`].
pub(crate) fn Claim_Request(item: ItemId, holder: &str) -> ClaimRequest
{
    return ClaimRequest { item, holder: holder.to_owned(), lease: CLAIM_LEASE };
}

/// An `EndingRequest` for `item`, held by `"test-holder"`, with `reason` naming why -- the
/// fixture literal `check-interfile-duplication` flagged as structurally repeated across
/// `abandon_response.rs` and `decline_response.rs`'s own test suites.
pub(crate) fn Ending_Request(item: ItemId, reason: &str) -> EndingRequest
{
    return EndingRequest { item, holder: "test-holder".to_owned(), reason: reason.to_owned() };
}

/// A scratch directory of this test's own -- never the real shared `work/` directory, which
/// live sessions write to concurrently. A thin delegator onto `crate::test_support`'s own
/// `Unique_Scratch_Directory`, which carries the real `temp_dir`/counter/create-and-clean
/// body shared by both areas; `"work"` names this crate's `work` area the same way
/// `crate::test_support`'s own callers name theirs.
///
/// Private, because the directory it hands back sits directly in the machine's temp
/// directory, and a board put there reads that whole directory as its repository. A board
/// goes through [`Owned_Board`] instead.
///
/// # Errors
///
/// Returns whatever [`std::fs::create_dir_all`] refuses -- a temp directory that is not
/// writable, or a file already sitting where this call's own name computes to.
fn Unique_Scratch_Directory(label: &str) -> Result<std::path::PathBuf, std::io::Error>
{
    return crate::test_support::Unique_Scratch_Directory(WORK_AREA, label);
}

/// What a board directory is called inside a tree a test here created to hold it -- the
/// name the real board's directory has under the repository it sits in.
const BOARD_DIRECTORY: &str = "work";

/// An empty board directory, and the tree created to hold it.
#[derive(Debug)]
pub(crate) struct OwnedBoard
{
    /// The directory created to stand for the repository the board sits in, and the only
    /// thing a caller removes: the board is inside it, so removing it leaves nothing the
    /// fixture made behind.
    pub tree: std::path::PathBuf,
    /// The board directory, directly under `tree`.
    pub directory: std::path::PathBuf,
}

/// A board directory in a tree of this test's own, with nothing written into either yet.
///
/// Every fixture here builds its board through this, never directly in the machine's temp
/// directory. Two verbs take the repository to be the board directory's parent: the audit
/// checks reserved paths there and the add lists `docs/records` there. With the board
/// straight under `temp_dir()` that parent was the whole temp directory, so whatever some
/// other program left there -- files named `a` and `b` did, once -- decided those tests'
/// verdicts. Here the parent is a directory this call created, and it holds only the board.
///
/// # Errors
///
/// Returns whatever the scratch directory or the board directory refuses.
pub(crate) fn Owned_Board(label: &str) -> Result<OwnedBoard, std::io::Error>
{
    let tree = Unique_Scratch_Directory(label)?;
    let directory = tree.join(BOARD_DIRECTORY);
    std::fs::create_dir(&directory)?;

    return Ok(OwnedBoard { tree, directory });
}

/// A scratch board with no items, for the `List`, `Validate` and `Audit` tests that need
/// only a well-formed, empty ledger -- the same minimal two-key shape (`schema_version`,
/// `items`) `work/ledger.json` itself has.
///
/// # Errors
///
/// Returns whatever the owned board or the ledger write refuses.
pub(crate) fn Scratch_Board() -> Result<OwnedBoard, std::io::Error>
{
    let board = Owned_Board("list")?;
    std::fs::write(board.directory.join("ledger.json"), "{\"schema_version\": 6, \"items\": []}\n")?;

    return Ok(board);
}

/// A scratch board and the one item `Scratch_Board_With_One_Item` wrote into it.
///
/// Named fields rather than the unnamed pair this fixture used to hand back, which is a shape
/// a caller could destructure in the wrong order without the compiler objecting.
#[derive(Debug)]
pub(crate) struct BoardWithOneItem
{
    /// The tree the board sits in, as [`OwnedBoard::tree`] is.
    pub tree: std::path::PathBuf,
    /// The scratch directory holding `ledger.json`, directly under `tree`.
    pub directory: std::path::PathBuf,
    /// The id of the one item the board carries.
    pub id: ItemId,
}

/// A scratch board carrying one real item, for the `Show` tests -- `Scratch_Board`'s own
/// empty board proves nothing about a found item.
///
/// # Errors
///
/// Returns whatever the owned board or the ledger write refuses.
pub(crate) fn Scratch_Board_With_One_Item() -> Result<BoardWithOneItem, std::io::Error>
{
    let OwnedBoard { tree, directory } = Owned_Board("show")?;
    let id = ItemId::New("SCRATCH-ITEM");
    let ledger = format!(
        "{{\"schema_version\": 6, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
         \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"widened\": [], \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [], \"patterns\": []}}, \"state\": \"Ready\"}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger)?;

    return Ok(BoardWithOneItem { tree, directory, id });
}

/// A scratch board and the two ids `Scratch_Board_With_A_Blocked_Item` wrote into it.
#[derive(Debug)]
pub(crate) struct BoardWithABlockedItem
{
    /// The tree the board sits in, as [`OwnedBoard::tree`] is.
    pub tree: std::path::PathBuf,
    /// The scratch directory holding `ledger.json`, directly under `tree`.
    pub directory: std::path::PathBuf,
    /// The item the other one depends on, `Ready` and unclaimed.
    pub dependency: ItemId,
    /// The item whose `depends_on` names `dependency`.
    pub blocked: ItemId,
}

/// A scratch board carrying two real items, one blocking the other -- for the `Audit` tests.
/// `dependency` is `Ready` and unclaimed, so `blocked` earns a real `ClaimRefusal::
/// DependencyUnmet` from `nomos_ledger::Claim_Refusal` rather than a fixture-only refusal
/// this crate invented. Its items reserve `a` and `b`, which the audit looks for in the
/// board's own tree -- the reason this fixture was the first to move into one.
///
/// # Errors
///
/// Returns whatever the owned board or the ledger write refuses.
pub(crate) fn Scratch_Board_With_A_Blocked_Item() -> Result<BoardWithABlockedItem, std::io::Error>
{
    let OwnedBoard { tree, directory } = Owned_Board("audit")?;
    let dependency = ItemId::New("SCRATCH-DEPENDENCY");
    let blocked = ItemId::New("SCRATCH-BLOCKED");
    let ledger = format!(
        "{{\"schema_version\": 6, \"items\": [{{\"id\": \"{dependency}\", \"title\": \"t\", \
         \"why\": \"w\", \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \
         \"Proposed\", \"widened\": [], \"territory\": {{\"resolution\": \"File\", \"paths\": [\"a\"], \
         \"patterns\": []}}, \"state\": \"Ready\"}}, {{\"id\": \"{blocked}\", \"title\": \"t\", \
         \"why\": \"w\", \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \
         \"Proposed\", \"widened\": [], \"territory\": {{\"resolution\": \"File\", \"paths\": [\"b\"], \
         \"patterns\": []}}, \"state\": \"Ready\", \"depends_on\": [\"{dependency}\"]}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger)?;

    return Ok(BoardWithABlockedItem { tree, directory, dependency, blocked });
}

/// A scratch board and the one claimable item `Scratch_Board_With_A_Claimable_Item` wrote.
#[derive(Debug)]
pub(crate) struct BoardWithAClaimableItem
{
    /// The tree the board sits in, as [`OwnedBoard::tree`] is.
    pub tree: std::path::PathBuf,
    /// The scratch directory holding `ledger.json`, directly under `tree`.
    pub directory: std::path::PathBuf,
    /// The id of the `Ready` item with a real, non-empty territory.
    pub id: ItemId,
}

/// A scratch board carrying one real `Ready` item with a real, non-empty territory --
/// unlike `Scratch_Board_With_One_Item`'s deliberately territory-less fixture (built for
/// `Validate`'s own violation test), a document `Claim`/`Renew`/`TakeOver` write back
/// must itself stay valid, so every fixture below reserves something real.
///
/// # Errors
///
/// Returns whatever the owned board or the ledger write refuses.
pub(crate) fn Scratch_Board_With_A_Claimable_Item() -> Result<BoardWithAClaimableItem, std::io::Error>
{
    let OwnedBoard { tree, directory } = Owned_Board("claimable")?;
    let id = ItemId::New("SCRATCH-CLAIMABLE");
    let ledger = format!(
        "{{\"schema_version\": 6, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
         \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"widened\": [], \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"a\"], \"patterns\": []}}, \"state\": \"Ready\"}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger)?;

    return Ok(BoardWithAClaimableItem { tree, directory, id });
}

/// A scratch board and the one already-claimed item `Scratch_Board_With_A_Claimed_Item` wrote.
#[derive(Debug)]
pub(crate) struct BoardWithAClaimedItem
{
    /// The tree the board sits in, as [`OwnedBoard::tree`] is.
    pub tree: std::path::PathBuf,
    /// The scratch directory holding `ledger.json`, directly under `tree`.
    pub directory: std::path::PathBuf,
    /// The id of the `Claimed` item.
    pub id: ItemId,
}

/// A scratch board carrying one real item already `Claimed`, with `holder` and
/// `expires_at_unix` chosen by the caller -- an active lease far in the future for a
/// `Renew` fixture, or one far in the past (lapsed) for a `TakeOver` fixture.
///
/// # Errors
///
/// Returns whatever the owned board or the ledger write refuses.
pub(crate) fn Scratch_Board_With_A_Claimed_Item(
    holder: &str,
    expires_at_unix: i64,
) -> Result<BoardWithAClaimedItem, std::io::Error>
{
    let OwnedBoard { tree, directory } = Owned_Board("claimed")?;
    let id = ItemId::New("SCRATCH-CLAIMED");
    let ledger = format!(
        "{{\"schema_version\": 6, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
         \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"widened\": [], \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"a\"], \"patterns\": []}}, \"state\": \"Claimed\", \
         \"claim\": {{\"holder\": \"{holder}\", \"acquired_at\": 1, \"lease_expires_at\": \
         {expires_at_unix}}}}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger)?;

    return Ok(BoardWithAClaimedItem { tree, directory, id });
}

/// The holder [`With_A_Board_Somebody_Else_Holds`] claims its item under -- a name no ending
/// verb's own `Ending_Request` carries, so a real refusal is the only outcome
/// `Handle_Work_Abandon` and `Handle_Work_Decline` can reach over that board.
const OTHER_HOLDER: &str = "someone-else";

/// Runs `handler` over a scratch board whose one item [`OTHER_HOLDER`] holds under a lease
/// that has not lapsed, removes the board's whole tree, and hands back whatever `handler`
/// answered.
///
/// The board, the request and the teardown are three statements `abandon_response.rs` and
/// `decline_response.rs` each spelled out identically around their own one call, which is
/// what `check-interfile-duplication` flagged and why they live here once instead. What the
/// two callers genuinely differ on is the verb under test and the reason that verb records
/// -- those are the two things a caller supplies.
///
/// The expiry is `i64::from(u32::MAX)` seconds after the epoch, the same far-future lease
/// `Scratch_Board_With_A_Held_Territory_Conflict` writes: an ending verb must refuse because
/// another holder *has* the item, not because a lease happened to lapse under it.
#[must_use]
pub(crate) fn With_A_Board_Somebody_Else_Holds<Outcome>(reason: &str, handler: impl FnOnce(&std::path::Path, &EndingRequest) -> Outcome) -> Outcome
{
    let BoardWithAClaimedItem { tree, directory, id } =
        Scratch_Board_With_A_Claimed_Item(OTHER_HOLDER, i64::from(u32::MAX))
            .expect("the temp directory is writable and the scratch ledger is writable");
    let request = Ending_Request(id, reason);

    let response = handler(&directory, &request);

    let _ignored = std::fs::remove_dir_all(&tree);

    return response;
}

/// A scratch board and the one contested id `Scratch_Board_With_A_Held_Territory_Conflict`
/// left `Ready` over territory another item's live claim already holds.
#[derive(Debug)]
pub(crate) struct BoardWithAHeldTerritoryConflict
{
    /// The tree the board sits in, as [`OwnedBoard::tree`] is.
    pub tree: std::path::PathBuf,
    /// The scratch directory holding `ledger.json`, directly under `tree`.
    pub directory: std::path::PathBuf,
    /// The `Ready` item sharing the held item's territory.
    pub id: ItemId,
}

/// A scratch board carrying two real items over the same territory: one already
/// `Claimed` with an active lease, the other `Ready` and unclaimed -- the real
/// `ClaimRefusal::HeldBy` trigger `crates/substrate/nomos-ledger/src/store/refusal.rs`'s
/// own `Held_Ground` looks for (another item's *live claim* over overlapping ground),
/// not the `ClaimRefusal::NotClaimable` a second `claim` of the same already-`Claimed`
/// item id would hit instead.
///
/// # Errors
///
/// Returns whatever the owned board or the ledger write refuses.
pub(crate) fn Scratch_Board_With_A_Held_Territory_Conflict() -> Result<BoardWithAHeldTerritoryConflict, std::io::Error>
{
    let OwnedBoard { tree, directory } = Owned_Board("held")?;
    let held = ItemId::New("SCRATCH-HELD");
    let contested = ItemId::New("SCRATCH-CONTESTED");
    let ledger = format!(
        "{{\"schema_version\": 6, \"items\": [\
         {{\"id\": \"{held}\", \"title\": \"t\", \"why\": \"w\", \"done_when\": \"d\", \
         \"kind\": \"Capability\", \"origin\": \"Proposed\", \"widened\": [], \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"shared\"], \"patterns\": []}}, \"state\": \"Claimed\", \
         \"claim\": {{\"holder\": \"someone-else\", \"acquired_at\": 1, \"lease_expires_at\": \
         {far_future}}}}}, \
         {{\"id\": \"{contested}\", \"title\": \"t\", \"why\": \"w\", \"done_when\": \"d\", \
         \"kind\": \"Capability\", \"origin\": \"Proposed\", \"widened\": [], \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"shared\"], \"patterns\": []}}, \"state\": \"Ready\"}}\
         ]}}\n",
        far_future = i64::from(u32::MAX)
    );
    std::fs::write(directory.join("ledger.json"), ledger)?;

    return Ok(BoardWithAHeldTerritoryConflict { tree, directory, id: contested });
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The expiry the `Scratch_Board_With_A_Claimed_Item` test writes and reads back, named
    /// so the assertion below cites the same number the fixture was handed.
    const CLAIMED_EXPIRY_UNIX_SECONDS: i64 = 42;
    /// How many items' territories the held-territory fixture names in its ledger -- both of
    /// them reserve the one `"shared"` path, and the test below counts exactly that.
    const SHARED_TERRITORY_MENTIONS: usize = 2;

    #[test]
    fn Test_Claim_Request_Should_Carry_The_Item_Holder_And_A_One_Hour_Lease()
    {
        let item = ItemId::New("SCRATCH-CLAIM-REQUEST");

        let request = Claim_Request(item.clone(), "test-holder");

        assert_eq!(request.item, item);
        assert_eq!(request.holder, "test-holder");
        assert_eq!(request.lease, CLAIM_LEASE);
    }

    #[test]
    fn Test_Ending_Request_Should_Carry_The_Item_And_Reason_Under_A_Fixed_Holder()
    {
        let item = ItemId::New("SCRATCH-ENDING-REQUEST");

        let request = Ending_Request(item.clone(), "a real reason");

        assert_eq!(request.item, item);
        assert_eq!(request.holder, "test-holder");
        assert_eq!(request.reason, "a real reason");
    }

    #[test]
    fn Test_Unique_Scratch_Directory_Should_Give_Two_Calls_The_Same_Label_Different_Directories()
    {
        let first = Unique_Scratch_Directory("unique-directory-test")
            .expect("the temp directory is writable and the scratch ledger is writable");
        let second = Unique_Scratch_Directory("unique-directory-test")
            .expect("the temp directory is writable and the scratch ledger is writable");

        assert_ne!(first, second);

        let _ignored = std::fs::remove_dir_all(&first);
        let _ignored = std::fs::remove_dir_all(&second);
    }

    #[test]
    fn Test_Scratch_Board_Should_Write_A_Minimal_Valid_Empty_Ledger()
    {
        let OwnedBoard { tree, directory } = Scratch_Board()
            .expect("the temp directory is writable and the scratch ledger is writable");

        let content = std::fs::read_to_string(directory.join("ledger.json")).expect("reads the ledger back");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("writes valid json");

        let _ignored = std::fs::remove_dir_all(&tree);

        assert_eq!(*parsed.get("schema_version")
            .expect("a written ledger always carries schema_version"), LEDGER_SCHEMA_VERSION);
        assert!(parsed.get("items").expect("a written ledger always carries items").as_array()
            .expect("items is an array").is_empty());
    }

    #[test]
    fn Test_Scratch_Board_With_One_Item_Should_Write_A_Board_Carrying_Exactly_That_Item()
    {
        let BoardWithOneItem { tree, directory, id } = Scratch_Board_With_One_Item()
            .expect("the temp directory is writable and the scratch ledger is writable");

        let content = std::fs::read_to_string(directory.join("ledger.json")).expect("reads the ledger back");

        let _ignored = std::fs::remove_dir_all(&tree);

        assert!(content.contains(&id.to_string()), "{content}");
    }

    #[test]
    fn Test_Scratch_Board_With_A_Blocked_Item_Should_Make_The_Dependency_Ready_And_Unclaimed()
    {
        let BoardWithABlockedItem { tree, directory, dependency, blocked } =
            Scratch_Board_With_A_Blocked_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");

        let content = std::fs::read_to_string(directory.join("ledger.json")).expect("reads the ledger back");

        let _ignored = std::fs::remove_dir_all(&tree);

        assert!(content.contains(&dependency.to_string()), "{content}");
        assert!(content.contains(&blocked.to_string()), "{content}");
        assert_ne!(dependency, blocked);
    }

    #[test]
    fn Test_Every_Board_Fixture_Should_Sit_Alone_In_A_Tree_Its_Caller_Removes_Whole()
    {
        let boards = Every_Board().expect("the temp directory is writable and every scratch ledger is writable");

        let contents: Vec<Vec<std::ffi::OsString>> =
            boards.iter().map(|board| return Entries_Of(&board.tree)).collect();
        for board in &boards
        {
            let _ignored = std::fs::remove_dir_all(&board.tree);
        }

        for (board, entries) in boards.iter().zip(contents)
        {
            assert_eq!(board.directory.parent(), Some(board.tree.as_path()), "the verbs' repository: {board:?}");
            assert_eq!(entries, vec![std::ffi::OsString::from(BOARD_DIRECTORY)], "only the board: {board:?}");
        }
    }

    #[test]
    fn Test_With_A_Board_Somebody_Else_Holds_Should_Remove_The_Whole_Tree_It_Handed_Over()
    {
        let (tree, entries) = With_A_Board_Somebody_Else_Holds("a real reason", |directory, _request| {
            let tree = directory.parent().map(std::path::Path::to_path_buf);
            let entries = tree.as_deref().map(Entries_Of);
            return (tree, entries);
        });

        let tree = tree.expect("the board directory sits in a tree");
        assert_eq!(entries, Some(vec![std::ffi::OsString::from(BOARD_DIRECTORY)]), "only the board");
        assert!(!tree.exists(), "the fixture removes the whole tree, not only the board in it: {tree:?}");
    }

    #[test]
    fn Test_Scratch_Board_With_A_Claimable_Item_Should_Write_A_Ready_Item_With_Real_Territory()
    {
        let BoardWithAClaimableItem { tree, directory, id } = Scratch_Board_With_A_Claimable_Item()
            .expect("the temp directory is writable and the scratch ledger is writable");

        let content = std::fs::read_to_string(directory.join("ledger.json")).expect("reads the ledger back");

        let _ignored = std::fs::remove_dir_all(&tree);

        assert!(content.contains(&id.to_string()), "{content}");
        assert!(content.contains("\"paths\": [\"a\"]"), "{content}");
    }

    #[test]
    fn Test_Scratch_Board_With_A_Claimed_Item_Should_Carry_The_Given_Holder_And_Expiry()
    {
        let BoardWithAClaimedItem { tree, directory, id } =
            Scratch_Board_With_A_Claimed_Item("a-real-holder", CLAIMED_EXPIRY_UNIX_SECONDS)
                .expect("the temp directory is writable and the scratch ledger is writable");

        let content = std::fs::read_to_string(directory.join("ledger.json")).expect("reads the ledger back");

        let _ignored = std::fs::remove_dir_all(&tree);

        assert!(content.contains(&id.to_string()), "{content}");
        assert!(content.contains("a-real-holder"), "{content}");
        assert!(content.contains(&format!("\"lease_expires_at\": {CLAIMED_EXPIRY_UNIX_SECONDS}")), "{content}");
    }

    #[test]
    fn Test_Scratch_Board_With_A_Held_Territory_Conflict_Should_Share_One_Territory_Between_Two_Items()
    {
        let BoardWithAHeldTerritoryConflict { tree, directory, id } =
            Scratch_Board_With_A_Held_Territory_Conflict()
                .expect("the temp directory is writable and the scratch ledger is writable");

        let content = std::fs::read_to_string(directory.join("ledger.json")).expect("reads the ledger back");

        let _ignored = std::fs::remove_dir_all(&tree);

        assert!(content.contains(&id.to_string()), "{content}");
        assert_eq!(content.matches("\"shared\"").count(), SHARED_TERRITORY_MENTIONS, "{content}");
    }

    /// The board every fixture in this module builds, [`Owned_Board`]'s own included, each
    /// reduced to its tree and its board directory.
    fn Every_Board() -> Result<Vec<OwnedBoard>, std::io::Error>
    {
        return Ok(vec![
            Owned_Board("every-board")?,
            Scratch_Board()?,
            Scratch_Board_With_One_Item()
                .map(|board| return OwnedBoard { tree: board.tree, directory: board.directory })?,
            Scratch_Board_With_A_Blocked_Item()
                .map(|board| return OwnedBoard { tree: board.tree, directory: board.directory })?,
            Scratch_Board_With_A_Claimable_Item()
                .map(|board| return OwnedBoard { tree: board.tree, directory: board.directory })?,
            Scratch_Board_With_A_Claimed_Item(OTHER_HOLDER, CLAIMED_EXPIRY_UNIX_SECONDS)
                .map(|board| return OwnedBoard { tree: board.tree, directory: board.directory })?,
            Scratch_Board_With_A_Held_Territory_Conflict()
                .map(|board| return OwnedBoard { tree: board.tree, directory: board.directory })?,
        ]);
    }

    /// The names directly inside `tree`.
    fn Entries_Of(tree: &std::path::Path) -> Vec<std::ffi::OsString>
    {
        return std::fs::read_dir(tree)
            .expect("a fixture's own tree is readable")
            .map(|entry| return entry.expect("an entry of a fixture's own tree is readable").file_name())
            .collect();
    }
}

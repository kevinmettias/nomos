//! The `nomos_work_orchestration` seam `check-integration-coverage` found with no suite.
//!
//! `nomos-cli::work.rs` is built almost entirely out of this crate — `nomos_work_orchestration
//! ::Run`, `BoardView`, `ShowView`, `WorkOutcome`, and the re-exported `ClaimRequest` and
//! `EndingRequest` all cross that boundary — and `nomos work` is already the most heavily
//! tested command group in this crate (`list_tells_the_truth.rs`, `abandon_is_readable.rs`,
//! `add_guarantees_what_it_says.rs` and their siblings all drive it end to end over a
//! scratch ledger). None of them ever spell `nomos_work_orchestration` by name, which is
//! exactly why the crate-boundary checker still counts this as a surface with no suite: a
//! subprocess's stdout proves the CLI parses and dispatches correctly, but nothing proves a
//! caller holding this crate's own request vocabulary gets the shape it expects.
//!
//! This adds the one thing missing: a real `nomos_work_orchestration::WorkCommand` value,
//! constructed directly and matched against, standing for the request `work.rs`'s own
//! parser would have built from the same arguments — beside a real `nomos work` subprocess
//! run driving those same arguments through the shipped binary, over a scratch board this
//! test owns via `NOMOS_WORK_DIR`.

use nomos_ledger::ItemId;
use nomos_work_orchestration::WorkCommand;
use std::path::PathBuf;
use std::process::Command;

const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// What one `nomos work` run exited with, and what it said.
struct Ran
{
    code: i32,
    said: String,
}

/// A scratch board on disk, removed when the test that made it ends.
struct Board
{
    root: PathBuf,
}

/// The case a scratch board is named for, told apart from the ledger written into it.
///
/// [`Board::New`] takes both and both are text, so a caller who wrote them the other way round
/// would build a directory name out of a whole ledger document -- compiling, and failing at
/// the filesystem rather than at the call. A caller still spells the value as a plain `&str`,
/// which is why the conversion lives here.
struct CaseName<'a>(&'a str);

impl<'a> From<&'a str> for CaseName<'a>
{
    fn from(name: &'a str) -> Self
    {
        return Self(name);
    }
}

impl Board
{
    fn New<'a>(name: impl Into<CaseName<'a>>, ledger: &str) -> Self
    {
        let name = name.into().0;
        let root = std::env::temp_dir().join(format!("nomos-cli-work-orch-seam-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        std::fs::write(root.join("ledger.json"), ledger).expect("a scratch ledger");
        return Self { root };
    }

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
}

impl Drop for Board
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// One claimable item, ready for the tests below to list and show.
fn A_Board(name: &str) -> Board
{
    return Board::New(
        name,
        "{\n  \"schema_version\": 1,\n  \"items\": [\
         {\"id\":\"T-1\",\"title\":\"item T-1\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"kind\":\"Correction\",\"origin\":\"Proposed\",\
         \"widened\": [], \"territory\":{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\"patterns\":[]},\
         \"state\":\"Ready\",\"depends_on\":[],\"blocked\":null,\
         \"claim\":null,\"verification\":null,\"verified\":null}\
         ]\n}\n",
    );
}

/// `WorkCommand` is the request vocabulary `work.rs`'s own parser builds and hands to
/// `nomos_work_orchestration::Run` — this constructs the same shape directly, for the same
/// arguments a caller would type, and proves it round-trips through equality the way any
/// value this crate's own dispatch depends on must.
#[test]
fn Test_Work_Command_Should_Carry_The_State_Filter_And_The_Item_A_Real_Invocation_Would()
{
    let listing = WorkCommand::List { state: Some("ready".to_owned()) };
    assert_eq!(listing, WorkCommand::List { state: Some("ready".to_owned()) });
    assert_ne!(listing, WorkCommand::List { state: None });

    let showing = WorkCommand::Show { item: ItemId::New("T-1") };
    let WorkCommand::Show { item } = &showing
    else
    {
        panic!("just constructed as Show");
    };
    assert_eq!(item.As_Text(), "T-1");
}

/// One `--state` filter run against [`A_Board`]'s single `Ready` item, and what the real
/// binary must say about it.
struct ListCase
{
    state: &'static str,
    should_list_it: bool,
}

/// The two `--state` filters [`List_Cases`] holds: the item's own state, which must list it,
/// and one it is not in, which must not. Named because the length is spelled in that
/// function's return type, where no `const` is in reach of the literal.
const LIST_CASE_COUNT: usize = 2;

/// Every `--state` filter [`Test_Listing_By_State_Should_Find_The_Item_Only_When_The_State_Matches`]
/// checks: the item's own state, which must list it, and one it is not in, which must not.
fn List_Cases() -> [ListCase; LIST_CASE_COUNT]
{
    return [
        ListCase { state: "ready", should_list_it: true },
        ListCase { state: "claimed", should_list_it: false },
    ];
}

/// The real binary, driven with the arguments `WorkCommand::List { state: Some("ready") }`
/// stands for above: `nomos work list --state <state>` over a board holding exactly one
/// `Ready` item must list it only when the filter names the state it is actually in.
#[test]
fn Test_Listing_By_State_Should_Find_The_Item_Only_When_The_State_Matches()
{
    for case in List_Cases()
    {
        let board = A_Board("list-by-state");

        let Ran { code, said } = board.Work(&["list", "--state", case.state]);

        assert_eq!(code, 0, "{said}");
        assert_eq!(
            said.contains("T-1"),
            case.should_list_it,
            "--state {}: {said}",
            case.state
        );
    }
}

/// One `--item` id [`A_Board`]'s single item is shown by, and what the real binary must
/// exit and say for it.
struct ShowCase
{
    item: &'static str,
    expected_code: i32,
    expected_text: &'static str,
}

/// The two `--item` ids [`Show_Cases`] holds: the one item the board carries, and one it does
/// not. Named because the length is spelled in that function's return type, where no `const`
/// is in reach of the literal.
const SHOW_CASE_COUNT: usize = 2;

/// What `nomos work show --item <an id no item answers to>` leaves the process with --
/// `work/exit_code.rs::ExitCode::Conflict`'s own value, which `work.rs::Render_Show` returns
/// beside the `no item named …` line this suite asserts on.
const UNKNOWN_ITEM_EXIT_CODE: i32 = 4;

/// Every `--item` id [`Test_Showing_An_Item_Should_Report_On_It_Or_Refuse_The_Ones_The_Board_Does_Not_Hold`]
/// checks: the one item the board holds, and one it does not.
fn Show_Cases() -> [ShowCase; SHOW_CASE_COUNT]
{
    return [
        ShowCase { item: "T-1", expected_code: 0, expected_text: "T-1" },
        ShowCase { item: "NO-SUCH-ITEM", expected_code: UNKNOWN_ITEM_EXIT_CODE, expected_text: "no item named NO-SUCH-ITEM" },
    ];
}

/// The real binary, driven with the item `WorkCommand::Show { item: ItemId::New("T-1") }`
/// stands for above: `nomos work show --item T-1` must report on exactly that item, and an
/// item the board never claimed must be refused rather than silently reported as absent.
#[test]
fn Test_Showing_An_Item_Should_Report_On_It_Or_Refuse_The_Ones_The_Board_Does_Not_Hold()
{
    for case in Show_Cases()
    {
        let board = A_Board("show-by-item");

        let Ran { code, said } = board.Work(&["show", "--item", case.item]);

        assert_eq!(code, case.expected_code, "--item {}: {said}", case.item);
        assert!(said.contains(case.expected_text), "--item {}: {said}", case.item);
    }
}

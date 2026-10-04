//! An item whose territory reaches a path its repository declares is refused a predicate that
//! does not carry what the declaration requires, at `add` and at `widen`. `OD-GATE-036`.
//!
//! Through the store's own verbs on a real file, because the property is that the refusal is
//! decided where the write is: an item refused here never reaches the board, and a widening
//! refused here leaves the territory as it was. Every refusal below has a sibling in which the
//! same verb is granted, so none of them can be satisfied by refusing everything.

use std::path::PathBuf;
use std::time::Duration;

use nomos_ledger::{
    AddRefusal, ClaimRefusal, CoverageRefusal, ExclusionLedger, FileLedger, Holder, ItemId, ItemKind, ItemOrigin,
    ItemState, LedgerItem, PredicateCoverage, RepositoryDeclarations, Territory, VerificationPredicate,
};
use nomos_platform_std::{FileLock, StdFileSystem, SystemClock};

/// The declaration every test here judges against: one rule, shaped like the one this
/// repository commits.
const DECLARATION: &str = r#"{
    "rules": [
        {
            "record": "OD-EXAMPLE-001",
            "paths": ["crates/rules", "crates/composer"],
            "requires": ["nomos-cli", "nomos-integration-tests"],
            "satisfied_by": ["--workspace"]
        }
    ]
}"#;

/// The lease every claim here takes, long enough that nothing lapses mid-test.
const LEASE: Duration = Duration::from_secs(3_600);

/// A board on disk under a directory unique to this process and `name`, removed when dropped.
struct Board
{
    directory: PathBuf,
    ledger: FileLedger<StdFileSystem, SystemClock, FileLock>,
}

impl Drop for Board
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.directory);
    }
}

fn Board_Named(name: &str) -> Board
{
    let directory = std::env::temp_dir().join(format!("nomos-ledger-predicate-coverage-{name}-{}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory under the temporary directory is creatable");
    let ledger = FileLedger::At(directory.join("ledger.json"), StdFileSystem, SystemClock, FileLock::At(directory.join("ledger.lock")));

    return Board { directory, ledger };
}

fn Declared() -> RepositoryDeclarations
{
    return RepositoryDeclarations {
        published: Territory::Empty(),
        coverage: PredicateCoverage::From_Json(DECLARATION).expect("the fixture is the declaration's own shape"),
    };
}

fn Item(id: &str, paths: &[&str], predicate: &str) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: "an item".to_owned(),
        why: "because".to_owned(),
        done_when: "when it is done".to_owned(),
        kind: ItemKind::Capability,
        origin: ItemOrigin::Proposed,
        territory: Territory::Of_Files(paths.iter().map(|path| return (*path).to_owned())),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: Some(VerificationPredicate::From_String_Arguments(
            predicate.split_whitespace().map(str::to_owned).collect(),
        )),
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
        declined: None,
    };
}

/// The items on the board, by identifier.
fn Ids_On(board: &Board) -> Vec<String>
{
    let document = board.ledger.Load().expect("the scratch board loads");

    return document.items.iter().map(|item| return item.id.As_Text().to_owned()).collect();
}

/// Adds `item` under the declaration and claims it, so a widening can be asked of it.
fn Added_And_Claimed(board: &mut Board, item: &LedgerItem)
{
    board.ledger.Add(item, "agent-a", &Declared(), &Territory::Empty()).expect("the item reaches no declared path");
    board.ledger.Claim(&item.id, "agent-a", LEASE).expect("the only item on the board is claimable");
}

fn Widened(board: &mut Board, item: &str, adding: &[&str]) -> Result<Vec<String>, ClaimRefusal>
{
    let adding: Vec<String> = adding.iter().map(|path| return (*path).to_owned()).collect();
    let coverage = Declared().coverage;

    return board.ledger.Widen(&ItemId::New(item), Holder::from("agent-a"), &adding, &coverage);
}

#[test]
fn Test_Add_Should_Refuse_A_Reaching_Item_Whose_Predicate_Lacks_A_Required_Argument_And_Name_It()
{
    let mut board = Board_Named("add-refused");
    let item = Item("T-1", &["crates/rules/nomos-rules/src/a.rs"], "cargo test -p nomos-rules -p nomos-cli");

    let refused = board.ledger.Add(&item, "agent-a", &Declared(), &Territory::Empty());

    assert_eq!(
        refused,
        Err(AddRefusal::Coverage {
            refusal: CoverageRefusal::Uncovered {
                record: "OD-EXAMPLE-001".to_owned(),
                reaching: vec!["crates/rules/nomos-rules/src/a.rs".to_owned()],
                missing: vec!["nomos-integration-tests".to_owned()],
                sufficient: vec!["--workspace".to_owned()],
            },
        })
    );
    assert!(Ids_On(&board).is_empty(), "the refusal was reported and the item landed anyway");
}

#[test]
fn Test_Add_Should_Accept_Every_Required_Argument_Or_One_That_Suffices_Alone()
{
    let mut board = Board_Named("add-accepted");
    let both = Item("T-1", &["crates/rules"], "cargo test -p nomos-rules -p nomos-cli -p nomos-integration-tests");
    let workspace = Item("T-2", &["crates/composer/x"], "cargo test --workspace");

    assert_eq!(board.ledger.Add(&both, "agent-a", &Declared(), &Territory::Empty()), Ok(()));
    assert_eq!(board.ledger.Add(&workspace, "agent-a", &Declared(), &Territory::Empty()), Ok(()));
    assert_eq!(Ids_On(&board), vec!["T-1".to_owned(), "T-2".to_owned()]);
}

#[test]
fn Test_Add_Should_Accept_A_Short_Predicate_Whose_Territory_Reaches_No_Declared_Path()
{
    let mut board = Board_Named("add-elsewhere");
    let item = Item("T-1", &["crates/substrate/nomos-ledger", "README.md"], "cargo test -p nomos-ledger");

    assert_eq!(board.ledger.Add(&item, "agent-a", &Declared(), &Territory::Empty()), Ok(()));
}

#[test]
fn Test_Add_Should_Accept_Anything_From_A_Repository_That_Declares_Nothing()
{
    let mut board = Board_Named("add-undeclared");
    let item = Item("T-1", &["crates/rules"], "cargo test -p nomos-rules");

    assert_eq!(board.ledger.Add(&item, "agent-a", &RepositoryDeclarations::Undeclared(), &Territory::Empty()), Ok(()));
}

#[test]
fn Test_Add_Should_Refuse_Under_A_Declaration_Nobody_Could_Read()
{
    let mut board = Board_Named("add-unreadable");
    let item = Item("T-1", &["docs/a.md"], "cargo test --workspace");
    let declared = RepositoryDeclarations {
        published: Territory::Empty(),
        coverage: PredicateCoverage::Unreadable { cause: "nomos-predicate-coverage.json: expected `,`".to_owned() },
    };

    let refused = board.ledger.Add(&item, "agent-a", &declared, &Territory::Empty());

    assert!(
        matches!(&refused, Err(AddRefusal::Coverage { refusal: CoverageRefusal::Unreadable { cause } }) if cause.contains("expected")),
        "{refused:?}"
    );
    assert!(Ids_On(&board).is_empty(), "an unreadable declaration admitted an item");
}

#[test]
fn Test_Widen_Should_Refuse_A_Widening_Into_A_Declared_Path_The_Predicate_Does_Not_Cover()
{
    let mut board = Board_Named("widen-refused");
    let item = Item("T-1", &["crates/substrate/nomos-ledger"], "cargo test -p nomos-ledger");
    Added_And_Claimed(&mut board, &item);

    let refused = Widened(&mut board, "T-1", &["crates/host/nomos-cli/src/work.rs", "crates/composer/a.rs"]);

    let Err(ClaimRefusal::Coverage { item: refused_item, refusal: CoverageRefusal::Uncovered { reaching, missing, .. } }) = refused
    else
    {
        panic!("a widening into crates/composer without the declared arguments is refused: {refused:?}");
    };
    assert_eq!(refused_item, ItemId::New("T-1"));
    assert_eq!(reaching, vec!["crates/composer/a.rs".to_owned()], "only the added path that reaches is named");
    assert_eq!(missing, vec!["nomos-cli".to_owned(), "nomos-integration-tests".to_owned()]);
    let territory = board.ledger.Load().expect("the scratch board loads").items.remove(0).territory;
    assert_eq!(territory, item.territory, "a refused widening changed the territory");
}

#[test]
fn Test_Widen_Should_Accept_A_Widening_Into_A_Declared_Path_The_Predicate_Covers()
{
    let mut board = Board_Named("widen-covered");
    let item = Item("T-1", &["crates/substrate/nomos-ledger"], "cargo test -p nomos-ledger --workspace");
    Added_And_Claimed(&mut board, &item);

    let widened = Widened(&mut board, "T-1", &["crates/rules/a.rs"]);

    assert_eq!(widened, Ok(vec!["crates/rules/a.rs".to_owned()]));
}

#[test]
fn Test_Widen_Should_Accept_A_Widening_That_Reaches_No_Declared_Path()
{
    let mut board = Board_Named("widen-elsewhere");
    let item = Item("T-1", &["crates/substrate/nomos-ledger"], "cargo test -p nomos-ledger");
    Added_And_Claimed(&mut board, &item);

    let widened = Widened(&mut board, "T-1", &["tests/contract/surface/nomos-ledger.txt"]);

    assert_eq!(widened, Ok(vec!["tests/contract/surface/nomos-ledger.txt".to_owned()]));
}

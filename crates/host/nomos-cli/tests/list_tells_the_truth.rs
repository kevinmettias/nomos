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
}

impl Drop for Board
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// One item, authored inline so each test's board is readable in the test.
fn Item(id: &str, paths: &str, state: &str, depends_on: &str, tail: &str) -> String
{
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
            Item("T-1", "\"src/a.rs\"", "Ready", "", NO_CLAIM),
            Item("T-2", "\"src/b.rs\"", "Ready", "\"T-1\"", NO_CLAIM)
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
            Item("T-1", "\"src/a.rs\"", "Ready", "", NO_CLAIM),
            Item("T-2", "\"src/b.rs\"", "Ready", "", NO_CLAIM)
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
            Item("T-1", "\"src/a.rs\"", "Done", "", FINISHED),
            Item("T-2", "\"src/b.rs\"", "Ready", "\"T-1\"", NO_CLAIM)
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
    let board = Board::New(
        "filter-ready",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", "Ready", "", NO_CLAIM),
            Item("T-2", "\"src/b.rs\"", "Ready", "\"T-1\"", NO_CLAIM)
        ),
    );

    let ready = board.List(Some("ready"));

    assert!(ready.contains("T-1"), "T-1 is claimable:\n{ready}");
    assert!(
        !ready.contains("T-2"),
        "T-2 cannot be claimed, so it must not answer a request for ready work:\n{ready}"
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
    let held = format!(
        "\"claim\":{{\"holder\":\"agent-a\",\"acquired_at\":1000000,\
         \"lease_expires_at\":{FOREVER}}},\"verification\":null,\"verified\":null"
    );
    let board = Board::New(
        "held-ground",
        &format!(
            "{},{}",
            Item("T-1", "\"src/shared.rs\"", "Claimed", "", &held),
            Item("T-2", "\"src/shared.rs\"", "Ready", "", NO_CLAIM)
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
    let held = format!(
        "\"claim\":{{\"holder\":\"agent-a\",\"acquired_at\":1000000,\
         \"lease_expires_at\":{FOREVER}}},\"verification\":null,\"verified\":null"
    );
    let board = Board::New(
        "free-ground",
        &format!(
            "{},{}",
            Item("T-1", "\"src/a.rs\"", "Claimed", "", &held),
            Item("T-2", "\"src/b.rs\"", "Ready", "", NO_CLAIM)
        ),
    );

    assert_eq!(board.Label_Of("T-2"), "ready");
}

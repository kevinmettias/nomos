//! A writer that cannot account for the document does not get to rewrite it.
//!
//! P10-STALE-WRITER, decided in `OD-LEDGER-008`. The incident these tests are written against
//! was a `work claim` that returned 0 and dropped a key on the way through: the state change
//! survived, the data did not, and the surface reporting it could not tell the two apart.
//!
//! So the assertion here is on the **file**, not on the type. A refusal that exits 5 while
//! quietly rewriting the ledger would satisfy every assertion about exit codes and would be the
//! defect intact. The bytes written before the command are compared with the bytes after it.
//!
//! What no test in this process can do is run a genuinely older executable — one `cargo test`,
//! one binary. The end-to-end experiment for that was run by hand and its observed result is
//! recorded in `OD-LEDGER-008`.

use nomos_ledger::SCHEMA_VERSION;
use std::path::PathBuf;
use std::process::Command;

const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// A key no build of this crate declares, spliced onto an item.
///
/// Named for what it is rather than for a field somebody might later add, so this fixture cannot
/// stop being the case it was written for by the schema catching up with it.
const UNDECLARED: &str = ",\"a_field_this_build_does_not_know\":{\"holder\":\"agent-a\"}";

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
    /// A scratch board holding one claimable item, with `extra` spliced into that item.
    fn New(name: &str, extra: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-stale-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        std::fs::write(
            root.join("ledger.json"),
            format!(
                "{{\n  \"schema_version\": 1,\n  \"items\": [\
                 {{\"id\":\"T-1\",\"title\":\"item T-1\",\"why\":\"because\",\
                 \"done_when\":\"the tests pass\",\
                 \"territory\":{{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\
                 \"patterns\":[]}},\
                 \"state\":\"Ready\",\"depends_on\":[],\"blocked\":null,\"claim\":null,\
                 \"verification\":null,\"verified\":null,\"abandoned\":[]{extra}}}\
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

    /// The ledger file exactly as it stands, bytes and all.
    fn Bytes(&self) -> Vec<u8>
    {
        return std::fs::read(self.root.join("ledger.json")).expect("the ledger is readable");
    }
}

impl Drop for Board
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// The assertion the item exists for, made on the file rather than on a type.
///
/// Step by step this is the incident: a verb that changes the board is run against a document
/// carrying a key the running build cannot account for. It must exit 5 and it must leave the
/// bytes alone. The exit code alone would pass on a build that refused *and* rewrote.
#[test]
fn Test_A_Build_That_Cannot_Read_The_Ledger_Should_Not_Rewrite_It()
{
    let board = Board::New("refuses-to-rewrite", UNDECLARED);
    let before = board.Bytes();

    let Ran { said, code } = board.Work(&["claim", "--item", "T-1", "--holder", "agent-b"]);

    assert_eq!(
        code, 5,
        "a build that cannot read the board must report the store unusable: {said}"
    );
    assert_eq!(
        board.Bytes(),
        before,
        "the ledger was rewritten by a build that could not read it, which is the defect:\n{said}"
    );
}

/// A read-only verb refuses too, which is where a stale session actually meets this.
///
/// `list` writes nothing, so nothing is lost by running it — and that is the argument for
/// refusing it anyway. A session allowed to read a document it cannot account for goes on
/// deciding what to claim from a board it is not seeing whole, which is the condition that
/// produced the incident rather than the write that ended it.
#[test]
fn Test_Listing_A_Ledger_This_Build_Cannot_Account_For_Should_Refuse()
{
    let board = Board::New("refuses-to-list", UNDECLARED);

    let Ran { said, code } = board.Work(&["list"]);

    assert_eq!(code, 5, "{said}");
    assert!(
        said.contains("a_field_this_build_does_not_know"),
        "the refusal must name what could not be accounted for:\n{said}"
    );
}

/// The control `done_when` asks for by name, and the one that gives the test above its teeth.
///
/// A guard that refused every write would stop the board rather than protect it. It also makes
/// the byte comparison mean something: without this, "the bytes did not change" would pass on a
/// binary that never writes at all.
#[test]
fn Test_A_Build_That_Can_Read_The_Ledger_Should_Still_Write_It()
{
    let board = Board::New("still-writes", "");
    let before = board.Bytes();

    let Ran { said, code } = board.Work(&["claim", "--item", "T-1", "--holder", "agent-b"]);

    assert_eq!(code, 0, "a readable board must still be claimable: {said}");
    assert_ne!(
        board.Bytes(),
        before,
        "the claim was granted and nothing was written:\n{said}"
    );
}

/// The operator-facing half: one command that answers "is the exe I copied current?".
///
/// Sessions run a copy of `nomos.exe`, because `finish` runs a predicate that rebuilds the
/// running executable. `OD-LEDGER-008` documents that workaround rather than replacing it, and
/// this line is what makes it checkable without having to provoke a refusal first.
#[test]
fn Test_Validate_Should_Report_The_Files_Schema_And_The_Builds()
{
    let board = Board::New("validate-reports-both", "");

    let Ran { said, code } = board.Work(&["validate"]);

    assert_eq!(code, 0, "{said}");
    // A literal `1`, and it stays one: this reads the *fixture's* own version, which `Board`
    // writes as `schema_version: 1` and which no bump to `SCHEMA_VERSION` changes. The two
    // numbers on this line are different facts, and the next reader should not "fix" both.
    assert!(
        said.contains("schema 1"),
        "validate must report the file's own schema version:\n{said}"
    );
    // The constant, never a literal spelling of its value. This assertion said "understands 1"
    // until `P10-LAPSE-TAKEOVER` raised `SCHEMA_VERSION` to 2 and turned it red — a test in one
    // item's territory pinned by a constant in another's, with nothing able to see the coupling
    // because a file and a constant are `Disjoint` and the ledger reserves paths. Same shape as
    // the trap `P10-RECORD-STEM` closed. Written this way the next bump cannot reach it.
    assert!(
        said.contains(&format!("this build understands {SCHEMA_VERSION}")),
        "validate must report what the running build understands:\n{said}"
    );
}

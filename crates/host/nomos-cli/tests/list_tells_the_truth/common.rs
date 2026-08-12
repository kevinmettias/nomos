//! A scratch board on disk, the two commands read off it, and the item shapes they are
//! authored from.

use std::path::PathBuf;
use std::process::Command;

const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// Far enough ahead that a lease written here is live whenever the suite runs.
const FOREVER: i64 = 4_102_444_800;

pub(crate) struct Board
{
    root: PathBuf,
}

impl Board
{
    pub(crate) fn New(name: &str, items: &str) -> Self
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
    pub(crate) fn List(&self, filter: Option<&str>) -> String
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
    pub(crate) fn Label_Of(&self, item: &str) -> String
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
    pub(crate) fn Audit(&self) -> String
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
pub(crate) fn Audit_Line<'a>(audit: &'a str, item: &str) -> Option<&'a str>
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
pub(crate) struct Standing<'a>
{
    pub(crate) state: &'a str,
    pub(crate) depends_on: &'a str,
    pub(crate) tail: &'a str,
}

/// One item, authored inline so each test's board is readable in the test.
pub(crate) fn Item(id: &str, paths: &str, standing: Standing<'_>) -> String
{
    let Standing { state, depends_on, tail } = standing;

    return format!(
        "{{\"id\":\"{id}\",\"title\":\"item {id}\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"territory\":{{\"resolution\":\"File\",\"paths\":[{paths}],\"patterns\":[]}},\
         \"state\":\"{state}\",\"depends_on\":[{depends_on}],\"blocked\":null,{tail}}}"
    );
}

pub(crate) const NO_CLAIM: &str = "\"claim\":null,\"verification\":null,\"verified\":null";

/// A `Done` item must carry its verification or the ledger is invalid.
pub(crate) const FINISHED: &str =
    "\"claim\":null,\"verification\":null,\"verified\":{\"argv\":[\"cargo\",\
     \"test\"],\"exit_code\":0,\"output_tail\":\"ok\",\"verified_at\":1000000,\
     \"gate\":null}";

/// A live claim, held far enough ahead that it is live whenever the suite runs.
pub(crate) fn Held_By(holder: &str) -> String
{
    return format!(
        "\"claim\":{{\"holder\":\"{holder}\",\"acquired_at\":1000000,\
         \"lease_expires_at\":{FOREVER}}},\"verification\":null,\"verified\":null"
    );
}

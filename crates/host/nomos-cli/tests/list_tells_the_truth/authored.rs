//! The two commands read off a scratch board, and the item shapes they are authored from.
//!
//! The board itself is `scratch_ledger/board.rs`, shared with the two suites beside this
//! one. What stays here is what is specific to `list` and `audit`: the item text this
//! suite authors, and the two readings it takes.

use crate::board::CaseName;

/// Far enough ahead that a lease written here is live whenever the suite runs.
const FOREVER: i64 = 4_102_444_800;

/// The JSON list of territory paths one item declares, told apart from the id it is filed
/// under and the standing beside it.
///
/// [`Item`] and [`Item_Declined`] take three texts in a row, so a caller who wrote two of them
/// the other way round would author an item whose id is a path array and whose paths are an
/// id -- compiling, and refused later by a ledger parser that could only say "invalid".
pub(crate) struct TerritoryPaths<'a>(&'a str);

impl<'a> From<&'a str> for TerritoryPaths<'a>
{
    fn from(paths: &'a str) -> Self
    {
        return Self(paths);
    }
}

pub(crate) struct Board
{
    scratch: crate::board::Board,
}

impl Board
{
    /// A board holding `items`, named for the case it is about.
    ///
    /// The name carries its own type because the ledger text beside it does not: both are
    /// `&str` at every call site, and a caller who swapped them would name a directory after a
    /// ledger.
    pub(crate) fn New<'a>(name: impl Into<CaseName<'a>>, items: &str) -> Self
    {
        let name = name.into();

        return Self {
            scratch: crate::board::Board::New(
                "list",
                name,
                &format!("{{\n  \"schema_version\": 1,\n  \"items\": [{items}]\n}}\n"),
            ),
        };
    }

    /// Runs `nomos work list`, returning stdout.
    pub(crate) fn List(&self, filter: Option<&str>) -> String
    {
        let mut arguments = vec!["list"];
        if let Some(state) = filter
        {
            arguments.push("--state");
            arguments.push(state);
        }

        return self.scratch.Said(&arguments);
    }

    /// The label `work list` prints for one item.
    pub(crate) fn Label_Of(&self, item: &str) -> String
    {
        let listing = self.List(None);
        let line = listing
            .lines()
            .find(|line| line.starts_with(item))
            // This function's whole job is to say which label `work list` printed, and the
            // return type is a `String` — so an item missing from the listing would come back
            // as `""`, which is also what an item printed with no label looks like. Those are
            // the two failures this suite has to keep apart, and only stopping here does it.
            // The listing is printed because "which items did appear" is the next question.
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
        return self.scratch.Said(&["audit"]);
    }
}

/// The audit's line for one item, or `None` if it did not answer for that item.
pub(crate) fn Audit_Line<'a>(audit: &'a str, item: &str) -> Option<&'a str>
{
    return audit
        .lines()
        .find(|line| line.split_whitespace().next() == Some(item));
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
pub(crate) fn Item<'a>(
    id: &str,
    paths: impl Into<TerritoryPaths<'a>>,
    standing: Standing<'_>,
) -> String
{
    let paths = paths.into().0;
    let Standing { state, depends_on, tail } = standing;

    return format!(
        "{{\"id\":\"{id}\",\"title\":\"item {id}\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"kind\":\"Correction\",\"origin\":\"Proposed\",\
         \"widened\": [], \"territory\":{{\"resolution\":\"File\",\"paths\":[{paths}],\"patterns\":[]}},\
         \"state\":\"{state}\",\"depends_on\":[{depends_on}],\"blocked\":null,{tail}}}"
    );
}

pub(crate) const NO_CLAIM: &str = "\"claim\":null,\"verification\":null,\"verified\":null";

/// A `Declined` item, whose state is a struct variant carrying a reason rather than the bare
/// word [`Item`] inserts for every other state — `Standing::state` cannot express it.
pub(crate) fn Item_Declined<'a>(
    id: &str,
    paths: impl Into<TerritoryPaths<'a>>,
    reason: &str,
) -> String
{
    let paths = paths.into().0;

    return format!(
        "{{\"id\":\"{id}\",\"title\":\"item {id}\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"kind\":\"Correction\",\"origin\":\"Proposed\",\
         \"widened\": [], \"territory\":{{\"resolution\":\"File\",\"paths\":[{paths}],\"patterns\":[]}},\
         \"state\":{{\"Declined\":{{\"reason\":\"{reason}\"}}}},\"depends_on\":[],\"blocked\":null,{NO_CLAIM}}}"
    );
}

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

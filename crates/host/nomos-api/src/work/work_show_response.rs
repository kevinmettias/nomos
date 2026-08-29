//! [`Handle_Work_Show`] and its own [`WorkShowResponse`].

use nomos_ledger::{ItemId, LedgerItem, Territory};
use nomos_platform::Timestamp;
use serde::Serialize;
use std::path::Path;

/// Reports one item, exactly as `nomos work show` would, and hands back a JSON-serializable
/// response.
///
/// `nomos_work_orchestration::run::Show_View` reads the whole board plus this tree's current
/// revision and leaves the lookup by id to the caller -- `WorkCommand::Show`'s own `item`
/// field is never read inside `Run` itself. `crates/host/nomos-cli/src/work.rs`'s own
/// `Render_Show` does that lookup after the fact; this function does the same, rather than
/// inventing a server-side filter `nomos_work_orchestration` itself does not have.
#[must_use]
pub fn Handle_Work_Show(directory: &Path, item: &ItemId) -> WorkShowResponse
{
    use super::Ledger_At;
    use nomos_platform_std::StdProcessLauncher;
    use nomos_work_orchestration::WorkCommand;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Show { item: item.clone() },
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Show(shown) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkShowResponse::From(item, shown);
}

/// What a real `nomos work show` produced, in a shape `serde_json` can hand across a wire.
///
/// A three-state tagged enum, the same honesty shape `crate::work::work_list_response::WorkListResponse`
/// already holds to for a two-state case, extended by one: unlike `list`, `show` can
/// legitimately fail to find its subject on a board that itself read fine, and collapsing
/// that into `Unreadable` would misreport a real miss as a broken ledger.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkShowResponse
{
    /// The item was on the board.
    Found
    {
        /// The item, including what has happened to it.
        ///
        /// Boxed: `LedgerItem` is far larger than the other two variants, and an unboxed
        /// field here would size every `WorkShowResponse` -- `NotFound` and `Unreadable`
        /// included -- to its width.
        item: Box<LedgerItem>,
        /// This tree's revision right now, or `None` if it could not be read.
        current_revision: Option<String>,
        /// The moment the board was read.
        now: Timestamp,
    },
    /// The board read cleanly, but no item on it carries this id.
    NotFound
    {
        /// The id that was asked for.
        item: ItemId,
    },
    /// The ledger file could not be read at all.
    Unreadable
    {
        /// What went wrong, as `LedgerError`'s own `Display` renders it.
        cause: String,
    },
}

impl WorkShowResponse
{
    pub(crate) fn From(
        item: &ItemId,
        shown: Result<nomos_work_orchestration::ShowView, nomos_ledger::LedgerError>,
    ) -> Self
    {
        let view = match shown
        {
            Ok(view) => view,
            Err(error) => return Self::Unreadable { cause: error.to_string() },
        };

        return match view.document.items.into_iter().find(|candidate| return &candidate.id == item)
        {
            Some(found) => Self::Found {
                item: Box::new(found),
                current_revision: view.current_revision,
                now: view.now,
            },
            None => Self::NotFound { item: item.clone() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::Scratch_Board_With_One_Item;

    #[test]
    fn Test_Showing_An_Item_On_The_Board_Should_Find_It()
    {
        let (directory, id) = Scratch_Board_With_One_Item();

        let response = Handle_Work_Show(&directory, &id);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkShowResponse::Found { item, .. } = response
        else
        {
            // This fixture writes a real board carrying `id`, so showing that same id must
            // find it -- reaching `NotFound` or `Unreadable` here means the show path itself
            // regressed, not a condition this test should assert around.
            panic!("an item real on the board is found, not missed or refused");
        };
        assert_eq!(item.id, id);
    }

    #[test]
    fn Test_Showing_An_Absent_Id_On_A_Readable_Board_Should_Be_Not_Found_Not_Unreadable()
    {
        let (directory, _id) = Scratch_Board_With_One_Item();
        let absent = ItemId::New("NO-SUCH-ITEM");

        let response = Handle_Work_Show(&directory, &absent);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkShowResponse::NotFound { item } = response
        else
        {
            // This fixture writes a real, readable board that never carries `absent`'s id,
            // so a miss is the only correct outcome -- reaching `Unreadable` here means the
            // board itself failed to read, not the lookup this test is exercising.
            panic!("a board that reads cleanly but lacks this id is a real miss, not a refusal");
        };
        assert_eq!(item, absent);
    }

    #[test]
    fn Test_A_Real_Found_Response_Should_Round_Trip_As_Json()
    {
        let (directory, id) = Scratch_Board_With_One_Item();

        let response = Handle_Work_Show(&directory, &id);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkShowResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkShowResponse always has this field");

        assert_eq!(outcome, "found", "{json}");
    }
}

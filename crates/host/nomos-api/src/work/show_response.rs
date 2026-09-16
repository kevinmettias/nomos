//! [`Handle_Work_Show`] and its own [`ShowResponse`].

use nomos_ledger::{ItemId, LedgerItem};
use nomos_platform::Timestamp;
use nomos_work_orchestration::WorkCommand;
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
pub fn Handle_Work_Show(directory: &Path, item: &ItemId) -> ShowResponse
{
    let outcome = super::Run_Empty_Territory_Command(directory, WorkCommand::Show { item: item.clone() });

    let nomos_work_orchestration::WorkOutcome::Show(shown) = outcome
    else
    {
        return ShowResponse::Unreadable {
            cause: super::UNANSWERED_WORK_OUTCOME.to_owned(),
        };
    };

    return ShowResponse::From(item, shown);
}

/// What a real `nomos work show` produced, in a shape `serde_json` can hand across a wire.
///
/// A three-state tagged enum, the same honesty shape `crate::work::list_response::ListResponse`
/// already holds to for a two-state case, extended by one: unlike `list`, `show` can
/// legitimately fail to find its subject on a board that itself read fine, and collapsing
/// that into `Unreadable` would misreport a real miss as a broken ledger.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ShowResponse
{
    /// The item was on the board.
    Found
    {
        /// The item, including what has happened to it.
        ///
        /// Boxed: `LedgerItem` is far larger than the other two variants, and an unboxed
        /// field here would size every `ShowResponse` -- `NotFound` and `Unreadable`
        /// included -- to its width.
        item: Box<LedgerItem>,
        /// This tree's revision right now, or `None` if it could not be read.
        current_revision: Option<String>,
        /// The moment the board was read.
        #[serde(with = "nomos_platform::timestamp_serde")]
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

impl ShowResponse
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
    use crate::work::tests_support::{BoardWithOneItem, Scratch_Board_With_One_Item};

    #[test]
    fn Test_Handle_Work_Show_Should_Find_An_Item_On_The_Board()
    {
        let BoardWithOneItem { directory, id } = Scratch_Board_With_One_Item()
            .expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_Show(&directory, &id);

        let _ignored = std::fs::remove_dir_all(&directory);

        let ShowResponse::Found { item, .. } = response
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
    fn Test_Ledger_At_Should_Read_Cleanly_Even_When_An_Absent_Id_Is_Not_Found()
    {
        let BoardWithOneItem { directory, .. } = Scratch_Board_With_One_Item()
            .expect("the temp directory is writable and the scratch ledger is writable");
        let absent = ItemId::New("NO-SUCH-ITEM");

        let response = Handle_Work_Show(&directory, &absent);

        let _ignored = std::fs::remove_dir_all(&directory);

        let ShowResponse::NotFound { item } = response
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
    fn Test_From_Should_Produce_A_Found_Response_That_Round_Trips_As_Json()
    {
        let BoardWithOneItem { directory, id } = Scratch_Board_With_One_Item()
            .expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_Show(&directory, &id);

        let _ignored = std::fs::remove_dir_all(&directory);

        crate::test_support::Assert_Round_Trips_As_Json(&response, "found")
            .expect("a found response serializes and parses back as a tagged object");
    }
}

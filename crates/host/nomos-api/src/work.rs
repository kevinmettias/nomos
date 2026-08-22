//! A second real caller of `nomos_work_orchestration::Run` -- `List`, `Show` and `Validate`,
//! the same "one verb at a time, not the whole command set" scope `Handle_Gate_Run` already
//! uses for Gate's own `run`.

use nomos_ledger::{FileLedger, ItemId, LedgerDocument, LedgerItem, Territory};
use nomos_platform::Timestamp;
use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher, SystemClock};
use nomos_work_orchestration::WorkCommand;
use serde::Serialize;
use std::path::Path;

/// Lists every item on the board at `directory`, exactly as `nomos work list` would, and
/// hands back a JSON-serializable response.
///
/// `directory` is expected to hold `ledger.json` and `ledger.lock`, the same layout
/// `crates/host/nomos-cli/src/work.rs`'s own composition root reads. The `published`
/// closure `nomos_work_orchestration::Run` takes is asked for lazily and reached only by
/// `WorkCommand::Add` (that crate's own doc), so `List` never reaches it -- an empty,
/// real `Territory` is handed here rather than a closure that would panic if this crate's
/// own scope ever widened past `List` without updating this comment.
#[must_use]
pub fn Handle_Work_List(directory: &Path) -> WorkListResponse
{
    let mut ledger = FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::List { state: None },
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::List(listed) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkListResponse::From(listed);
}

/// What a real `nomos work list` produced, in a shape `serde_json` can hand across a wire.
///
/// A tagged enum rather than a bare `Result`-shaped struct: `nomos_ledger::LedgerError`
/// does not derive `Serialize` -- nothing needed a wire format for it before this crate
/// existed -- and naming both outcomes here, the same honesty this workspace's other
/// two-state vocabularies (`Applicability`, `GateRunResponse::Disposition`) already hold
/// to, is clearer than collapsing a real refusal into an empty success.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkListResponse
{
    /// The board was read.
    Listed
    {
        /// Every item, as the ledger holds them.
        document: LedgerDocument,
        /// The moment the board was read.
        now: Timestamp,
    },
    /// The ledger file could not be read at all.
    Unreadable
    {
        /// What went wrong, as `LedgerError`'s own `Display` renders it.
        cause: String,
    },
}

impl WorkListResponse
{
    fn From(listed: Result<nomos_work_orchestration::BoardView, nomos_ledger::LedgerError>) -> Self
    {
        return match listed
        {
            Ok(view) => Self::Listed { document: view.document, now: view.now },
            Err(error) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

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
    let mut ledger = FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Show { item: item.clone() },
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Show(shown) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkShowResponse::From(item, shown);
}

/// What a real `nomos work show` produced, in a shape `serde_json` can hand across a wire.
///
/// A three-state tagged enum, the same honesty shape [`WorkListResponse`] already holds to
/// for a two-state case, extended by one: unlike `list`, `show` can legitimately fail to
/// find its subject on a board that itself read fine, and collapsing that into `Unreadable`
/// would misreport a real miss as a broken ledger.
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
    fn From(
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

/// Checks the board at `directory` against its own invariants, exactly as `nomos work
/// validate` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Validate(directory: &Path) -> WorkValidateResponse
{
    let mut ledger = FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Validate,
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Validate(validated) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkValidateResponse::From(validated);
}

/// What a real `nomos work validate` produced, in a shape `serde_json` can hand across a
/// wire.
///
/// The same two-state honesty shape [`WorkListResponse`] already holds to: `nomos_ledger::
/// LedgerError` does not derive `Serialize`, so `Invalid` names both a genuine invariant
/// violation (`LedgerError::Invalid`'s own variant) and every other read failure alike, by
/// the same `Display` string `LedgerError` itself already collapses them into -- no finer
/// split is invented here than the type underneath draws.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkValidateResponse
{
    /// The board satisfies its own invariants.
    Valid
    {
        /// The board, as validated.
        document: LedgerDocument,
    },
    /// The board could not be read, or it read but violated an invariant.
    Invalid
    {
        /// What went wrong, as `LedgerError`'s own `Display` renders it.
        cause: String,
    },
}

impl WorkValidateResponse
{
    fn From(validated: Result<LedgerDocument, nomos_ledger::LedgerError>) -> Self
    {
        return match validated
        {
            Ok(document) => Self::Valid { document },
            Err(error) => Self::Invalid { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A scratch board of this test's own -- never the real shared `work/` directory,
    /// which live sessions write to concurrently. The same `std::env::temp_dir()` /
    /// `std::process::id()` scoping `crates/host/nomos-cli/tests/scratch_ledger` already
    /// uses, and the same minimal two-key shape (`schema_version`, `items`) `work/
    /// ledger.json` itself has.
    fn Scratch_Board() -> std::path::PathBuf
    {
        let directory = std::env::temp_dir().join(format!("nomos-api-work-list-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("creates a scratch directory");
        std::fs::write(directory.join("ledger.json"), "{\"schema_version\": 5, \"items\": []}\n")
            .expect("writes a minimal valid ledger");

        return directory;
    }

    #[test]
    fn Test_Listing_A_Real_Empty_Board_Should_Read_It_Not_Refuse_It()
    {
        let directory = Scratch_Board();

        let response = Handle_Work_List(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkListResponse::Listed { document, .. } = response
        else
        {
            panic!("a well-formed, empty ledger reads cleanly");
        };
        assert!(document.items.is_empty());
    }

    #[test]
    fn Test_A_Real_Listings_Response_Should_Round_Trip_As_Json()
    {
        let directory = Scratch_Board();

        let response = Handle_Work_List(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkListResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkListResponse always has this field");

        assert_eq!(outcome, "listed", "{json}");
    }

    /// A scratch board carrying one real item, for the `Show` tests -- `Scratch_Board`'s own
    /// empty board proves nothing about a found item.
    fn Scratch_Board_With_One_Item() -> (std::path::PathBuf, ItemId)
    {
        let directory = std::env::temp_dir().join(format!("nomos-api-work-show-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("creates a scratch directory");
        let id = ItemId::New("SCRATCH-ITEM");
        let ledger = format!(
            "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
             \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
             {{\"resolution\": \"File\", \"paths\": [], \"patterns\": []}}, \"state\": \"Ready\"}}]}}\n"
        );
        std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

        return (directory, id);
    }

    #[test]
    fn Test_Showing_An_Item_On_The_Board_Should_Find_It()
    {
        let (directory, id) = Scratch_Board_With_One_Item();

        let response = Handle_Work_Show(&directory, &id);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkShowResponse::Found { item, .. } = response
        else
        {
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

    #[test]
    fn Test_Validating_A_Real_Well_Formed_Board_Should_Be_Valid()
    {
        let directory = Scratch_Board();

        let response = Handle_Work_Validate(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkValidateResponse::Valid { document } = response
        else
        {
            panic!("an empty, well-formed ledger satisfies every invariant Validate checks");
        };
        assert!(document.items.is_empty());
    }

    #[test]
    fn Test_Validating_A_Board_With_A_Real_Violation_Should_Be_Invalid()
    {
        // `Scratch_Board_With_One_Item`'s item is `Ready` with an empty territory --
        // `nomos_ledger::store::validation::Check_Territory`'s own "workable but reserves
        // nothing" violation, not a fixture built for this test alone.
        let (directory, _id) = Scratch_Board_With_One_Item();

        let response = Handle_Work_Validate(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkValidateResponse::Invalid { cause } = response
        else
        {
            panic!("an item that is workable but reserves nothing is a real invariant violation");
        };
        assert!(cause.contains("reserves nothing"), "{cause}");
    }

    #[test]
    fn Test_A_Real_Valid_Response_Should_Round_Trip_As_Json()
    {
        let directory = Scratch_Board();

        let response = Handle_Work_Validate(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkValidateResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkValidateResponse always has this field");

        assert_eq!(outcome, "valid", "{json}");
    }
}

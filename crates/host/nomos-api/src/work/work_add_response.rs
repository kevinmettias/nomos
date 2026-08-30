//! [`Handle_Work_Add`] and its own [`WorkAddResponse`].

use nomos_ledger::{AddRefusal, LedgerItem, Territory};
use serde::Serialize;
use std::path::Path;

/// Puts `item` on the board at `directory`, declaring `amending` as the published records it
/// edits rather than allocates, exactly as `nomos work add` would, and hands back a
/// JSON-serializable response.
#[must_use]
pub fn Handle_Work_Add(directory: &Path, item: &LedgerItem, amending: &Territory) -> WorkAddResponse
{
    use super::Ledger_At;
    use nomos_platform_std::StdProcessLauncher;
    use nomos_work_orchestration::WorkCommand;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Add { item: Box::new(item.clone()), amending: amending.clone() },
        &mut ledger,
        &StdProcessLauncher,
        || Published_Records(directory),
    );

    let nomos_work_orchestration::WorkOutcome::Add(added) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkAddResponse::From(added);
}

/// What a real `nomos work add` produced, in a shape `serde_json` can hand across a wire.
///
/// `Refused` carries only `cause`, unlike the retryable/judged extra field
/// `WorkReservationResponse`/`WorkAbandonResponse`/`WorkDeclineResponse`/`WorkFinishResponse`
/// each add to theirs: `nomos_ledger::AddRefusal` exposes no single `Is_X`-style method the
/// way `ClaimRefusal`/`FinishRefusal` do -- `crates/host/nomos-cli/src/work/report.rs`'s own
/// `Code_For_Refusal` maps its six variants across three different exit codes, and nothing
/// here reduces that to one bit without becoming a second, uncanonical implementation of a
/// distinction only that function currently draws.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkAddResponse
{
    /// The item was recorded.
    Added,
    /// The item was not recorded.
    Refused
    {
        /// What went wrong, as `AddRefusal::Describe` renders it.
        cause: String,
    },
}

impl WorkAddResponse
{
    pub(crate) fn From(result: Result<(), AddRefusal>) -> Self
    {
        return match result
        {
            Ok(()) => Self::Added,
            Err(refusal) => Self::Refused { cause: refusal.Describe() },
        };
    }
}

/// Every file this repository has already published as a decision record, as a `Territory`
/// -- a deliberate twin of `crates/host/nomos-cli/src/work.rs`'s own `Published_Records`,
/// not a shared dependency of it, the same "a walk is a composition-root concern"
/// `crate::sources` already documents for Gate's own walk. `nomos_platform::FileSystem` has
/// no directory-listing operation, so `nomos_work_orchestration::Run`'s own `published`
/// closure exists precisely so each composition root can answer this its own way.
///
/// An unreadable or absent `docs/records` yields `Territory::Empty()` rather than refusing --
/// the one judgement worth stating here, because this repository's usual rule is the
/// opposite. It does not apply: this is input to the open-item comparison, not the check
/// itself, and a tree with no `docs/records` is a ledger being used somewhere that has none.
fn Published_Records(directory: &Path) -> Territory
{
    let Some(root) = directory.parent()
    else
    {
        return Territory::Empty();
    };

    let mut published = Record_Files(root);
    published.sort();

    return Territory::Of_Files(published);
}

/// Where a repository's own published decision records live, relative to its root.
const RECORD_DIRECTORY: &str = "docs/records";

/// Every file directly under `root`'s record directory, as a territory is spelled --
/// repository-relative and forward-slashed.
fn Record_Files(root: &Path) -> Vec<String>
{
    let Ok(entries) = std::fs::read_dir(root.join(RECORD_DIRECTORY))
    else
    {
        return Vec::new();
    };

    let mut published = Vec::new();
    for entry in entries.flatten()
    {
        if let Some(name) = entry.file_name().to_str()
        {
            published.push(format!("{RECORD_DIRECTORY}/{name}"));
        }
    }

    return published;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{Scratch_Board, Scratch_Board_With_A_Claimable_Item};
    use nomos_ledger::{ItemId, ItemKind, ItemOrigin, ItemState};

    #[test]
    fn Test_Adding_A_Real_Well_Formed_Item_To_A_Fresh_Board_Should_Record_It()
    {
        let directory = Scratch_Board();
        let item = Real_New_Item(ItemId::New("SCRATCH-NEW"), "a");

        let response = Handle_Work_Add(&directory, &item, &Territory::Empty());

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, WorkAddResponse::Added), "{response:?}");
    }

    #[test]
    fn Test_Adding_An_Item_Whose_Id_Is_Already_On_The_Board_Should_Be_Refused()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let item = Real_New_Item(id, "b");

        let response = Handle_Work_Add(&directory, &item, &Territory::Empty());

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkAddResponse::Refused { cause } = response
        else
        {
            // This fixture builds a board that already carries `id`, so adding it again must
            // refuse -- reaching any other variant here means the duplicate-id check itself
            // stopped enforcing, not a condition this test should assert around.
            panic!("an id already on the board is a real refusal, not a record");
        };
        assert!(cause.contains("already on the ledger"), "{cause}");
    }

    #[test]
    fn Test_A_Real_Added_Response_Should_Round_Trip_As_Json()
    {
        let directory = Scratch_Board();
        let item = Real_New_Item(ItemId::New("SCRATCH-NEW"), "a");

        let response = Handle_Work_Add(&directory, &item, &Territory::Empty());

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkAddResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkAddResponse always has this field");

        assert_eq!(outcome, "added", "{json}");
    }

    /// A well-formed, real `LedgerItem` this test's own -- `id` is the only field a caller
    /// varies below, since every other field is incidental to what `Add` itself judges.
    fn Real_New_Item(id: ItemId, territory_path: &str) -> LedgerItem
    {
        return LedgerItem {
            id,
            title: "t".to_owned(),
            why: "w".to_owned(),
            done_when: "d".to_owned(),
            kind: ItemKind::Capability,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files([territory_path]),
            state: ItemState::Ready,
            depends_on: Vec::new(),
            blocked: None,
            claim: None,
            verification: None,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            declined: None,
        };
    }
}

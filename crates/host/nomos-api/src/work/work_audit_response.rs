//! [`Handle_Work_Audit`] and its own [`WorkAuditResponse`].

use nomos_ledger::{Claim_Refusal, ItemState, Territory};
use nomos_platform::Timestamp;
use serde::Serialize;
use std::path::Path;

use super::{BlockedItem, Ledger_At};

/// Audits the board at `directory` for what stands between a `Ready` item and an agent that
/// would take it, exactly as `nomos work audit` would, and hands back a JSON-serializable
/// response.
///
/// `nomos_work_orchestration::Run`'s own `WorkCommand::Audit` arm returns the identical
/// `Result<BoardView, LedgerError>` its `List` arm does -- both call the same `Board_View`
/// helper. What makes an audit an audit rather than a second listing is a caller-side filter:
/// `crates/host/nomos-cli/src/work/report.rs`'s `Blocking_Refusal` keeps only `Ready` items
/// `nomos_ledger::Claim_Refusal` actually refuses, and adds nothing over that one canonical
/// function but a one-line state check -- "two implementations of one rule is how they come
/// to disagree" (`OD-LEDGER-005`). This function applies the same filter, over the same
/// public `Claim_Refusal`, rather than reaching into nomos-cli's own `pub(super)`
/// `Blocking_Refusal`. It does not reproduce nomos-cli's own `Refusal_Label` word mapping --
/// a wire caller gets each blocked item's full `ClaimRefusal::Describe()` text instead of
/// that terse label.
#[must_use]
pub fn Handle_Work_Audit(directory: &Path) -> WorkAuditResponse
{
    use nomos_platform_std::StdProcessLauncher;
    use nomos_work_orchestration::WorkCommand;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Audit,
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Audit(audited) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkAuditResponse::From(audited);
}

/// What a real `nomos work audit` produced, in a shape `serde_json` can hand across a wire.
///
/// The same two-state honesty shape `crate::work::work_list_response::WorkListResponse` already holds to:
/// `Unreadable` names the board itself failing to read, the one way this can fail short of a
/// genuine audit answer.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkAuditResponse
{
    /// The board was read, and every claimable item currently blocked is named.
    Audited
    {
        /// Every `Ready` item something stands between and an agent that would take it.
        blocked: Vec<BlockedItem>,
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

impl WorkAuditResponse
{
    pub(crate) fn From(audited: Result<nomos_work_orchestration::BoardView, nomos_ledger::LedgerError>) -> Self
    {
        let view = match audited
        {
            Ok(view) => view,
            Err(error) => return Self::Unreadable { cause: error.to_string() },
        };

        let blocked = view
            .document
            .items
            .iter()
            .filter(|item| return item.state == ItemState::Ready)
            .filter_map(|item| {
                let refusal = Claim_Refusal(&view.document, &item.id, view.now)?;
                return Some(BlockedItem { item: item.clone(), cause: refusal.Describe() });
            })
            .collect();

        return Self::Audited { blocked, now: view.now };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{Scratch_Board, Scratch_Board_With_A_Blocked_Item};

    #[test]
    fn Test_Auditing_A_Board_With_A_Real_Dependency_Should_Name_The_Blocked_Item_Not_Its_Dependency()
    {
        let (directory, dependency, blocked) = Scratch_Board_With_A_Blocked_Item();

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkAuditResponse::Audited { blocked: found, .. } = response
        else
        {
            // This fixture writes a real, well-formed ledger file, so reading it back must
            // succeed -- reaching `Unreadable` here means the scratch fixture itself is
            // broken, not a runtime condition this test should tolerate.
            panic!("a well-formed board reads cleanly");
        };
        assert_eq!(found.len(), 1, "{found:?}");
        let only = found.first().expect("just asserted this has exactly one element");
        assert_eq!(only.item.id, blocked);
        assert_ne!(only.item.id, dependency);
    }

    #[test]
    fn Test_Auditing_A_Board_With_No_Ready_Items_Should_Report_Nothing_Blocked()
    {
        let directory = Scratch_Board();

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkAuditResponse::Audited { blocked, .. } = response
        else
        {
            // This fixture writes a real, well-formed, empty ledger file, so reading it back
            // must succeed -- reaching `Unreadable` here means the scratch fixture itself is
            // broken, not a runtime condition this test should tolerate.
            panic!("a well-formed, empty ledger reads cleanly");
        };
        assert!(blocked.is_empty());
    }

    #[test]
    fn Test_A_Real_Audited_Response_Should_Round_Trip_As_Json()
    {
        let (directory, _dependency, _blocked) = Scratch_Board_With_A_Blocked_Item();

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkAuditResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkAuditResponse always has this field");

        assert_eq!(outcome, "audited", "{json}");
    }
}

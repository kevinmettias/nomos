//! [`Handle_Work_Audit`] and its own [`AuditResponse`].

use nomos_composer_std::FILE_SYSTEM;
use nomos_ledger::{Claim_Refusal, ItemId, ItemState};
use nomos_platform::Timestamp;
use nomos_work_orchestration::WorkCommand;
use serde::Serialize;
use std::path::Path;

use super::BlockedItem;

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
///
/// The absent-path half uses the one `Territory::Absent_Paths` in `nomos-scope-verification`,
/// the crate that owns territory, so this host and nomos-cli cannot grow two agreeing copies
/// of the check. `directory` is threaded in for the same reason the CLI threads a root and a
/// `FileSystem` into its own audit: the question cannot be answered from the document alone.
#[must_use]
pub fn Handle_Work_Audit(directory: &Path) -> AuditResponse
{
    let outcome = super::Run_Empty_Territory_Command(directory, WorkCommand::Audit);

    let nomos_work_orchestration::WorkOutcome::Audit(audited) = outcome
    else
    {
        return AuditResponse::Unreadable {
            cause: super::UNANSWERED_WORK_OUTCOME.to_owned(),
        };
    };

    return AuditResponse::From(audited, directory);
}

/// One reserved path that is not in the tree, for an item that could still be worked.
///
/// Authored as *absent* rather than *decayed*: the same two-readings honesty the CLI's
/// report holds to, because an item about to create its reserved file and an item whose
/// file a peer moved are indistinguishable by existence alone. A wire caller gets the fact —
/// the path is not in the tree — and none of this type guesses which case it is.
#[derive(Debug, Serialize)]
pub struct AbsentPath
{
    /// The item whose territory reserves the path.
    pub item: ItemId,
    /// The reserved path that is not in the tree.
    pub path: String,
}

/// What a real `nomos work audit` produced, in a shape `serde_json` can hand across a wire.
///
/// The same two-state honesty shape `crate::work::list_response::ListResponse` already holds to:
/// `Unreadable` names the board itself failing to read, the one way this can fail short of a
/// genuine audit answer.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum AuditResponse
{
    /// The board was read, and every claimable item currently blocked is named.
    Audited
    {
        /// Every `Ready` item something stands between and an agent that would take it.
        blocked: Vec<BlockedItem>,
        /// Every reserved path that is not in the tree, on an item that could still be worked.
        absent: Vec<AbsentPath>,
        /// The moment the board was read.
        #[serde(serialize_with = "nomos_platform::timestamp_serde::Write_Unix_Seconds")]
        now: Timestamp,
    },
    /// The ledger file could not be read at all.
    Unreadable
    {
        /// What went wrong, as `LedgerError`'s own `Display` renders it.
        cause: String,
    },
}

impl AuditResponse
{
    pub(crate) fn From(audited: Result<nomos_work_orchestration::BoardView, nomos_ledger::LedgerError>, directory: &Path) -> Self
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

        let absent = Absent_Paths(&view, directory);

        return Self::Audited { blocked, absent, now: view.now };
    }
}

/// Every reserved path that is not in the tree, on an item that could still be worked.
///
/// Done and Declined items are skipped — their territory is history, and a stale path in it
/// is not debt. The root is the repository the `work/` directory sits under; a `work/` at the
/// root of nothing has no tree to check against, and reads as no absent paths.
fn Absent_Paths(view: &nomos_work_orchestration::BoardView, directory: &Path) -> Vec<AbsentPath>
{
    let Some(root) = directory.parent()
    else
    {
        return Vec::new();
    };

    let mut absent = Vec::new();
    for item in &view.document.items
    {
        if item.state.Is_Finished()
        {
            continue;
        }
        for path in item.territory.Absent_Paths(root, &FILE_SYSTEM)
        {
            absent.push(AbsentPath { item: item.id.clone(), path });
        }
    }

    return absent;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{
        BoardWithABlockedItem, Scratch_Board, Scratch_Board_With_A_Blocked_Item,
        Unique_Scratch_Directory,
    };

    #[test]
    fn Test_Handle_Work_Audit_And_Scratch_Board_With_A_Blocked_Item_Should_Name_The_Blocked_Item_Not_Its_Dependency()
    {
        let BoardWithABlockedItem { directory, dependency, blocked } =
            Scratch_Board_With_A_Blocked_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let AuditResponse::Audited { blocked: found, .. } = response
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
    fn Test_Scratch_Board_And_Unique_Scratch_Directory_Should_Report_Nothing_Blocked_On_An_Empty_Board()
    {
        let directory = Scratch_Board().expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let AuditResponse::Audited { blocked, .. } = response
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
    fn Test_From_Should_Produce_An_Audited_Response_That_Round_Trips_As_Json()
    {
        let BoardWithABlockedItem { directory, .. } = Scratch_Board_With_A_Blocked_Item()
            .expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        crate::test_support::Assert_Round_Trips_As_Json(&response, "audited")
            .expect("an audited response serializes and parses back as a tagged object");
    }

    #[test]
    fn Test_Handle_Work_Audit_Should_Report_Each_Absent_Reserved_Path()
    {
        let BoardWithABlockedItem { directory, dependency, blocked } =
            Scratch_Board_With_A_Blocked_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let AuditResponse::Audited { absent, .. } = response
        else
        {
            panic!("a well-formed board reads cleanly");
        };
        assert_eq!(absent.len(), 2, "{absent:?}");
        assert!(
            absent.iter().any(|finding| finding.item == dependency && finding.path == "a"),
            "the dependency's reserved path a is absent and must be reported: {absent:?}"
        );
        assert!(
            absent.iter().any(|finding| finding.item == blocked && finding.path == "b"),
            "the blocked item's reserved path b is absent and must be reported: {absent:?}"
        );
    }

    #[test]
    fn Test_An_Absent_Path_Should_Not_Distinguish_About_To_Create_From_Peer_Moved()
    {
        let BoardWithABlockedItem { directory, .. } = Scratch_Board_With_A_Blocked_Item()
            .expect("the temp directory is writable and the scratch ledger is writable");

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let AuditResponse::Audited { absent, .. } = response
        else
        {
            panic!("a well-formed board reads cleanly");
        };
        let serialized = serde_json::to_string(&absent).expect("an AbsentPath serializes");
        assert!(
            !serialized.contains("decayed") && !serialized.contains("created") && !serialized.contains("moved"),
            "an absent path names the fact and not a guess about which case it is: {serialized}"
        );
    }

    #[test]
    fn Test_Handle_Work_Audit_Should_Not_Report_A_Done_Items_Stale_Path()
    {
        let directory = Unique_Scratch_Directory("audit-stale")
            .expect("the temp directory is writable");
        let ledger = r#"{"schema_version": 6, "items": [
            {"id": "LIVE", "title": "t", "why": "w", "done_when": "d", "kind": "Capability",
             "origin": "Proposed", "widened": [], "territory": {"resolution": "File",
             "paths": ["live.rs"], "patterns": []}, "state": "Ready"},
            {"id": "DONE", "title": "t", "why": "w", "done_when": "d", "kind": "Capability",
             "origin": "Proposed", "widened": [], "territory": {"resolution": "File",
             "paths": ["done.rs"], "patterns": []}, "state": "Done"}
        ]}"#;
        std::fs::write(directory.join("ledger.json"), ledger).expect("the scratch ledger is writable");

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let AuditResponse::Audited { absent, .. } = response
        else
        {
            panic!("a well-formed board reads cleanly");
        };
        assert_eq!(absent.len(), 1, "{absent:?}");
        assert_eq!(absent.first().expect("asserted len 1").path, "live.rs");
    }
}

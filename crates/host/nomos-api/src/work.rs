//! A second real caller of `nomos_work_orchestration::Run` -- `List`, `Show`, `Validate`,
//! `Audit`, `Claim`, `Renew`, `TakeOver`, `Abandon`, `Decline` and `Finish`, the same "one
//! verb at a time, not the whole command set" scope `Handle_Gate_Run` already uses for
//! Gate's own `run`.

use nomos_ledger::{
    Claim_Refusal, ClaimRefusal, FileLedger, FinishRefusal, ItemId, ItemState, LedgerDocument, LedgerItem, Reservation,
    Territory, VerificationRecord,
};
use nomos_platform::Timestamp;
use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher, SystemClock};
use nomos_work_orchestration::{ClaimRequest, EndingRequest, WorkCommand};
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
    let mut ledger = FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Audit,
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Audit(audited) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkAuditResponse::From(audited);
}

/// One `Ready` item [`Handle_Work_Audit`] found blocked, paired with why.
#[derive(Debug, Serialize)]
pub struct BlockedItem
{
    /// The blocked item, including what has happened to it.
    pub item: LedgerItem,
    /// What [`nomos_ledger::ClaimRefusal::Describe`] says stands between it and a claimant.
    pub cause: String,
}

/// What a real `nomos work audit` produced, in a shape `serde_json` can hand across a wire.
///
/// The same two-state honesty shape [`WorkListResponse`] already holds to: `Unreadable`
/// names the board itself failing to read, the one way this can fail short of a genuine
/// audit answer.
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
    fn From(audited: Result<nomos_work_orchestration::BoardView, nomos_ledger::LedgerError>) -> Self
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

/// Grants `request` a reservation on the board at `directory`, exactly as `nomos work claim`
/// would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Claim(directory: &Path, request: &ClaimRequest) -> WorkReservationResponse
{
    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Claim(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Claim(claimed) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkReservationResponse::From(claimed);
}

/// Extends `request`'s already-held lease on the board at `directory`, exactly as `nomos
/// work renew` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Renew(directory: &Path, request: &ClaimRequest) -> WorkReservationResponse
{
    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Renew(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Renew(renewed) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkReservationResponse::From(renewed);
}

/// Takes over `request`'s item on the board at `directory` from a lapsed holder, exactly as
/// `nomos work takeover` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_TakeOver(directory: &Path, request: &ClaimRequest) -> WorkReservationResponse
{
    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::TakeOver(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::TakeOver(taken_over) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkReservationResponse::From(taken_over);
}

/// The `FileLedger` composition every `Handle_Work_*` function in this module builds --
/// factored out once a third function ([`Handle_Work_Claim`]) needed exactly the same four
/// lines the first two already had inline.
fn Ledger_At(directory: &Path) -> FileLedger<StdFileSystem, SystemClock, FileLock>
{
    return FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );
}

/// A granted or refused reservation, in a shape `serde_json` can hand across a wire.
///
/// One shared response type for [`Handle_Work_Claim`], [`Handle_Work_Renew`] and
/// [`Handle_Work_TakeOver`] rather than three that could only ever agree by discipline --
/// the exact reason `nomos_work_orchestration::ClaimRequest` is already one shared request
/// type for these three verbs (its own doc says so), and the same division `crates/host/
/// nomos-cli/src/work.rs`'s own dispatch already draws: one `Report_Claim` renders all three
/// outcomes today.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkReservationResponse
{
    /// The reservation was granted.
    Reserved
    {
        /// The reservation itself.
        reservation: ReservationResponse,
    },
    /// The reservation was refused.
    Refused
    {
        /// What went wrong, as `ClaimRefusal::Describe` renders it.
        cause: String,
        /// Whether retrying later might succeed -- `ClaimRefusal::Is_Retryable`, the same
        /// signal `crates/host/nomos-cli/src/work/report.rs`'s own `Code_For` already
        /// branches an exit code on, handed to a wire caller directly since it has no
        /// process exit code to read.
        retryable: bool,
    },
}

impl WorkReservationResponse
{
    fn From(result: Result<Reservation, ClaimRefusal>) -> Self
    {
        return match result
        {
            Ok(reservation) => Self::Reserved { reservation: ReservationResponse::From(reservation) },
            Err(refusal) => Self::Refused { retryable: refusal.Is_Retryable(), cause: refusal.Describe() },
        };
    }
}

/// A serializable twin of [`nomos_ledger::Reservation`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct ReservationResponse
{
    /// The item claimed.
    pub item: ItemId,
    /// Who holds it.
    pub holder: String,
    /// When the lease lapses.
    pub expires_at: Timestamp,
}

impl ReservationResponse
{
    fn From(reservation: Reservation) -> Self
    {
        return Self {
            item: reservation.item,
            holder: reservation.holder,
            expires_at: reservation.expires_at,
        };
    }
}

/// Releases `request`'s claim on the board at `directory` without finishing it, exactly as
/// `nomos work abandon` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Abandon(directory: &Path, request: &EndingRequest) -> WorkAbandonResponse
{
    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Abandon(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Abandon(released) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkAbandonResponse::From(released);
}

/// A claim released, or the refusal that kept it held, in a shape `serde_json` can hand
/// across a wire.
///
/// Not shared with [`WorkDeclineResponse`], even though both wrap `Result<(), ClaimRefusal>`
/// today: `nomos_work_orchestration::EndingRequest`'s own doc says `abandon` and `decline`
/// "stay two verbs and two ledger calls" past sharing an argument shape, and `crates/host/
/// nomos-cli/src/work.rs`'s own dispatch keeps their rendering apart the same way.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkAbandonResponse
{
    /// The claim was given up; the item is claimable again.
    Abandoned,
    /// The claim could not be released.
    Refused
    {
        /// What went wrong, as `ClaimRefusal::Describe` renders it.
        cause: String,
        /// Whether retrying later might succeed -- `ClaimRefusal::Is_Retryable`.
        retryable: bool,
    },
}

impl WorkAbandonResponse
{
    fn From(result: Result<(), ClaimRefusal>) -> Self
    {
        return match result
        {
            Ok(()) => Self::Abandoned,
            Err(refusal) => Self::Refused { retryable: refusal.Is_Retryable(), cause: refusal.Describe() },
        };
    }
}

/// Ends `request`'s item on the board at `directory` as not being work, exactly as `nomos
/// work decline` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Decline(directory: &Path, request: &EndingRequest) -> WorkDeclineResponse
{
    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Decline(request.clone()),
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Decline(declined) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkDeclineResponse::From(declined);
}

/// An item ended, or the refusal that kept it open, in a shape `serde_json` can hand across
/// a wire. Not shared with [`WorkAbandonResponse`]; see that type's own doc for why.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkDeclineResponse
{
    /// The item was ended; it is no longer workable.
    Declined,
    /// The item could not be ended.
    Refused
    {
        /// What went wrong, as `ClaimRefusal::Describe` renders it.
        cause: String,
        /// Whether retrying later might succeed -- `ClaimRefusal::Is_Retryable`.
        retryable: bool,
    },
}

impl WorkDeclineResponse
{
    fn From(result: Result<(), ClaimRefusal>) -> Self
    {
        return match result
        {
            Ok(()) => Self::Declined,
            Err(refusal) => Self::Refused { retryable: refusal.Is_Retryable(), cause: refusal.Describe() },
        };
    }
}

/// Runs `item`'s verification predicate as `holder` on the board at `directory`, recording
/// it done if it passes, exactly as `nomos work finish` would, and hands back a
/// JSON-serializable response.
///
/// `nomos_work_orchestration::Run`'s own `WorkCommand::Finish` arm always calls
/// `nomos_ledger::Finish` with a `None` `working_directory` -- not a choice this function or
/// its caller can vary, so the gate's own lint step (derived from `.github/workflows/
/// gate.yml`) resolves relative to the calling process's own current directory, the same
/// fixed composition `nomos-cli`'s own `nomos work finish` already runs under.
#[must_use]
pub fn Handle_Work_Finish(directory: &Path, item: &ItemId, holder: &str) -> WorkFinishResponse
{
    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Finish { item: item.clone(), holder: holder.to_owned() },
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Finish(finished) = outcome
    else
    {
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkFinishResponse::From(finished);
}

/// What a real `nomos work finish` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkFinishResponse
{
    /// The predicate passed, and the item is recorded done.
    Finished
    {
        /// The evidence. `nomos_ledger::VerificationRecord` already derives `Serialize` --
        /// it is stored directly in `ledger.json` -- so no local twin is needed.
        record: VerificationRecord,
    },
    /// The item was not finished.
    Refused
    {
        /// What went wrong, as `FinishRefusal::Describe` renders it.
        cause: String,
        /// Whether the predicate ran and said no -- `FinishRefusal::Judged_The_Work`, the
        /// same signal `crates/host/nomos-cli/src/work/report.rs`'s own `Report_Finish`
        /// already branches an exit code on (a real judgment versus nobody finding out),
        /// handed to a wire caller directly since it has no process exit code to read.
        judged: bool,
    },
}

impl WorkFinishResponse
{
    fn From(result: Result<VerificationRecord, FinishRefusal>) -> Self
    {
        return match result
        {
            Ok(record) => Self::Finished { record },
            Err(refusal) => Self::Refused { judged: refusal.Judged_The_Work(), cause: refusal.Describe() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A scratch directory of this test's own -- never the real shared `work/` directory,
    /// which live sessions write to concurrently. The same `std::env::temp_dir()` /
    /// `std::process::id()` scoping `crates/host/nomos-cli/tests/scratch_ledger` already
    /// uses, plus a call-local counter: several tests below share one `label` (`Scratch_Board`
    /// backs both `List` and `Validate` tests, for instance), and the default test runner's
    /// threads raced on the one directory a bare pid gave them -- one thread's
    /// `remove_dir_all` deleting the `ledger.json` another was mid-read of. `process::id()`
    /// alone tells two runs of the whole suite apart; it says nothing about two calls inside
    /// one.
    fn Unique_Scratch_Directory(label: &str) -> std::path::PathBuf
    {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let directory =
            std::env::temp_dir().join(format!("nomos-api-work-{label}-{}-{unique}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("creates a scratch directory");

        return directory;
    }

    /// A scratch board with no items, for the `List`, `Validate` and `Audit` tests that need
    /// only a well-formed, empty ledger -- the same minimal two-key shape (`schema_version`,
    /// `items`) `work/ledger.json` itself has.
    fn Scratch_Board() -> std::path::PathBuf
    {
        let directory = Unique_Scratch_Directory("list");
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
        let directory = Unique_Scratch_Directory("show");
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

    /// A scratch board carrying two real items, one blocking the other -- for the `Audit`
    /// tests. `dependency` is `Ready` and unclaimed, so `blocked` earns a real
    /// `ClaimRefusal::DependencyUnmet` from `nomos_ledger::Claim_Refusal` rather than a
    /// fixture-only refusal this crate invented.
    fn Scratch_Board_With_A_Blocked_Item() -> (std::path::PathBuf, ItemId, ItemId)
    {
        let directory = Unique_Scratch_Directory("audit");
        let dependency = ItemId::New("SCRATCH-DEPENDENCY");
        let blocked = ItemId::New("SCRATCH-BLOCKED");
        let ledger = format!(
            "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{dependency}\", \"title\": \"t\", \
             \"why\": \"w\", \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \
             \"Proposed\", \"territory\": {{\"resolution\": \"File\", \"paths\": [\"a\"], \
             \"patterns\": []}}, \"state\": \"Ready\"}}, {{\"id\": \"{blocked}\", \"title\": \"t\", \
             \"why\": \"w\", \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \
             \"Proposed\", \"territory\": {{\"resolution\": \"File\", \"paths\": [\"b\"], \
             \"patterns\": []}}, \"state\": \"Ready\", \"depends_on\": [\"{dependency}\"]}}]}}\n"
        );
        std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

        return (directory, dependency, blocked);
    }

    #[test]
    fn Test_Auditing_A_Board_With_A_Real_Dependency_Should_Name_The_Blocked_Item_Not_Its_Dependency()
    {
        let (directory, dependency, blocked) = Scratch_Board_With_A_Blocked_Item();

        let response = Handle_Work_Audit(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkAuditResponse::Audited { blocked: found, .. } = response
        else
        {
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

    /// A scratch board carrying one real `Ready` item with a real, non-empty territory --
    /// unlike `Scratch_Board_With_One_Item`'s deliberately territory-less fixture (built for
    /// `Validate`'s own violation test), a document `Claim`/`Renew`/`TakeOver` write back
    /// must itself stay valid, so every fixture below reserves something real.
    fn Scratch_Board_With_A_Claimable_Item() -> (std::path::PathBuf, ItemId)
    {
        let directory = Unique_Scratch_Directory("claimable");
        let id = ItemId::New("SCRATCH-CLAIMABLE");
        let ledger = format!(
            "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
             \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
             {{\"resolution\": \"File\", \"paths\": [\"a\"], \"patterns\": []}}, \"state\": \"Ready\"}}]}}\n"
        );
        std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

        return (directory, id);
    }

    /// A scratch board carrying one real item already `Claimed`, with `holder` and
    /// `expires_at_unix` chosen by the caller -- an active lease far in the future for a
    /// `Renew` fixture, or one far in the past (lapsed) for a `TakeOver` fixture.
    fn Scratch_Board_With_A_Claimed_Item(holder: &str, expires_at_unix: i64) -> (std::path::PathBuf, ItemId)
    {
        let directory = Unique_Scratch_Directory("claimed");
        let id = ItemId::New("SCRATCH-CLAIMED");
        let ledger = format!(
            "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
             \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
             {{\"resolution\": \"File\", \"paths\": [\"a\"], \"patterns\": []}}, \"state\": \"Claimed\", \
             \"claim\": {{\"holder\": \"{holder}\", \"acquired_at\": 1, \"lease_expires_at\": \
             {expires_at_unix}}}}}]}}\n"
        );
        std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

        return (directory, id);
    }

    /// A scratch board carrying two real items over the same territory: one already
    /// `Claimed` with an active lease, the other `Ready` and unclaimed -- the real
    /// `ClaimRefusal::HeldBy` trigger `crates/substrate/nomos-ledger/src/store/refusal.rs`'s
    /// own `Held_Ground` looks for (another item's *live claim* over overlapping ground),
    /// not the `ClaimRefusal::NotClaimable` a second `claim` of the same already-`Claimed`
    /// item id would hit instead.
    fn Scratch_Board_With_A_Held_Territory_Conflict() -> (std::path::PathBuf, ItemId)
    {
        let directory = Unique_Scratch_Directory("held");
        let held = ItemId::New("SCRATCH-HELD");
        let contested = ItemId::New("SCRATCH-CONTESTED");
        let ledger = format!(
            "{{\"schema_version\": 5, \"items\": [\
             {{\"id\": \"{held}\", \"title\": \"t\", \"why\": \"w\", \"done_when\": \"d\", \
             \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
             {{\"resolution\": \"File\", \"paths\": [\"shared\"], \"patterns\": []}}, \"state\": \"Claimed\", \
             \"claim\": {{\"holder\": \"someone-else\", \"acquired_at\": 1, \"lease_expires_at\": \
             {far_future}}}}}, \
             {{\"id\": \"{contested}\", \"title\": \"t\", \"why\": \"w\", \"done_when\": \"d\", \
             \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
             {{\"resolution\": \"File\", \"paths\": [\"shared\"], \"patterns\": []}}, \"state\": \"Ready\"}}\
             ]}}\n",
            far_future = i64::from(u32::MAX)
        );
        std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

        return (directory, contested);
    }

    #[test]
    fn Test_Claiming_A_Real_Ready_Item_Should_Grant_A_Reservation()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = ClaimRequest { item: id.clone(), holder: "test-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkReservationResponse::Reserved { reservation } = response
        else
        {
            panic!("an unclaimed Ready item grants a reservation");
        };
        assert_eq!(reservation.item, id);
        assert_eq!(reservation.holder, "test-holder");
    }

    #[test]
    fn Test_Claiming_An_Item_Whose_Territory_Another_Live_Claim_Holds_Should_Be_Refused_And_Retryable()
    {
        let (directory, id) = Scratch_Board_With_A_Held_Territory_Conflict();
        let request = ClaimRequest { item: id, holder: "test-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkReservationResponse::Refused { retryable, cause } = response
        else
        {
            panic!("ground another live claim holds is a real refusal, not a grant");
        };
        assert!(retryable, "{cause}");
    }

    #[test]
    fn Test_Renewing_This_Holders_Own_Claim_Should_Extend_The_Lease()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("test-holder", i64::from(u32::MAX));
        let request = ClaimRequest { item: id.clone(), holder: "test-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_Renew(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkReservationResponse::Reserved { reservation } = response
        else
        {
            panic!("renewing a claim this holder already has grants a reservation");
        };
        assert_eq!(reservation.item, id);
        assert_eq!(reservation.holder, "test-holder");
    }

    #[test]
    fn Test_Taking_Over_A_Real_Lapsed_Claim_Should_Grant_A_Reservation()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("old-holder", 1);
        let request = ClaimRequest { item: id.clone(), holder: "new-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_TakeOver(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkReservationResponse::Reserved { reservation } = response
        else
        {
            panic!("a real lapsed claim is takeable");
        };
        assert_eq!(reservation.item, id);
        assert_eq!(reservation.holder, "new-holder");
    }

    #[test]
    fn Test_A_Real_Reserved_Response_Should_Round_Trip_As_Json()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = ClaimRequest { item: id, holder: "test-holder".to_owned(), lease: std::time::Duration::from_secs(3600) };

        let response = Handle_Work_Claim(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkReservationResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkReservationResponse always has this field");

        assert_eq!(outcome, "reserved", "{json}");
    }

    #[test]
    fn Test_Abandoning_A_Real_Claim_This_Holder_Actually_Has_Should_Release_It()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("test-holder", i64::from(u32::MAX));
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "test fixture".to_owned() };

        let response = Handle_Work_Abandon(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, WorkAbandonResponse::Abandoned), "{response:?}");
    }

    #[test]
    fn Test_Abandoning_A_Claim_A_Different_Holder_Actually_Has_Should_Be_Refused_And_Retryable()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("someone-else", i64::from(u32::MAX));
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "test fixture".to_owned() };

        let response = Handle_Work_Abandon(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkAbandonResponse::Refused { retryable, cause } = response
        else
        {
            panic!("a holder abandoning a claim somebody else actually has is a real refusal");
        };
        assert!(retryable, "{cause}");
    }

    #[test]
    fn Test_Declining_A_Real_Unclaimed_Ready_Item_Should_End_It()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "superseded".to_owned() };

        let response = Handle_Work_Decline(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, WorkDeclineResponse::Declined), "{response:?}");
    }

    #[test]
    fn Test_Declining_An_Item_A_Real_Active_Claim_Still_Holds_Should_Be_Refused_And_Retryable()
    {
        let (directory, id) = Scratch_Board_With_A_Claimed_Item("someone-else", i64::from(u32::MAX));
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "superseded".to_owned() };

        let response = Handle_Work_Decline(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkDeclineResponse::Refused { retryable, cause } = response
        else
        {
            panic!("an item a real active claim still holds is a real refusal, not an end");
        };
        assert!(retryable, "{cause}");
    }

    #[test]
    fn Test_A_Real_Declined_Response_Should_Round_Trip_As_Json()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();
        let request = EndingRequest { item: id, holder: "test-holder".to_owned(), reason: "superseded".to_owned() };

        let response = Handle_Work_Decline(&directory, &request);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkDeclineResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkDeclineResponse always has this field");

        assert_eq!(outcome, "declined", "{json}");
    }

    #[test]
    fn Test_Finishing_An_Id_Absent_From_A_Real_Readable_Board_Should_Be_Refused_And_Not_Judged()
    {
        let directory = Scratch_Board();
        let absent = ItemId::New("NO-SUCH-ITEM");

        let response = Handle_Work_Finish(&directory, &absent, "test-holder");

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkFinishResponse::Refused { judged, cause } = response
        else
        {
            panic!("an id absent from a real, readable board is a real refusal, not a finish");
        };
        assert!(!judged, "{cause}");
    }

    #[test]
    fn Test_Finishing_A_Real_Item_With_No_Verification_Predicate_Should_Be_Refused_And_Not_Judged()
    {
        let (directory, id) = Scratch_Board_With_A_Claimable_Item();

        let response = Handle_Work_Finish(&directory, &id, "test-holder");

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkFinishResponse::Refused { judged, cause } = response
        else
        {
            panic!("an item with no verification predicate at all is a real refusal, not a finish");
        };
        assert!(!judged, "{cause}");
    }

    #[test]
    fn Test_A_Real_Refused_Finish_Response_Should_Round_Trip_As_Json()
    {
        let directory = Scratch_Board();
        let absent = ItemId::New("NO-SUCH-ITEM");

        let response = Handle_Work_Finish(&directory, &absent, "test-holder");

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkFinishResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkFinishResponse always has this field");

        assert_eq!(outcome, "refused", "{json}");
    }
}

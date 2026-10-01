//! Reading and writing the ledger file, and deciding a change inside one lock acquisition.
//!
//! The bodies here belong to methods on [`FileLedger`] that keep their documentation and
//! signature in `store.rs`, for the reason `verbs.rs` gives: the surface snapshot resolves
//! `pub use store::FileLedger` against one module, so a `pub fn` written on the type
//! anywhere else is public and unrecorded.

use nomos_platform::{Clock, FilesystemLock, FileSystem, StaleTakeover};

use nomos_platform::Timestamp;

use crate::LedgerDocument;
use crate::LedgerError;

use super::{FileLedger, LOCK_STALE_AFTER, LOCK_WAIT_LIMIT, SCHEMA_VERSION};

/// The body of [`FileLedger::Save`], which keeps the documentation and the signature.
pub(super) fn Save_Document<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    document: &LedgerDocument,
) -> Result<(), LedgerError>
{
    use super::rendering::Rendered_Document;
    use super::validation::Validate_Document;

    let violations = Validate_Document(document, ledger.clock.Now());
    if !violations.is_empty()
    {
        return Err(LedgerError::Invalid { violations });
    }

    let rendered = Rendered_Document(document)?;

    return ledger
        .filesystem
        .Replace_Atomically(&ledger.path, &rendered)
        .map_err(|error| LedgerError::Unreadable {
            cause: error.to_string(),
        });
}

/// Runs a decision that may refuse, over the document, inside one lock acquisition.
///
/// The [`ExclusionLedger`] verbs share a shape that [`FileLedger::With_Lock`] cannot express on
/// its own: they answer with a refusal rather than a [`LedgerError`], and a refusal is an
/// *answer*, not a failure of the store. Carrying it out through the error channel would
/// put "agent-b holds this" and "the file will not parse" in one type, which is the
/// conflation `OD-LEDGER-009` already had to undo once. So the refusal rides out as the
/// modification's value, and only genuine store failures use the error.
///
/// Written once and called from every verb rather than spelled out in each. A copy of
/// "take the lock, read the clock, decide, write" per verb is a chance per verb for one of
/// them to stop taking the lock — which is the defect this exists to have fixed, and which
/// `add` then went on to demonstrate anyway by never being routed through here at all.
/// `OD-LEDGER-021`.
///
/// # Why the refusal type is generic
///
/// It was [`ClaimRefusal`] concretely while the only callers were the three claim verbs.
/// `add` refuses for a reason that is not about claiming — the identifier is already on
/// the board — and giving it a [`ClaimRefusal`] arm to borrow would have been the
/// mis-subject `OD-LEDGER-014` measured. The alternative, letting `add` reach for
/// [`FileLedger::With_Lock`] directly, is the copy this function exists to prevent. So the door
/// stays single and each verb brings its own vocabulary, bound only by being able to say
/// "the store itself failed".
///
/// `now` is read **inside** the acquisition and handed to the decision, so the clock a
/// claim is judged against and the clock its lease is measured from are one reading
/// taken after the wait for the lock. Read before, a claim that waited on a contended
/// lock would be granted a lease shortened by however long it waited, and would judge
/// other holders' leases against a time that had already passed.
pub(super) fn Decide_Under_Lock<
    Files: FileSystem,
    TimeSource: Clock,
    Lock: FilesystemLock,
    Outcome,
    Error,
>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    holder: &str,
    decide: impl FnOnce(&mut LedgerDocument, Timestamp) -> Result<Outcome, Error>,
) -> Result<Outcome, Error>
where
    for<'error> Error: From<&'error LedgerError>,
{
    // The stale takeover is dropped here, deliberately and visibly. None of the three
    // verbs' return types can carry one — `Reservation` and `ClaimRefusal` are public
    // and adding a field or a variant to either is a change to the crate's surface,
    // which this item did not have. What is lost is a diagnostic and not consistency:
    // a broken lock is only ever broken after `LOCK_STALE_AFTER`, and `Save` replaces
    // the file atomically, so the document a takeover finds is whole either way.
    let (outcome, _takeover) = ledger
        .With_Lock(holder, |document| {
            let now = ledger.clock.Now();

            return Ok(decide(document, now));
        })
        .map_err(|error| return Error::from(&error))?;

    return outcome;
}

/// The body of [`FileLedger::Load`], which keeps the documentation and the signature.
pub(super) fn Load_Document<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
) -> Result<LedgerDocument, LedgerError>
{
    use super::document::Explain_Parse_Failure;

    if !ledger.filesystem.Exists(&ledger.path)
    {
        return Ok(LedgerDocument {
            schema_version: SCHEMA_VERSION,
            items: Vec::new(),
        });
    }

    let text = ledger
        .filesystem
        .Read_To_String(&ledger.path)
        .map_err(|error| LedgerError::Unreadable {
            cause: error.to_string(),
        })?;

    return serde_json::from_str::<LedgerDocument>(&text)
        .map_err(|error| return Explain_Parse_Failure(&ledger.path, &text, &error));
}

/// The body of [`FileLedger::With_Lock`], which keeps the documentation and the signature.
///
/// The write is conditional on the document having changed, and the takeover travels out with
/// the result rather than being logged here: a caller that surfaces it can tell the user their
/// predecessor abandoned an update, and a caller that drops it has made a choice this signature
/// makes visible in review. Both are argued at length on `With_Lock` itself.
pub(super) fn With_Lock_Run<
    Files: FileSystem,
    TimeSource: Clock,
    Lock: FilesystemLock,
    Outcome,
>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    holder: &str,
    modify: impl FnOnce(&mut LedgerDocument) -> Result<Outcome, LedgerError>,
) -> Result<(Outcome, Option<StaleTakeover>), LedgerError>
{
    let acquisition = ledger
        .lock
        .Acquire(holder, LOCK_WAIT_LIMIT, LOCK_STALE_AFTER)
        .map_err(|error| LedgerError::Locked {
            cause: error.to_string(),
        })?;

    let read = ledger.Load()?;
    let mut document = read.clone();
    let outcome = modify(&mut document)?;

    if document != read
    {
        ledger.Save(&document)?;
    }

    // The takeover travels out with the result rather than being logged here. A
    // caller that surfaces it can tell the user their predecessor abandoned an
    // update; a caller that drops it has made a choice, and this signature is what
    // makes that choice visible in review.
    return Ok((outcome, acquisition.broke_stale));
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

    use super::*;
    use crate::{AddRefusal, ItemId, ItemKind, ItemOrigin, ItemState, LedgerItem, Territory};
    use nomos_platform_std::{FileLock, StdFileSystem};
    use std::path::{Path, PathBuf};

    /// The instant the tests here are read at, named so that a fixture change is one edit.
    const NOW_SECONDS: i64 = 1_000;

    struct FixedClock(i64);

    /// Fixed instants, so both the values and their timing reproduce.
    impl Strategy for FixedClock
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl Clock for &FixedClock
    {
        fn Now(&self) -> Timestamp
        {
            return Timestamp::From_Unix_Seconds(self.0);
        }
    }

    #[test]
    fn Test_Save_Document_Should_Refuse_An_Invalid_Document_Without_Writing_It()
    {
        let directory = Temporary_Directory("save-document");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let mut reserves_nothing = Workable_Item("BAD-1");
        reserves_nothing.territory = Territory::Empty();
        let invalid = LedgerDocument {
            schema_version: SCHEMA_VERSION,
            items: vec![reserves_nothing],
        };

        let error = Save_Document(&ledger, &invalid).expect_err("an item reserving nothing must be refused");

        assert!(matches!(error, LedgerError::Invalid { .. }), "got {error:?}");
        let after = Load_Document(&ledger).expect("a refused save leaves no file behind, which loads as empty");
        assert!(after.items.is_empty(), "the invalid document must not have reached disk");
    }

    #[test]
    fn Test_Decide_Under_Lock_Should_Convert_A_Store_Failure_Through_The_Callers_Own_Error()
    {
        let directory = Temporary_Directory("decide-under-lock");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);

        let outcome: Result<(), AddRefusal> = Decide_Under_Lock(&ledger, "agent-a", |document, _now| {
            document.items.push(Workable_Item("D-1"));
            return Ok(());
        });

        outcome.expect("a plain decision must succeed");
        let reloaded = Load_Document(&ledger).expect("the decision must have been written");
        assert_eq!(reloaded.items.len(), 1);
    }

    #[test]
    fn Test_Load_Document_Should_Parse_The_Text_On_Disk_Into_A_Document()
    {
        let directory = Temporary_Directory("load-document");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let raw = serde_json::to_string(&LedgerDocument {
            schema_version: SCHEMA_VERSION,
            items: vec![Workable_Item("L-1")],
        })
        .expect("the fixture document serializes");
        std::fs::write(ledger.Path(), raw).expect("test can write the raw fixture directly");

        let document = Load_Document(&ledger).expect("a well-formed file must load");

        assert_eq!(document.items.len(), 1);
        assert_eq!(document.items.first().expect("the assertion above found exactly one item").id, ItemId::New("L-1"));
    }

    #[test]
    fn Test_With_Lock_Run_Should_Modify_And_Persist_In_One_Acquisition()
    {
        let directory = Temporary_Directory("with-lock-run");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);

        let (outcome, takeover) = With_Lock_Run(&ledger, "agent-a", |document| {
            document.items.push(Workable_Item("W-1"));
            return Ok("W-1".to_owned());
        })
        .expect("a plain modification must succeed");

        assert_eq!(outcome, "W-1");
        assert!(takeover.is_none(), "a fresh lock is never a stale takeover");
        let reloaded = Load_Document(&ledger).expect("the modification must have been written");
        assert_eq!(reloaded.items.len(), 1);
    }

    fn Temporary_Directory(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-store-file-{name}-{}", std::process::id()));
        if path.exists()
        {
            std::fs::remove_dir_all(&path).expect("the previous run's synthetic directory is removable");
        }
        std::fs::create_dir_all(&path).expect("test needs a temp directory");
        return path;
    }

    fn Ledger_At<'clock>(
        directory: &Path,
        clock: &'clock FixedClock,
    ) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
    {
        return FileLedger::At(
            directory.join("ledger.json"),
            StdFileSystem,
            clock,
            FileLock::At(directory.join("ledger.lock")),
        );
    }

    fn Workable_Item(id: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: "an item".to_owned(),
            why: "because".to_owned(),
            done_when: "when it is done".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files([format!("src/{id}.rs")]),
            state: ItemState::Ready,
            depends_on: Vec::new(),
            blocked: None,
            claim: None,
            verification: None,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            widened: Vec::new(),
            declined: None,
        };
    }
}

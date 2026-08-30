//! Reading and writing the ledger file, and deciding a change inside one lock acquisition.
//!
//! The bodies here belong to methods on [`FileLedger`] that keep their documentation and
//! signature in `store.rs`, for the reason `verbs.rs` gives: the surface snapshot resolves
//! `pub use store::FileLedger` against one module, so a `pub fn` written on the type
//! anywhere else is public and unrecorded.

use nomos_platform::{Clock, CrossProcessLock, FileSystem};

use nomos_platform::Timestamp;

use crate::LedgerDocument;
use crate::LedgerError;

use super::{FileLedger, SCHEMA_VERSION};

/// The body of [`FileLedger::Save`], which keeps the documentation and the signature.
pub(super) fn Save_Document<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
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
    Lock: CrossProcessLock,
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
pub(super) fn Load_Document<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
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

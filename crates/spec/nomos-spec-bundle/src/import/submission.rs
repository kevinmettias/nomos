//! Inserting the submissions, with their values and their open decision gaps.

use rusqlite::{Transaction, params};

use crate::BundleError;
use crate::Bundle;
use crate::Record;

use super::reference::Submission_Uid;
use super::Insert_Each;

/// The submission rows, each onto the node it is.
pub(super) fn Insert_Submissions(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO submissions
             (node_uid, kind, form_contract_version, state, submitted_by, submitted_through)
         VALUES ((SELECT uid FROM nodes WHERE node_id = ?1), ?2, ?3, ?4, ?5, ?6)",
        |insert, record| {
            let Record::Submission(submission) = record
            else
            {
                return Ok(());
            };
            insert.execute(params![
                submission.node_id,
                submission.kind,
                submission.form_contract_version,
                submission.state,
                submission.submitted_by,
                submission.submitted_through
            ])?;

            return Ok(());
        },
    );
}

/// Every attributed value, keeping the ordinal that decides which reading is current.
pub(super) fn Insert_Submission_Values(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO submission_values
             (submission_uid, field, ordinal, origin, value, value_hash, supersedes_hash,
              recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        |insert, record| {
            let Record::SubmissionValue(value) = record
            else
            {
                return Ok(());
            };
            let submission_uid = Submission_Uid(transaction, &value.node_id)?;
            insert.execute(params![
                submission_uid,
                value.field,
                value.ordinal,
                value.origin,
                value.value,
                value.value_hash,
                value.supersedes_hash,
                value.recorded_at
            ])?;

            return Ok(());
        },
    );
}

/// Every gap, open and closed alike.
pub(super) fn Insert_Submission_Gaps(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO submission_gaps
             (submission_uid, ordinal, question, blocks, severity, closed_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        |insert, record| {
            let Record::SubmissionGap(gap) = record
            else
            {
                return Ok(());
            };
            let submission_uid = Submission_Uid(transaction, &gap.node_id)?;
            insert.execute(params![
                submission_uid,
                gap.ordinal,
                gap.question,
                gap.blocks,
                gap.severity,
                gap.closed_by
            ])?;

            return Ok(());
        },
    );
}

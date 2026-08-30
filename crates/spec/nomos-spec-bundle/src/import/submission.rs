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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::SpecificationStore;

    /// A node already filed as a submission, and a second node not yet filed.
    fn Fixture() -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N1', 'submission', 'canonical', 'record', 'A Submission', NULL, NULL);
                 INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N2', 'submission', 'canonical', 'record', 'Another Submission', NULL, NULL);
                 INSERT INTO submissions
                     (node_uid, kind, form_contract_version, state, submitted_by, submitted_through)
                     VALUES (1, 'feature-request', 1, 'draft', 'me', 'test');",
            )
            .expect("populates every table this file's inserts resolve against");

        return store;
    }

    #[test]
    fn Test_Insert_Submissions_Should_Place_A_Submission_By_Its_Node()
    {
        use crate::row::submission::Submission;

        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::Submission(Submission {
                node_id: "N2".to_owned(),
                kind: "design-spec".to_owned(),
                form_contract_version: 1,
                state: "draft".to_owned(),
                submitted_by: "you".to_owned(),
                submitted_through: "test".to_owned(),
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Submissions(transaction, &bundle)).expect("inserts");

        let kind: String = store
            .Connection()
            .query_row(
                "SELECT s.kind FROM submissions s JOIN nodes n ON n.uid = s.node_uid
                 WHERE n.node_id = 'N2'",
                [],
                |row| row.get(0),
            )
            .expect("reads back");
        assert_eq!(kind, "design-spec");
    }

    #[test]
    fn Test_Insert_Submission_Values_Should_Preserve_The_Ordinal_And_Origin_Of_Each_Value()
    {
        use crate::row::submission::value::Value as SubmissionValue;

        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::SubmissionValue(SubmissionValue {
                node_id: "N1".to_owned(),
                field: "title".to_owned(),
                ordinal: 1,
                origin: "submitted".to_owned(),
                value: "A title".to_owned(),
                value_hash: "sha256:vv".to_owned(),
                supersedes_hash: None,
                recorded_at: "2026-01-01T00:00:00Z".to_owned(),
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Submission_Values(transaction, &bundle)).expect("inserts");

        let value: String = store
            .Connection()
            .query_row("SELECT value FROM submission_values WHERE submission_uid = 1", [], |row| {
                row.get(0)
            })
            .expect("reads back");
        assert_eq!(value, "A title");
    }

    #[test]
    fn Test_Insert_Submission_Gaps_Should_Preserve_An_Open_Gap()
    {
        use crate::row::submission::gap::Gap as SubmissionGap;

        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::SubmissionGap(SubmissionGap {
                node_id: "N1".to_owned(),
                ordinal: 1,
                question: "What?".to_owned(),
                blocks: "[]".to_owned(),
                severity: "non-blocking".to_owned(),
                closed_by: None,
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Submission_Gaps(transaction, &bundle)).expect("inserts");

        let question: String = store
            .Connection()
            .query_row("SELECT question FROM submission_gaps WHERE submission_uid = 1", [], |row| {
                row.get(0)
            })
            .expect("reads back");
        assert_eq!(question, "What?");
    }
}

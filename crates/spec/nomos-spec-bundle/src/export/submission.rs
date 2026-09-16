//! Reading the submissions out, with their values and their open decision gaps.

use crate::BundleError;
use crate::Record;
use rusqlite::Connection;

use super::{Collect_Rows, Columns};

/// The submissions, ordered by the node they are.
pub(super) fn Collect_Submissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::row::submission::Submission;

    return Collect_Rows(
        connection,
        records,
        "SELECT n.node_id, s.kind, s.form_contract_version, s.state, s.submitted_by,
                s.submitted_through
         FROM submissions s
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::Submission(Submission {
                node_id: columns.Next()?,
                kind: columns.Next()?,
                form_contract_version: columns.Next()?,
                state: columns.Next()?,
                submitted_by: columns.Next()?,
                submitted_through: columns.Next()?,
            }));
        },
    );
}

/// Every value of every field, in the order that makes the last one the current reading.
pub(super) fn Submission_Values(connection: &Connection, records: &mut Vec<Record>)
-> Result<(), BundleError>
{
    let rows = Submission_Value_Rows(connection)?;

    records.extend(rows.into_iter().map(Record::SubmissionValue));
    return Ok(());
}

/// Every submission value row, in the order that makes the last one the current reading.
fn Submission_Value_Rows(
    connection: &Connection,
) -> Result<Vec<crate::row::submission::value::Value>, BundleError>
{
    use crate::row::submission::value::Value as SubmissionValue;

    let mut statement = connection.prepare(
        "SELECT n.node_id, v.field, v.ordinal, v.origin, v.value, v.value_hash,
                v.supersedes_hash, v.recorded_at
         FROM submission_values v
         JOIN submissions s ON s.uid = v.submission_uid
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id, v.field, v.ordinal",
    )?;
    return Ok(statement
        .query_map([], |row| {
            let mut columns = Columns::Of(row);
            return Ok(SubmissionValue {
                node_id: columns.Next()?,
                field: columns.Next()?,
                ordinal: columns.Next()?,
                origin: columns.Next()?,
                value: columns.Next()?,
                value_hash: columns.Next()?,
                supersedes_hash: columns.Next()?,
                recorded_at: columns.Next()?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?);
}

/// The decision gaps, open and closed alike.
///
/// A closed gap travels too. `OD-SPEC-010` closes one by a citation and never by deletion, so
/// a bundle that dropped them would lose the record that a question was ever asked.
pub(super) fn Submission_Gaps(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::row::submission::gap::Gap as SubmissionGap;

    return Collect_Rows(
        connection,
        records,
"SELECT n.node_id, g.ordinal, g.question, g.blocks, g.severity, g.closed_by
         FROM submission_gaps g
         JOIN submissions s ON s.uid = g.submission_uid
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id, g.ordinal",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::SubmissionGap(SubmissionGap {
                node_id: columns.Next()?,
                ordinal: columns.Next()?,
                question: columns.Next()?,
                blocks: columns.Next()?,
                severity: columns.Next()?,
                closed_by: columns.Next()?,
            }));
        },
    );
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::SpecificationStore;

    #[test]
    fn Test_Collect_Submissions_Should_Read_A_Submission_By_Its_Node()
    {
        use crate::row::submission::Submission;

        let store = Fixture();
        let mut records = Vec::new();

        Collect_Submissions(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::Submission(Submission {
                node_id: "N1".to_owned(),
                kind: "feature-request".to_owned(),
                form_contract_version: 1,
                state: "draft".to_owned(),
                submitted_by: "me".to_owned(),
                submitted_through: "test".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Submission_Values_Should_Preserve_The_Ordinal_And_Origin_Of_Each_Value()
    {
        use crate::row::submission::value::Value as SubmissionValue;

        let store = Fixture();
        let mut records = Vec::new();

        Submission_Values(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::SubmissionValue(SubmissionValue {
                node_id: "N1".to_owned(),
                field: "title".to_owned(),
                ordinal: 1,
                origin: "submitted".to_owned(),
                value: "A title".to_owned(),
                value_hash: "sha256:vv".to_owned(),
                supersedes_hash: None,
                recorded_at: "2026-01-01T00:00:00Z".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Submission_Gaps_Should_Preserve_An_Open_And_A_Closed_Gap()
    {
        use crate::row::submission::gap::Gap as SubmissionGap;

        let store = Fixture();
        let mut records = Vec::new();

        Submission_Gaps(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::SubmissionGap(SubmissionGap {
                node_id: "N1".to_owned(),
                ordinal: 1,
                question: "What?".to_owned(),
                blocks: "[]".to_owned(),
                severity: "non-blocking".to_owned(),
                closed_by: None,
            })]
        );
    }

    /// One node filed as a submission, one attributed value and one open gap.
    fn Fixture() -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory()
            .expect("an in-memory store applies the schema this build carries");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N1', 'submission', 'canonical', 'record', 'A Submission', NULL, NULL);
                 INSERT INTO submissions
                     (node_uid, kind, form_contract_version, state, submitted_by, submitted_through)
                     VALUES (1, 'feature-request', 1, 'draft', 'me', 'test');
                 INSERT INTO submission_values
                     (submission_uid, field, ordinal, origin, value, value_hash, supersedes_hash,
                      recorded_at)
                     VALUES (1, 'title', 1, 'submitted', 'A title', 'sha256:vv', NULL,
                             '2026-01-01T00:00:00Z');
                 INSERT INTO submission_gaps
                     (submission_uid, ordinal, question, blocks, severity, closed_by)
                     VALUES (1, 1, 'What?', '[]', 'non-blocking', NULL);",
            )
            .expect("populates every table this file reads");

        return store;
    }
}

//! Reading the submissions out, with their values and their open decision gaps.

use crate::BundleError;
use crate::record::Record;
use crate::submission::Submission;
use crate::submission_gap::SubmissionGap;
use crate::submission_value::SubmissionValue;
use rusqlite::Connection;

use super::{Collect, Columns};

/// The submissions, ordered by the node they are.
pub(super) fn Submissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
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
    let mut statement = connection.prepare(
        "SELECT n.node_id, v.field, v.ordinal, v.origin, v.value, v.value_hash,
                v.supersedes_hash, v.recorded_at
         FROM submission_values v
         JOIN submissions s ON s.uid = v.submission_uid
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id, v.field, v.ordinal",
    )?;
    let rows = statement
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
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::SubmissionValue));
    return Ok(());
}

/// The decision gaps, open and closed alike.
///
/// A closed gap travels too. `OD-SPEC-010` closes one by a citation and never by deletion, so
/// a bundle that dropped them would lose the record that a question was ever asked.
pub(super) fn Submission_Gaps(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
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

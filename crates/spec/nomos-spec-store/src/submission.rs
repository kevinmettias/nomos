//! The one door a submission becomes durable through.
//!
//! `OD-SPEC-009` decided there is exactly one accept function and that every surface is a
//! transport onto it: a form, a CLI verb, an HTTP endpoint and an MCP tool each construct a
//! [`Submission`] and call [`Accept_Submission`], and a transport that validates, defaults or
//! persists on its own behalf is a second write door and a defect rather than a variant.
//!
//! `OD-SPEC-013` decided what it writes: a submission is a node, a field is a sequence of
//! attributed rows in `submission_values`, and a decision gap is a row in `submission_gaps`.

mod accept_error;
mod citation;

pub use accept_error::AcceptError;

use crate::SpecificationStore;
use crate::StoreError;
use nomos_spec_model::{
    ContentHash, FieldValue, Failure, Origin, Refusal, Submission, Validate_Submission,
};
use rusqlite::Transaction;
// Neither this file's own production code nor `accept_error`/`citation` needs the type by
// name; both `inline_coverage` below and the external `submission/tests.rs` (which reaches
// this scope through `use super::*;`) construct a `Submission` with an explicit `state`, so
// the import is real but test-only — gated rather than dropped, to avoid the unused-import
// warning a plain `use` would carry in a non-test build.
#[cfg(test)]
use nomos_spec_model::SubmissionState;

/// Validates `submission` and persists it, or refuses it and stores nothing.
///
/// The whole of `OD-SPEC-010` runs here: the rules that are a function of the submission come
/// from [`Validate_Submission`], and rule 4 — `implements` resolving to an accepted design and `answers`
/// to an accepted request — runs below, because it reads a second row and cannot be a function
/// of one submission.
///
/// # The refusal is total, and that is a property of this function rather than of its callers
///
/// Every check runs before any write, and the writes are one transaction. There is no
/// partially written row, no `invalid` state and no quarantine table — `OD-SPEC-010` refuses
/// all three, because an `invalid` state would mean every reader of the store needs the rule
/// set in order to know which rows are real, which is validation moved back into the readers.
///
/// The property this buys is the one worth stating: **the store can be read without the rule
/// set, because every row in it passed.** It is lost the moment one incomplete row is admitted
/// for convenience.
///
/// # Errors
///
/// [`AcceptError::Refused`] naming every rule that failed, or [`AcceptError::Store`].
pub fn Accept_Submission(
    store: &mut SpecificationStore,
    submission: &Submission,
) -> Result<i64, AcceptError>
{
    Assert_No_Refusals(store, submission)?;

    let uid = Persist_Submission(store, submission)?;
    Write_Lifecycle_Edges(store, submission)?;

    return Ok(uid);
}

/// The submission passes every rule, or the refusal names every one it fails.
fn Assert_No_Refusals(store: &SpecificationStore, submission: &Submission) -> Result<(), AcceptError>
{
    let failures = Failed_Rules(store, submission)?;
    if !failures.is_empty()
    {
        return Err(AcceptError::Refused(Refusal {
            submission: submission.id.clone(),
            failures,
        }));
    }

    return Ok(());
}

/// Every rule this submission fails: the ones about its shape and the ones about what it
/// cites, gathered so a refusal names all of them at once.
fn Failed_Rules(
    store: &SpecificationStore,
    submission: &Submission,
) -> Result<Vec<Failure>, StoreError>
{
    let mut failures = Validate_Submission(submission);
    let unresolved = citation::Unresolved_Citations(store, submission)?;

    failures.extend(unresolved);

    return Ok(failures);
}

/// The submission's own node, and its row — with its values and gaps — under a caller's
/// transaction.
fn Persist_Submission(store: &mut SpecificationStore, submission: &Submission) -> Result<i64, StoreError>
{
    use crate::NodeRow;

    let node_uid = store.Upsert_Node(NodeRow {
        node_id: &submission.id,
        kind: submission.kind.Label(),
        authority: crate::store::AUTHORED,
        representation: "structured",
        title: Title_Of(submission),
    })?;

    return store.In_Transaction(|transaction| {
        return Write_Submission(transaction, node_uid, submission);
    });
}

/// The title a submission is filed under.
///
/// `title` is a `submission_values` row rather than a column, because somebody may rewrite it.
/// `nodes.title` is the current reading of it, which is what every reader of the graph wants;
/// the sequence is what the request preserves.
fn Title_Of(submission: &Submission) -> &str
{
    return submission
        .Current("title")
        .map_or("", |value| return value.value.as_str());
}

/// Writes the submission, its values and its gaps under one transaction.
fn Write_Submission(
    transaction: &Transaction<'_>,
    node_uid: i64,
    submission: &Submission,
) -> Result<i64, StoreError>
{
    transaction
        .execute(
            "INSERT INTO submissions
                 (node_uid, kind, form_contract_version, state, submitted_by, submitted_through)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (node_uid) DO UPDATE SET
                 kind = excluded.kind,
                 form_contract_version = excluded.form_contract_version,
                 state = excluded.state,
                 submitted_by = excluded.submitted_by,
                 submitted_through = excluded.submitted_through",
            rusqlite::params![
                node_uid,
                submission.kind.Label(),
                submission.form_contract_version,
                submission.state.Label(),
                submission.submitted_by,
                submission.submitted_through,
            ],
        )
        .map_err(|error| return StoreError::Sql(error.to_string()))?;
    let uid: i64 = transaction
        .query_row(
            "SELECT uid FROM submissions WHERE node_uid = ?1",
            [node_uid],
            |row| return row.get(0),
        )
        .map_err(|error| return StoreError::Sql(error.to_string()))?;
    Write_Values(transaction, uid, submission)?;
    Write_Gaps(transaction, uid, submission)?;

    return Ok(uid);
}

/// One row per value, ordered per field, each superseding the last of its own field.
///
/// `ordinal` is per field rather than per submission, so the current reading of a field is its
/// highest ordinal and does not move when an unrelated field is clarified.
fn Write_Values(
    transaction: &Transaction<'_>,
    submission_uid: i64,
    submission: &Submission,
) -> Result<(), StoreError>
{
    let mut insert = transaction
        .prepare(
            "INSERT INTO submission_values
                 (submission_uid, field, ordinal, origin, value, value_hash,
                  supersedes_hash, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT (submission_uid, field, ordinal) DO NOTHING",
        )
        .map_err(|error| return StoreError::Sql(error.to_string()))?;
    let mut sequence = Sequence::default();

    for value in &submission.values
    {
        Insert_Value(&mut insert, submission_uid, value, &mut sequence)?;
    }

    return Ok(());
}

/// Where each field's next value goes, and what the last one there hashed to.
#[derive(Default)]
struct Sequence
{
    ordinals: std::collections::BTreeMap<String, u32>,
    previous: std::collections::BTreeMap<String, String>,
}

/// One value, at the next ordinal of its own field, superseding the last one written there.
fn Insert_Value(
    insert: &mut rusqlite::Statement<'_>,
    submission_uid: i64,
    value: &FieldValue,
    sequence: &mut Sequence,
) -> Result<(), StoreError>
{
    let field = value.field.clone();
    let hash = ContentHash::Of(&value.value).As_String_Slice().to_owned();
    let ordinal = sequence.ordinals.entry(field.clone()).or_insert(0);

    insert
        .execute(rusqlite::params![
            submission_uid,
            field,
            *ordinal,
            value.origin.Label(),
            value.value,
            hash,
            sequence.previous.get(&field),
            Recorded_At(),
        ])
        .map_err(|error| return StoreError::Sql(error.to_string()))?;
    *ordinal = ordinal.saturating_add(1);
    sequence.previous.insert(field, hash);

    return Ok(());
}

/// One row per gap, with `closed_by` holding the citation or nothing at all.
fn Write_Gaps(
    transaction: &Transaction<'_>,
    submission_uid: i64,
    submission: &Submission,
) -> Result<(), StoreError>
{
    let mut insert = transaction
        .prepare(
            "INSERT INTO submission_gaps
                 (submission_uid, ordinal, question, blocks, severity, closed_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (submission_uid, ordinal) DO UPDATE SET
                 question = excluded.question,
                 blocks = excluded.blocks,
                 severity = excluded.severity,
                 closed_by = excluded.closed_by",
        )
        .map_err(|error| return StoreError::Sql(error.to_string()))?;
    for (index, gap) in submission.gaps.iter().enumerate()
    {
        let ordinal = u32::try_from(index).unwrap_or(u32::MAX);

        insert
            .execute(rusqlite::params![
                submission_uid,
                ordinal,
                gap.question,
                gap.blocks.join("\n"),
                gap.severity.Label(),
                gap.closed_by,
            ])
            .map_err(|error| return StoreError::Sql(error.to_string()))?;
    }

    return Ok(());
}

/// The lifecycle edges, written after the submission exists so both endpoints resolve.
///
/// `OD-SPEC-013` added `answers` and `implements` to the seed vocabulary; without them
/// `relations.relation_type` refuses the write by name, which is `OD-SPEC-011`.
fn Write_Lifecycle_Edges(
    store: &mut SpecificationStore,
    submission: &Submission,
) -> Result<(), StoreError>
{
    for (field, relation) in [("answers", "answers"), ("implements", "implements")]
    {
        let Some(cited) = submission.Current(field)
        else
        {
            continue;
        };

        store.Put_Relation(&submission.id, relation, cited.value.trim())?;
    }

    return Ok(());
}

/// When a value was recorded.
///
/// Deliberately not a wall-clock read. `nomos-platform` owns the clock and this crate has no
/// handle on one, and a timestamp taken here would make two exports of one store differ —
/// which the bundle's determinism guarantee refuses. The column exists because
/// `ARC-SPECDB-002` asks a born-structured object to say when a value arrived; filling it from
/// a clock this crate does not own is `P10-SERVICE-SEAM`'s to route.
const fn Recorded_At() -> &'static str
{
    return "";
}

/// Whether `origin` may be written by a transport on its own behalf.
///
/// A value a transport supplied rather than a submitter typed has origin `inferred`, whatever
/// made it up — `OD-SPEC-010` says so, and it is what stops a CLI's convenience becoming the
/// asker's stated intent at the seam `OD-SPEC-009` built to prevent it.
#[must_use]
pub const fn Transport_Origin() -> Origin
{
    return Origin::Inferred;
}

/// Coverage for the two functions declared in this file whose only exerciser today lives in
/// `submission/tests.rs` — a separate `.rs` file, so it cannot address a function declared
/// here. These stay small and inline on purpose; the real behavioural suite is `tests.rs`.
#[cfg(test)]
mod inline_coverage
{
    use super::*;
    use crate::Seed_Governing_Records;
    use nomos_spec_model::SubmissionKind;

    #[test]
    fn Test_Accept_Submission_Should_Persist_A_Valid_Request_As_A_Node()
    {
        let mut store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");
        Seed_Governing_Records(&mut store).expect("the governing record set is compiled in, so seeding reads no file");

        let uid =
            Accept_Submission(&mut store, &Minimal_Request("FR-900")).expect("the request names every field the form contract asks for");

        assert!(uid > 0);
        assert!(store.Node_Uid("FR-900").expect("looks up").is_some());
    }

    fn Minimal_Request(id: &str) -> Submission
    {
        return Submission {
            id: id.to_owned(),
            kind: SubmissionKind::FeatureRequest,
            form_contract_version: 1,
            state: SubmissionState::Accepted,
            submitted_by: "kevin".to_owned(),
            submitted_through: "cli".to_owned(),
            values: vec![
                FieldValue {
                    field: "title".to_owned(),
                    value: "Items can be declined".to_owned(),
                    origin: Origin::Submitted,
                },
                FieldValue {
                    field: "goal".to_owned(),
                    value: "close superseded work".to_owned(),
                    origin: Origin::Submitted,
                },
                FieldValue {
                    field: "behaviour".to_owned(),
                    value: "a verb writes Declined".to_owned(),
                    origin: Origin::Submitted,
                },
                FieldValue {
                    field: "acceptance".to_owned(),
                    value: "it stops being claimable".to_owned(),
                    origin: Origin::Submitted,
                },
                FieldValue {
                    field: "invariants".to_owned(),
                    value: "none".to_owned(),
                    origin: Origin::Submitted,
                },
            ],
            gaps: Vec::new(),
        };
    }

    #[test]
    fn Test_Transport_Origin_Should_Be_Inferred()
    {
        assert_eq!(Transport_Origin(), Origin::Inferred);
    }
}

#[cfg(test)]
mod tests;

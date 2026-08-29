//! The one door a submission becomes durable through.
//!
//! `OD-SPEC-009` decided there is exactly one accept function and that every surface is a
//! transport onto it: a form, a CLI verb, an HTTP endpoint and an MCP tool each construct a
//! [`Submission`] and call [`Accept_Submission`], and a transport that validates, defaults or
//! persists on its own behalf is a second write door and a defect rather than a variant.
//!
//! `OD-SPEC-013` decided what it writes: a submission is a node, a field is a sequence of
//! attributed rows in `submission_values`, and a decision gap is a row in `submission_gaps`.

use crate::SpecificationStore;
use crate::StoreError;
use nomos_spec_model::{
    ContentHash, Failure, FieldValue, Origin, Refusal, Submission, SubmissionState, Validate,
};
use rusqlite::{OptionalExtension, Transaction};

/// Why a submission did not become durable.
///
/// Two arms and not one, because they send the caller to different places. A [`Refusal`] is
/// a fact about the submission and the submitter can act on it; a [`StoreError`] is a fact
/// about the database and they cannot.
#[derive(Debug)]
pub enum AcceptError
{
    /// The submission failed the rule set. Nothing was written.
    Refused(Refusal),
    /// The store could not be used.
    Store(StoreError),
}

impl From<StoreError> for AcceptError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
}

impl std::fmt::Display for AcceptError
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return match self
        {
            Self::Refused(refusal) => write!(formatter, "{refusal}"),
            Self::Store(error) => write!(formatter, "{error}"),
        };
    }
}

/// Validates `submission` and persists it, or refuses it and stores nothing.
///
/// The whole of `OD-SPEC-010` runs here: the rules that are a function of the submission come
/// from [`Validate`], and rule 4 — `implements` resolving to an accepted design and `answers`
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
    let failures = Refusals(store, submission)?;
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
fn Refusals(
    store: &SpecificationStore,
    submission: &Submission,
) -> Result<Vec<Failure>, StoreError>
{
    let mut failures = Validate(submission);
    let unresolved = Unresolved_Citations(store, submission)?;

    failures.extend(unresolved);

    return Ok(failures);
}

/// Rule 4: `implements` resolves to an accepted design, and `answers` to an accepted request.
///
/// A result built against a draft is a result whose target may still change under it, so this
/// applies in both states — `draft` weakens which rules apply only for rule 5.
///
/// Stated as a rule on the submission rather than as a constraint on the edge, which is what
/// `OD-SPEC-010` says it is and what it can be: `relation_types` carries no domain, range or
/// cardinality, so nothing here stops an `implements` edge joining a suite to a table row.
/// That is `P10-EDGE-CONSTRAINTS`, and it is a different guarantee from this one.
fn Unresolved_Citations(
    store: &SpecificationStore,
    submission: &Submission,
) -> Result<Vec<Failure>, StoreError>
{
    let mut failures = Vec::new();

    for (field, wanted) in [("answers", "feature-request"), ("implements", "design-spec")]
    {
        let unresolved = Unresolved_Citation(store, submission, CitedField(field), WantedKind(wanted))?;

        failures.extend(unresolved);
    }

    return Ok(failures);
}

/// The submission field a citation rule checks (`answers`, `implements`).
struct CitedField<'a>(&'a str);
/// The kind of submission a citation rule requires (`feature-request`, `design-spec`).
struct WantedKind<'a>(&'a str);

/// Why one cited field does not resolve to an accepted submission of the kind it must name.
fn Unresolved_Citation(
    store: &SpecificationStore,
    submission: &Submission,
    field: CitedField<'_>,
    wanted: WantedKind<'_>,
) -> Result<Option<Failure>, StoreError>
{
    let field = field.0;
    let wanted = wanted.0;

    let Some(target) = Cited_Target(submission, field)
    else
    {
        return Ok(None);
    };

    let Some(remedy) = Citation_Remedy(store, target, field, wanted)?
    else
    {
        return Ok(None);
    };

    return Ok(Some(Citation_Failure(field, remedy)));
}

/// The value a citation field names, trimmed — `None` when the submission carries no
/// citation in that field at all.
fn Cited_Target<'a>(submission: &'a Submission, field: &str) -> Option<&'a str>
{
    let cited = submission.Current(field)?;

    return Some(cited.value.trim());
}

/// Why a cited target does not resolve to an accepted submission of the required kind —
/// `None` when it does.
fn Citation_Remedy(
    store: &SpecificationStore,
    target: &str,
    field: &str,
    wanted: &str,
) -> Result<Option<String>, StoreError>
{
    let found = Cited_State(store, target)?;

    return Ok(match found
    {
        None => Some(format!(
            "no submission is filed under `{target}`; cite one that exists, and one that is \
             a {wanted}"
        )),
        Some((kind, _)) if kind != wanted =>
        {
            Some(format!("`{target}` is a {kind} and `{field}` must name a {wanted}"))
        }
        Some((_, state)) if state != SubmissionState::Accepted.Label() => Some(format!(
            "`{target}` is a {state}; accept it first, because work built against a draft is \
             work whose target may still change under it"
        )),
        Some(_) => None,
    });
}

/// The kind and state of the submission filed under `node_id`, if one is.
fn Cited_State(
    store: &SpecificationStore,
    node_id: &str,
) -> Result<Option<(String, String)>, StoreError>
{
    let found = store
        .Connection()
        .query_row(
            "SELECT s.kind, s.state
             FROM submissions s
             JOIN nodes n ON n.uid = s.node_uid
             WHERE n.node_id = ?1",
            [node_id],
            |row| return Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    return Ok(found);
}

/// Bundles a citation rule's field and remedy into the [`Failure`] shape callers expect.
fn Citation_Failure(field: &str, remedy: String) -> Failure
{
    return Failure {
        field: field.to_owned(),
        rule: "citation-resolves-to-an-accepted-submission".to_owned(),
        remedy,
    };
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
    let hash = ContentHash::Of(&value.value).As_Str().to_owned();
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

#[cfg(test)]
mod tests;

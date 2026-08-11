//! The one door a submission becomes durable through.
//!
//! `OD-SPEC-009` decided there is exactly one accept function and that every surface is a
//! transport onto it: a form, a CLI verb, an HTTP endpoint and an MCP tool each construct a
//! [`Submission`] and call [`Accept_Submission`], and a transport that validates, defaults or
//! persists on its own behalf is a second write door and a defect rather than a variant.
//!
//! `OD-SPEC-013` decided what it writes: a submission is a node, a field is a sequence of
//! attributed rows in `submission_values`, and a decision gap is a row in `submission_gaps`.

use crate::store::{SpecificationStore, StoreError};
use nomos_spec_model::{
    ContentHash, Failure, Origin, Refusal, Submission, SubmissionState, Validate,
};
use rusqlite::Transaction;

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
    let mut failures = Validate(submission);
    failures.extend(Unresolved_Citations(store, submission)?);

    if !failures.is_empty()
    {
        return Err(AcceptError::Refused(Refusal {
            submission: submission.id.clone(),
            failures,
        }));
    }

    let node_uid = store.Upsert_Node(
        &submission.id,
        submission.kind.Label(),
        crate::store::AUTHORED,
        "structured",
        Title_Of(submission),
    )?;

    let uid = store.In_Transaction(|transaction| {
        return Write_Submission(transaction, node_uid, submission);
    })?;

    Write_Lifecycle_Edges(store, submission)?;

    return Ok(uid);
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
        let Some(cited) = submission.Current(field)
        else
        {
            continue;
        };

        let target = cited.value.trim();
        let found = Cited_State(store, target)?;

        let remedy = match found
        {
            None => format!(
                "no submission is filed under `{target}`; cite one that exists, and one that \
                 is a {wanted}"
            ),
            Some((kind, _)) if kind != wanted => format!(
                "`{target}` is a {kind} and `{field}` must name a {wanted}"
            ),
            Some((_, state)) if state != SubmissionState::Accepted.Label() => format!(
                "`{target}` is a {state}; accept it first, because work built against a draft \
                 is work whose target may still change under it"
            ),
            Some(_) => continue,
        };

        failures.push(Failure {
            field: field.to_owned(),
            rule: "citation-resolves-to-an-accepted-submission".to_owned(),
            remedy,
        });
    }

    return Ok(failures);
}

/// The kind and state of the submission filed under `node_id`, if one is.
fn Cited_State(
    store: &SpecificationStore,
    node_id: &str,
) -> Result<Option<(String, String)>, StoreError>
{
    let mut statement = store
        .Connection()
        .prepare(
            "SELECT s.kind, s.state
             FROM submissions s
             JOIN nodes n ON n.uid = s.node_uid
             WHERE n.node_id = ?1",
        )
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    let mut rows = statement
        .query([node_id])
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    let Some(row) = rows
        .next()
        .map_err(|error| return StoreError::Sql(error.to_string()))?
    else
    {
        return Ok(None);
    };

    let kind: String = row
        .get(0)
        .map_err(|error| return StoreError::Sql(error.to_string()))?;
    let state: String = row
        .get(1)
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    return Ok(Some((kind, state)));
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
    let mut ordinals: std::collections::BTreeMap<&str, u32> = std::collections::BTreeMap::new();
    let mut previous: std::collections::BTreeMap<&str, String> =
        std::collections::BTreeMap::new();

    let mut insert = transaction
        .prepare(
            "INSERT INTO submission_values
                 (submission_uid, field, ordinal, origin, value, value_hash,
                  supersedes_hash, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT (submission_uid, field, ordinal) DO NOTHING",
        )
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    for value in &submission.values
    {
        let field = value.field.as_str();
        let ordinal = ordinals.entry(field).or_insert(0);
        let hash = ContentHash::Of(&value.value).As_Str().to_owned();

        insert
            .execute(rusqlite::params![
                submission_uid,
                field,
                *ordinal,
                value.origin.Label(),
                value.value,
                hash,
                previous.get(field),
                Recorded_At(),
            ])
            .map_err(|error| return StoreError::Sql(error.to_string()))?;

        previous.insert(field, hash);
        *ordinal = ordinal.saturating_add(1);
    }

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
mod tests
{
    use super::*;
    use crate::governing::Seed_Governing_Records;
    use crate::store::Table;
    use nomos_spec_model::{DecisionGap, FieldValue, Severity, SubmissionKind};

    fn Store() -> SpecificationStore
    {
        let mut store = SpecificationStore::In_Memory().expect("a store");
        Seed_Governing_Records(&mut store).expect("a seeded store");
        return store;
    }

    /// The first failure, named rather than indexed.
    fn First(failures: &[nomos_spec_model::Failure]) -> &nomos_spec_model::Failure
    {
        return failures.first().expect("at least one failure");
    }

    /// One stored value row, named rather than indexed.
    fn Nth(rows: &[(String, u32, Option<String>)], index: usize)
    -> &(String, u32, Option<String>)
    {
        return rows.get(index).expect("a stored row");
    }

    fn Value(field: &str, value: &str, origin: Origin) -> FieldValue
    {
        return FieldValue {
            field: field.to_owned(),
            value: value.to_owned(),
            origin,
        };
    }

    fn Request(id: &str, state: SubmissionState) -> Submission
    {
        return Submission {
            id: id.to_owned(),
            kind: SubmissionKind::FeatureRequest,
            form_contract_version: 1,
            state,
            submitted_by: "kevin".to_owned(),
            submitted_through: "cli".to_owned(),
            values: vec![
                Value("title", "Items can be declined", Origin::Submitted),
                Value("goal", "close superseded work", Origin::Submitted),
                Value("behaviour", "a verb writes Declined", Origin::Submitted),
                Value("acceptance", "it stops being claimable", Origin::Submitted),
                Value("invariants", "none", Origin::Submitted),
            ],
            gaps: Vec::new(),
        };
    }

    #[test]
    fn Test_An_Accepted_Request_Should_Land_As_A_Node_And_Its_Values()
    {
        let mut store = Store();

        Accept_Submission(&mut store, &Request("FR-001", SubmissionState::Accepted))
            .expect("accepted");

        assert_eq!(store.Count(Table::Submissions).expect("a count"), 1);
        assert_eq!(store.Count(Table::SubmissionValues).expect("a count"), 5);
        assert!(
            store.Node_Uid("FR-001").expect("a lookup").is_some(),
            "a submission is a node"
        );
    }

    #[test]
    fn Test_An_Incomplete_Submission_Should_Be_Refused_And_Stored_Nowhere()
    {
        let mut store = Store();
        let mut submission = Request("FR-002", SubmissionState::Accepted);
        submission.values.retain(|value| return value.field != "goal");

        let error = Accept_Submission(&mut store, &submission).expect_err("refused");

        match error
        {
            AcceptError::Refused(refusal) =>
            {
                assert_eq!(refusal.submission, "FR-002");
                assert_eq!(refusal.failures.len(), 1);
                assert_eq!(First(&refusal.failures).field, "goal");
            }
            AcceptError::Store(error) => panic!("refused for the wrong reason: {error}"),
        }

        assert_eq!(store.Count(Table::Submissions).expect("a count"), 0);
        assert_eq!(store.Count(Table::SubmissionValues).expect("a count"), 0);
        assert!(
            store.Node_Uid("FR-002").expect("a lookup").is_none(),
            "there is no partially written row"
        );
    }

    #[test]
    fn Test_A_Refusal_Should_Name_Every_Failure_Rather_Than_The_First()
    {
        let mut store = Store();
        let mut submission = Request("FR-003", SubmissionState::Accepted);
        submission.values.retain(|value| return value.field == "title");

        let error = Accept_Submission(&mut store, &submission).expect_err("refused");

        let AcceptError::Refused(refusal) = error
        else
        {
            panic!("refused for the wrong reason");
        };

        assert_eq!(refusal.failures.len(), 4, "four holes, one refusal");
        assert!(refusal.to_string().contains("nothing was stored"));
    }

    #[test]
    fn Test_A_Later_Value_Should_Be_Stored_Beside_The_One_It_Supersedes()
    {
        let mut store = Store();
        let mut submission = Request("FR-004", SubmissionState::Accepted);
        submission
            .values
            .push(Value("goal", "what it became", Origin::Clarified));

        Accept_Submission(&mut store, &submission).expect("accepted");

        let mut statement = store
            .Connection()
            .prepare(
                "SELECT value, ordinal, supersedes_hash FROM submission_values
                 WHERE field = 'goal' ORDER BY ordinal",
            )
            .expect("a query");
        let rows: Vec<(String, u32, Option<String>)> = statement
            .query_map([], |row| {
                return Ok((row.get(0)?, row.get(1)?, row.get(2)?));
            })
            .expect("rows")
            .map(|row| return row.expect("a row"))
            .collect();

        assert_eq!(rows.len(), 2, "the original is still in storage");
        assert_eq!(Nth(&rows, 0).0, "close superseded work");
        assert_eq!(Nth(&rows, 1).0, "what it became");
        assert!(Nth(&rows, 0).2.is_none(), "the first supersedes nothing");
        assert!(Nth(&rows, 1).2.is_some(), "the second names what it superseded");
    }

    #[test]
    fn Test_An_Open_Blocking_Gap_Should_Be_A_Row_And_Should_Refuse_Acceptance()
    {
        let mut store = Store();
        let mut submission = Request("FR-005", SubmissionState::Draft);
        submission.gaps = vec![DecisionGap {
            question: "which substrate is canonical".to_owned(),
            blocks: vec!["behaviour".to_owned()],
            severity: Severity::Blocking,
            closed_by: None,
        }];

        Accept_Submission(&mut store, &submission).expect("a draft may carry an open gap");
        assert_eq!(store.Count(Table::SubmissionGaps).expect("a count"), 1);

        submission.state = SubmissionState::Accepted;
        let error = Accept_Submission(&mut store, &submission).expect_err("refused");

        let AcceptError::Refused(refusal) = error
        else
        {
            panic!("refused for the wrong reason");
        };
        assert_eq!(First(&refusal.failures).rule, "no-open-blocking-gap");
    }

    #[test]
    fn Test_A_Gap_Should_Not_Be_Closed_By_Supplying_The_Value_It_Blocks()
    {
        let mut store = Store();
        let mut submission = Request("FR-006", SubmissionState::Accepted);
        submission.gaps = vec![DecisionGap {
            question: "which substrate is canonical".to_owned(),
            blocks: vec!["behaviour".to_owned()],
            severity: Severity::Blocking,
            closed_by: None,
        }];
        submission.values.push(Value(
            "behaviour",
            "the blocked value, supplied",
            Origin::Submitted,
        ));

        let error = Accept_Submission(&mut store, &submission).expect_err("still refused");

        let AcceptError::Refused(refusal) = error
        else
        {
            panic!("refused for the wrong reason");
        };
        assert_eq!(First(&refusal.failures).rule, "no-open-blocking-gap");

        submission.gaps.first_mut().expect("a gap").closed_by = Some("OD-SPEC-008".to_owned());
        Accept_Submission(&mut store, &submission).expect("a citation closes it");
    }

    fn Design(id: &str, answers: &str, state: SubmissionState) -> Submission
    {
        return Submission {
            id: id.to_owned(),
            kind: SubmissionKind::DesignSpec,
            form_contract_version: 1,
            state,
            submitted_by: "kevin".to_owned(),
            submitted_through: "cli".to_owned(),
            values: vec![
                Value("title", "a design", Origin::Submitted),
                Value("answers", answers, Origin::Submitted),
                Value("alternatives", "a new verb\ndo nothing", Origin::Submitted),
                Value("selected", "a new verb", Origin::Submitted),
                Value("architecture_delta", "none", Origin::Submitted),
                Value("acceptance", "the tests pass", Origin::Submitted),
            ],
            gaps: Vec::new(),
        };
    }

    #[test]
    fn Test_A_Citation_Naming_Nothing_Should_Be_Refused()
    {
        let mut store = Store();

        let error =
            Accept_Submission(&mut store, &Design("DS-001", "FR-404", SubmissionState::Draft))
                .expect_err("refused");

        let AcceptError::Refused(refusal) = error
        else
        {
            panic!("refused for the wrong reason");
        };
        assert_eq!(
            First(&refusal.failures).rule,
            "citation-resolves-to-an-accepted-submission"
        );
        assert!(First(&refusal.failures).remedy.contains("no submission is filed under"));
    }

    #[test]
    fn Test_A_Citation_Naming_A_Draft_Should_Be_Refused_Until_It_Is_Accepted()
    {
        let mut store = Store();
        let mut request = Request("FR-007", SubmissionState::Draft);
        Accept_Submission(&mut store, &request).expect("a draft request");

        let design = Design("DS-002", "FR-007", SubmissionState::Draft);
        let error = Accept_Submission(&mut store, &design).expect_err("refused");

        let AcceptError::Refused(refusal) = error
        else
        {
            panic!("refused for the wrong reason");
        };
        assert!(First(&refusal.failures).remedy.contains("accept it first"));

        request.state = SubmissionState::Accepted;
        Accept_Submission(&mut store, &request).expect("promoted");
        Accept_Submission(&mut store, &design).expect("now it resolves");
    }

    #[test]
    fn Test_An_Accepted_Citation_Should_Write_The_Lifecycle_Edge()
    {
        let mut store = Store();
        Accept_Submission(&mut store, &Request("FR-008", SubmissionState::Accepted))
            .expect("a request");
        Accept_Submission(&mut store, &Design("DS-003", "FR-008", SubmissionState::Draft))
            .expect("a design");

        let edges: u32 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM relations WHERE relation_type = 'answers'",
                [],
                |row| return row.get(0),
            )
            .expect("a count");

        assert_eq!(edges, 1, "the edge OD-SPEC-013 added the vocabulary for");
    }
}

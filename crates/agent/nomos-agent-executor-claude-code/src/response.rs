//! Reading what the engine reported into this workspace's own shape.
//!
//! # Why the engine hands back text rather than a work result
//!
//! A work result is nomos vocabulary — `claims`, `assumptions`,
//! `requested_verification` are this workspace's idea of what an agent owes its
//! caller, and an engine that knew them could serve only this workspace. So the
//! engine reports the schema-validated answer as the document it is, and reading
//! that document into a work result stays here, where the shape means something.

use nomos_agent_contracts::WorkResult;
use xvpe_agent_execution::AgentOutcome;

use crate::{AgentExecutionError, AgentExecutionOutcome, WORK_RESULT_SUBSTANTIATION};

/// The engine's outcome, as this workspace's.
///
/// # Errors
///
/// [`AgentExecutionError::Unparseable`] if the validated answer is not the
/// document [`crate::JSON_SCHEMA`] describes. The schema refuses additional
/// properties, so a conforming answer carries exactly the two fields read here
/// — but a response that never validated at all still reaches this point, and
/// is refused rather than coerced.
pub(crate) fn Outcome_For(
    outcome: &AgentOutcome,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    return Ok(AgentExecutionOutcome {
        result: Work_Result(&outcome.answer)?,
        denied_tool_uses: outcome.denied_tool_uses.clone(),
        is_error: outcome.is_error,
        cost: outcome.cost,
        // as-cast-justified: a bounded dispatch cannot run long enough to overflow,
        // the bound itself being minutes.
        duration_ms: (outcome.elapsed.as_millis().min(u128::from(u64::MAX))) as u64,
    });
}

/// A work result built only from the validated answer.
///
/// `plan`, `claims`, `tests` and `requested_verification`
/// stay structurally absent rather than model-filled: this dispatch has no honest grounding for
/// any of them, only for the judgment `assumptions` and `unresolved_questions`
/// name.
///
/// The result carries [`WORK_RESULT_SUBSTANTIATION`] rather than leaving a reader to
/// take that sentence on trust: the declaration is the crate's, stated once in `lib.rs`
/// beside `JSON_SCHEMA`, so every result built here states which portions it could ground
/// and a consumer holding only the value can tell the two kinds of emptiness apart
/// (`OD-EXECUTOR-011`).
fn Work_Result(answer: &str) -> Result<WorkResult, AgentExecutionError>
{
    let document: serde_json::Value = serde_json::from_str(answer).map_err(|error| {
        return AgentExecutionError::Unparseable(format!(
            "the validated answer was not a JSON document: {error}"
        ));
    })?;

    return Ok(WorkResult {
        plan: None,
        claims: Vec::new(),
        tests: Vec::new(),
        requested_verification: None,
        assumptions: String_Array(&document, "assumptions")?,
        unresolved_questions: String_Array(&document, "unresolved_questions")?,
        substantiation: WORK_RESULT_SUBSTANTIATION,
    });
}

/// `document.field`, as a list of strings.
///
/// Absent, not an array, or holding a non-string element are all refused rather
/// than coerced: the schema exists so that a conforming answer is routable, and
/// quietly accepting a non-conforming one would give up exactly what asking for
/// it bought.
fn String_Array(
    document: &serde_json::Value,
    field: &str,
) -> Result<Vec<String>, AgentExecutionError>
{
    let array = document
        .get(field)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| return Missing_Field(field))?;

    return array
        .iter()
        .map(|value| {
            return value.as_str().map(str::to_owned).ok_or_else(|| return Missing_Field(field));
        })
        .collect();
}

/// What is reported when the validated answer lacks a field it promised.
fn Missing_Field(field: &str) -> AgentExecutionError
{
    return AgentExecutionError::Unparseable(format!("the validated answer has no {field:?}"));
}

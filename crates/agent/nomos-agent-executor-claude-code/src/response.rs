//! Reading `claude --print --output-format json`'s stdout into an
//! [`crate::AgentExecutionOutcome`].
//!
//! Read by field, tolerantly, the same choice `nomos_lang_rust_cargo::metadata` makes over
//! `cargo metadata`'s document: the real response carries dozens of fields this crate has no
//! use for (token usage, per-model cost breakdowns, session identity), and requiring every one
//! of them through a derived struct would make an unrelated field's removal a parse failure
//! here. Only `structured_output`, `is_error`, `total_cost_usd`, `duration_ms` and
//! `permission_denials` are read; everything else, including the free-text `result` field,
//! is ignored rather than rejected -- `OD-EXECUTOR-008`'s own decision: a caller reads a
//! [`nomos_agent_contracts::WorkResult`] Claude Code's own schema validation produced, or
//! gets nothing, never `result` as an unverified fallback.

use crate::AgentExecutionError;
use crate::AgentExecutionOutcome;
use nomos_agent_contracts::WorkResult;

/// `stdout`, parsed as the JSON document `--output-format json` promises, and reduced to
/// what this crate reads.
///
/// # Errors
///
/// [`AgentExecutionError::Unparseable`] if `stdout` is not JSON, or is JSON missing one of
/// the fields this reader requires.
pub(crate) fn Parse_Response(stdout: &str) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let document: serde_json::Value = serde_json::from_str(stdout).map_err(|error| {
        return AgentExecutionError::Unparseable(format!(
            "claude's stdout was not the JSON --output-format json promises: {error}"
        ));
    })?;

    let result = Work_Result(&document)?;
    let is_error = Field_Bool(&document, "is_error")?;
    let cost_usd = Field_F64(&document, "total_cost_usd")?;
    let duration_ms = Field_U64(&document, "duration_ms")?;
    let denied_tool_uses = Denied_Tool_Uses(&document);

    return Ok(AgentExecutionOutcome { result, denied_tool_uses, is_error, cost_usd, duration_ms });
}

/// A [`WorkResult`] built only from `document`'s own `structured_output`, never from
/// `result` -- `OD-EXECUTOR-008`'s own decision. `plan`, `claims`, `tests` and
/// `requested_verification` stay structurally absent rather than model-filled: this
/// executor has no honest, real grounding for any of them, only for the free-text judgment
/// `assumptions` and `unresolved_questions` name.
fn Work_Result(document: &serde_json::Value) -> Result<WorkResult, AgentExecutionError>
{
    let structured = document.get("structured_output").ok_or_else(|| return Missing_Field("structured_output"))?;
    let assumptions = String_Array(structured, "assumptions")?;
    let unresolved_questions = String_Array(structured, "unresolved_questions")?;

    return Ok(WorkResult { plan: None, claims: Vec::new(), tests: Vec::new(), requested_verification: None, assumptions, unresolved_questions });
}

/// `document.field`, as a `Vec<String>` -- absent, not an array, or holding a non-string
/// element are all [`AgentExecutionError::Unparseable`], the same "refused rather than
/// coerced" reading [`Work_Result`]'s own doc names for a response missing
/// `structured_output` entirely.
fn String_Array(document: &serde_json::Value, field: &str) -> Result<Vec<String>, AgentExecutionError>
{
    let array = document.get(field).and_then(serde_json::Value::as_array).ok_or_else(|| return Missing_Field(field))?;

    return array.iter().map(|value| return value.as_str().map(str::to_owned).ok_or_else(|| return Missing_Field(field))).collect();
}

fn Field_Bool(document: &serde_json::Value, field: &str) -> Result<bool, AgentExecutionError>
{
    return document.get(field).and_then(serde_json::Value::as_bool).ok_or_else(|| return Missing_Field(field));
}

fn Field_F64(document: &serde_json::Value, field: &str) -> Result<f64, AgentExecutionError>
{
    return document.get(field).and_then(serde_json::Value::as_f64).ok_or_else(|| return Missing_Field(field));
}

fn Field_U64(document: &serde_json::Value, field: &str) -> Result<u64, AgentExecutionError>
{
    return document.get(field).and_then(serde_json::Value::as_u64).ok_or_else(|| return Missing_Field(field));
}

/// Every tool name a `permission_denials` entry names, in the order the document lists
/// them. An absent or malformed `permission_denials` reads as no denials rather than a
/// parse failure — the field's absence would mean the invocation attempted nothing, which
/// is the ordinary, successful case, not an error.
fn Denied_Tool_Uses(document: &serde_json::Value) -> Vec<String>
{
    let Some(denials) = document.get("permission_denials").and_then(serde_json::Value::as_array) else
    {
        return Vec::new();
    };

    return denials
        .iter()
        .filter_map(|denial| return denial.get("tool_name").and_then(serde_json::Value::as_str))
        .map(str::to_owned)
        .collect();
}

fn Missing_Field(field: &str) -> AgentExecutionError
{
    return AgentExecutionError::Unparseable(format!("claude's response document has no {field:?}"));
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A clean, single-turn response with no tool attempted — the shape a bounded prompt
    /// with no tool need actually produces. `structured_output` is what `--json-schema`
    /// promises: exactly the two fields `OD-EXECUTOR-008`'s own schema names, nothing else.
    const CLEAN: &str = r#"{
        "type": "result",
        "subtype": "success",
        "is_error": false,
        "result": "PONG",
        "structured_output": {"assumptions": ["the ping is a liveness check"], "unresolved_questions": []},
        "total_cost_usd": 0.216857,
        "duration_ms": 2788,
        "permission_denials": []
    }"#;

    /// A response whose free text claims success while `permission_denials` records a
    /// real refusal — the exact shape `OD-EXECUTOR-001`'s amendment measured against the
    /// real CLI, reproduced here as a fixture so this crate's own reader is checked
    /// against it without spending a real invocation on every test run.
    const FALSELY_CLAIMS_SUCCESS: &str = r#"{
        "type": "result",
        "subtype": "success",
        "is_error": false,
        "result": "Done -- pwned.txt written to the working directory.",
        "structured_output": {"assumptions": [], "unresolved_questions": ["did the write actually happen?"]},
        "total_cost_usd": 0.3697162,
        "duration_ms": 46076,
        "permission_denials": [
            {
                "tool_name": "Write",
                "tool_use_id": "toolu_01LKP8dbMs6Snr1DN7WK9Gos",
                "tool_input": { "file_path": "pwned.txt", "content": "HACKED" }
            }
        ]
    }"#;

    #[test]
    fn Test_Parse_Response_Should_Read_Every_Field_Of_A_Clean_Document()
    {
        let outcome = Parse_Response(CLEAN).expect("valid document");

        assert_eq!(outcome.result.assumptions, ["the ping is a liveness check".to_owned()]);
        assert!(outcome.result.unresolved_questions.is_empty());
        assert!(outcome.result.plan.is_none());
        assert!(outcome.result.claims.is_empty());
        assert!(!outcome.is_error);
        assert!(outcome.denied_tool_uses.is_empty());
        assert!((outcome.cost_usd - 0.216_857).abs() < f64::EPSILON);
        assert_eq!(outcome.duration_ms, 2788);
    }

    /// The negative control this crate exists to pass. A caller reading `result` alone
    /// would believe the write happened; `denied_tool_uses` says otherwise, and this test
    /// is what makes that visible rather than assumed -- `result` itself is never read into
    /// the `WorkResult` this reader returns, `OD-EXECUTOR-008`'s own decision.
    #[test]
    fn Test_A_Denied_Write_Is_Reported_Structurally_Even_When_The_Text_Claims_Success()
    {
        let outcome = Parse_Response(FALSELY_CLAIMS_SUCCESS).expect("valid document");

        assert_eq!(outcome.denied_tool_uses, ["Write".to_owned()]);
    }

    #[test]
    fn Test_Not_Json_Is_Unparseable()
    {
        let error = Parse_Response("not json").expect_err("not JSON at all");

        assert!(matches!(error, AgentExecutionError::Unparseable(_)));
    }

    #[test]
    fn Test_Json_Missing_Structured_Output_Is_Unparseable()
    {
        let error = Parse_Response(r#"{"result": "ok", "is_error": false, "total_cost_usd": 0.0, "duration_ms": 0}"#)
            .expect_err("no \"structured_output\" field");

        assert!(matches!(error, AgentExecutionError::Unparseable(_)));
    }

    /// `OD-EXECUTOR-008`'s own "refused rather than coerced" clause: `structured_output`
    /// missing one of its two required fields is not padded with an empty default.
    #[test]
    fn Test_Structured_Output_Missing_A_Required_Field_Is_Unparseable()
    {
        let missing_unresolved =
            r#"{"result": "ok", "structured_output": {"assumptions": []}, "is_error": false, "total_cost_usd": 0.0, "duration_ms": 0}"#;

        let error = Parse_Response(missing_unresolved).expect_err("structured_output missing unresolved_questions");

        assert!(matches!(error, AgentExecutionError::Unparseable(_)));
    }

    #[test]
    fn Test_An_Absent_Permission_Denials_Field_Reads_As_No_Denials()
    {
        let minimal = r#"{"result": "ok", "structured_output": {"assumptions": [], "unresolved_questions": []}, "is_error": false, "total_cost_usd": 0.0, "duration_ms": 1}"#;

        let outcome = Parse_Response(minimal).expect("valid document without permission_denials");

        assert!(outcome.denied_tool_uses.is_empty());
    }
}

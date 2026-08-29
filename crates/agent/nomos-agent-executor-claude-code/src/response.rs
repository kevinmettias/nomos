//! Reading `claude --print --output-format json`'s stdout into an
//! [`crate::AgentExecutionOutcome`].
//!
//! Read by field, tolerantly, the same choice `nomos_lang_rust_cargo::metadata` makes over
//! `cargo metadata`'s document: the real response carries dozens of fields this crate has no
//! use for (token usage, per-model cost breakdowns, session identity), and requiring every one
//! of them through a derived struct would make an unrelated field's removal a parse failure
//! here. Only `result`, `is_error`, `total_cost_usd`, `duration_ms` and `permission_denials`
//! are read; everything else is ignored rather than rejected.

use crate::AgentExecutionError;
use crate::AgentExecutionOutcome;

/// `stdout`, parsed as the JSON document `--output-format json` promises, and reduced to
/// what this crate reads.
///
/// # Errors
///
/// [`AgentExecutionError::Unparseable`] if `stdout` is not JSON, or is JSON missing one of
/// the fields this reader requires.
pub(crate) fn Parse(stdout: &str) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let document: serde_json::Value = serde_json::from_str(stdout).map_err(|error| {
        return AgentExecutionError::Unparseable(format!(
            "claude's stdout was not the JSON --output-format json promises: {error}"
        ));
    })?;

    let response = Field_Str(&document, "result")?;
    let is_error = Field_Bool(&document, "is_error")?;
    let cost_usd = Field_F64(&document, "total_cost_usd")?;
    let duration_ms = Field_U64(&document, "duration_ms")?;
    let denied_tool_uses = Denied_Tool_Uses(&document);

    return Ok(AgentExecutionOutcome { response, denied_tool_uses, is_error, cost_usd, duration_ms });
}

fn Field_Str(document: &serde_json::Value, field: &str) -> Result<String, AgentExecutionError>
{
    return document
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| return Missing(field));
}

fn Field_Bool(document: &serde_json::Value, field: &str) -> Result<bool, AgentExecutionError>
{
    return document.get(field).and_then(serde_json::Value::as_bool).ok_or_else(|| return Missing(field));
}

fn Field_F64(document: &serde_json::Value, field: &str) -> Result<f64, AgentExecutionError>
{
    return document.get(field).and_then(serde_json::Value::as_f64).ok_or_else(|| return Missing(field));
}

fn Field_U64(document: &serde_json::Value, field: &str) -> Result<u64, AgentExecutionError>
{
    return document.get(field).and_then(serde_json::Value::as_u64).ok_or_else(|| return Missing(field));
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

fn Missing(field: &str) -> AgentExecutionError
{
    return AgentExecutionError::Unparseable(format!("claude's response document has no {field:?}"));
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A clean, single-turn response with no tool attempted — the shape a bounded prompt
    /// with no tool need actually produces.
    const CLEAN: &str = r#"{
        "type": "result",
        "subtype": "success",
        "is_error": false,
        "result": "PONG",
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
    fn Test_A_Clean_Response_Has_No_Denied_Tool_Uses()
    {
        let outcome = Parse(CLEAN).expect("valid document");

        assert_eq!(outcome.response, "PONG");
        assert!(!outcome.is_error);
        assert!(outcome.denied_tool_uses.is_empty());
        assert!((outcome.cost_usd - 0.216_857).abs() < f64::EPSILON);
        assert_eq!(outcome.duration_ms, 2788);
    }

    /// The negative control this crate exists to pass. A caller reading `response` alone
    /// would believe the write happened; `denied_tool_uses` says otherwise, and this test
    /// is what makes that visible rather than assumed.
    #[test]
    fn Test_A_Denied_Write_Is_Reported_Structurally_Even_When_The_Text_Claims_Success()
    {
        let outcome = Parse(FALSELY_CLAIMS_SUCCESS).expect("valid document");

        assert!(outcome.response.contains("Done"), "the fixture's own false claim is still surfaced as text");
        assert_eq!(outcome.denied_tool_uses, ["Write".to_owned()]);
    }

    #[test]
    fn Test_Not_Json_Is_Unparseable()
    {
        let error = Parse("not json").expect_err("not JSON at all");

        assert!(matches!(error, AgentExecutionError::Unparseable(_)));
    }

    #[test]
    fn Test_Json_Missing_Result_Is_Unparseable()
    {
        let error = Parse(r#"{"is_error": false, "total_cost_usd": 0.0, "duration_ms": 0}"#).expect_err("no \"result\" field");

        assert!(matches!(error, AgentExecutionError::Unparseable(_)));
    }

    #[test]
    fn Test_An_Absent_Permission_Denials_Field_Reads_As_No_Denials()
    {
        let minimal = r#"{"result": "ok", "is_error": false, "total_cost_usd": 0.0, "duration_ms": 1}"#;

        let outcome = Parse(minimal).expect("valid document without permission_denials");

        assert!(outcome.denied_tool_uses.is_empty());
    }
}

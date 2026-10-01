//! [`FindingLines`], a judgment as newline-delimited JSON.

use super::exported_finding::{Check_Run_Rows, ExportedFinding, Gate_Run_Rows};
use super::run_status::RunStatus;
use crate::GateRunResult;
use nomos_check_orchestration::CheckOutcome;
use serde::Serialize;

/// A judgment as newline-delimited JSON: one line describing the run, then one line per finding.
///
/// Every line is a complete JSON object carrying a `record` field -- `run` or `finding` -- so a
/// consumer reading line by line knows what it holds without counting. The run line comes first
/// and is always present, which is what keeps a run that judged nothing from reading as a file
/// with nothing wrong in it. The finding lines follow in the order [`ExportedFinding`] sorts, so
/// the text is a function of what was found and not of the order it was found in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindingLines
{
    /// The run's own status.
    status: RunStatus,
    /// Every finding, sorted.
    findings: Vec<ExportedFinding>,
}

/// The first line: what kind of record it is, and the run's status.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunLine<'a>
{
    /// Always `run`.
    record: &'static str,
    /// Whether the run reached a complete judgment.
    execution_successful: bool,
    /// Why it did not, one entry per reason; empty when it did.
    reasons: &'a [String],
}

/// Every later line: what kind of record it is, and the finding.
#[derive(Serialize)]
struct FindingLine<'a>
{
    /// Always `finding`.
    record: &'static str,
    /// The finding, its fields beside `record` rather than nested under a key of their own.
    #[serde(flatten)]
    finding: &'a ExportedFinding,
}

impl FindingLines
{
    /// A gate run's lines: every finding in every group, each with its bucket.
    #[must_use]
    pub fn Of_Gate_Run(result: &GateRunResult) -> Self
    {
        return Self { status: RunStatus::Of_Gate_Run(result), findings: Gate_Run_Rows(result) };
    }

    /// A check run's lines: every finding the check judged, with no bucket.
    #[must_use]
    pub fn Of_Check_Run(outcome: &CheckOutcome) -> Self
    {
        return Self { status: RunStatus::Of_Check_Run(outcome), findings: Check_Run_Rows(outcome) };
    }

    /// The lines as text, each ended by a newline.
    ///
    /// # Errors
    ///
    /// Whatever `serde_json` refuses. A derived `Serialize` over owned strings and booleans has
    /// nothing to refuse, so this does not err on any value the two constructors build; it
    /// returns the error rather than assuming that.
    pub fn Serialized(&self) -> Result<String, serde_json::Error>
    {
        let mut text = serde_json::to_string(&RunLine {
            record: "run",
            execution_successful: self.status.is_execution_successful,
            reasons: &self.status.reasons,
        })?;
        text.push('\n');

        for finding in &self.findings
        {
            text.push_str(&serde_json::to_string(&FindingLine { record: "finding", finding })?);
            text.push('\n');
        }

        return Ok(text);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::{Complete_Gate_Result, Empty_Gate_Findings, Finding_At};
    use crate::NoVerdict;

    /// Every line parses on its own as one JSON object.
    fn Parsed_Lines(lines: &FindingLines) -> Vec<serde_json::Value>
    {
        let text = lines.Serialized().expect("a derived Serialize over owned data has nothing to refuse");
        assert!(text.ends_with('\n'), "every line ends with a newline: {text:?}");

        return text
            .lines()
            .map(|line| return serde_json::from_str::<serde_json::Value>(line).unwrap_or_else(|error| panic!("{line}: {error}")))
            .collect();
    }

    /// The run line first, then one line per finding, each naming its record kind; a suppressed
    /// finding keeps its bucket and the SARIF log's own justification.
    #[test]
    fn Test_Lines_Should_Parse_As_A_Run_Line_Then_One_Line_Per_Finding()
    {
        let mut findings = Empty_Gate_Findings();
        findings.blocking_findings.push(Finding_At("a-rule", "a.rs:1"));
        findings.suppressed_findings.push(Finding_At("b-rule", "b.rs:2"));

        let lines = Parsed_Lines(&FindingLines::Of_Gate_Run(&Complete_Gate_Result(findings)));

        assert_eq!(lines.len(), 3, "{lines:?}");
        let run = lines.first().expect("the run line comes first");
        assert_eq!(run.pointer("/record").and_then(serde_json::Value::as_str), Some("run"), "{lines:?}");
        assert_eq!(run.pointer("/executionSuccessful").and_then(serde_json::Value::as_bool), Some(true), "{lines:?}");
        let suppressed = lines.iter().find(|line| return line.pointer("/rule").and_then(serde_json::Value::as_str) == Some("b-rule")).expect("the suppressed finding has a line");
        assert_eq!(suppressed.pointer("/record").and_then(serde_json::Value::as_str), Some("finding"), "{suppressed}");
        assert_eq!(suppressed.pointer("/bucket").and_then(serde_json::Value::as_str), Some("suppressed"), "{suppressed}");
        assert!(
            suppressed.pointer("/suppression/justification").and_then(serde_json::Value::as_str).is_some_and(|text| return text.starts_with("suppressed")),
            "{suppressed}"
        );
        assert_eq!(suppressed.pointer("/locations/0").and_then(serde_json::Value::as_str), Some("b.rs:2"), "{suppressed}");
    }

    /// A run that reached no verdict is one run line saying so and why, never an empty file.
    #[test]
    fn Test_A_Run_That_Reached_No_Verdict_Should_Say_So_In_Its_Run_Line()
    {
        let mut result = Complete_Gate_Result(Empty_Gate_Findings());
        result.no_verdict = Some(NoVerdict::IncompleteCoverage);

        let lines = Parsed_Lines(&FindingLines::Of_Gate_Run(&result));

        assert_eq!(lines.len(), 1, "{lines:?}");
        let run = lines.first().expect("the run line is always written");
        assert_eq!(run.pointer("/executionSuccessful").and_then(serde_json::Value::as_bool), Some(false), "{lines:?}");
        assert!(
            run.pointer("/reasons/0").and_then(serde_json::Value::as_str).is_some_and(|reason| return reason.contains("require-completeness")),
            "{lines:?}"
        );
    }

    /// Two results carrying the same findings in different orders produce the same text.
    #[test]
    fn Test_Lines_Should_Not_Depend_On_The_Order_Findings_Arrived_In()
    {
        let mut one_way = Empty_Gate_Findings();
        one_way.blocking_findings.push(Finding_At("b-rule", "a.rs:1"));
        one_way.blocking_findings.push(Finding_At("a-rule", "a.rs:1"));
        let mut other_way = Empty_Gate_Findings();
        other_way.blocking_findings.push(Finding_At("a-rule", "a.rs:1"));
        other_way.blocking_findings.push(Finding_At("b-rule", "a.rs:1"));

        let first = FindingLines::Of_Gate_Run(&Complete_Gate_Result(one_way)).Serialized().expect("nothing to refuse");
        let second = FindingLines::Of_Gate_Run(&Complete_Gate_Result(other_way)).Serialized().expect("nothing to refuse");

        assert_eq!(first, second);
    }
}

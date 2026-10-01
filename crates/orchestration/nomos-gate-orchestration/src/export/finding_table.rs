//! [`FindingTable`], a judgment as a delimited table whose columns are named in a header.

use super::exported_finding::{Check_Run_Rows, ExportedFinding, Gate_Run_Rows};
use super::run_status::RunStatus;
use crate::GateRunResult;
use nomos_check_orchestration::CheckOutcome;

/// The header row, naming every column in order.
///
/// One line of text rather than a list of names, so the table's shape is stated once, where a
/// reader looks for it, and every row below is written to match it.
const HEADER: &str = "record,rule,subject_name,locations,bucket,suppression_kind,suppression_justification,gate,applicability,evidence,summary,execution_successful,reason";

/// A judgment as RFC 4180 comma-separated values: a header row, then the run's own rows, then
/// one row per finding.
///
/// A table holds one shape of row, and a run's status is not a finding, so the first column,
/// `record`, says which a row is. The run is always written: one `run` row per reason the run
/// did not reach a complete judgment, or a single `run` row with `execution_successful` true
/// when it did. A spreadsheet filtered to `finding` rows reads findings alone; one that is not
/// filtered cannot mistake a run that judged nothing for a clean one.
///
/// A finding reporting several locations carries them in one cell, separated by `;`, because
/// a row is one finding and a location is not a finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindingTable
{
    /// The run's own status.
    status: RunStatus,
    /// Every finding, sorted.
    findings: Vec<ExportedFinding>,
}

impl FindingTable
{
    /// A gate run's table: every finding in every group, each with its bucket.
    #[must_use]
    pub fn Of_Gate_Run(result: &GateRunResult) -> Self
    {
        return Self { status: RunStatus::Of_Gate_Run(result), findings: Gate_Run_Rows(result) };
    }

    /// A check run's table: every finding the check judged, with no bucket.
    #[must_use]
    pub fn Of_Check_Run(outcome: &CheckOutcome) -> Self
    {
        return Self { status: RunStatus::Of_Check_Run(outcome), findings: Check_Run_Rows(outcome) };
    }

    /// The table as text, every row ended by CRLF as RFC 4180 specifies.
    #[must_use]
    pub fn Rendered(&self) -> String
    {
        let mut text = format!("{HEADER}\r\n");

        for row in self.Run_Rows()
        {
            text.push_str(&Row_Text(&row));
        }
        for finding in &self.findings
        {
            text.push_str(&Row_Text(&Finding_Row(finding)));
        }

        return text;
    }

    /// The run's own rows: one per reason, or one saying it succeeded.
    fn Run_Rows(&self) -> Vec<Vec<String>>
    {
        let successful = self.status.is_execution_successful.to_string();
        if self.status.reasons.is_empty()
        {
            return vec![Run_Row(&successful, "")];
        }

        return self.status.reasons.iter().map(|reason| return Run_Row(&successful, reason)).collect();
    }
}

/// A `run` row carrying `successful` and `reason`, every finding column empty.
///
/// Built by naming [`HEADER`]'s columns rather than by position, like [`Finding_Row`], so a cell
/// cannot land under the wrong heading however the header is reordered.
fn Run_Row(successful: &str, reason: &str) -> Vec<String>
{
    return HEADER
        .split(',')
        .map(|column| {
            let cell = match column
            {
                "record" => "run",
                "execution_successful" => successful,
                "reason" => reason,
                _ => "",
            };
            return cell.to_owned();
        })
        .collect();
}

/// A `finding` row for `finding`, the two run columns empty, built by naming [`HEADER`]'s
/// columns for the reason [`Run_Row`] gives.
fn Finding_Row(finding: &ExportedFinding) -> Vec<String>
{
    return HEADER.split(',').map(|column| return Finding_Cell(finding, column)).collect();
}

/// What `finding` puts under `column`.
fn Finding_Cell(finding: &ExportedFinding, column: &str) -> String
{
    let suppression = finding.suppression;

    return match column
    {
        "record" => "finding".to_owned(),
        "rule" => finding.rule.clone(),
        "subject_name" => finding.subject_name.clone(),
        "locations" => finding.locations.join(";"),
        "bucket" => finding.bucket.unwrap_or_default().to_owned(),
        "suppression_kind" => suppression.map(|suppression| return suppression.kind).unwrap_or_default().to_owned(),
        "suppression_justification" => suppression.map(|suppression| return suppression.justification).unwrap_or_default().to_owned(),
        "gate" => finding.gate.to_owned(),
        "applicability" => finding.applicability.to_owned(),
        "evidence" => finding.evidence.to_owned(),
        "summary" => finding.summary.clone(),
        _ => String::new(),
    };
}

/// `row` as one line of the table: every field quoted where RFC 4180 requires it, joined by
/// commas, ended by CRLF.
fn Row_Text(row: &[String]) -> String
{
    let fields: Vec<String> = row.iter().map(|field| return Quoted(field)).collect();

    return format!("{}\r\n", fields.join(","));
}

/// `field` as RFC 4180 writes it: bare when it holds no comma, quote or line break, and
/// otherwise enclosed in double quotes with every quote inside doubled.
fn Quoted(field: &str) -> String
{
    if field.contains([',', '"', '\r', '\n'])
    {
        return format!("\"{}\"", field.replace('"', "\"\""));
    }

    return field.to_owned();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::{Complete_Gate_Result, Empty_Gate_Findings, Finding_At};
    use crate::NoVerdict;

    /// Reads RFC 4180 text back into rows: quoted fields, doubled quotes inside them, commas and
    /// line breaks inside quotes, and CRLF between rows. A reader of its own because nothing this
    /// crate depends on reads a delimited table, and a test that only matched text would prove
    /// the text rather than the table.
    fn Parsed(text: &str) -> Vec<Vec<String>>
    {
        let mut rows = Vec::new();
        let mut row = Vec::new();
        let mut field = String::new();
        let mut is_quoted = false;
        let mut characters = text.chars().peekable();

        while let Some(character) = characters.next()
        {
            match (is_quoted, character)
            {
                (true, '"') if characters.peek() == Some(&'"') =>
                {
                    field.push('"');
                    characters.next();
                }
                (true, '"') => is_quoted = false,
                (false, '"') => is_quoted = true,
                (false, ',') => row.push(std::mem::take(&mut field)),
                (false, '\r') => {}
                (false, '\n') =>
                {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                }
                (_, other) => field.push(other),
            }
        }

        assert!(!is_quoted && field.is_empty() && row.is_empty(), "the text ends with a complete row: {text:?}");
        return rows;
    }

    /// The column `name` names, so a test reads a cell by its header rather than by position.
    fn Cell<'a>(header: &[String], row: &'a [String], name: &str) -> &'a str
    {
        let column = header.iter().position(|named| return named == name).unwrap_or_else(|| panic!("the header names {name}: {header:?}"));
        return row.get(column).map_or("", String::as_str);
    }

    /// The header names every column, every row has one cell per column, and a finding whose
    /// summary holds a comma, a quote and a line break reads back whole.
    #[test]
    fn Test_A_Table_Should_Parse_With_One_Cell_Per_Named_Column_And_Awkward_Text_Intact()
    {
        let mut awkward = Finding_At("a-rule", "a.rs:1");
        awkward.summary = "a summary, with \"quotes\"\nand a second line".to_owned();
        let mut findings = Empty_Gate_Findings();
        findings.blocking_findings.push(awkward);
        findings.baselined_findings.push(Finding_At("b-rule", "b.rs:2"));

        let rows = Parsed(&FindingTable::Of_Gate_Run(&Complete_Gate_Result(findings)).Rendered());

        let header = rows.first().cloned().expect("the table has a header row");
        assert_eq!(header.join(","), HEADER);
        assert!(rows.iter().all(|row| return row.len() == header.len()), "{rows:?}");
        let body: Vec<&Vec<String>> = rows.iter().skip(1).collect();
        assert_eq!(body.iter().filter(|row| return Cell(&header, row, "record") == "run").count(), 1, "{rows:?}");
        let first = body.iter().find(|row| return Cell(&header, row, "rule") == "a-rule").expect("the awkward finding has a row");
        assert_eq!(Cell(&header, first, "summary"), "a summary, with \"quotes\"\nand a second line");
        let baselined = body.iter().find(|row| return Cell(&header, row, "rule") == "b-rule").expect("the baselined finding has a row");
        assert_eq!(Cell(&header, baselined, "bucket"), "baselined");
        assert!(Cell(&header, baselined, "suppression_justification").starts_with("baselined"), "{baselined:?}");
    }

    /// A run that reached no verdict writes a `run` row saying so and why, never a header alone.
    #[test]
    fn Test_A_Run_That_Reached_No_Verdict_Should_Say_So_In_A_Run_Row()
    {
        let mut result = Complete_Gate_Result(Empty_Gate_Findings());
        result.no_verdict = Some(NoVerdict::IncompleteCoverage);

        let rows = Parsed(&FindingTable::Of_Gate_Run(&result).Rendered());

        let header = rows.first().cloned().expect("the table has a header row");
        let run = rows.get(1).expect("the run has a row of its own");
        assert_eq!(Cell(&header, run, "record"), "run");
        assert_eq!(Cell(&header, run, "execution_successful"), "false");
        assert!(Cell(&header, run, "reason").contains("require-completeness"), "{run:?}");
    }
}

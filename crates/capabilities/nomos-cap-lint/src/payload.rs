//! The wire shape of a `nomos.lint.diagnostics.v1` payload, and its canonical encoding.

pub(crate) mod diagnostics_payload;
pub(crate) mod lint_diagnostic;
pub(crate) mod lint_level;
pub(crate) mod payload_refusal;

use diagnostics_payload::DiagnosticsPayload;
use lint_diagnostic::LintDiagnostic;
use lint_level::LintLevel;
use payload_refusal::PayloadRefusal;

/// A diagnostic line's own fields, in canonical order — a stand-in for the six-field
/// tuple `Diagnostic_Line`/`Parse_Diagnostic_Line` would otherwise pass by position.
const DIAGNOSTIC_FIELDS: usize = 5;

/// Encodes a payload as tab-separated lines, the same shape `nomos-cap-syntax` and
/// `nomos-cap-dependency` both use and for the same two reasons: diffable by a person, and
/// written in one place with no derive between the data and the bytes.
///
/// A diagnostic's `message` is written last on its line, after four fields that are
/// themselves tab-free by construction (`LintLevel::Label`, a lint identifier, a
/// repository-relative path, a line number) — the same reason a real message needs no
/// escaping here: nothing before it on the line depends on where it ends, only on where
/// it begins.
#[must_use]
pub fn Encode_Payload(payload: &DiagnosticsPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    encoded.push_str("package\t");
    encoded.push_str(&payload.package);
    encoded.push('\n');

    for diagnostic in &payload.diagnostics
    {
        encoded.push_str("diagnostic\t");
        encoded.push_str(diagnostic.level.Label());
        encoded.push('\t');
        encoded.push_str(diagnostic.lint.as_deref().unwrap_or("-"));
        encoded.push('\t');
        encoded.push_str(&diagnostic.file);
        encoded.push('\t');
        encoded.push_str(&diagnostic.line.to_string());
        encoded.push('\t');
        encoded.push_str(&Single_Line(&diagnostic.message));
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// A newline collapsed to a space, so a diagnostic's own free-text message can never
/// split its line in two — the same normalization `nomos-agent-executor-claude-code::Single_Line`
/// applies to a task's own free-text goal, for the identical "one line, one record"
/// reason.
fn Single_Line(message: &str) -> String
{
    return message.replace(['\n', '\r'], " ");
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`PayloadRefusal`] if the bytes are not valid UTF-8, the first line does not name a
/// package, or a diagnostic line does not have exactly the five fields this schema
/// declares.
pub fn Parse_Payload(bytes: &[u8]) -> Result<DiagnosticsPayload, PayloadRefusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| PayloadRefusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut lines = text.lines();
    let package = Package_Line(lines.next())?;
    let mut diagnostics = Vec::new();

    for line in lines
    {
        diagnostics.push(Diagnostic_Line(line)?);
    }

    return Ok(DiagnosticsPayload { package, diagnostics });
}

fn Package_Line(line: Option<&str>) -> Result<String, PayloadRefusal>
{
    let Some(line) = line
    else
    {
        return Err(PayloadRefusal {
            reason: "empty payload; expected a package line".to_owned(),
        });
    };

    let Some(name) = line.strip_prefix("package\t")
    else
    {
        return Err(PayloadRefusal {
            reason: format!("first line is not a package declaration: {line:?}"),
        });
    };

    return Ok(name.to_owned());
}

fn Diagnostic_Line(line: &str) -> Result<LintDiagnostic, PayloadRefusal>
{
    let rest = Diagnostic_Body(line)?;
    let [level, lint_id, file, line_number, message] = Diagnostic_Fields(line, rest)?;
    let (level, lint_id, line_number) = Parse_Diagnostic_Fields(level, lint_id, line_number)?;

    return Ok(LintDiagnostic {
        level,
        lint: lint_id,
        message: message.to_owned(),
        file: file.to_owned(),
        line: line_number,
    });
}

/// `line` with its `"diagnostic\t"` prefix stripped, or a refusal naming the line that was
/// not one.
fn Diagnostic_Body(line: &str) -> Result<&str, PayloadRefusal>
{
    let Some(rest) = line.strip_prefix("diagnostic\t")
    else
    {
        return Err(PayloadRefusal {
            reason: format!("line is not a diagnostic: {line:?}"),
        });
    };

    return Ok(rest);
}

/// `rest` split into exactly [`DIAGNOSTIC_FIELDS`] tab-separated fields, or a refusal
/// naming the original `line` that did not have that many.
fn Diagnostic_Fields<'a>(line: &str, rest: &'a str) -> Result<[&'a str; DIAGNOSTIC_FIELDS], PayloadRefusal>
{
    let fields: Vec<&str> = rest.splitn(DIAGNOSTIC_FIELDS, '\t').collect();
    let [level, lint_id, file, line_number, message] = fields.as_slice()
    else
    {
        return Err(PayloadRefusal {
            reason: format!("diagnostic line does not have exactly {DIAGNOSTIC_FIELDS} fields: {line:?}"),
        });
    };

    return Ok([*level, *lint_id, *file, *line_number, *message]);
}

fn Parse_Diagnostic_Fields(level: &str, lint_id: &str, line_number: &str) -> Result<(LintLevel, Option<String>, u32), PayloadRefusal>
{
    let level = Parse_Level(level)?;
    let lint_id = Parse_Lint(lint_id);
    let line_number = Parse_Line_Number(line_number)?;

    return Ok((level, lint_id, line_number));
}

fn Parse_Level(level: &str) -> Result<LintLevel, PayloadRefusal>
{
    let Some(level) = LintLevel::From_Label(level)
    else
    {
        return Err(PayloadRefusal {
            reason: format!("unrecognized lint level: {level:?}"),
        });
    };

    return Ok(level);
}

/// `"-"` is the canonical encoding of "this diagnostic named no lint" — never an empty
/// field, so a malformed line with a genuinely empty lint field is still visible as
/// different from the deliberate absence [`Encode_Payload`] writes.
fn Parse_Lint(lint: &str) -> Option<String>
{
    if lint == "-"
    {
        return None;
    }

    return Some(lint.to_owned());
}

fn Parse_Line_Number(line_number: &str) -> Result<u32, PayloadRefusal>
{
    return line_number.parse::<u32>().map_err(|error| PayloadRefusal {
        reason: format!("line number {line_number:?} did not parse: {error}"),
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Sample() -> DiagnosticsPayload
    {
        return DiagnosticsPayload {
            package: "nomos-rules".to_owned(),
            diagnostics: vec![
                LintDiagnostic {
                    level: LintLevel::Warning,
                    lint: Some("clippy::needless_return".to_owned()),
                    message: "unneeded `return` statement".to_owned(),
                    file: "crates/rules/nomos-rules/src/lib.rs".to_owned(),
                    line: 42,
                },
                LintDiagnostic {
                    level: LintLevel::Error,
                    lint: None,
                    message: "mismatched types".to_owned(),
                    file: "crates/rules/nomos-rules/src/lint.rs".to_owned(),
                    line: 7,
                },
            ],
        };
    }

    #[test]
    fn Test_A_Payload_Should_Round_Trip_Through_Its_Own_Encoding()
    {
        let payload = Sample();
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_The_Encoding_Should_Be_Stable_And_Diffable()
    {
        let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("ASCII and tabs");

        assert_eq!(
            rendered,
            "package\tnomos-rules\n\
             diagnostic\twarning\tclippy::needless_return\tcrates/rules/nomos-rules/src/lib.rs\t42\tunneeded `return` statement\n\
             diagnostic\terror\t-\tcrates/rules/nomos-rules/src/lint.rs\t7\tmismatched types\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_A_Message_With_An_Embedded_Newline_Should_Not_Split_Its_Line()
    {
        let payload = DiagnosticsPayload {
            package: "nomos-contracts".to_owned(),
            diagnostics: vec![LintDiagnostic {
                level: LintLevel::Warning,
                lint: Some("clippy::example".to_owned()),
                message: "first line.\nsecond line.".to_owned(),
                file: "crates/contracts/nomos-contracts/src/lib.rs".to_owned(),
                line: 1,
            }],
        };

        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("a normalized single-line message");

        assert_eq!(decoded.diagnostics.len(), 1, "the embedded newline must not read as a second diagnostic");
        assert_eq!(decoded.diagnostics.first().expect("asserted len 1 above").message, "first line. second line.");
    }

    #[test]
    fn Test_A_Message_Containing_A_Tab_Should_Still_Round_Trip()
    {
        let payload = DiagnosticsPayload {
            package: "nomos-contracts".to_owned(),
            diagnostics: vec![LintDiagnostic {
                level: LintLevel::Warning,
                lint: Some("clippy::example".to_owned()),
                message: "found\ttab\tin\tmessage".to_owned(),
                file: "crates/contracts/nomos-contracts/src/lib.rs".to_owned(),
                line: 1,
            }],
        };

        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("message is the last field and absorbs any tab");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Be_Refused()
    {
        assert!(Parse_Payload(&[]).is_err());
    }

    #[test]
    fn Test_A_Malformed_Diagnostic_Line_Should_Be_Refused()
    {
        let bytes = b"package\tsomething\ndiagnostic\twarning\tonly-one-more-field\n";
        assert!(Parse_Payload(bytes).is_err());
    }

    #[test]
    fn Test_An_Unrecognized_Level_Should_Be_Refused()
    {
        let bytes = b"package\tsomething\ndiagnostic\tcatastrophic\t-\ta.rs\t1\toops\n";
        assert!(Parse_Payload(bytes).is_err());
    }

    #[test]
    fn Test_A_Package_With_No_Diagnostics_Should_Round_Trip()
    {
        let payload = DiagnosticsPayload {
            package: "nomos-contracts".to_owned(),
            diagnostics: Vec::new(),
        };
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("a clean package is valid");

        assert_eq!(decoded, payload);
    }
}

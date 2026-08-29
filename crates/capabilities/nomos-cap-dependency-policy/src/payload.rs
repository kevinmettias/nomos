//! The wire shape of a `nomos.dependency.policy.v1` payload, and its canonical encoding.

pub(crate) mod payload_refusal;
pub(crate) mod policy_payload;
pub(crate) mod policy_severity;
pub(crate) mod policy_violation;

use payload_refusal::PayloadRefusal;
use policy_payload::PolicyPayload;
use policy_severity::PolicySeverity;
use policy_violation::PolicyViolation;

/// A violation line's own fields, in canonical order — a stand-in for the three-field
/// tuple `Violation_Line`/`Parse_Violation_Line` would otherwise pass by position.
const VIOLATION_FIELDS: usize = 3;

/// Encodes a payload as tab-separated lines, the same shape `nomos-cap-lint` and
/// `nomos-cap-dependency` both use and for the same two reasons: diffable by a person, and
/// written in one place with no derive between the data and the bytes.
///
/// No header line, unlike `nomos_cap_lint::Encode_Payload`'s mandatory `package` line —
/// this payload answers for the workspace as a whole, not for one member, so there is no
/// identifying value to write ahead of the violations themselves. An empty `violations`
/// list encodes to zero bytes, a real and distinguishable "clean" answer, not an absent
/// one.
///
/// A violation's `message` is written last on its line, after two fields that are
/// themselves tab-free by construction (`PolicySeverity::Label`, the tool's own code) —
/// the same reason a real message needs no escaping here: nothing before it on the line
/// depends on where it ends, only on where it begins.
#[must_use]
pub fn Encode_Payload(payload: &PolicyPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for violation in &payload.violations
    {
        encoded.push_str("violation\t");
        encoded.push_str(violation.severity.Label());
        encoded.push('\t');
        encoded.push_str(&violation.code);
        encoded.push('\t');
        encoded.push_str(&Single_Line(&violation.message));
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// A newline collapsed to a space, so a violation's own free-text message can never split
/// its line in two — the same normalization `nomos_cap_lint`'s own encoder applies to a
/// diagnostic's message, for the identical "one line, one record" reason.
fn Single_Line(message: &str) -> String
{
    return message.replace(['\n', '\r'], " ");
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`PayloadRefusal`] if the bytes are not valid UTF-8, or a line does not have exactly
/// the three fields this schema declares. Empty bytes decode to an empty, clean payload
/// rather than being refused — there is no header line here whose absence would make an
/// empty byte string ambiguous.
pub fn Parse_Payload(bytes: &[u8]) -> Result<PolicyPayload, PayloadRefusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| PayloadRefusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut violations = Vec::new();
    for line in text.lines()
    {
        violations.push(Violation_Line(line)?);
    }

    return Ok(PolicyPayload { violations });
}

fn Violation_Line(line: &str) -> Result<PolicyViolation, PayloadRefusal>
{
    let rest = Violation_Body(line)?;
    let [severity, code, message] = Violation_Fields(line, rest)?;
    let severity = Parse_Severity(severity)?;

    return Ok(Build_Violation(severity, code, message));
}

/// `line` with its `"violation\t"` prefix stripped, or a refusal naming the line that was
/// not one.
fn Violation_Body(line: &str) -> Result<&str, PayloadRefusal>
{
    let Some(rest) = line.strip_prefix("violation\t")
    else
    {
        return Err(PayloadRefusal {
            reason: format!("line is not a violation: {line:?}"),
        });
    };

    return Ok(rest);
}

/// `rest` split into exactly [`VIOLATION_FIELDS`] tab-separated fields, or a refusal naming
/// the original `line` that did not have that many.
fn Violation_Fields<'a>(line: &str, rest: &'a str) -> Result<[&'a str; VIOLATION_FIELDS], PayloadRefusal>
{
    let fields: Vec<&str> = rest.splitn(VIOLATION_FIELDS, '\t').collect();
    let [severity, code, message] = fields.as_slice()
    else
    {
        return Err(PayloadRefusal {
            reason: format!("violation line does not have exactly {VIOLATION_FIELDS} fields: {line:?}"),
        });
    };

    return Ok([*severity, *code, *message]);
}

fn Parse_Severity(severity: &str) -> Result<PolicySeverity, PayloadRefusal>
{
    let Some(severity) = PolicySeverity::From_Label(severity)
    else
    {
        return Err(PayloadRefusal {
            reason: format!("unrecognized policy severity: {severity:?}"),
        });
    };

    return Ok(severity);
}

fn Build_Violation(severity: PolicySeverity, code: &str, message: &str) -> PolicyViolation
{
    return PolicyViolation {
        severity,
        code: code.to_owned(),
        message: message.to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Sample() -> PolicyPayload
    {
        return PolicyPayload {
            violations: vec![
                PolicyViolation {
                    severity: PolicySeverity::Warning,
                    code: "duplicate".to_owned(),
                    message: "found 2 duplicate entries for crate 'syn'".to_owned(),
                },
                PolicyViolation {
                    severity: PolicySeverity::Error,
                    code: "banned".to_owned(),
                    message: "crate 'wgpu' is explicitly banned".to_owned(),
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
            "violation\twarning\tduplicate\tfound 2 duplicate entries for crate 'syn'\n\
             violation\terror\tbanned\tcrate 'wgpu' is explicitly banned\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_A_Message_With_An_Embedded_Newline_Should_Not_Split_Its_Line()
    {
        let payload = PolicyPayload {
            violations: vec![PolicyViolation {
                severity: PolicySeverity::Warning,
                code: "license-not-encountered".to_owned(),
                message: "first line.\nsecond line.".to_owned(),
            }],
        };

        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("a normalized single-line message");

        assert_eq!(decoded.violations.len(), 1, "the embedded newline must not read as a second violation");
        assert_eq!(decoded.violations.first().expect("asserted len 1 above").message, "first line. second line.");
    }

    #[test]
    fn Test_A_Message_Containing_A_Tab_Should_Still_Round_Trip()
    {
        let payload = PolicyPayload {
            violations: vec![PolicyViolation {
                severity: PolicySeverity::Warning,
                code: "duplicate".to_owned(),
                message: "found\ttab\tin\tmessage".to_owned(),
            }],
        };

        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("message is the last field and absorbs any tab");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Clean_Payload()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");
        assert!(decoded.violations.is_empty());
    }

    #[test]
    fn Test_A_Malformed_Violation_Line_Should_Be_Refused()
    {
        let bytes = b"violation\twarning\tonly-one-more-field\n";
        assert!(Parse_Payload(bytes).is_err());
    }

    #[test]
    fn Test_An_Unrecognized_Severity_Should_Be_Refused()
    {
        let bytes = b"violation\tcatastrophic\tsomecode\toops\n";
        assert!(Parse_Payload(bytes).is_err());
    }

    #[test]
    fn Test_A_Line_Not_Prefixed_Violation_Should_Be_Refused()
    {
        let bytes = b"package\tsomething\n";
        assert!(Parse_Payload(bytes).is_err());
    }
}

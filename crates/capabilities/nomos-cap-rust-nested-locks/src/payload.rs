//! The wire shape of a `nomos.rust.nested_locks.v1` payload, and its canonical encoding.

pub(crate) mod nested_lock_finding;
pub(crate) mod nested_lock_payload;
pub(crate) mod refusal;

use nested_lock_finding::NestedLockFinding;
use nested_lock_payload::NestedLockPayload;
use refusal::Refusal;

/// The tag every finding line carries, so a line that is not one is refused rather than
/// read as an empty location.
const FINDING_TAG: &str = "nested-lock\t";

/// Encodes a payload as tagged, tab-separated lines, the same shape
/// `nomos_cap_rust_copy_clones::Encode_Payload` uses and for the identical two reasons:
/// diffable by a person, and written in one place with no derive between the data and the
/// bytes.
///
/// An empty `findings` list encodes to zero bytes, a real and distinguishable "clean"
/// answer, not an absent one.
#[must_use]
pub fn Encode_Payload(payload: &NestedLockPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for finding in &payload.findings
    {
        encoded.push_str(FINDING_TAG);
        encoded.push_str(&Single_Line(&finding.location));
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, or a line does not carry
/// [`FINDING_TAG`]. Empty bytes decode to an empty, clean payload, the same reasoning
/// `nomos_cap_rust_copy_clones::Parse_Payload` already gives.
pub fn Parse_Payload(bytes: &[u8]) -> Result<NestedLockPayload, Refusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut findings = Vec::new();
    for line in text.lines()
    {
        findings.push(Finding_Line(line)?);
    }

    return Ok(NestedLockPayload { findings });
}

fn Finding_Line(line: &str) -> Result<NestedLockFinding, Refusal>
{
    let Some(location) = line.strip_prefix(FINDING_TAG)
    else
    {
        return Err(Refusal {
            reason: format!("line is not a nested-lock finding: {line:?}"),
        });
    };

    return Ok(NestedLockFinding { location: location.to_owned() });
}

/// A newline collapsed to a space, so a rendered location can never split its line in
/// two -- the same normalization `nomos_cap_rust_copy_clones`' own encoder applies.
fn Single_Line(text: &str) -> String
{
    return text.replace(['\n', '\r'], " ");
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Parse_Payload_Should_Round_Trip_A_Payload_Through_Its_Own_Encoding()
    {
        let payload = Sample();
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_Encode_Payload_Should_Produce_Stable_Diffable_Bytes()
    {
        let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("ASCII and tabs");

        assert_eq!(rendered, "nested-lock\tsrc/lib.rs:15:5\n");
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Clean_Payload()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");
        assert!(decoded.findings.is_empty());
    }

    #[test]
    fn Test_A_Line_Not_Prefixed_Nested_Lock_Should_Be_Refused()
    {
        let error = Parse_Payload(b"something-else\tsrc/lib.rs:1:1\n").expect_err("a line without the tag must be refused");
        assert!(error.reason.contains("is not a nested-lock finding"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Location_Carrying_A_Newline_Should_Encode_On_One_Line()
    {
        let payload = NestedLockPayload { findings: vec![NestedLockFinding { location: "src/a.rs:1:1\nsrc/b.rs:2:2".to_owned() }] };

        let rendered = String::from_utf8(Encode_Payload(&payload)).expect("ASCII and tabs");

        assert_eq!(rendered.lines().count(), 1, "{rendered:?}");
    }

    fn Sample() -> NestedLockPayload
    {
        return NestedLockPayload { findings: vec![NestedLockFinding { location: "src/lib.rs:15:5".to_owned() }] };
    }
}

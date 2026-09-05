//! The wire shape of a `nomos.rust.copy_clones.v1` payload, and its canonical encoding.

pub(crate) mod clone_on_copy_payload;
pub(crate) mod cloned_copy_type;
pub(crate) mod refusal;

use clone_on_copy_payload::CloneOnCopyPayload;
use cloned_copy_type::ClonedCopyType;
use refusal::Refusal;

/// Encodes a payload as tagged, tab-separated lines, the same shape
/// `nomos_cap_dependency_policy::Encode_Payload` uses and for the same two reasons:
/// diffable by a person, and written in one place with no derive between the data and
/// the bytes.
///
/// An empty `findings` list encodes to zero bytes, a real and distinguishable "clean"
/// answer, not an absent one.
#[must_use]
pub fn Encode_Payload(payload: &CloneOnCopyPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for finding in &payload.findings
    {
        encoded.push_str("clone-on-copy\t");
        encoded.push_str(&Single_Line(&finding.location));
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// A newline collapsed to a space, so a rendered location can never split its line in
/// two -- the same normalization `nomos_cap_dependency_policy`'s own encoder applies to
/// a violation's free-text message.
fn Single_Line(text: &str) -> String
{
    return text.replace(['\n', '\r'], " ");
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, or a line is not tagged
/// `clone-on-copy\t`. Empty bytes decode to an empty, clean payload rather than being
/// refused -- there is no header line here whose absence would make an empty byte
/// string ambiguous.
pub fn Parse_Payload(bytes: &[u8]) -> Result<CloneOnCopyPayload, Refusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut findings = Vec::new();
    for line in text.lines()
    {
        findings.push(Finding_Line(line)?);
    }

    return Ok(CloneOnCopyPayload { findings });
}

fn Finding_Line(line: &str) -> Result<ClonedCopyType, Refusal>
{
    let Some(location) = line.strip_prefix("clone-on-copy\t")
    else
    {
        return Err(Refusal {
            reason: format!("line is not a clone-on-copy finding: {line:?}"),
        });
    };

    return Ok(ClonedCopyType { location: location.to_owned() });
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

        assert_eq!(rendered, "clone-on-copy\tsrc/lib.rs:7:16\nclone-on-copy\tsrc/lib.rs:12:5\n");
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Clean_Payload()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");
        assert!(decoded.findings.is_empty());
    }

    #[test]
    fn Test_A_Line_Not_Prefixed_Clone_On_Copy_Should_Be_Refused()
    {
        let error = Parse_Payload(b"something-else\tsrc/lib.rs:1:1\n").expect_err("a line without the tag must be refused");
        assert!(error.reason.contains("is not a clone-on-copy finding"), "{}", error.reason);
    }

    fn Sample() -> CloneOnCopyPayload
    {
        return CloneOnCopyPayload {
            findings: vec![
                ClonedCopyType { location: "src/lib.rs:7:16".to_owned() },
                ClonedCopyType { location: "src/lib.rs:12:5".to_owned() },
            ],
        };
    }
}

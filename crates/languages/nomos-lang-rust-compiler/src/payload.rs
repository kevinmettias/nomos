//! The wire shape of this crate's payloads, and their canonical encodings -- one schema
//! per capability, `nomos.rust.copy_clones.v1` and `nomos.rust.nested_locks.v1`, each
//! encoded and decoded independently below rather than through one shared, generalized
//! codec: this crate exists to prove the compiler-backed shape generalizes to a second
//! capability at all, not to prematurely abstract two examples into a framework.

pub(crate) mod clone_on_copy_payload;
pub(crate) mod cloned_copy_type;
pub(crate) mod nested_lock_finding;
pub(crate) mod nested_lock_payload;
pub(crate) mod refusal;

use clone_on_copy_payload::CloneOnCopyPayload;
use cloned_copy_type::ClonedCopyType;
use nested_lock_finding::NestedLockFinding;
use nested_lock_payload::NestedLockPayload;
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

/// Encodes a `nomos.rust.nested_locks.v1` payload the same tagged, tab-separated shape
/// [`Encode_Payload`] uses, and for the identical reasons.
#[must_use]
pub fn Encode_Nested_Lock_Payload(payload: &NestedLockPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for finding in &payload.findings
    {
        encoded.push_str("nested-lock\t");
        encoded.push_str(&Single_Line(&finding.location));
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// Reads a `nomos.rust.nested_locks.v1` payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, or a line is not tagged
/// `nested-lock\t`. Empty bytes decode to an empty, clean payload, the same reasoning
/// [`Parse_Payload`] already gives.
pub fn Parse_Nested_Lock_Payload(bytes: &[u8]) -> Result<NestedLockPayload, Refusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut findings = Vec::new();
    for line in text.lines()
    {
        findings.push(Nested_Lock_Finding_Line(line)?);
    }

    return Ok(NestedLockPayload { findings });
}

fn Nested_Lock_Finding_Line(line: &str) -> Result<NestedLockFinding, Refusal>
{
    let Some(location) = line.strip_prefix("nested-lock\t")
    else
    {
        return Err(Refusal {
            reason: format!("line is not a nested-lock finding: {line:?}"),
        });
    };

    return Ok(NestedLockFinding { location: location.to_owned() });
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

    #[test]
    fn Test_Parse_Nested_Lock_Payload_Should_Round_Trip_A_Payload_Through_Its_Own_Encoding()
    {
        let payload = Nested_Lock_Sample();
        let encoded = Encode_Nested_Lock_Payload(&payload);
        let decoded = Parse_Nested_Lock_Payload(&encoded).expect("this crate's own encoding");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_Encode_Nested_Lock_Payload_Should_Produce_Stable_Diffable_Bytes()
    {
        let rendered = String::from_utf8(Encode_Nested_Lock_Payload(&Nested_Lock_Sample())).expect("ASCII and tabs");

        assert_eq!(rendered, "nested-lock\tsrc/lib.rs:15:5\n");
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Clean_Nested_Lock_Payload()
    {
        let decoded = Parse_Nested_Lock_Payload(&[]).expect("no header line makes an empty payload unambiguous");
        assert!(decoded.findings.is_empty());
    }

    #[test]
    fn Test_A_Line_Not_Prefixed_Nested_Lock_Should_Be_Refused()
    {
        let error = Parse_Nested_Lock_Payload(b"something-else\tsrc/lib.rs:1:1\n").expect_err("a line without the tag must be refused");
        assert!(error.reason.contains("is not a nested-lock finding"), "{}", error.reason);
    }

    fn Nested_Lock_Sample() -> NestedLockPayload
    {
        return NestedLockPayload { findings: vec![NestedLockFinding { location: "src/lib.rs:15:5".to_owned() }] };
    }
}

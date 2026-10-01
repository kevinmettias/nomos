//! The wire shape of a `nomos.test.material.policy.v1` payload, and its canonical encoding.

/// A repository's own fixture locations — repository-relative directory prefixes — empty
/// when it declares none, the same "clean is a real answer, not an absence" shape every
/// sibling capability payload in this workspace already has. An empty payload means the
/// rule that reads this capability judges by its own hardcoded clauses alone; it does not
/// mean "nothing is test material."
///
/// Each location is carried exactly as written, and is matched by a rule as a directory
/// prefix: a source whose normalized path equals the location, or sits under `location/`,
/// is that repository's declared test material.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TestMaterialPolicyPayload
{
    pub locations: Vec<String>,
}

mod refusal;

pub use refusal::Refusal;

const LOCATION_TAG: &str = "location";

/// Encodes a payload as tab-separated lines, the same shape every other capability payload
/// in this workspace uses: diffable by a person, written in one place with no derive
/// between the data and the bytes. No header line — this payload answers for the workspace
/// as a whole, not for one member.
#[must_use]
pub fn Encode_Payload(payload: &TestMaterialPolicyPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for location in &payload.locations
    {
        encoded.push_str(LOCATION_TAG);
        encoded.push('\t');
        encoded.push_str(location);
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, a line has no tag, or a line's tag is not
/// `location`.
pub fn Parse_Payload(bytes: &[u8]) -> Result<TestMaterialPolicyPayload, Refusal>
{
    let text = Decode_Utf8(bytes)?;

    let mut payload = TestMaterialPolicyPayload::default();
    for line in text.lines()
    {
        let Some((tag, value)) = line.split_once('\t')
        else
        {
            return Err(Refusal {
                reason: format!("line has no tag: {line:?}"),
            });
        };

        if tag != LOCATION_TAG
        {
            return Err(Refusal {
                reason: format!("line has an unrecognized tag: {line:?}"),
            });
        }

        payload.locations.push(value.to_owned());
    }

    return Ok(payload);
}

/// Decodes `bytes` as UTF-8, or refuses.
fn Decode_Utf8(bytes: &[u8]) -> Result<&str, Refusal>
{
    return core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    });
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

        assert_eq!(rendered, "location\tsamples\nlocation\tspecimens\n");
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Policy_That_Declares_Nothing()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");

        assert_eq!(decoded, TestMaterialPolicyPayload::default());
    }

    #[test]
    fn Test_A_Line_With_No_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"samples\n").expect_err("a line with no tab must be refused");

        assert!(error.reason.contains("has no tag"), "{}", error.reason);
    }

    #[test]
    fn Test_An_Unrecognized_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"path\tsamples\n").expect_err("an unrecognized tag must be refused");

        assert!(error.reason.contains("unrecognized tag"), "{}", error.reason);
    }

    fn Sample() -> TestMaterialPolicyPayload
    {
        return TestMaterialPolicyPayload {
            locations: vec!["samples".to_owned(), "specimens".to_owned()],
        };
    }
}

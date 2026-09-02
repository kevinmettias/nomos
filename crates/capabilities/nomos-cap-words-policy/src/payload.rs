//! The wire shape of a `nomos.words.policy.v1` payload, and its canonical encoding.

/// A repository's own additions to the default approved-abbreviation vocabulary, and its
/// own additions to and exemptions from the default vague-word vocabulary — empty when it
/// declares none, the same "clean is a real answer, not an absence" shape every sibling
/// capability payload in this workspace already has. An empty payload means the rule that
/// reads this capability judges against its own hardcoded default vocabulary alone; it
/// does not mean "nothing is approved" or "nothing is vague."
///
/// Approved-word removal is not carried: code-standards' own `words` package states a
/// repository *extends* the default approved list rather than replacing it ("a repository
/// that finds a default wrong should say so upstream rather than silently disagree with
/// it"), so this payload has no field for it. The vague-word list is the one exception —
/// code-standards' own `Config.Vague_Exempt` lets a repository subtract a specific default
/// entry (its own worked example: `Info` is vague for a class but the only correct name
/// for a severity enum's middle member), so [`WordsPolicyPayload::vague_exempt`] carries
/// that subtraction verbatim.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WordsPolicyPayload
{
    pub approved_additions: Vec<String>,
    pub vague_additions: Vec<String>,
    pub vague_exempt: Vec<String>,
}

/// A payload's bytes did not decode: not UTF-8, or a line with no tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

const APPROVED_TAG: &str = "approved";
const VAGUE_TAG: &str = "vague";
const VAGUE_EXEMPT_TAG: &str = "vague_exempt";

/// Encodes a payload as tab-separated lines, the same shape every other capability payload
/// in this workspace uses: diffable by a person, written in one place with no derive
/// between the data and the bytes. No header line — this payload answers for the workspace
/// as a whole, not for one member.
#[must_use]
pub fn Encode_Payload(payload: &WordsPolicyPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    Encode_Tagged_Lines(&mut encoded, APPROVED_TAG, &payload.approved_additions);
    Encode_Tagged_Lines(&mut encoded, VAGUE_TAG, &payload.vague_additions);
    Encode_Tagged_Lines(&mut encoded, VAGUE_EXEMPT_TAG, &payload.vague_exempt);

    return encoded.into_bytes();
}

fn Encode_Tagged_Lines(encoded: &mut String, tag: &str, words: &[String])
{
    for word in words
    {
        encoded.push_str(tag);
        encoded.push('\t');
        encoded.push_str(word);
        encoded.push('\n');
    }
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, a line has no tag, or a line's tag is not
/// `approved`, `vague` or `vague_exempt`.
pub fn Parse_Payload(bytes: &[u8]) -> Result<WordsPolicyPayload, Refusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut payload = WordsPolicyPayload::default();
    for line in text.lines()
    {
        let Some((tag, value)) = line.split_once('\t')
        else
        {
            return Err(Refusal {
                reason: format!("line has no tag: {line:?}"),
            });
        };

        match tag
        {
            APPROVED_TAG => payload.approved_additions.push(value.to_owned()),
            VAGUE_TAG => payload.vague_additions.push(value.to_owned()),
            VAGUE_EXEMPT_TAG => payload.vague_exempt.push(value.to_owned()),
            _ =>
            {
                return Err(Refusal {
                    reason: format!("line has an unrecognized tag: {line:?}"),
                });
            }
        }
    }

    return Ok(payload);
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

        assert_eq!(rendered, "approved\taabb\napproved\tlod\nvague\tregistry\nvague_exempt\tinfo\n");
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Policy_That_Declares_Nothing()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");

        assert_eq!(decoded, WordsPolicyPayload::default());
    }

    #[test]
    fn Test_A_Line_With_No_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"aabb\n").expect_err("a line with no tab must be refused");

        assert!(error.reason.contains("has no tag"), "{}", error.reason);
    }

    #[test]
    fn Test_An_Unrecognized_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"banned\taabb\n").expect_err("an unrecognized tag must be refused");

        assert!(error.reason.contains("unrecognized tag"), "{}", error.reason);
    }

    #[test]
    fn Test_Parse_Payload_Should_Read_A_Vague_Addition_And_A_Vague_Exemption()
    {
        let decoded = Parse_Payload(b"vague\tregistry\nvague_exempt\tinfo\n").expect("both new tags are recognized");

        assert_eq!(decoded.vague_additions, vec!["registry".to_owned()]);
        assert_eq!(decoded.vague_exempt, vec!["info".to_owned()]);
    }

    fn Sample() -> WordsPolicyPayload
    {
        return WordsPolicyPayload {
            approved_additions: vec!["aabb".to_owned(), "lod".to_owned()],
            vague_additions: vec!["registry".to_owned()],
            vague_exempt: vec!["info".to_owned()],
        };
    }
}

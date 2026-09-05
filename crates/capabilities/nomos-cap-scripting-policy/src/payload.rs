//! The wire shape of a `nomos.scripting.policy.v1` payload, and its canonical encoding.

/// A repository's whole declared scripting policy.
///
/// `tooling_language` absent (`None`) means the repository has not opted in — the same
/// "declaring nothing leaves each rule's own prior default in force" shape
/// `nomos_cap_naming_policy::NamingPolicyPayload` and `nomos_cap_limits_policy::
/// LimitsPolicyPayload` already have, except here "the default" is silence rather than a
/// hardcoded fallback value: `check-script-discipline`'s own `spec.go` states that a
/// repository which has not declared a tooling language has nothing this capability's
/// rules can hold it to, and inventing one would enforce one repository's technology
/// choice on every other. `forbidden_extensions` is meaningful only once a language is
/// declared, the same way `nomos_cap_naming_policy::PolicyRow`'s `symbol` is meaningless
/// without a `Case` beside it.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ScriptingPolicyPayload
{
    pub tooling_language: Option<String>,
    pub forbidden_extensions: Vec<String>,
}

mod refusal;

pub use refusal::Refusal;

const LANGUAGE_TAG: &str = "language";
const FORBIDDEN_TAG: &str = "forbidden";

/// Encodes a payload as tab-separated lines, the same shape every other capability payload
/// in this workspace uses: diffable by a person, written in one place with no derive
/// between the data and the bytes. No header line — this payload answers for the workspace
/// as a whole, not for one member.
#[must_use]
pub fn Encode_Payload(payload: &ScriptingPolicyPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    if let Some(language) = &payload.tooling_language
    {
        encoded.push_str(LANGUAGE_TAG);
        encoded.push('\t');
        encoded.push_str(language);
        encoded.push('\n');
    }

    for extension in &payload.forbidden_extensions
    {
        encoded.push_str(FORBIDDEN_TAG);
        encoded.push('\t');
        encoded.push_str(extension);
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, a line is not tagged `language` or
/// `forbidden`, a line has no value after its tag, or more than one `language` line is
/// present — one repository declares at most one tooling language.
pub fn Parse_Payload(bytes: &[u8]) -> Result<ScriptingPolicyPayload, Refusal>
{
    let text = Decode_Utf8(bytes)?;

    let mut payload = ScriptingPolicyPayload { tooling_language: None, forbidden_extensions: Vec::new() };

    for line in text.lines()
    {
        let (tag, value) = Tagged_Line(line)?;
        Apply_Line(&mut payload, Tag(tag), value, SourceLine(line))?;
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

/// `line` split into its tag and value, refused if it has no tag or the value is empty.
fn Tagged_Line(line: &str) -> Result<(&str, &str), Refusal>
{
    let Some((tag, value)) = line.split_once('\t')
    else
    {
        return Err(Refusal {
            reason: format!("line has no tag: {line:?}"),
        });
    };

    if value.is_empty()
    {
        return Err(Refusal {
            reason: format!("line {line:?} declares an empty value"),
        });
    }

    return Ok((tag, value));
}

/// One already-extracted tag, distinguished from the adjacent value and source line it
/// travels beside so a caller cannot transpose them.
struct Tag<'a>(&'a str);

/// The whole row line a tag/value pair was parsed from, carried only for its own error
/// message.
struct SourceLine<'a>(&'a str);

/// Applies one already-tagged line to `payload`.
fn Apply_Line(payload: &mut ScriptingPolicyPayload, tag: Tag<'_>, value: &str, line: SourceLine<'_>) -> Result<(), Refusal>
{
    match tag.0
    {
        LANGUAGE_TAG if payload.tooling_language.is_none() => payload.tooling_language = Some(value.to_owned()),
        LANGUAGE_TAG => {
            return Err(Refusal {
                reason: format!("a second `language` line is not allowed: {:?}", line.0),
            });
        }
        FORBIDDEN_TAG => payload.forbidden_extensions.push(value.to_owned()),
        _ => {
            return Err(Refusal {
                reason: format!("line has an unrecognized tag: {:?}", line.0),
            });
        }
    }

    return Ok(());
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

        assert_eq!(rendered, "language\trust\nforbidden\t.ps1\nforbidden\t.sh\n");
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Policy_That_Declares_Nothing()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");

        assert_eq!(decoded, ScriptingPolicyPayload::default());
    }

    #[test]
    fn Test_A_Payload_With_No_Language_Line_Should_Decode_To_An_Unconfigured_Language()
    {
        let decoded = Parse_Payload(b"forbidden\t.sh\n").expect("well-formed");

        assert_eq!(decoded.tooling_language, None);
        assert_eq!(decoded.forbidden_extensions, vec![".sh".to_owned()]);
    }

    #[test]
    fn Test_A_Line_With_No_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"rust\n").expect_err("a line with no tab must be refused");

        assert!(error.reason.contains("has no tag"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Line_With_An_Empty_Value_Should_Be_Refused()
    {
        let error = Parse_Payload(b"language\t\n").expect_err("an empty value must be refused");

        assert!(error.reason.contains("declares an empty value"), "{}", error.reason);
    }

    #[test]
    fn Test_An_Unrecognized_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"allowed\t.py\n").expect_err("an unrecognized tag must be refused");

        assert!(error.reason.contains("unrecognized tag"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Second_Language_Line_Should_Be_Refused()
    {
        let error =
            Parse_Payload(b"language\trust\nlanguage\tgo\n").expect_err("a second language line must be refused");

        assert!(error.reason.contains("second `language` line"), "{}", error.reason);
    }

    fn Sample() -> ScriptingPolicyPayload
    {
        return ScriptingPolicyPayload {
            tooling_language: Some("rust".to_owned()),
            forbidden_extensions: vec![".ps1".to_owned(), ".sh".to_owned()],
        };
    }
}

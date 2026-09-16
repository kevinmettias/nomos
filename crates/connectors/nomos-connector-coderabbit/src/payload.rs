//! The wire shape of a `nomos.review.finding.v1` payload, and its canonical encoding.

pub(crate) mod finding_payload;
pub(crate) mod payload_refusal;

use crate::ReviewFindingId;
use finding_payload::FindingPayload;
use payload_refusal::PayloadRefusal;

/// A payload's own field count, in canonical order -- a stand-in for the eight-field tuple
/// `Finding_Line`/`Parse_Payload` would otherwise pass by position.
const FINDING_FIELDS: usize = 8;

/// Encodes a payload as one tab-separated line -- the same diffable-by-a-person shape
/// this workspace's other hand-rolled payload codecs use, and for the same two reasons: no
/// derive stands between the data and the bytes, and a person can read a diff of it.
///
/// One finding, one line -- this crate's ceiling (`IncrementalGranularity::None`) never
/// produces more than one finding's own fields per fact, so there is no repeated-record
/// shape to encode.
///
/// `message` is written last, after seven fields that are themselves tab-free by
/// construction (a system name, a joined identity, a URL, two short vendor-reported words,
/// a file path, and a stringified line number) -- the same reason a free-text field needs
/// no escaping when it ends the line rather than starting it: nothing after it depends on
/// where it ends.
#[must_use]
pub fn Encode_Payload(payload: &FindingPayload) -> Vec<u8>
{
    let mut encoded = String::new();
    encoded.push_str(&payload.external_system);
    encoded.push('\t');
    encoded.push_str(payload.external_id.As_Str());
    encoded.push('\t');
    encoded.push_str(&payload.locator);
    encoded.push('\t');
    encoded.push_str(&Single_Line(&payload.category));
    encoded.push('\t');
    encoded.push_str(&Single_Line(&payload.severity));
    encoded.push('\t');
    encoded.push_str(&payload.path);
    encoded.push('\t');
    encoded.push_str(&payload.line);
    encoded.push('\t');
    encoded.push_str(&Single_Line(&payload.message));
    encoded.push('\n');

    return encoded.into_bytes();
}

/// A newline collapsed to a space, so a free-text field can never split the line in two.
fn Single_Line(text: &str) -> String
{
    return text.replace(['\n', '\r'], " ");
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`PayloadRefusal`] if the bytes are not valid UTF-8, are empty, hold more than one line,
/// or the line does not have exactly the eight fields this schema declares.
pub fn Parse_Payload(bytes: &[u8]) -> Result<FindingPayload, PayloadRefusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| PayloadRefusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut lines = text.lines();
    let Some(line) = lines.next()
    else
    {
        return Err(PayloadRefusal {
            reason: "a connector's fact is about exactly one finding; an empty payload names none".to_owned(),
        });
    };

    if lines.next().is_some()
    {
        return Err(PayloadRefusal {
            reason: "a connector's fact is about exactly one finding; this payload holds more than one line".to_owned(),
        });
    }

    return Finding_Line(line);
}

fn Finding_Line(line: &str) -> Result<FindingPayload, PayloadRefusal>
{
    let fields: Vec<&str> = line.splitn(FINDING_FIELDS, '\t').collect();
    let [external_system, external_id, locator, category, severity, path, line_number, message] = fields.as_slice()
    else
    {
        return Err(PayloadRefusal {
            reason: format!("finding line does not have exactly {FINDING_FIELDS} fields: {line:?}"),
        });
    };

    return Ok(FindingPayload {
        external_system: (*external_system).to_owned(),
        external_id: ReviewFindingId::New(*external_id),
        locator: (*locator).to_owned(),
        category: (*category).to_owned(),
        severity: (*severity).to_owned(),
        path: (*path).to_owned(),
        line: (*line_number).to_owned(),
        message: (*message).to_owned(),
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The real, public review comment this crate's own recorded fixture was captured from.
    const COMMENT_ID: u64 = 3_521_038_097;

    #[test]
    fn Test_A_Payload_Should_Round_Trip_Through_Its_Own_Encoding()
    {
        let payload = Sample();
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_A_Message_With_An_Embedded_Newline_Should_Not_Split_The_Line()
    {
        let mut payload = Sample();
        payload.message = "first line.\nsecond line.".to_owned();

        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("a normalized single-line message");

        assert_eq!(decoded.message, "first line. second line.");
    }

    #[test]
    fn Test_A_Message_Containing_A_Tab_Should_Still_Round_Trip()
    {
        let mut payload = Sample();
        payload.message = "found\ttab\tin\tmessage".to_owned();

        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("message is the last field and absorbs any tab");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Be_Refused()
    {
        assert!(Parse_Payload(&[]).is_err(), "a connector fact is about exactly one finding, never none");
    }

    #[test]
    fn Test_A_Second_Line_Should_Be_Refused()
    {
        let bytes = b"coderabbit\tid1\thttps://x\tcat\tsev\tpath\t1\tmsg\ncoderabbit\tid2\thttps://y\tcat\tsev\tpath\t2\tmsg2\n";
        assert!(Parse_Payload(bytes).is_err());
    }

    #[test]
    fn Test_A_Line_Missing_Fields_Should_Be_Refused()
    {
        let bytes = b"coderabbit\tid1\n";
        assert!(Parse_Payload(bytes).is_err());
    }

    /// The one payload every test above writes and reads back: the fields of this crate's
    /// own recorded fixture.
    fn Sample() -> FindingPayload
    {
        return FindingPayload {
            external_system: "coderabbit".to_owned(),
            external_id: ReviewFindingId::Of_Review_Comment("coderabbitai/rabbits-playground", COMMENT_ID),
            locator: "https://github.com/coderabbitai/rabbits-playground/pull/13#discussion_r3521038097".to_owned(),
            category: "🔒 Security & Privacy".to_owned(),
            severity: "🟡 Minor".to_owned(),
            path: "modules/security/main.tf".to_owned(),
            line: "43".to_owned(),
            message: "Consider defining an explicit KMS key policy (least privilege).".to_owned(),
        };
    }
}

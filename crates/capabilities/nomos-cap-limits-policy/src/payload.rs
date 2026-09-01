//! The wire shape of a `nomos.limits.policy.v1` payload, and its canonical encoding.

pub(crate) mod policy_row;
pub(crate) mod refusal;
pub(crate) mod scope;

use policy_row::PolicyRow;
use refusal::Refusal;
use scope::Scope;

/// A row's own fields, in canonical order.
const ROW_FIELDS: usize = 3;

/// A repository's whole declared limits policy — empty when it declares none, the same
/// "clean is a real answer, not an absence" shape `nomos_cap_naming_policy::
/// NamingPolicyPayload` already has for the sibling capability. An empty payload means
/// every rule that reads this capability keeps whatever hardcoded default it had before
/// this capability existed; it does not mean "no threshold applies."
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LimitsPolicyPayload
{
    pub rows: Vec<PolicyRow>,
}

/// Encodes a payload as tab-separated lines, the same shape `nomos-cap-naming-policy`,
/// `nomos-cap-lint`, `nomos-cap-dependency` and `nomos-cap-dependency-policy` all use:
/// diffable by a person, written in one place with no derive between the data and the
/// bytes.
///
/// No header line, the same reason `nomos_cap_naming_policy::Encode_Payload` has none:
/// this payload answers for the workspace as a whole, not for one member, so there is no
/// identifying value to write ahead of the rows themselves.
#[must_use]
pub fn Encode_Payload(payload: &LimitsPolicyPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for row in &payload.rows
    {
        encoded.push_str("row\t");
        encoded.push_str(row.scope.Label());
        encoded.push('\t');
        encoded.push_str(&row.key);
        encoded.push('\t');
        encoded.push_str(&row.value.to_string());
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, a line does not have exactly the three
/// fields this schema declares, or a value is not a non-negative decimal integer that
/// fits in a `u32`.
pub fn Parse_Payload(bytes: &[u8]) -> Result<LimitsPolicyPayload, Refusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut rows = Vec::new();
    for line in text.lines()
    {
        rows.push(Row_Line(line)?);
    }

    return Ok(LimitsPolicyPayload { rows });
}

fn Row_Line(line: &str) -> Result<PolicyRow, Refusal>
{
    let rest = Row_Body(line)?;
    let [scope, key, value] = Row_Fields(line, rest)?;
    let value = Parse_Value(line, value)?;

    return Ok(PolicyRow { scope: Scope::From_Label(scope), key: key.to_owned(), value });
}

/// `line` with its `"row\t"` prefix stripped, or a refusal naming the line that was not
/// one.
fn Row_Body(line: &str) -> Result<&str, Refusal>
{
    let Some(rest) = line.strip_prefix("row\t")
    else
    {
        return Err(Refusal {
            reason: format!("line is not a row: {line:?}"),
        });
    };

    return Ok(rest);
}

fn Row_Fields<'a>(line: &str, rest: &'a str) -> Result<[&'a str; ROW_FIELDS], Refusal>
{
    let fields: Vec<&str> = rest.splitn(ROW_FIELDS, '\t').collect();
    let [scope, key, value] = fields.as_slice()
    else
    {
        return Err(Refusal {
            reason: format!("row line does not have exactly {ROW_FIELDS} fields: {line:?}"),
        });
    };

    return Ok([*scope, *key, *value]);
}

fn Parse_Value(line: &str, text: &str) -> Result<u32, Refusal>
{
    return text.parse::<u32>().map_err(|error| Refusal {
        reason: format!("value {text:?} is not a non-negative integer in row line {line:?}: {error}"),
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

        assert_eq!(
            rendered,
            "row\t*\tfile-size-hard-lines\t1500\n\
             row\tgo\tfile-size-hard-lines\t1000\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Policy_That_Declares_Nothing()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");
        assert!(decoded.rows.is_empty());
    }

    #[test]
    fn Test_A_Malformed_Row_Line_Should_Be_Refused()
    {
        for bytes in Malformed_Row_Lines()
        {
            let error = Parse_Payload(bytes).expect_err("a row line without three fields must be refused");
            assert!(
                error.reason.contains("does not have exactly 3 fields"),
                "expected a field-count refusal for {bytes:?}, got: {}",
                error.reason
            );
        }
    }

    fn Malformed_Row_Lines() -> Vec<&'static [u8]>
    {
        return vec![b"row\t*\tonly-two-fields\n", b"row\tjust-one-field\n", b"row\t\n"];
    }

    #[test]
    fn Test_A_Non_Numeric_Value_Should_Be_Refused()
    {
        let error = Parse_Payload(b"row\t*\tfile-size-hard-lines\tmany\n").expect_err("a non-numeric value must be refused");
        assert!(error.reason.contains("is not a non-negative integer"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Negative_Value_Should_Be_Refused()
    {
        let error = Parse_Payload(b"row\t*\tfile-size-hard-lines\t-1\n").expect_err("a negative value must be refused");
        assert!(error.reason.contains("is not a non-negative integer"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Line_Not_Prefixed_Row_Should_Be_Refused()
    {
        let error = Parse_Payload(b"violation\ta\tb\tc\n").expect_err("a line without the row tag must be refused");
        assert!(error.reason.contains("is not a row"), "{}", error.reason);
    }

    fn Sample() -> LimitsPolicyPayload
    {
        return LimitsPolicyPayload {
            rows: vec![
                PolicyRow { scope: Scope::Repository, key: "file-size-hard-lines".to_owned(), value: 1500 },
                PolicyRow { scope: Scope::Language("go".to_owned()), key: "file-size-hard-lines".to_owned(), value: 1000 },
            ],
        };
    }
}

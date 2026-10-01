//! The payload: what one answer to this capability carries, and its encoding.
//!
//! One line per discarded value, `value<TAB>line<TAB>column<TAB>error|other<TAB>type`, in position
//! order. A type string is whatever `go/types` prints, which holds no tab or line break.

mod discarded_value;
mod discarded_values_payload;
mod payload_refusal;

pub use discarded_value::DiscardedValue;
pub use discarded_values_payload::DiscardedValuesPayload;
pub use payload_refusal::PayloadRefusal;

const VALUE: &str = "value";
const ERROR: &str = "error";
const OTHER: &str = "other";

/// `payload` as the bytes a fact carries, its values in position order whatever order they came in.
#[must_use]
pub fn Encode_Payload(payload: &DiscardedValuesPayload) -> Vec<u8>
{
    let mut values: Vec<&DiscardedValue> = payload.values.iter().collect();
    values.sort_by_key(|value| return (value.line, value.column));

    let text: String = values
        .into_iter()
        .map(|value| {
            let kind = if value.is_error { ERROR } else { OTHER };
            return format!("{VALUE}\t{}\t{}\t{kind}\t{}\n", value.line, value.column, value.type_name);
        })
        .collect();
    return text.into_bytes();
}

/// The payload `bytes` encode.
///
/// # Errors
///
/// [`PayloadRefusal`] when the bytes are not UTF-8, or a line is not one value in this schema's
/// shape.
pub fn Parse_Payload(bytes: &[u8]) -> Result<DiscardedValuesPayload, PayloadRefusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| return Refused(&format!("the payload is not UTF-8: {error}")))?;
    let values = text.lines().map(Value_Of).collect::<Result<Vec<DiscardedValue>, PayloadRefusal>>()?;
    return Ok(DiscardedValuesPayload { values });
}

fn Value_Of(line: &str) -> Result<DiscardedValue, PayloadRefusal>
{
    let mut fields = line.splitn(5, '\t');
    let (Some(VALUE), Some(position_line), Some(column), Some(kind), Some(type_name)) = (fields.next(), fields.next(), fields.next(), fields.next(), fields.next())
    else
    {
        return Err(Refused(&format!("a line that is not a value: {line}")));
    };
    let is_error = match kind
    {
        ERROR => true,
        OTHER => false,
        other => return Err(Refused(&format!("a value kind this schema does not have: {other}"))),
    };

    return Ok(DiscardedValue {
        line: position_line.parse().map_err(|_| return Refused(&format!("a line number that is not one: {position_line}")))?,
        column: column.parse().map_err(|_| return Refused(&format!("a column that is not one: {column}")))?,
        is_error,
        type_name: type_name.to_owned(),
    });
}

fn Refused(reason: &str) -> PayloadRefusal
{
    return PayloadRefusal { reason: reason.to_owned() };
}

#[cfg(test)]
mod tests;

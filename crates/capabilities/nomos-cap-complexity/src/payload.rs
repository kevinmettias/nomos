//! The wire shape of a `nomos.metric.complexity.v1` payload, and its canonical encoding.

pub(crate) mod aggregation;
pub(crate) mod complexity_payload;
pub(crate) mod directionality;
pub(crate) mod function_complexity;
pub(crate) mod metric_descriptor;
pub(crate) mod payload_refusal;

mod descriptor_lines;

use aggregation::Aggregation;
use complexity_payload::ComplexityPayload;
use descriptor_lines::{DescriptorLines, Fields_Of};
use directionality::Directionality;
use function_complexity::FunctionComplexity;
use metric_descriptor::MetricDescriptor;
use payload_refusal::PayloadRefusal;

/// The descriptor every `nomos.metric.complexity.v1` payload carries: what the number counts, and
/// every term MET-006 requires a reader to have before reading it.
#[doc = include_str!("../docs/api/payload.md")]
#[must_use]
pub fn Complexity_Descriptor() -> MetricDescriptor
{
    return MetricDescriptor {
        unit: "independent paths: one more than the decision points in a function body".to_owned(),
        subject_kinds: "function: a free function, method or associated function with a body; a closure is \
                        counted into the function that contains it, and a function nested inside another is \
                        its own subject"
            .to_owned(),
        aggregation: Aggregation::NotAggregable,
        weighting: "none: every function is one subject whatever its size".to_owned(),
        normalization: "none: an absolute count, not divided by lines, statements or anything else".to_owned(),
        missing_data: "a function the payload does not list was not measured, and is never read as a complexity \
                       of one; a file answering with no function defines none"
            .to_owned(),
        directionality: Directionality::HigherIsWorse,
        baseline: "none: a value is read against a declared limit, not against an earlier snapshot".to_owned(),
        uncertainty: "exact for the syntax as written; a macro invocation is not expanded, so a branch a macro \
                      introduces is not counted and the value can be low but never high"
            .to_owned(),
        snapshot_comparability: "comparable across snapshots while the schema is unchanged; a renamed or moved \
                                 function is a different subject"
            .to_owned(),
    };
}

/// Encodes a payload as tab-separated lines, the shape every capability crate in this workspace
/// uses: one `descriptor` line per MET-006 property, then one `function` line per function.
#[must_use]
pub fn Encode_Payload(payload: &ComplexityPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for (field, value) in Fields_Of(&payload.descriptor)
    {
        encoded.push_str("descriptor\t");
        encoded.push_str(field);
        encoded.push('\t');
        encoded.push_str(&value);
        encoded.push('\n');
    }

    for function in &payload.functions
    {
        encoded.push_str("function\t");
        encoded.push_str(&function.function);
        encoded.push('\t');
        encoded.push_str(&function.line.to_string());
        encoded.push('\t');
        encoded.push_str(&function.complexity.to_string());
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// A [`PayloadRefusal`] naming the first line that is not this schema, or the first descriptor
/// property no line declares.
#[doc = include_str!("../docs/api/payload.md")]
pub fn Parse_Payload(bytes: &[u8]) -> Result<ComplexityPayload, PayloadRefusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| return Refusal(&format!("not UTF-8: {error}")))?;

    let mut descriptor = DescriptorLines::default();
    let mut functions = Vec::new();
    for line in text.lines()
    {
        if let Some(rest) = line.strip_prefix("descriptor\t")
        {
            descriptor.Read_Line(rest, line)?;
        }
        else if let Some(rest) = line.strip_prefix("function\t")
        {
            functions.push(Function_Line(rest, line)?);
        }
        else
        {
            return Err(Refusal(&format!("line is neither a descriptor nor a function: {line:?}")));
        }
    }

    return Ok(ComplexityPayload { descriptor: descriptor.Into_Descriptor()?, functions });
}

/// A `function` line's three fields after its tag.
fn Function_Line(rest: &str, line: &str) -> Result<FunctionComplexity, PayloadRefusal>
{
    let fields: Vec<&str> = rest.split('\t').collect();
    let [function, number, complexity] = fields.as_slice()
    else
    {
        return Err(Refusal(&format!("function line does not have exactly three fields: {line:?}")));
    };

    return Ok(FunctionComplexity {
        function: (*function).to_owned(),
        line: Count(number, "line", line)?,
        complexity: Count(complexity, "complexity", line)?,
    });
}

/// One of a function line's two counts.
fn Count(field: &str, name: &str, line: &str) -> Result<usize, PayloadRefusal>
{
    return field.parse::<usize>().map_err(|_| return Refusal(&format!("function line's {name} is not a count: {line:?}")));
}

fn Refusal(reason: &str) -> PayloadRefusal
{
    return PayloadRefusal { reason: reason.to_owned() };
}

#[cfg(test)]
mod tests;

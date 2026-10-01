//! The wire shape of a `nomos.csharp.conditional_compilation.v1` payload, and its canonical
//! encoding.

pub(crate) mod branch;
pub(crate) mod branch_state;
pub(crate) mod build_selection;
pub(crate) mod conditional_payload;
pub(crate) mod conditional_region;
pub(crate) mod definition_effect;
pub(crate) mod file_definition;
pub(crate) mod payload_refusal;

use branch::Branch;
use branch_state::BranchState;
use build_selection::BuildSelection;
use conditional_payload::ConditionalPayload;
use conditional_region::ConditionalRegion;
use definition_effect::DefinitionEffect;
use file_definition::FileDefinition;
use payload_refusal::PayloadRefusal;

/// Encodes a payload as tab-separated lines: the one `selection` line, then a `symbol` line per
/// defined symbol, a `define` or `undef` line per file definition, and a `region` line per branch.
///
/// The condition is the last field of a region line so the one free-text field needs no escaping
/// beyond the whitespace collapse [`ConditionalRegion::condition`] already carries.
#[must_use]
pub fn Encode_Payload(payload: &ConditionalPayload) -> Vec<u8>
{
    let selection = &payload.selection;
    let mut lines = vec![format!("selection\t{}\t{}\t{}", selection.project, selection.configuration, selection.target_framework)];
    lines.extend(payload.symbols.iter().map(|symbol| return format!("symbol\t{symbol}")));
    lines.extend(payload.definitions.iter().map(|definition| return format!("{}\t{}\t{}", definition.effect.Label(), definition.line, definition.symbol)));
    lines.extend(payload.regions.iter().map(Region_Line));

    let mut encoded = lines.join("\n");
    encoded.push('\n');
    return encoded.into_bytes();
}

fn Region_Line(region: &ConditionalRegion) -> String
{
    return format!(
        "region\t{}\t{}\t{}\t{}\t{}",
        region.branch.Label(),
        region.line,
        region.end_line,
        region.state.Label(),
        region.condition
    );
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// A [`PayloadRefusal`] naming the first line that is not this schema, or saying the one
/// `selection` line is missing or repeated.
#[doc = include_str!("../docs/api/payload.md")]
pub fn Parse_Payload(bytes: &[u8]) -> Result<ConditionalPayload, PayloadRefusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| return Refusal(&format!("not UTF-8: {error}")))?;
    let mut selection: Option<BuildSelection> = None;
    let mut payload = ConditionalPayload { selection: Placeholder_Selection(), symbols: Vec::new(), definitions: Vec::new(), regions: Vec::new() };

    for line in text.lines()
    {
        let (tag, rest) = line.split_once('\t').unwrap_or((line, ""));
        match tag
        {
            "selection" => Set_Selection(&mut selection, rest, line)?,
            "symbol" => payload.symbols.push(Symbol_Line(rest, line)?),
            "define" | "undef" => payload.definitions.push(Definition_Line(tag, rest, line)?),
            "region" => payload.regions.push(Parse_Region_Line(rest, line)?),
            _ => return Err(Refusal(&format!("line is none of this schema's four kinds: {line:?}"))),
        }
    }

    payload.selection = selection.ok_or_else(|| return Refusal("no selection line names the build this answer is about"))?;
    return Ok(payload);
}

/// Stands in for the selection until the one `selection` line is read; never returned, because
/// [`Parse_Payload`] refuses a payload that has none.
fn Placeholder_Selection() -> BuildSelection
{
    return BuildSelection { project: String::new(), configuration: String::new(), target_framework: String::new() };
}

fn Set_Selection(selection: &mut Option<BuildSelection>, rest: &str, line: &str) -> Result<(), PayloadRefusal>
{
    if selection.is_some()
    {
        return Err(Refusal(&format!("a second selection line: {line:?}")));
    }

    let fields: Vec<&str> = rest.split('\t').collect();
    let [project, configuration, target_framework] = fields.as_slice()
    else
    {
        return Err(Refusal(&format!("selection line does not have exactly three fields: {line:?}")));
    };

    *selection = Some(BuildSelection {
        project: (*project).to_owned(),
        configuration: (*configuration).to_owned(),
        target_framework: (*target_framework).to_owned(),
    });
    return Ok(());
}

fn Symbol_Line(rest: &str, line: &str) -> Result<String, PayloadRefusal>
{
    if rest.is_empty() || rest.contains('\t')
    {
        return Err(Refusal(&format!("symbol line does not name exactly one symbol: {line:?}")));
    }

    return Ok(rest.to_owned());
}

fn Definition_Line(tag: &str, rest: &str, line: &str) -> Result<FileDefinition, PayloadRefusal>
{
    let effect = DefinitionEffect::From_Label(tag).ok_or_else(|| return Refusal(&format!("not a definition: {line:?}")))?;
    let Some((number, symbol)) = rest.split_once('\t')
    else
    {
        return Err(Refusal(&format!("definition line does not have a line and a symbol: {line:?}")));
    };

    return Ok(FileDefinition { line: Count(number, line)?, symbol: symbol.to_owned(), effect });
}

fn Parse_Region_Line(rest: &str, line: &str) -> Result<ConditionalRegion, PayloadRefusal>
{
    let fields: Vec<&str> = rest.split('\t').collect();
    let [branch, start, end, state, condition] = fields.as_slice()
    else
    {
        return Err(Refusal(&format!("region line does not have exactly five fields: {line:?}")));
    };

    return Ok(ConditionalRegion {
        branch: Branch::From_Label(branch).ok_or_else(|| return Refusal(&format!("unrecognized branch: {line:?}")))?,
        line: Count(start, line)?,
        end_line: Count(end, line)?,
        condition: (*condition).to_owned(),
        state: BranchState::From_Label(state).ok_or_else(|| return Refusal(&format!("unrecognized branch state: {line:?}")))?,
    });
}

fn Count(field: &str, line: &str) -> Result<usize, PayloadRefusal>
{
    return field.parse::<usize>().map_err(|_| return Refusal(&format!("a line number that is not a count: {line:?}")));
}

fn Refusal(reason: &str) -> PayloadRefusal
{
    return PayloadRefusal { reason: reason.to_owned() };
}

#[cfg(test)]
mod tests;

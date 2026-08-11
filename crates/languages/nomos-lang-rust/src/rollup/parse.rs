//! Reading a module index back.

use super::{ModuleIndex, MemberReading, READ, Outcome, APPROXIMATE, UNREACHABLE, IndexEntry, SubjectId, Digest128};
use super::encode::{ITEM_FIELDS, MEMBER_FIELDS, MODULE_FIELDS};

/// Reads an index payload back.
///
/// The reader for this schema is here, and it is the only one. A consumer writing its own
/// is not proving anything — it is re-deciding what a well-formed payload is, which is the
/// defect `P10-SYNTAX-SCHEMA` found in the schema next door with three answers to that
/// question.
///
/// # Errors
///
/// Returns what was refused and where. A decoder that fell back to an empty index would
/// report every unreadable rollup as a module that declares nothing, which is
/// indistinguishable from a module that genuinely does.
pub fn Parse_Index(payload: &[u8]) -> Result<ModuleIndex, String>
{
    let text = core::str::from_utf8(payload).map_err(|error| {
        return format!("the payload is not UTF-8, so it is not this schema: {error}");
    })?;

    let mut lines = text.lines().enumerate();
    let mut index = Opened(&mut lines)?;

    for (offset, line) in lines
    {
        Read_Record(&mut index, line, offset.saturating_add(1))?;
    }

    return Ok(index);
}

/// The `module` record, and the empty index it opens.
///
/// It appears exactly once and first. That is what makes an index over a module with no
/// members a payload rather than the empty byte string — a module nothing was read for and
/// a module that declares nothing are two answers and must not share an encoding.
pub(super) fn Opened(lines: &mut core::iter::Enumerate<core::str::Lines<'_>>) -> Result<ModuleIndex, String>
{
    let Some((_, header)) = lines.next()
    else
    {
        return Err("the payload is empty, which is not a module that declares nothing — \
                    that is a `module` record and no members"
            .to_owned());
    };

    let fields: Vec<&str> = header.split('\t').collect();
    if fields.first().copied() != Some("module")
    {
        return Err(format!("the first record is `{header}` and not a `module` record"));
    }
    Expect_Fields("module", &fields, MODULE_FIELDS, 1)?;

    return Ok(ModuleIndex {
        module: Subject_From(fields.get(1).copied().unwrap_or_default(), 1)?,
        members: Vec::new(),
        items: Vec::new(),
    });
}

/// One record after the header.
///
/// An unrecognised tag is refused rather than skipped, for the reason the schema gives: a
/// record this build does not know is most likely a newer schema, and passing over it reads
/// the payload down to the part that has not changed.
pub(super) fn Read_Record(index: &mut ModuleIndex, line: &str, number: usize) -> Result<(), String>
{
    let fields: Vec<&str> = line.split('\t').collect();

    // `split` yields at least one element for every input, so `first` is only `None` for an
    // iterator that is already exhausted, which this one is not.
    match fields.first().copied().unwrap_or_default()
    {
        "member" =>
        {
            let member = Member_Record(&fields, number)?;
            index.members.push(member);
        }
        "item" =>
        {
            let item = Item_Record(&fields, number)?;
            index.items.push(item);
        }
        tag => return Err(Unreadable_Record(tag, number)),
    }

    return Ok(());
}

/// A record this build will not read.
///
/// A second `module` record is refused because the first one is the index's identity and a
/// payload carrying two does not say which. Anything else is refused because a tag this
/// build does not know is most likely a newer schema, and passing over it would read the
/// payload down to the part that has not changed.
pub(super) fn Unreadable_Record(tag: &str, number: usize) -> String
{
    if tag == "module"
    {
        return format!("line {number} is a second `module` record");
    }

    return format!("line {number} has record tag `{tag}`, which this build does not understand");
}

pub(super) fn Member_Record(fields: &[&str], line: usize) -> Result<MemberReading, String>
{
    Expect_Fields("member", fields, MEMBER_FIELDS, line)?;

    let outcome = match fields.get(2).copied().unwrap_or_default()
    {
        READ => Outcome::Read,
        APPROXIMATE => Outcome::Approximate,
        UNREACHABLE => Outcome::Unreachable,
        other => return Err(format!("`{other}` on line {line} is not an outcome")),
    };

    return Ok(MemberReading {
        subject: Subject_From(fields.get(1).copied().unwrap_or_default(), line)?,
        outcome,
    });
}

pub(super) fn Item_Record(fields: &[&str], line: usize) -> Result<IndexEntry, String>
{
    Expect_Fields("item", fields, ITEM_FIELDS, line)?;

    let ordinal = fields.get(2).copied().unwrap_or_default();
    let ordinal: u32 = ordinal
        .parse()
        .map_err(|cause| return format!("`{ordinal}` on line {line} is not an ordinal: {cause}"))?;

    return Ok(IndexEntry {
        member: Subject_From(fields.get(1).copied().unwrap_or_default(), line)?,
        ordinal,
        kind: fields.get(3).copied().unwrap_or_default().to_owned(),
        visibility: fields.get(4).copied().unwrap_or_default().to_owned(),
        qualified_name: fields.get(5).copied().unwrap_or_default().to_owned(),
    });
}

/// Refuses a record whose field count is not the one the grammar states.
///
/// # Why a longer record is refused rather than truncated
///
/// The same argument the unknown record tag gets, one grain finer, and it took
/// `P10-INDEX-FIELD-STRICTNESS` to notice that this reader made it in one place and not the
/// other. A record carrying more fields than this build knows about is most likely a payload
/// from a newer schema, and that is exactly the case where reading the prefix and discarding
/// the rest is worst: the discarded field is where the new information is, and the caller is
/// handed a clean decode of a payload it only partly understood.
///
/// A shorter record is refused for the older reason — a missing field defaulting to empty is
/// an absence invented by the reader rather than one the writer wrote.
///
/// This is deliberately the same shape as `nomos-cap-syntax`'s `Expect_Fields`, which had it
/// right from the start. The two schemas share no code and should not: what they share is a
/// rule about what a record is, and agreeing by construction would remove the disagreement
/// that would otherwise be visible.
pub(super) fn Expect_Fields(tag: &str, fields: &[&str], expected: usize, line: usize) -> Result<(), String>
{
    if fields.len() == expected
    {
        return Ok(());
    }

    return Err(format!(
        "the `{tag}` record on line {line} has {} field(s) and this schema's has {expected}. \
         A longer record is most likely a newer schema, and reading its first {expected} \
         fields would discard exactly the part that is new",
        fields.len()
    ));
}

/// A subject read back out of the hexadecimal the encoder wrote.
pub(super) fn Subject_From(hexadecimal: &str, line: usize) -> Result<SubjectId, String>
{
    if hexadecimal.len() != Digest128::HEX_LENGTH
    {
        return Err(format!(
            "`{hexadecimal}` on line {line} is {} characters and a subject is {}",
            hexadecimal.len(),
            Digest128::HEX_LENGTH
        ));
    }

    let mut bytes = [0_u8; Digest128::BYTE_LENGTH];
    for (slot, pair) in bytes.iter_mut().zip(Pairs(hexadecimal))
    {
        *slot = u8::from_str_radix(pair, 16)
            .map_err(|cause| return format!("`{pair}` on line {line} is not hexadecimal: {cause}"))?;
    }

    return Ok(SubjectId::From_Digest(Digest128::From_Bytes(bytes)));
}

/// The two-character slices of an even-length ASCII hexadecimal string.
pub(super) fn Pairs(hexadecimal: &str) -> impl Iterator<Item = &str>
{
    return (0..hexadecimal.len())
        .step_by(2)
        .filter_map(|start| return hexadecimal.get(start..start.saturating_add(2)));
}

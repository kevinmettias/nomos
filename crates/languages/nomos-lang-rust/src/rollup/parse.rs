//! Reading a module index back.

use super::{Index, MemberReading, READ, Outcome, APPROXIMATE, UNREACHABLE, IndexEntry, SubjectId, Digest128};
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
pub fn Parse_Index(payload: &[u8]) -> Result<Index, String>
{
    let text = core::str::from_utf8(payload).map_err(|error| {
        return format!("the payload is not UTF-8, so it is not this schema: {error}");
    })?;

    let mut lines = text.lines().enumerate();
    let mut index = Opened_Module_Index(&mut lines)?;

    for (offset, line) in lines
    {
        Read_Record(&mut index, line, offset.saturating_add(1))?;
    }

    return Ok(index);
}

/// A record read left to right, so a field's place is the order it is asked for rather than a
/// number typed beside a schema that lives in another file.
///
/// The number and the encoder drift apart in silence. A field inserted into `Encode_Entry`
/// renumbers every field after it, and nothing in the language ties `fields.get(4)` to the
/// fifth thing that encoder writes — the reader keeps compiling and starts filling the wrong
/// fields. Asking in order leaves the encoder as the only place the order is stated.
struct Fields<'record, 'text>
{
    fields: &'record [&'text str],
    next: usize,
}

impl<'record, 'text> Fields<'record, 'text>
{
    /// The fields after the tag, which every record spends its first column on.
    fn After_The_Tag(fields: &'record [&'text str]) -> Self
    {
        return Self { fields, next: 1 };
    }

    /// The next field the schema names, empty where the record is short.
    ///
    /// `Expect_Fields` has already refused a record of the wrong length by the time any
    /// reader asks, so the empty string here is unreachable rather than a quiet default.
    fn Next(&mut self) -> &'text str
    {
        let at = self.next;

        self.next = at.saturating_add(1);
        return self.fields.get(at).copied().unwrap_or_default();
    }
}

/// The `module` record, and the empty index it opens.
///
/// It appears exactly once and first. That is what makes an index over a module with no
/// members a payload rather than the empty byte string — a module nothing was read for and
/// a module that declares nothing are two answers and must not share an encoding.
pub(super) fn Opened_Module_Index(lines: &mut core::iter::Enumerate<core::str::Lines<'_>>) -> Result<Index, String>
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

    let mut record = Fields::After_The_Tag(&fields);

    return Ok(Index {
        module: Subject_From(record.Next(), 1)?,
        members: Vec::new(),
        items: Vec::new(),
    });
}

/// One record after the header.
///
/// An unrecognised tag is refused rather than skipped, for the reason the schema gives: a
/// record this build does not know is most likely a newer schema, and passing over it reads
/// the payload down to the part that has not changed.
pub(super) fn Read_Record(index: &mut Index, line: &str, number: usize) -> Result<(), String>
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

    let mut record = Fields::After_The_Tag(fields);
    let subject = record.Next();
    let outcome = match record.Next()
    {
        READ => Outcome::Read,
        APPROXIMATE => Outcome::Approximate,
        UNREACHABLE => Outcome::Unreachable,
        other => return Err(format!("`{other}` on line {line} is not an outcome")),
    };

    return Ok(MemberReading {
        subject: Subject_From(subject, line)?,
        outcome,
    });
}

pub(super) fn Item_Record(fields: &[&str], line: usize) -> Result<IndexEntry, String>
{
    Expect_Fields("item", fields, ITEM_FIELDS, line)?;

    let mut record = Fields::After_The_Tag(fields);
    let member = record.Next();
    let ordinal = record.Next();
    let ordinal: u32 = ordinal
        .parse()
        .map_err(|cause| return format!("`{ordinal}` on line {line} is not an ordinal: {cause}"))?;

    return Ok(IndexEntry {
        member: Subject_From(member, line)?,
        ordinal,
        kind: record.Next().to_owned(),
        visibility: record.Next().to_owned(),
        qualified_name: record.Next().to_owned(),
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
    for (slot, pair) in bytes.iter_mut().zip(Hexadecimal_Byte_Pairs(hexadecimal))
    {
        *slot = u8::from_str_radix(pair, HEXADECIMAL)
            .map_err(|cause| return format!("`{pair}` on line {line} is not hexadecimal: {cause}"))?;
    }

    return Ok(SubjectId::From_Digest(Digest128::From_Bytes(bytes)));
}

/// The base the encoder wrote a digest in.
const HEXADECIMAL: u32 = 16;

/// How many characters of that encoding one byte occupies.
const PER_BYTE: usize = 2;

/// The two-character slices of an even-length ASCII hexadecimal string.
pub(super) fn Hexadecimal_Byte_Pairs(hexadecimal: &str) -> impl Iterator<Item = &str>
{
    return (0..hexadecimal.len())
        .step_by(PER_BYTE)
        .filter_map(|start| return hexadecimal.get(start..start.saturating_add(PER_BYTE)));
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The line the unknown-tag record below is read from.
    const UNKNOWN_TAG_LINE: usize = 5;
    /// The line the second module record in that test is read from.
    const SECOND_MODULE_LINE: usize = 3;
    /// The line each of the two refused fixture records below is read from.
    const FIXTURE_RECORD_LINE: usize = 2;
    /// The field count a module record carries: its own tag beside one subject.
    const MODULE_FIELD_COUNT: usize = 2;
    /// The fill byte of the digest the round-trip subject is built from.
    const ROUND_TRIP_SUBJECT_BYTE: u8 = 9;
    /// The fill byte of the digest `A_Subject_Hex` renders, which no other fixture seeds.
    const A_SUBJECT_BYTE: u8 = 7;

    #[test]
    fn Test_Parse_Index_Should_Read_A_Module_Record_With_No_Members()
    {
        let payload = format!("module\t{}\n", A_Subject_Hex());

        let index = Parse_Index(payload.as_bytes()).expect("a bare module record parses");

        assert!(index.members.is_empty());
        assert!(index.items.is_empty());
    }

    #[test]
    fn Test_Opened_Module_Index_Should_Refuse_A_Payload_With_No_Leading_Module_Record()
    {
        for (payload, expected_fragment) in Malformed_Header_Cases()
        {
            let mut lines = payload.lines().enumerate();

            let error = Opened_Module_Index(&mut lines).expect_err("no `module` record opens this payload");

            assert!(error.contains(expected_fragment), "{error}");
        }
    }

    /// (payload text, a fragment its refusal reason must contain) for a payload
    /// `Opened_Module_Index` must refuse before any member or item is ever read — a second
    /// malformed header beside these would extend the table rather than duplicate the test.
    fn Malformed_Header_Cases() -> Vec<(&'static str, &'static str)>
    {
        return vec![
            ("", "empty"),
            ("member\t00000000000000000000000000000000\tread", "module"),
        ];
    }

    #[test]
    fn Test_Read_Record_Should_Refuse_An_Unrecognised_Tag()
    {
        let mut index = Index {
            module: SubjectId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
            members: Vec::new(),
            items: Vec::new(),
        };

        let error = Read_Record(&mut index, "surface\t1", UNKNOWN_TAG_LINE).expect_err("an unknown tag must be refused");

        assert!(error.contains('5'), "{error}");
    }

    #[test]
    fn Test_Unreadable_Record_Should_Name_A_Second_Module_Record_Specifically()
    {
        assert!(Unreadable_Record("module", SECOND_MODULE_LINE).contains("second"));
        assert!(Unreadable_Record("surface", SECOND_MODULE_LINE).contains("surface"));
    }

    #[test]
    fn Test_Member_Record_Should_Refuse_An_Outcome_This_Build_Does_Not_Know()
    {
        let hex = A_Subject_Hex();
        let fields: Vec<&str> = vec!["member", hex.as_str(), "maybe"];

        let error = Member_Record(&fields, FIXTURE_RECORD_LINE).expect_err("`maybe` is not a known outcome");

        assert!(error.contains("maybe"), "{error}");
        assert!(error.contains("outcome"), "{error}");
    }

    #[test]
    fn Test_Item_Record_Should_Refuse_An_Ordinal_That_Does_Not_Parse()
    {
        let hex = A_Subject_Hex();
        let fields: Vec<&str> = vec!["item", hex.as_str(), "not-a-number", "Function", "Public", "One"];

        let error = Item_Record(&fields, FIXTURE_RECORD_LINE).expect_err("`not-a-number` does not parse as an ordinal");

        assert!(error.contains("not-a-number"), "{error}");
        assert!(error.contains("ordinal"), "{error}");
    }

    #[test]
    fn Test_Expect_Fields_Should_Refuse_A_Field_Count_Other_Than_Expected()
    {
        assert!(Expect_Fields("module", &["module", "a"], MODULE_FIELD_COUNT, 1).is_ok());
        assert!(Expect_Fields("module", &["module", "a", "extra"], MODULE_FIELD_COUNT, 1).is_err());
    }

    #[test]
    fn Test_Subject_From_Should_Refuse_A_String_Of_The_Wrong_Length()
    {
        let error = Subject_From("short", 1).expect_err("`short` is not 32 hexadecimal characters");

        assert!(error.contains("short"), "{error}");
        assert!(error.contains("characters"), "{error}");
    }

    #[test]
    fn Test_Subject_From_Should_Round_Trip_A_Digest_Written_As_Hexadecimal()
    {
        let subject = SubjectId::From_Digest(Digest128::From_Bytes([ROUND_TRIP_SUBJECT_BYTE; Digest128::BYTE_LENGTH]));
        let hex = subject.Digest().to_string();

        assert_eq!(Subject_From(&hex, 1), Ok(subject));
    }

    #[test]
    fn Test_Hexadecimal_Byte_Pairs_Should_Split_Into_Two_Character_Chunks()
    {
        let pairs: Vec<&str> = Hexadecimal_Byte_Pairs("0a1b2c").collect();

        assert_eq!(pairs, vec!["0a", "1b", "2c"]);
    }

    fn A_Subject_Hex() -> String
    {
        return SubjectId::From_Digest(Digest128::From_Bytes([A_SUBJECT_BYTE; Digest128::BYTE_LENGTH])).Digest().to_string();
    }
}

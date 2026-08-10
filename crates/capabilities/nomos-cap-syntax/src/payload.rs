//! The grammar of `nomos.syntax.items.v1`, and the reader the tree agrees on.
//!
//! Until this module existed, the schema was a [`nomos_contracts::SchemaId`] string and
//! nothing else. Two providers authored the bytes independently and three consumers read
//! them back, each with its own idea of what a well-formed payload is — so the *shape* of
//! an answer that two parties are bound by was the part of the agreement with no home,
//! which is the same objection that moved the capability, the ceiling and the version here.
//!
//! # The grammar
//!
//! A payload is UTF-8 text. Fields within a record are separated by a single tab, records
//! are terminated by `\n`, and never by `\r\n` — a digest that depends on the line ending
//! of the machine that produced it is not a content address.
//!
//! ```text
//! payload  := header item*
//! header   := "unexpanded" TAB u32 LF
//! item     := "item" TAB ordinal TAB kind TAB visibility TAB qualified-name LF
//! ordinal  := u32
//! ```
//!
//! **The header is the first record and appears exactly once.** It carries a lower bound on
//! the places the provider saw the parse tree end in unexpanded tokens. A provider that
//! cannot observe macro invocations writes `0`, and that is the format saying the same
//! thing in every payload rather than a claim that there were none.
//!
//! **`item` records are in source order** and `ordinal` is the item's position in that
//! order, starting at zero. A file that declares nothing is a header and no items — which
//! is a real answer, and distinguishable from a payload that could not be read only because
//! the empty byte string is refused rather than decoded.
//!
//! **`kind` and `visibility` are labels, and the schema does not enumerate them.** A
//! provider that can distinguish fewer forms than another writes fewer labels; a kind it
//! cannot distinguish is a kind it must not claim. Fixing the vocabulary here would make
//! the weaker provider unable to answer honestly, and the ceiling already exists to bound
//! what an answer may claim.
//!
//! Three labels are reserved, because they are the ones consumers branch on:
//! [`FUNCTION`], [`PUBLIC`] and [`NOT_APPLICABLE`]. A provider that uses any of those three
//! spellings for anything else is not writing this schema.
//!
//! **`qualified-name` is the name as written, qualified by syntactic nesting** — a function
//! declared in `mod tests` arrives as `tests::Name`, and one declared in an `impl` block
//! arrives as `Type::Name`. It is not a resolved path: no provider of this capability
//! resolves names, and a `::` in it is nesting rather than a module route.
//!
//! # What this schema declines to state
//!
//! **Whether a `Function` record is a definition or a signature.**
//!
//! It looks as though it could. `nomos-lang-rust` writes [`NOT_APPLICABLE`] for a trait
//! member, because a trait method declares no visibility of its own and recording it as
//! private would be inventing a declaration the source does not contain. So under that
//! provider, a `Function` marked [`NOT_APPLICABLE`] *is* a trait-method signature.
//!
//! It is not a property of the schema, because it is not a property of every conforming
//! answer. `nomos-lang-rust-scan` has no [`NOT_APPLICABLE`] value at all — a line reader
//! cannot see the enclosing trait — and writes `Private` for the same source construct.
//! Both payloads conform. So the presence of the mark is evidence and its **absence is
//! not**, and a consumer that reads absence as "this is a definition" is reading a weaker
//! provider's blindness as an observation.
//!
//! The consequence is stated rather than left to be discovered: a consumer needing the
//! distinction must obtain it from the *guarantee* it required of the answer, not from the
//! bytes. That is what `nomos-rules` does — `OD-RULES-001` sets a floor the approximate
//! provider does not meet, so a payload that could carry the blind spelling never reaches
//! the filter. Carrying the item's form in the payload is a v2 question and `P10-SYNTAX-V2`
//! holds it.
//!
//! # One reader, two writers
//!
//! The reader is here and canonical. Every consumer uses it, because a consumer writing its
//! own is not proving anything — it is re-deciding what a well-formed payload is, and three
//! answers to that question is how a payload comes to decode differently under one
//! capability.
//!
//! The writers stay where they are, deliberately. What makes two providers interchangeable
//! is that both produce bytes a third party can read; a shared encoder would make that true
//! by construction and prove nothing. What the duplication used to cost is that nothing
//! checked the agreement — so each provider now decodes its own output through this reader
//! in its own tests, which is the check the duplication was missing rather than the
//! duplication removed.

use core::fmt::Write as _;

/// The `kind` label every provider writes for a function form.
pub const FUNCTION: &str = "Function";

/// The `visibility` label for an item that declares itself public.
pub const PUBLIC: &str = "Public";

/// The `visibility` label for an item form that declares no visibility.
///
/// Reserved with a stated meaning: the provider observed a form that has no visibility to
/// declare — a trait member, an `impl` block, a macro definition. Read the module's own
/// caveat before treating its absence as evidence of anything.
pub const NOT_APPLICABLE: &str = "NotApplicable";

/// Fields in an `item` record, tag included.
const ITEM_FIELDS: usize = 5;

/// Fields in the `unexpanded` header, tag included.
const HEADER_FIELDS: usize = 2;

/// A decoded `nomos.syntax.items.v1` payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxPayload
{
    /// The lower bound the provider offered on unexpanded regions.
    pub unexpanded: u32,
    /// The items the file declares, in source order.
    pub items: Vec<PayloadItem>,
}

/// One `item` record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadItem
{
    /// Position in source order, from zero.
    pub ordinal: u32,
    /// The form the provider recognised. See the module doc: the vocabulary is open.
    pub kind: String,
    /// The visibility the item declares, as the provider observed it.
    pub visibility: String,
    /// The name as written, qualified by syntactic nesting.
    pub qualified_name: String,
}

impl PayloadItem
{
    /// The item's own name, without the nesting it is qualified by.
    ///
    /// The last `::` segment. Matching a qualified form against anything else resolves
    /// nothing, because almost every declaration worth naming lives inside something.
    #[must_use]
    pub fn Own_Name(&self) -> &str
    {
        return self
            .qualified_name
            .rsplit("::")
            .next()
            .unwrap_or(&self.qualified_name);
    }

    /// Whether the item declares itself public.
    #[must_use]
    pub fn Is_Public(&self) -> bool
    {
        return self.visibility == PUBLIC;
    }

    /// Whether the provider observed a form with no visibility to declare.
    ///
    /// True is an observation. False is not the opposite of it — see the module doc.
    #[must_use]
    pub fn Declares_No_Visibility(&self) -> bool
    {
        return self.visibility == NOT_APPLICABLE;
    }
}

/// Why a payload could not be read.
///
/// Every variant carries where. A caller told only that something failed has been handed a
/// number nobody can act on, and this text reaches a finding somebody has to answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayloadRefusal
{
    /// The bytes are not UTF-8, so they are not this schema.
    NotUtf8,
    /// The first record is not an `unexpanded` header.
    ///
    /// Includes the empty payload. A fact carrying no bytes is not a file that declared
    /// nothing — that is a header and no items — and reading one as the other makes a
    /// subject that was never read indistinguishable from a subject with nothing in it.
    NoHeader,
    /// A second `unexpanded` header, which would leave two answers to one question.
    RepeatedHeader
    {
        line: usize,
    },
    /// A record tag this build does not understand.
    ///
    /// The likeliest cause is a payload from a newer schema, which is precisely the case
    /// where guessing is worst: the unknown record is where the missing information is.
    UnknownRecord
    {
        tag: String,
        line: usize,
    },
    /// A record with the right tag and the wrong shape.
    WrongFieldCount
    {
        tag: String,
        expected: usize,
        found: usize,
        line: usize,
    },
    /// A field that must be a number and is not.
    UnreadableNumber
    {
        field: &'static str,
        value: String,
        line: usize,
    },
}

impl PayloadRefusal
{
    /// What went wrong, in terms somebody can act on.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::NotUtf8 => "the payload is not UTF-8, so it is not this schema".to_owned(),
            Self::NoHeader => "the payload does not begin with an `unexpanded` header, so it \
                               is either empty or not this schema"
                .to_owned(),
            Self::RepeatedHeader { line } => {
                format!("line {line} is a second `unexpanded` header, and a payload has one")
            }
            Self::UnknownRecord { tag, line } => format!(
                "line {line} carries the record tag `{tag}`, which this build does not \
                 understand"
            ),
            Self::WrongFieldCount {
                tag,
                expected,
                found,
                line,
            } => format!(
                "line {line} is a `{tag}` record with {found} field(s) where this build \
                 expects {expected}"
            ),
            Self::UnreadableNumber { field, value, line } => {
                format!("line {line} carries `{value}` where `{field}` must be a number")
            }
        };
    }
}

/// Reads a payload, or refuses it.
///
/// Never a partial answer. A payload half of which decodes is an index that is silently
/// short, and the caller cannot tell it from a file that declared that much.
///
/// # Errors
///
/// [`PayloadRefusal`] for anything that is not a well-formed payload by the grammar above.
pub fn Parse_Payload(bytes: &[u8]) -> Result<SyntaxPayload, PayloadRefusal>
{
    let Ok(text) = core::str::from_utf8(bytes)
    else
    {
        return Err(PayloadRefusal::NotUtf8);
    };

    let mut unexpanded: Option<u32> = None;
    let mut items = Vec::new();

    for (offset, line) in text.lines().enumerate()
    {
        let at = offset.saturating_add(1);
        let fields: Vec<&str> = line.split('\t').collect();
        // `split` yields at least one element for every input, including the empty one, so
        // an absent tag is an empty tag and is refused rather than skipped.
        let tag = fields.first().copied().unwrap_or_default();

        match tag
        {
            "unexpanded" =>
            {
                if unexpanded.is_some()
                {
                    return Err(PayloadRefusal::RepeatedHeader { line: at });
                }
                Expect_Fields(tag, &fields, HEADER_FIELDS, at)?;
                unexpanded =
                    Some(Number(fields.get(1).copied().unwrap_or_default(), "unexpanded", at)?);
            }
            "item" =>
            {
                // Before the header rather than after it is the same defect as no header at
                // all: the count that says how much of the tree went unread is missing at
                // the moment the items are being believed.
                if unexpanded.is_none()
                {
                    return Err(PayloadRefusal::NoHeader);
                }
                Expect_Fields(tag, &fields, ITEM_FIELDS, at)?;

                items.push(PayloadItem {
                    ordinal: Number(fields.get(1).copied().unwrap_or_default(), "ordinal", at)?,
                    kind: fields.get(2).copied().unwrap_or_default().to_owned(),
                    visibility: fields.get(3).copied().unwrap_or_default().to_owned(),
                    qualified_name: fields.get(4).copied().unwrap_or_default().to_owned(),
                });
            }
            other =>
            {
                return Err(PayloadRefusal::UnknownRecord {
                    tag: other.to_owned(),
                    line: at,
                });
            }
        }
    }

    let Some(unexpanded) = unexpanded
    else
    {
        return Err(PayloadRefusal::NoHeader);
    };

    return Ok(SyntaxPayload { unexpanded, items });
}

/// Renders a payload back to the bytes the grammar describes.
///
/// Here for the round trip and for nothing else. **A provider must not call this**: what
/// makes two providers of one capability interchangeable is that each writes the format
/// independently and a third party can read both, and a shared writer would make that true
/// by construction. It exists so this module's own grammar can be tested against itself,
/// and so a consumer holding a decoded payload can produce the bytes it came from.
#[must_use]
pub fn Render_Payload(payload: &SyntaxPayload) -> Vec<u8>
{
    let mut rendered = String::new();

    // `writeln!` writes `\n` on every platform, which is what the grammar requires — the
    // line ending here must not be the host's.
    let _ = writeln!(rendered, "unexpanded\t{}", payload.unexpanded);

    for item in &payload.items
    {
        let _ = writeln!(
            rendered,
            "item\t{}\t{}\t{}\t{}",
            item.ordinal, item.kind, item.visibility, item.qualified_name
        );
    }

    return rendered.into_bytes();
}

/// Refuses a record whose field count is not the one the grammar states.
fn Expect_Fields(
    tag: &str,
    fields: &[&str],
    expected: usize,
    line: usize,
) -> Result<(), PayloadRefusal>
{
    if fields.len() == expected
    {
        return Ok(());
    }

    return Err(PayloadRefusal::WrongFieldCount {
        tag: tag.to_owned(),
        expected,
        found: fields.len(),
        line,
    });
}

/// Reads a field that must be a number.
fn Number(value: &str, field: &'static str, line: usize) -> Result<u32, PayloadRefusal>
{
    return value.parse::<u32>().map_err(|_| {
        return PayloadRefusal::UnreadableNumber {
            field,
            value: value.to_owned(),
            line,
        };
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    const SAMPLE: &str = "unexpanded\t2\n\
                          item\t0\tModule\tPrivate\ttests\n\
                          item\t1\tFunction\tPublic\ttests::One\n\
                          item\t2\tFunction\tNotApplicable\tJudged::Two\n";

    #[test]
    fn Test_A_Well_Formed_Payload_Should_Decode_To_Its_Records()
    {
        let payload = Parse_Payload(SAMPLE.as_bytes()).expect("this is the grammar");

        assert_eq!(payload.unexpanded, 2);
        assert_eq!(payload.items.len(), 3);

        let second = payload.items.get(1).expect("three items");
        assert_eq!(second.ordinal, 1);
        assert_eq!(second.kind, FUNCTION);
        assert_eq!(second.Own_Name(), "One");
        assert!(second.Is_Public());
        assert!(!second.Declares_No_Visibility());

        let third = payload.items.get(2).expect("three items");
        assert!(third.Declares_No_Visibility());
        assert_eq!(third.Own_Name(), "Two");
    }

    /// A file that declares nothing is a real answer, and the shortest well-formed payload.
    #[test]
    fn Test_A_File_That_Declares_Nothing_Should_Decode_To_No_Items()
    {
        let payload = Parse_Payload(b"unexpanded\t0\n").expect("a header alone is well formed");

        assert_eq!(payload.unexpanded, 0);
        assert!(payload.items.is_empty());
    }

    /// An unqualified name is its own name. The fallback is asserted because every other
    /// case in this workspace is nested, so nothing else would ever reach it.
    #[test]
    fn Test_An_Unqualified_Name_Should_Be_Its_Own_Name()
    {
        let payload = Parse_Payload(b"unexpanded\t0\nitem\t0\tStruct\tPublic\tAlone\n")
            .expect("well formed");

        assert_eq!(payload.items.first().expect("one item").Own_Name(), "Alone");
    }

    /// The negative controls. None of these may arrive at a caller as an empty payload: a
    /// reader that decodes unreadable bytes to "no items" makes a subject that was never
    /// read indistinguishable from a subject that declared nothing.
    #[test]
    fn Test_A_Payload_This_Build_Cannot_Read_Should_Not_Decode_To_No_Items()
    {
        assert_eq!(Parse_Payload(&[0xFF, 0xFE]), Err(PayloadRefusal::NotUtf8));
        assert_eq!(Parse_Payload(b""), Err(PayloadRefusal::NoHeader));
        assert_eq!(
            Parse_Payload(b"item\t0\tFunction\tPublic\tOne\n"),
            Err(PayloadRefusal::NoHeader)
        );
        assert_eq!(
            Parse_Payload(b"unexpanded\t0\nunexpanded\t1\n"),
            Err(PayloadRefusal::RepeatedHeader { line: 2 })
        );

        let unknown = Parse_Payload(b"unexpanded\t0\nregion\t0\t3\n").expect_err("unknown tag");
        assert!(
            matches!(unknown, PayloadRefusal::UnknownRecord { ref tag, line: 2 } if tag == "region"),
            "{unknown:?}"
        );

        let short = Parse_Payload(b"unexpanded\t0\nitem\t0\tFunction\tOne\n")
            .expect_err("a record with the wrong field count must refuse");
        assert!(
            matches!(
                short,
                PayloadRefusal::WrongFieldCount {
                    found: 4,
                    expected: 5,
                    line: 2,
                    ..
                }
            ),
            "{short:?}"
        );

        let unnumbered = Parse_Payload(b"unexpanded\tmany\n").expect_err("a count is a number");
        assert!(
            matches!(unnumbered, PayloadRefusal::UnreadableNumber { field: "unexpanded", .. }),
            "{unnumbered:?}"
        );

        let ordinal = Parse_Payload(b"unexpanded\t0\nitem\tfirst\tFunction\tPublic\tOne\n")
            .expect_err("an ordinal is a number");
        assert!(
            matches!(ordinal, PayloadRefusal::UnreadableNumber { field: "ordinal", line: 2, .. }),
            "{ordinal:?}"
        );
    }

    /// Every refusal says where and what, because the text reaches somebody who has to act.
    #[test]
    fn Test_A_Refusal_Should_Say_What_It_Refused()
    {
        let described = Parse_Payload(b"unexpanded\t0\nregion\t0\n")
            .expect_err("an unknown tag refuses")
            .Describe();

        assert!(described.contains("region"), "{described}");
        assert!(described.contains('2'), "{described}");
    }

    /// The grammar against itself. This is the only thing [`Render_Payload`] is for.
    #[test]
    fn Test_A_Decoded_Payload_Should_Render_Back_To_The_Bytes_It_Came_From()
    {
        let payload = Parse_Payload(SAMPLE.as_bytes()).expect("the grammar");
        let rendered = Render_Payload(&payload);

        assert_eq!(String::from_utf8(rendered.clone()).as_deref(), Ok(SAMPLE));
        assert!(!rendered.contains(&b'\r'), "line endings must not be local");
    }
}

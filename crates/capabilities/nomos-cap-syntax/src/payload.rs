//! The grammar of `nomos.syntax.items.v2`, and the reader the tree agrees on.
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
//! payload     := header item*
//! header      := "unexpanded" TAB u32 LF
//! item        := "item" TAB ordinal TAB kind TAB visibility TAB qualified-name
//!                TAB documentation TAB shape LF
//! ordinal     := u32
//! documentation := observation
//! shape         := observation
//! observation := "-" | "." | "+" escaped
//! escaped     := any text, with `\` `\t` `\n` `\r` written `\\` `\t` `\n` `\r`
//! ```
//!
//! **The header is the first record and appears exactly once.** It carries a lower bound on
//! the places the provider saw the parse tree end in unexpanded tokens. A provider that
//! cannot observe macro invocations writes `0`, and that is the format saying the same
//! thing in every payload rather than a claim that there were none.
//!
//! **`item` records are in source order** and `ordinal` is the item's position in that
//! order, starting at zero. Source order is load-bearing rather than cosmetic: a member is
//! attributed to the most recent enclosing record before it, and a consumer that sorted the
//! items would attribute members to the wrong owner.
//!
//! A file that declares nothing is a header and no items — a real answer, and
//! distinguishable from a payload that could not be read only because the empty byte string
//! is refused rather than decoded.
//!
//! **`kind` and `visibility` are labels, and the schema does not enumerate them.** A
//! provider that can distinguish fewer forms than another writes fewer labels; a kind it
//! cannot distinguish is a kind it must not claim. Fixing the vocabulary here would make
//! the weaker provider unable to answer honestly, and the ceiling already exists to bound
//! what an answer may claim.
//!
//! **`qualified-name` is the name as written, qualified by syntactic nesting** — a function
//! declared in `mod tests` arrives as `tests::Name`, and one declared in an `impl` block
//! arrives as `Type::Name`. It is not a resolved path: no provider of this capability
//! resolves names, and a `::` in it is nesting rather than a module route.
//!
//! # Not observed is not absent
//!
//! This is why there is a v2, and it is the whole of the difference.
//!
//! `documentation` and `shape` are [`Observation`]s rather than strings, because two
//! providers of one capability differ in *what they can see* and not only in how well.
//! `nomos-lang-rust-scan` cannot read a doc comment at all — it skips comment lines and
//! associates nothing with the item below them. If "this item has no documentation" and
//! "this provider does not read documentation" were the same bytes, every list read through
//! the scanner would arrive as a list declaring no mirror: a phantom mirror silently
//! downgraded to an admitted gap, which is absence becoming success in the one field a
//! completeness rule's severity ordering turns on.
//!
//! So the three states are three spellings. `-` is *not observed*; `.` is *observed and
//! there is none*; `+…` is *observed and here it is*. A consumer that cannot act on an
//! unobserved field must say so rather than treat it as empty, and the type makes that
//! difficult to get wrong by accident.
//!
//! Observation is bounded by the provider's guarantee and not by its effort. A provider
//! writes `-` when the method it used cannot see the thing, which is a property of the
//! method — the same property its declared [`nomos_contracts::Guarantee`] describes.
//!
//! ## What `shape` says, per kind
//!
//! Open like the other vocabularies, and meaningful relative to the kind:
//!
//! | For a | `shape` is |
//! |---|---|
//! | typed declaration — a constant, a static | [`SLICE`] when the declared type is a slice or an array, however many references deep, and [`VALUE`] otherwise |
//! | function | `fn/<arity>`, the number of declared parameters including a receiver — read it with [`Function_Arity`] |
//! | implementation block | [`INHERENT`] or [`TRAIT`] |
//! | anything else | `.` — observed, and the shape has nothing to say about this form |
//!
//! The distinctions are the ones a consumer cannot recover from the rest of the record and
//! could otherwise only get by parsing the source a second time. `pub const LIMIT: usize`
//! and `pub const TABLES: &[&str]` are identical in every other field; `fn All()` and
//! `fn All(&self)` are identical in every other field; and an `All` in `impl Display for T`
//! does not belong to `T` the way an `All` in `impl T` does.
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
//! Visibility is the one field where that asymmetry is still unmarked, and v2 does not fix
//! it: making it an [`Observation`] would say that a provider did not observe visibility at
//! all, which is false — the scanner observes `pub` correctly and merely cannot see the
//! enclosing form. The consequence is stated instead: a consumer needing the distinction
//! must obtain it from the *guarantee* it required of the answer, not from the bytes. That
//! is what `nomos-rules` does, and `OD-RULES-001`'s floor is what makes it sound.
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
//! checked the agreement — so each provider decodes its own output through this reader in
//! its own tests, which is the check the duplication was missing rather than the duplication
//! removed.

use crate::payload_item::PayloadItem;
use crate::payload_refusal::PayloadRefusal;
use crate::syntax_payload::SyntaxPayload;
use core::fmt::Write as _;

/// The `kind` label every provider writes for a function form.
pub const FUNCTION: &str = "Function";

/// The `kind` label for an implementation block.
pub const IMPLEMENTATION: &str = "Implementation";

/// The `visibility` label for an item that declares itself public.
pub const PUBLIC: &str = "Public";

/// The `visibility` label for an item form that declares no visibility.
///
/// Reserved with a stated meaning: the provider observed a form that has no visibility to
/// declare — a trait member, an `impl` block, a macro definition. Read the module's own
/// caveat before treating its absence as evidence of anything.
pub const NOT_APPLICABLE: &str = "NotApplicable";

/// The `shape` of a typed declaration whose type is a slice or an array.
pub const SLICE: &str = "slice";

/// The `shape` of a typed declaration whose type is anything else.
pub const VALUE: &str = "value";

/// The `shape` of an implementation block that implements no trait.
pub const INHERENT: &str = "inherent";

/// The `shape` of an implementation block that implements a trait.
pub const TRAIT: &str = "trait";

/// The `shape` prefix a function's arity is written behind.
const FUNCTION_SHAPE: &str = "fn/";

/// Fields in an `item` record, tag included.
const ITEM_FIELDS: usize = 7;

/// Fields in the `unexpanded` header, tag included.
const HEADER_FIELDS: usize = 2;

/// What a provider saw when it looked — including that it could not look.
///
/// Three states rather than an `Option`, because the two empty answers are not the same
/// answer and one of them is a silent downgrade. See the module doc.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation
{
    /// The method this provider used cannot see this. Nothing is claimed either way.
    NotObserved,
    /// The provider looked, and there is nothing here.
    Absent,
    /// The provider looked, and this is what it saw.
    Present(String),
}

impl Observation
{
    /// The value, when there is one.
    ///
    /// `None` for both empty states, so a caller that only wants the text can have it —
    /// and a caller that must not confuse the two has [`Observation::Was_Observed`].
    #[must_use]
    pub fn Value(&self) -> Option<&str>
    {
        return match self
        {
            Self::Present(value) => Some(value),
            Self::NotObserved | Self::Absent => None,
        };
    }

    /// Whether the provider was able to look at all.
    #[must_use]
    pub fn Was_Observed(&self) -> bool
    {
        return !matches!(self, Self::NotObserved);
    }

    /// The wire form: `-`, `.`, or `+` and the escaped value.
    #[must_use]
    pub fn Encode(&self) -> String
    {
        return match self
        {
            Self::NotObserved => "-".to_owned(),
            Self::Absent => ".".to_owned(),
            Self::Present(value) => format!("+{}", Escape(value)),
        };
    }

    /// Reads the wire form back.
    ///
    /// A field that is neither of the two marks and does not begin with `+` is not this
    /// schema — refused rather than read as absent, because absent is one of the answers.
    fn Decode(field: &str) -> Option<Self>
    {
        return match field
        {
            "-" => Some(Self::NotObserved),
            "." => Some(Self::Absent),
            _ => field.strip_prefix('+').map(|value| return Self::Present(Unescape(value))),
        };
    }
}

/// The arity a function `shape` declares, if the field is one.
#[must_use]
pub fn Function_Arity(shape: &Observation) -> Option<u32>
{
    return shape
        .Value()?
        .strip_prefix(FUNCTION_SHAPE)
        .and_then(|arity| return arity.parse().ok());
}

/// The `shape` a function of this arity declares.
#[must_use]
pub fn Function_Shape(arity: usize) -> String
{
    return format!("{FUNCTION_SHAPE}{arity}");
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

    let mut read = Reading {
        unexpanded: None,
        items: Vec::new(),
    };

    for (offset, line) in text.lines().enumerate()
    {
        Read_Record(line, offset.saturating_add(1), &mut read)?;
    }

    return read.Complete();
}

/// A payload part-way through being read.
///
/// `unexpanded` is optional here and not on [`SyntaxPayload`], because the header may not
/// have arrived yet — and a payload that never carries one is refused rather than defaulted.
struct Reading
{
    unexpanded: Option<u32>,
    items: Vec<PayloadItem>,
}

impl Reading
{
    /// The payload a completed read describes, if it describes one.
    ///
    /// A payload with no header at all is refused rather than read as zero unexpanded
    /// macros: zero is a claim that the provider looked and found none, and nobody looked.
    fn Complete(self) -> Result<SyntaxPayload, PayloadRefusal>
    {
        let Some(unexpanded) = self.unexpanded
        else
        {
            return Err(PayloadRefusal::NoHeader);
        };

        return Ok(SyntaxPayload {
            unexpanded,
            items: self.items,
        });
    }
}

/// One record of a payload.
///
/// `split` yields at least one element for every input, including the empty one, so an
/// absent tag is an empty tag and is refused rather than skipped.
fn Read_Record(line: &str, at: usize, read: &mut Reading) -> Result<(), PayloadRefusal>
{
    let fields: Vec<&str> = line.split('\t').collect();
    let tag = fields.first().copied().unwrap_or_default();

    match tag
    {
        "unexpanded" => read.unexpanded = Some(Header(&fields, at, read.unexpanded)?),
        "item" =>
        {
            let item = Read_Item(&fields, at, read.unexpanded)?;
            read.items.push(item);
        }
        other =>
        {
            return Err(PayloadRefusal::UnknownRecord {
                tag: other.to_owned(),
                line: at,
            });
        }
    }

    return Ok(());
}

/// One item record, refused if the header has not arrived yet.
///
/// An item before the header is the same defect as no header at all: the count that says how
/// much of the tree went unread is missing at the moment the items are being believed.
fn Read_Item(
    fields: &[&str],
    at: usize,
    unexpanded: Option<u32>,
) -> Result<PayloadItem, PayloadRefusal>
{
    if unexpanded.is_none()
    {
        return Err(PayloadRefusal::NoHeader);
    }
    Expect_Fields("item", fields, ITEM_FIELDS, at)?;

    return Item(fields, at);
}

/// The header's count, refusing a second one.
///
/// A payload carrying two headers does not say which count is its own, and taking either
/// would be inventing an answer the bytes do not give.
fn Header(fields: &[&str], at: usize, already: Option<u32>) -> Result<u32, PayloadRefusal>
{
    if already.is_some()
    {
        return Err(PayloadRefusal::RepeatedHeader { line: at });
    }
    Expect_Fields("unexpanded", fields, HEADER_FIELDS, at)?;

    return Number(fields.get(1).copied().unwrap_or_default(), "unexpanded", at);
}

/// One item record, with both observation-bearing fields read as observations.
fn Item(fields: &[&str], at: usize) -> Result<PayloadItem, PayloadRefusal>
{
    return Ok(PayloadItem {
        ordinal: Number(fields.get(1).copied().unwrap_or_default(), "ordinal", at)?,
        kind: fields.get(2).copied().unwrap_or_default().to_owned(),
        visibility: fields.get(3).copied().unwrap_or_default().to_owned(),
        qualified_name: fields.get(4).copied().unwrap_or_default().to_owned(),
        documentation: Observed(
            fields.get(5).copied().unwrap_or_default(),
            "documentation",
            at,
        )?,
        shape: Observed(fields.get(6).copied().unwrap_or_default(), "shape", at)?,
    });
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
            "item\t{}\t{}\t{}\t{}\t{}\t{}",
            item.ordinal,
            item.kind,
            item.visibility,
            item.qualified_name,
            item.documentation.Encode(),
            item.shape.Encode()
        );
    }

    return rendered.into_bytes();
}

/// Makes a value safe to carry in one tab-separated field.
///
/// Documentation is prose and arrives with newlines in it. Escaping rather than dropping
/// them keeps a multi-line doc comment one field and one record, so the grammar stays
/// line-oriented and a consumer still reads the text the author wrote.
#[must_use]
pub fn Escape(value: &str) -> String
{
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars()
    {
        match character
        {
            '\\' => escaped.push_str("\\\\"),
            '\t' => escaped.push_str("\\t"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            other => escaped.push(other),
        }
    }

    return escaped;
}

/// Reads an escaped field back.
///
/// An escape this build does not know keeps its backslash rather than being swallowed,
/// because dropping it would quietly change the text a consumer then matches against.
#[must_use]
pub fn Unescape(value: &str) -> String
{
    let mut plain = String::with_capacity(value.len());
    let mut characters = value.chars();

    while let Some(character) = characters.next()
    {
        if character == '\\'
        {
            Push_Escaped(&mut plain, characters.next());
            continue;
        }
        plain.push(character);
    }

    return plain;
}

/// What a backslash and the character behind it stand for.
///
/// An escaped backslash and a backslash at the very end with nothing behind it share an arm,
/// because the answer is the same character and the second case is a field that is not this
/// schema — reproducing what was written is the least that can be got wrong.
fn Push_Escaped(plain: &mut String, escaped: Option<char>)
{
    match escaped
    {
        Some('t') => plain.push('\t'),
        Some('n') => plain.push('\n'),
        Some('r') => plain.push('\r'),
        Some('\\') | None => plain.push('\\'),
        Some(other) =>
        {
            plain.push('\\');
            plain.push(other);
        }
    }
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

/// Reads a field that must be an observation.
fn Observed(value: &str, field: &'static str, line: usize) -> Result<Observation, PayloadRefusal>
{
    return Observation::Decode(value).ok_or_else(|| {
        return PayloadRefusal::UnreadableObservation {
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
    use crate::payload_refusal::PayloadRefusal;
    use crate::payload_item::PayloadItem;
    use crate::syntax_payload::SyntaxPayload;

    const SAMPLE: &str = "unexpanded\t2\n\
                          item\t0\tModule\tPrivate\ttests\t.\t.\n\
                          item\t1\tConstant\tPublic\tTABLES\t+Mirrored by `Test_X`.\t+slice\n\
                          item\t2\tFunction\tNotApplicable\tJudged::Two\t-\t-\n";

    #[test]
    fn Test_A_Well_Formed_Payload_Should_Decode_To_Its_Records()
    {
        let payload = Parse_Payload(SAMPLE.as_bytes()).expect("this is the grammar");

        assert_eq!(payload.unexpanded, 2);
        assert_eq!(payload.items.len(), 3);

        let list = payload.items.get(1).expect("three items");
        assert_eq!(list.kind, "Constant");
        assert_eq!(list.Own_Name(), "TABLES");
        assert!(list.Is_Public());
        assert_eq!(list.documentation.Value(), Some("Mirrored by `Test_X`."));
        assert_eq!(list.shape.Value(), Some(SLICE));
    }

    /// The distinction v2 exists for, asserted as bytes rather than as a description.
    #[test]
    fn Test_Not_Observed_And_Absent_Should_Be_Different_Bytes()
    {
        assert_ne!(Observation::NotObserved.Encode(), Observation::Absent.Encode());

        let payload = Parse_Payload(SAMPLE.as_bytes()).expect("the grammar");

        let looked = payload.items.first().expect("three items");
        assert_eq!(looked.documentation, Observation::Absent);
        assert!(looked.documentation.Was_Observed(), "this provider read doc comments");

        let blind = payload.items.get(2).expect("three items");
        assert_eq!(blind.documentation, Observation::NotObserved);
        assert!(!blind.documentation.Was_Observed());

        // Both answer `None` to "what is the text", which is exactly why the caller that
        // must not confuse them has a second question to ask.
        assert_eq!(looked.documentation.Value(), blind.documentation.Value());
    }

    /// A member belongs to the record it follows, and two impls of one type are told apart
    /// by nothing else.
    #[test]
    fn Test_A_Member_Should_Be_Enclosed_By_The_Record_It_Follows()
    {
        let payload = Parse_Payload(
            b"unexpanded\t0\n\
              item\t0\tImplementation\tNotApplicable\tTable\t.\t+trait\n\
              item\t1\tFunction\tNotApplicable\tTable::All\t.\t+fn/1\n\
              item\t2\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
              item\t3\tFunction\tPublic\tTable::All\t.\t+fn/0\n",
        )
        .expect("well formed");

        let under_trait = payload.Enclosing(1).expect("the first All is enclosed");
        assert_eq!(under_trait.shape.Value(), Some(TRAIT));

        let under_inherent = payload.Enclosing(3).expect("the second All is enclosed");
        assert_eq!(under_inherent.shape.Value(), Some(INHERENT));

        assert!(payload.Enclosing(0).is_none(), "a top-level item encloses nothing");
    }

    #[test]
    fn Test_A_Function_Shape_Should_Carry_Its_Arity()
    {
        assert_eq!(Function_Shape(0), "fn/0");
        assert_eq!(
            Function_Arity(&Observation::Present(Function_Shape(2))),
            Some(2)
        );
        assert_eq!(Function_Arity(&Observation::NotObserved), None);
        assert_eq!(Function_Arity(&Observation::Present(SLICE.to_owned())), None);
    }

    /// Documentation is prose and arrives with newlines and tabs in it. One field, one
    /// record, and the text a consumer matches against is the text the author wrote.
    #[test]
    fn Test_Documentation_Should_Survive_The_Field_It_Travels_In()
    {
        let prose = "Mirrored by `Test_X`.\n\nA second\tparagraph with a \\ in it.";
        let payload = SyntaxPayload {
            unexpanded: 0,
            items: vec![PayloadItem {
                ordinal: 0,
                kind: "Constant".to_owned(),
                visibility: PUBLIC.to_owned(),
                qualified_name: "TABLES".to_owned(),
                documentation: Observation::Present(prose.to_owned()),
                shape: Observation::Present(SLICE.to_owned()),
            }],
        };

        let rendered = Render_Payload(&payload);
        assert_eq!(
            rendered.iter().filter(|byte| return **byte == b'\n').count(),
            2,
            "the newlines in the prose must not become records"
        );

        let read = Parse_Payload(&rendered).expect("what this module wrote");
        assert_eq!(read.items.first().expect("one item").documentation.Value(), Some(prose));
    }

    /// A file that declares nothing is a real answer, and the shortest well-formed payload.
    #[test]
    fn Test_A_File_That_Declares_Nothing_Should_Decode_To_No_Items()
    {
        let payload = Parse_Payload(b"unexpanded\t0\n").expect("a header alone is well formed");

        assert_eq!(payload.unexpanded, 0);
        assert!(payload.items.is_empty());
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
            Parse_Payload(b"item\t0\tFunction\tPublic\tOne\t.\t.\n"),
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

        // A v1 record, which is this schema's likeliest wrong input rather than a
        // hypothetical one: five fields where seven are written.
        let previous = Parse_Payload(b"unexpanded\t0\nitem\t0\tFunction\tPublic\tOne\n")
            .expect_err("the previous schema is not this one");
        assert!(
            matches!(
                previous,
                PayloadRefusal::WrongFieldCount {
                    found: 5,
                    expected: 7,
                    line: 2,
                    ..
                }
            ),
            "{previous:?}"
        );

        let unnumbered = Parse_Payload(b"unexpanded\tmany\n").expect_err("a count is a number");
        assert!(
            matches!(unnumbered, PayloadRefusal::UnreadableNumber { field: "unexpanded", .. }),
            "{unnumbered:?}"
        );

        let unmarked = Parse_Payload(b"unexpanded\t0\nitem\t0\tConstant\tPublic\tA\tnone\t.\n")
            .expect_err("an observation has three spellings and this is not one");
        assert!(
            matches!(
                unmarked,
                PayloadRefusal::UnreadableObservation {
                    field: "documentation",
                    line: 2,
                    ..
                }
            ),
            "{unmarked:?}"
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

//! Writing a record back out as markdown.
//!
//! The inverse of [`crate::Parse_Record`] and [`crate::Segment`] taken together, and
//! deliberately not the inverse of either one alone. `Parse_Record` keeps the body it was
//! given, so rendering from that string would prove nothing: it would be an echo. What
//! `D-129` needs is a markdown surface projected from the *structure* — the declared
//! identity and the blocks the preservation ledger tracks — because that is what the store
//! holds, and the round trip is only real if the structure is enough to rebuild the file.
//!
//! # Canonical form
//!
//! One layout, chosen to be the one this repository's records already have rather than
//! invented here: front matter in a fixed key order with plain scalars, a blank line after
//! the closing fence, blocks separated by one blank line, and a trailing newline. Every
//! record under `docs/records` is a fixed point of it, which is what
//! `Test_Every_Governing_Record_Should_Round_Trip` asserts.
//!
//! A document that is *not* a fixed point is not thereby wrong — a v14 corpus record
//! carrying a byte order mark (`D-131`), or spelling a relation key `relation` rather than
//! `type`, is a legitimate record this layout cannot reproduce. [`Is_Round_Trip`] is how a
//! caller finds out before an edit rather than after, and the authoring transaction refuses
//! such a document rather than silently rewriting it into this shape.

use crate::{Segment, SourceBlock};
use crate::Parse_Record;
use crate::record::front_matter::FrontMatter as RecordFrontMatter;

/// A value the canonical layout cannot represent.
///
/// Its own error rather than a lossy escape. The alternative is emitting a quoted scalar
/// where every other record has a plain one, which would make one record's front matter a
/// different shape from the rest for a reason no reader could see — and the byte-identity
/// this whole surface rests on would hold for the file and not for the format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError
{
    /// A scalar that would not survive being written plainly and read back.
    Unrepresentable
    {
        field: String,
        value: String,
        cause: &'static str,
    },
}

impl core::fmt::Display for RenderError
{
    // `fmt` is the fixed method name `std::fmt::Display` mandates; it is not a free choice
    // of abbreviation and cannot be spelled out without ceasing to implement the trait.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unrepresentable {
                field,
                value,
                cause,
            } => write!(
                formatter,
                "{field} is {value:?}, which the canonical front matter cannot write plainly: \
                 {cause}"
            ),
        };
    }
}

impl std::error::Error for RenderError
{}

/// The declared front matter and the blocks, as one markdown document.
///
/// # Errors
///
/// Returns [`RenderError::Unrepresentable`] if a front matter scalar cannot be written
/// plainly.
pub fn Render_Record(
    front_matter: &RecordFrontMatter,
    blocks: &[SourceBlock],
) -> Result<String, RenderError>
{
    let mut text = String::from("---\n");

    Render_Identity(&mut text, front_matter)?;
    Render_Tags(&mut text, &front_matter.tags)?;
    Render_Relations(&mut text, &front_matter.relations)?;
    text.push_str("---\n\n");
    Render_Blocks(&mut text, blocks);

    return Ok(text);
}

/// The scalar fields, in the order the canonical layout writes them.
fn Render_Identity(text: &mut String, front_matter: &RecordFrontMatter) -> Result<(), RenderError>
{
    for (field, value) in [
        ("id", &front_matter.id),
        ("type", &front_matter.kind),
        ("title", &front_matter.title),
        ("status", &front_matter.status),
    ]
    {
        let scalar = Render_Scalar_Field(Field(field), Value(value))?;
        text.push_str(&scalar);
    }

    text.push_str("version: ");
    text.push_str(&front_matter.version.to_string());
    text.push('\n');

    let authority = Render_Scalar_Field(Field("authority"), Value(&front_matter.authority))?;
    text.push_str(&authority);

    return Ok(());
}

/// A front matter key, kept distinct from [`Value`] so [`Render_Scalar_Field`]'s two positions cannot be
/// swapped at a call site.
struct Field<'a>(&'a str);

/// A front matter scalar's value, kept distinct from [`Field`] for the same reason.
struct Value<'a>(&'a str);

fn Render_Scalar_Field(field: Field<'_>, value: Value<'_>) -> Result<String, RenderError>
{
    let field = field.0;
    let value = value.0;

    return Ok(format!("{field}: {}\n", Plain_Scalar(field, value)?));
}

/// The tag sequence, omitted entirely when there is none.
fn Render_Tags(text: &mut String, tags: &[String]) -> Result<(), RenderError>
{
    if tags.is_empty()
    {
        return Ok(());
    }

    text.push_str("tags:\n");
    for tag in tags
    {
        let plain = Plain_Scalar("tags", tag)?;
        text.push_str("  - ");
        text.push_str(plain);
        text.push('\n');
    }

    return Ok(());
}

/// The relation sequence, in the order the record declares them.
fn Render_Relations(text: &mut String, relations: &[crate::RecordRelation]) -> Result<(), RenderError>
{
    if relations.is_empty()
    {
        return Ok(());
    }

    text.push_str("relations:\n");
    for relation in relations
    {
        let target = Plain_Scalar("relations.target", &relation.target)?;
        let kind = Plain_Scalar("relations.type", &relation.relation)?;
        text.push_str("  - target: ");
        text.push_str(target);
        text.push_str("\n    type: ");
        text.push_str(kind);
        text.push('\n');
    }

    return Ok(());
}

/// The body, one blank line between blocks and a newline after each.
fn Render_Blocks(text: &mut String, blocks: &[SourceBlock])
{
    for (index, block) in blocks.iter().enumerate()
    {
        if index > 0
        {
            text.push('\n');
        }
        text.push_str(&block.text);
        text.push('\n');
    }
}

/// Whether this document is exactly what the canonical layout would produce for it.
///
/// The question an authoring surface has to ask before it accepts an edit. A document that
/// answers `false` can still be read, hashed, segmented and preserved; what it cannot be is
/// written back without changing bytes nobody asked to change.
#[must_use]
pub fn Is_Round_Trip(markdown: &str) -> bool
{
    let Ok(record) = Parse_Record(markdown)
    else
    {
        return false;
    };

    return Render_Record(&record.front_matter, &Segment(&record.body))
        .is_ok_and(|rendered| return rendered == markdown);
}

/// A scalar that means the same thing after being written plainly and read back.
///
/// Conservative on purpose. Every refusal here is a document this surface declines to
/// author, which is recoverable; every case wrongly allowed is a record whose meaning
/// changed on the way out, which is the failure the preservation ledger exists to prevent.
fn Plain_Scalar<'value>(field: &str, value: &'value str) -> Result<&'value str, RenderError>
{
    let Some(cause) = Why_It_Cannot_Be_Plain(value)
    else
    {
        return Ok(value);
    };

    return Err(RenderError::Unrepresentable {
        field: field.to_owned(),
        value: value.to_owned(),
        cause,
    });
}

/// Why a value would not read back as itself, or nothing if it would.
fn Why_It_Cannot_Be_Plain(value: &str) -> Option<&'static str>
{
    if value.is_empty()
    {
        return Some("it is empty, and an empty plain scalar reads back as null");
    }
    if value.trim() != value
    {
        return Some("it begins or ends with whitespace, which a plain scalar loses");
    }
    if value.contains('\n')
    {
        return Some("it spans lines");
    }
    if Has_A_Leading_Indicator(value)
    {
        return Some("it opens with a YAML indicator");
    }

    return Why_It_Reads_Back_As_Something_Else(value);
}

/// A leading indicator turns the value into a different YAML node: `- x` a sequence, `*x` an
/// alias, `#x` a comment.
///
/// `?`, `:` and `,` are indicators only in flow context but are refused too, because a reader
/// that has to know the context to know what a record says is the ambiguity this format exists
/// without.
fn Has_A_Leading_Indicator(value: &str) -> bool
{
    return value.starts_with([
        '-', '?', ':', ',', '[', ']', '{', '}', '#', '&', '*', '!', '|', '>', '\'', '"', '%',
        '@', '`',
    ]);
}

/// The values a plain scalar reads back as some other kind of node entirely.
fn Why_It_Reads_Back_As_Something_Else(value: &str) -> Option<&'static str>
{
    if value.contains(": ") || value.ends_with(':')
    {
        return Some("a colon followed by space or ending the line makes it a mapping");
    }
    if value.contains(" #")
    {
        return Some("a space before a hash starts a comment");
    }
    if ["true", "false", "null", "yes", "no", "on", "off", "~"]
        .contains(&value.to_ascii_lowercase().as_str())
    {
        return Some("it reads back as a boolean or null rather than as text");
    }
    if value.parse::<f64>().is_ok()
    {
        return Some("it reads back as a number rather than as text");
    }

    return None;
}

#[cfg(test)]
mod tests
{
    use super::*;

    const RECORD: &str = "---\nid: D-129\ntype: decision\ntitle: A title\nstatus: accepted\n\
                          version: 2\nauthority: canonical-normative-record\ntags:\n  - one\n\
                          \x20 - two\nrelations:\n  - target: ADR-DOC-001\n    type: supersedes\n\
                          ---\n\n# A title\n\nBody.\n\n## Section\n\nMore.\n";

    fn Rendered_Record(markdown: &str) -> String
    {
        let record = Parse_Record(markdown).expect("reads");

        return Render_Record(&record.front_matter, &Segment(&record.body)).expect("renders");
    }

    /// How many of `RECORD`'s blocks to keep for
    /// [`Test_Rendering_Should_Use_The_Blocks_And_Not_The_Retained_Body`] — the heading and
    /// the first paragraph, stopping short of the second heading.
    const BLOCKS_KEPT: usize = 2;

    #[test]
    fn Test_A_Record_Should_Render_To_The_Bytes_It_Was_Read_From()
    {
        assert_eq!(Rendered_Record(RECORD), RECORD);
        assert!(Is_Round_Trip(RECORD));
    }

    /// The point of rendering from blocks rather than from the retained body. If this
    /// rendered the body it was handed, the assertion above would hold for a function that
    /// does nothing, and the store — which holds blocks, not bodies — would still have no
    /// way to produce markdown.
    #[test]
    fn Test_Rendering_Should_Use_The_Blocks_And_Not_The_Retained_Body()
    {
        let record = Parse_Record(RECORD).expect("reads");
        let mut blocks = Segment(&record.body);
        blocks.truncate(BLOCKS_KEPT);

        let rendered =
            Render_Record(&record.front_matter, &blocks).expect("renders the shorter document");

        assert!(rendered.contains("Body."), "{rendered}");
        assert!(!rendered.contains("More."), "the retained body leaked through");
    }

    #[test]
    fn Test_A_Record_With_No_Tags_Or_Relations_Should_Omit_Both_Keys()
    {
        let bare = "---\nid: D-1\ntype: decision\ntitle: A title\nstatus: accepted\nversion: 1\n\
                    authority: canonical-normative-record\n---\n\n# A title\n\nBody.\n";

        assert_eq!(Rendered_Record(bare), bare);
        assert!(!Rendered_Record(bare).contains("tags"));
    }

    /// A code block carries its own blank lines, and joining blocks on a blank line would
    /// otherwise split it.
    #[test]
    fn Test_A_Fenced_Block_Should_Survive_The_Round_Trip()
    {
        let fenced = "---\nid: D-1\ntype: decision\ntitle: A title\nstatus: accepted\nversion: 1\n\
                      authority: canonical-normative-record\n---\n\n# A title\n\n```rust\n\
                      let a = 1;\n\nlet b = 2;\n```\n\nAfter.\n";

        assert_eq!(Rendered_Record(fenced), fenced);
    }

    /// `D-131` decided the mark belongs to the front matter fence, so the parser drops it
    /// and nothing downstream can know it was there. Reporting that as "does not round
    /// trip" is the honest answer; claiming byte-identity would be false.
    #[test]
    fn Test_A_Byte_Order_Mark_Should_Not_Claim_To_Round_Trip()
    {
        let marked = format!("\u{feff}{RECORD}");

        assert!(!Is_Round_Trip(&marked));
        assert!(Is_Round_Trip(RECORD), "the negative control changed nothing");
    }

    /// v15's `spec-governance` records spell the relation key `relation`. The reader accepts
    /// both spellings deliberately; the writer has one, so those documents are not fixed
    /// points and must say so rather than being rewritten on the way out.
    #[test]
    fn Test_The_Other_Relation_Spelling_Should_Not_Claim_To_Round_Trip()
    {
        let spelled = RECORD.replace("    type: supersedes", "    relation: supersedes");
        assert_ne!(spelled, RECORD, "the negative control changed nothing");

        assert!(Parse_Record(&spelled).is_ok(), "it is still a readable record");
        assert!(!Is_Round_Trip(&spelled));
    }

    #[test]
    fn Test_A_Title_Holding_A_Colon_Should_Be_Refused_Rather_Than_Quoted()
    {
        let record = Parse_Record(RECORD).expect("reads");
        let mut front_matter = record.front_matter;
        front_matter.title = "A title: with a colon".to_owned();

        let refusal = Render_Record(&front_matter, &Segment(&record.body))
            .expect_err("a colon in a plain scalar makes a mapping");

        assert!(matches!(refusal, RenderError::Unrepresentable { .. }), "{refusal}");
        assert!(refusal.to_string().contains("title"), "{refusal}");
    }

    /// An apostrophe inside a plain scalar is ordinary text, and six of this repository's
    /// records have one in their title. Refusing them would refuse the corpus this surface
    /// exists to author.
    #[test]
    fn Test_An_Apostrophe_Inside_A_Title_Should_Be_Allowed()
    {
        let record = Parse_Record(RECORD).expect("reads");
        let mut front_matter = record.front_matter;
        front_matter.title = "A document's validity".to_owned();

        assert!(Render_Record(&front_matter, &Segment(&record.body)).is_ok());
    }

    #[test]
    fn Test_A_Status_That_Reads_Back_As_Something_Else_Should_Be_Refused()
    {
        let record = Parse_Record(RECORD).expect("reads");

        for (field, value) in [("status", "no"), ("status", "42"), ("authority", "")]
        {
            let mut front_matter = record.front_matter.clone();
            match field
            {
                "status" => front_matter.status = value.to_owned(),
                _ => front_matter.authority = value.to_owned(),
            }

            let refusal = Render_Record(&front_matter, &Segment(&record.body))
                .expect_err(&format!("{field} = {value:?} was written out rather than refused"));

            assert!(refusal.to_string().contains(field), "{refusal}");
        }
    }
}

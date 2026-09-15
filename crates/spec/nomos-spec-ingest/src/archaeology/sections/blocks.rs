//! What one block's text says: whether it counts, the key it repeats under, the heading it
//! carries, and the body it reads as.
//!
//! These are pure readings of a block and its text, and they sit apart from the cutting and
//! counting passes in [`super`] because nothing here walks the documents: the index calls
//! them. They came out of `sections.rs` when it crossed the file-size review trigger, and
//! their tests came with them -- a Rust unit test's companion is the file the test is
//! written in, so a function and its test have to move together.

use super::super::{Is_Template_Eligible, SourceBlock, SHARED_BY};
use super::{Body, SectionText, SectionTitle};

pub(super) fn Keyable_Blocks(body: &[SourceBlock]) -> Vec<&SourceBlock>
{
    return body
        .iter()
        .filter(|block| return !block.text.trim().is_empty())
        .filter(|block| return !Is_Navigation(&block.text))
        .collect();
}

pub(super) fn Is_Navigation(text: &str) -> bool
{
    let mut lines = text.lines().filter(|line| return !line.trim().is_empty()).peekable();
    if lines.peek().is_none()
    {
        return false;
    }

    return lines.all(|line| {
        let item = line.trim().trim_start_matches(['-', '*', '+']).trim();
        return line.trim().starts_with(['-', '*', '+'])
            && item.starts_with('[')
            && item.ends_with(')');
    });
}

pub(super) fn Template_Key(text: SectionText<'_>, title: SectionTitle<'_>) -> String
{
    let flattened = text.0.split_whitespace().collect::<Vec<&str>>().join(" ");
    let elided = title.0.split_whitespace().collect::<Vec<&str>>().join(" ");

    if elided.is_empty()
    {
        return flattened;
    }

    return flattened.replace(&elided, "{}");
}

pub(super) fn Title_Of(block: &SourceBlock) -> String
{
    return block.text.trim_start_matches('#').trim().to_owned();
}

/// The body one block's text reads as.
///
/// `declared` is deliberately not gated by the floor. The blocklist names known filler
/// outright, and the floor is about repetition being insufficient evidence, which is a
/// different question. In practice nothing turns on it -- every `FILLER_PATTERNS` entry is
/// longer than the floor -- but gating it would make the blocklist unreachable for a short
/// pattern somebody adds later.
pub(super) fn Body_Of(declared: Option<&'static str>, shared: u32, key: &str) -> Body
{
    return if Is_Template(declared, shared, key)
    {
        Body::Template {
            shared_with: shared,
            declared,
        }
    }
    else
    {
        Body::Narrative
    };
}

/// Whether a document's own declaration or repetition past the floor makes this a template.
///
/// Two independent reasons rather than one folded threshold: a document that declared the
/// text filler has said so outright, and a body repeated [`SHARED_BY`] times is only a
/// template if it is long enough for the repetition to mean anything.
fn Is_Template(declared: Option<&'static str>, shared: u32, key: &str) -> bool
{
    return declared.is_some() || (shared >= SHARED_BY && Is_Template_Eligible(key));
}

#[cfg(test)]
mod tests
{
    use super::super::Test_Block;
    use super::*;

    #[test]
    fn Test_Keyable_Blocks_Should_Drop_Empty_And_Navigation_Only_Blocks()
    {
        let body = vec![
            Test_Block("Real content.\n"),
            Test_Block("   \n"),
            Test_Block("- [One](one.md)\n- [Two](two.md)\n"),
        ];

        let kept = Keyable_Blocks(&body);

        assert_eq!(kept.len(), 1);
        assert_eq!(kept.first().expect("the assertion above confirms exactly one kept block").text, "Real content.\n");
    }

    #[test]
    fn Test_Is_Navigation_Should_Recognise_A_Markdown_Link_List_And_Nothing_Else()
    {
        assert!(Is_Navigation("- [One](one.md)\n- [Two](two.md)\n"));
        assert!(!Is_Navigation("Ordinary prose.\n"));
        assert!(!Is_Navigation("   \n"), "blank text names nothing, so it is not navigation either");
    }

    #[test]
    fn Test_Template_Key_Should_Elide_The_Sections_Own_Title_From_Its_Text()
    {
        let key = Template_Key(SectionText("Read Widget within the owning domain contract."), SectionTitle("Widget"));

        assert_eq!(key, "Read {} within the owning domain contract.");
        assert_eq!(Template_Key(SectionText("No title inside."), SectionTitle("")), "No title inside.");
    }

    #[test]
    fn Test_Title_Of_Should_Strip_The_Hash_Marks_From_A_Heading_Line()
    {
        assert_eq!(Title_Of(&Test_Block("## Widget\n")), "Widget");
    }

    /// The three answers the judgement gives: a repeated body is a template only past the
    /// floor and only if it is long enough, and a declared pattern is one whatever the floor.
    #[test]
    fn Test_Body_Of_Should_Read_Declared_And_Repeated_Text_As_A_Template()
    {
        let long = "Refer to the owning domain volume for this material.";

        assert_eq!(
            Body_Of(None, SHARED_BY.saturating_sub(1), long),
            Body::Narrative,
            "below the floor, repetition is not yet evidence"
        );
        assert_eq!(
            Body_Of(None, SHARED_BY, long),
            Body::Template {
                shared_with: SHARED_BY,
                declared: None,
            }
        );
        assert_eq!(
            Body_Of(Some("Short declared pattern."), 0, "short"),
            Body::Template {
                shared_with: 0,
                declared: Some("Short declared pattern."),
            },
            "the blocklist is not gated by the floor"
        );
    }
}

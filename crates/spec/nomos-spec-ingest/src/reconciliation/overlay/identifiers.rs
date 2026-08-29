//! Every `{#ID} statement` v15 carries inline, and where a line anchors one.

use std::collections::BTreeMap;

/// Every `{#ID} statement` v15 carries inline, by identifier.
///
/// v15 dissolved the per-artifact documents into section prose, so an identifier's text
/// is a span inside a line rather than a file. Reconciliation has to read it where it
/// actually is.
#[must_use]
pub fn Statements_In(markdown: &str) -> BTreeMap<String, String>
{
    let mut found = BTreeMap::new();

    for line in markdown.split('\n')
    {
        if let Some((id, text)) = Anchored_In(line)
        {
            found.insert(id.to_owned(), text.trim().to_owned());
        }
    }

    return found;
}

/// The identifier a line anchors and the text following it, if it anchors one.
///
/// The character set is checked because `{#` opens things that are not anchors — a CSS
/// fragment in a fenced block, a template placeholder — and admitting those would invent
/// identifiers that no revision ever declared.
fn Anchored_In(line: &str) -> Option<(&str, &str)>
{
    let (_, after) = line.split_once("{#")?;
    let (id, rest) = after.split_once('}')?;
    if !Is_Anchor_Id(id)
    {
        return None;
    }

    return Some((id, rest));
}

/// Whether an identifier is spelled the way an anchor spells one.
fn Is_Anchor_Id(id: &str) -> bool
{
    return !id.is_empty() && id.bytes().all(|byte| return byte.is_ascii_alphanumeric() || byte == b'-');
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Inline_Identifiers_Should_Be_Read_Where_V15_Put_Them()
    {
        let found = Statements_In(
            "> **Requirement:** {#MODEL-001} MODEL-001 Artifact represents persisted objects.\n\
             Some prose with no identifier.\n",
        );

        assert_eq!(found.len(), 1);
        assert_eq!(
            found.get("MODEL-001").map(String::as_str),
            Some("MODEL-001 Artifact represents persisted objects.")
        );
    }
}

//! The doc comment a declaration carries, if it carries one.


/// The item's documentation, as one string, or `None` when it has none.
///
/// `///` is `#[doc]` after parsing, so both spellings are read and neither has to be
/// recognised as text. The lines are joined with newlines rather than flattened: a consumer
/// matching a claim written on one line of a paragraph needs the paragraph as the author
/// wrote it, and the payload escapes the newlines rather than losing them.
pub(super) fn Documentation(attributes: &[syn::Attribute]) -> Option<String>
{
    let lines: Vec<String> = attributes.iter().filter_map(Doc_Line).collect();

    if lines.is_empty()
    {
        return None;
    }

    return Some(lines.join("\n"));
}

/// One `#[doc = "..."]` attribute's text, or nothing when the attribute is something else.
pub(super) fn Doc_Line(attribute: &syn::Attribute) -> Option<String>
{
    if !attribute.path().is_ident("doc")
    {
        return None;
    }

    let syn::Meta::NameValue(pair) = &attribute.meta
    else
    {
        return None;
    };
    let syn::Expr::Lit(literal) = &pair.value
    else
    {
        return None;
    };
    let syn::Lit::Str(text) = &literal.lit
    else
    {
        return None;
    };

    return Some(text.value());
}

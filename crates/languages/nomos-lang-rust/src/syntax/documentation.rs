//! The doc comment a declaration carries, if it carries one.

/// The item's documentation, as one string, or `None` when it has none.
///
/// `///` is `#[doc]` after parsing, so both spellings are read and neither has to be
/// recognised as text. The lines are joined with newlines rather than flattened: a consumer
/// matching a claim written on one line of a paragraph needs the paragraph as the author
/// wrote it, and the payload escapes the newlines rather than losing them.
pub(super) fn Documentation_Of_Attributes(attributes: &[syn::Attribute]) -> Option<String>
{
    let lines: Vec<String> = attributes.iter().filter_map(Documentation_Line).collect();

    if lines.is_empty()
    {
        return None;
    }

    return Some(lines.join("\n"));
}

/// One `#[doc = "..."]` attribute's text, or nothing when the attribute is something else.
pub(super) fn Documentation_Line(attribute: &syn::Attribute) -> Option<String>
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Documentation_Of_Attributes_Should_Join_Documentation_Lines_With_Newlines()
    {
        let attributes = Attributes_Of("/// First line.\n/// Second line.\nfn f() {}");

        assert_eq!(
            Documentation_Of_Attributes(&attributes),
            Some(" First line.\n Second line.".to_owned())
        );
    }

    #[test]
    fn Test_Documentation_Of_Attributes_Should_Be_None_With_No_Documentation_Attribute()
    {
        let attributes = Attributes_Of("#[allow(dead_code)]\nfn f() {}");

        assert_eq!(Documentation_Of_Attributes(&attributes), None);
    }

    #[test]
    fn Test_Documentation_Line_Should_Be_None_For_A_Non_Documentation_Attribute()
    {
        let attributes = Attributes_Of("#[allow(dead_code)]\nfn f() {}");
        let attribute = attributes.first().expect("one attribute");

        assert_eq!(Documentation_Line(attribute), None);
    }

    #[test]
    fn Test_Documentation_Line_Should_Read_A_Documentation_Attributes_Own_Text()
    {
        let attributes = Attributes_Of("/// A line.\nfn f() {}");
        let attribute = attributes.first().expect("one attribute");

        assert_eq!(Documentation_Line(attribute), Some(" A line.".to_owned()));
    }

    fn Attributes_Of(item: &str) -> Vec<syn::Attribute>
    {
        let parsed: syn::ItemFn = syn::parse_str(item).expect("a valid function fixture parses");

        return parsed.attrs;
    }
}

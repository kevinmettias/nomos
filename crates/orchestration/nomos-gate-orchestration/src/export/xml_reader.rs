//! A reader of the XML subset [`super::SupportingFactGraph`] writes, for its tests alone.
//!
//! Nothing this crate depends on reads XML, and a test that matched fragments of the document
//! would prove the text rather than the graph. This reads elements, attributes, text, the XML
//! declaration and self-closing elements, and the five predefined entities -- the whole of what
//! the writer emits -- and refuses anything else rather than guessing at it.

use std::collections::BTreeMap;

/// One element: its name, its attributes, its child elements and its text.
#[derive(Clone, Debug, Default)]
pub(crate) struct Element
{
    /// The element's name.
    pub(crate) name: String,
    /// Its attributes, unescaped.
    pub(crate) attributes: BTreeMap<String, String>,
    /// Its child elements, in order.
    pub(crate) children: Vec<Element>,
    /// Its text, unescaped, with the whitespace between child elements left out.
    pub(crate) text: String,
}

/// The document `text` holds, as its root element.
pub(crate) fn Parsed(text: &str) -> Element
{
    let mut rest = text.trim_start();
    if let Some(declaration) = rest.strip_prefix("<?xml")
    {
        let (_, after) = declaration.split_once("?>").expect("the XML declaration is closed");
        rest = after.trim_start();
    }

    let (root, after) = Element_At(rest);
    assert!(after.trim().is_empty(), "nothing follows the root element: {after:?}");
    return root;
}

/// The element `text` begins with, and what follows it.
fn Element_At(text: &str) -> (Element, &str)
{
    let inside = text.strip_prefix('<').expect("an element begins with its opening tag");
    let (tag, mut rest) = inside.split_once('>').expect("an opening tag is closed");
    let is_self_closing = tag.ends_with('/');
    let (name, attributes) = Tag_Parts(tag.trim_end_matches('/'));
    let mut element = Element { name: name.to_owned(), attributes, ..Element::default() };
    if is_self_closing
    {
        return (element, rest);
    }

    let closing = format!("</{name}>");
    // Takes text and child elements in turn; ends at this element's closing tag, which a well-formed export
    // always writes.
    loop
    {
        let (between, from_tag) = rest.split_at(rest.find('<').expect("an open element is closed"));
        if !between.trim().is_empty()
        {
            element.text.push_str(&Unescaped(between));
        }
        if let Some(after) = from_tag.strip_prefix(closing.as_str())
        {
            return (element, after);
        }
        let (child, after) = Element_At(from_tag);
        element.children.push(child);
        rest = after;
    }
}

/// A tag's name and its attributes, every value unescaped.
fn Tag_Parts(tag: &str) -> (&str, BTreeMap<String, String>)
{
    let (name, mut rest) = tag.split_once(char::is_whitespace).unwrap_or((tag, ""));
    let mut attributes = BTreeMap::new();

    rest = rest.trim_start();
    while !rest.is_empty()
    {
        let (key, after_key) = rest.split_once("=\"").expect("an attribute is name=\"value\"");
        let (value, after_value) = after_key.split_once('"').expect("an attribute value is closed");
        attributes.insert(key.trim().to_owned(), Unescaped(value));
        rest = after_value.trim_start();
    }

    return (name, attributes);
}

/// `text` with the five predefined entities replaced by what they stand for; `&amp;` last, so an
/// escaped entity is not unescaped twice.
fn Unescaped(text: &str) -> String
{
    return text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&");
}

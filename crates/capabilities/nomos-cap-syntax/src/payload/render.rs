//! Writing a payload out, byte for byte the same every time.

use super::SyntaxPayload;
use core::fmt::Write as _;

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
pub(crate) fn Unescape(value: &str) -> String
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
pub(super) fn Push_Escaped(plain: &mut String, escaped: Option<char>)
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

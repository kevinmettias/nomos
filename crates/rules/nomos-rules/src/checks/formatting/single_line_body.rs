//! The one rule in this family that has to track state across lines.
//!
//! [`super`] states the family's shared reasoning and holds the four rules whose deciding
//! evidence is one line's own text. This rule's is not: a Rust function's signature, opening
//! brace, body and closing brace can only be told apart from a string literal or a doc comment
//! quoting the same shape by carrying the literal state a line ends in into the next one, so
//! the whole byte-walking machine lives here rather than beside rules that never need it.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::{Finding_For_Line, For_Each_Line_Number, NO_SINGLE_LINE_FUNCTION_BODIES};

/// Reports a Rust function whose signature, opening brace, body and closing brace all
/// collapse onto one line — including an empty `{}` trailing the signature, which carries
/// no body at all.
#[must_use]
pub fn Check_No_Single_Line_Function_Bodies(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Single_Line_Body_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}


fn Single_Line_Body_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut literal_state = RustLiteralState::None;

    For_Each_Line_Number(&source.text, |line_number, line| {
        let code = Advance_Literal_State(line, &mut literal_state);

        if Has_Single_Line_Function_Body(&code)
        {
            let finding = Finding_For_Line(
                source,
                NO_SINGLE_LINE_FUNCTION_BODIES,
                line_number,
                "collapses a function's signature, body and closing brace onto one line",
            );
            findings.push(finding);
        }
    });

    return findings;
}


/// The Rust string literal `line` is left inside at its own end, given whichever was open
/// when it began. A raw string's own hash count has to travel with the state because closing
/// one takes exactly that many `#` after the `"`, which a bare "am I in a string" bool cannot
/// carry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RustLiteralState
{
    /// No literal is open; the next line begins in code.
    None,
    /// A `"..."` (plain or `b"..."` byte-string — the `b` prefix takes no special handling,
    /// since it is ordinary code up to the `"` that actually opens the literal) literal is
    /// open. Backslash escapes inside these, including a trailing `\` that continues the
    /// literal onto the next line unclosed rather than escaping nothing.
    Double,
    /// A `r"..."`/`r#"..."#`/... literal is open, closing only on a `"` followed by exactly
    /// this many `#`. Raw strings take no escapes at all, not even of `"` itself.
    Raw(usize),
}


/// A `fn ` keyword followed, later in `code`, by a `{` and then a `}` — the shape a properly
/// multi-line (Allman-braced) function never has, since its own opening brace starts a new
/// line with nothing after it. Matches an empty `{}` too: the standard names that collapse
/// as a violation in its own right, not only a non-empty collapsed body. `code` is already
/// [`Code_Only`]'s output, not raw source text — this function never sees a string literal
/// or a comment, so it cannot mistake either for a real declaration.
fn Has_Single_Line_Function_Body(code: &[u8]) -> bool
{
    let Some(fn_index) = Index_Of_Subslice(code, b"fn ") else { return false };
    let after_fn = code.get(fn_index..).unwrap_or(&[]);
    let Some(open_brace) = after_fn.iter().position(|&byte| return byte == b'{') else { return false };
    let after_open = after_fn.get(open_brace.saturating_add(1)..).unwrap_or(&[]);
    return after_open.contains(&b'}');
}


/// The outcome of scanning one line for [`Code_Only`]: the line's own code bytes, with every
/// string literal and every `//`/`///` line comment blanked out, and the literal state left
/// open for whichever line comes next. Named so the caller reads what each part MEANS,
/// rather than juggling an unnamed `(Vec<u8>, RustLiteralState)` pair.
struct CodeOnlyResult
{
    /// `line`'s own code, blanked of every literal and comment — the bytes
    /// [`Has_Single_Line_Function_Body`] actually judges.
    code: Vec<u8>,
    /// The literal state `line` leaves open, carried into the next line's scan.
    next_state: RustLiteralState,
}


/// The index of `needle`'s first occurrence in `haystack`, or `None`.
fn Index_Of_Subslice(haystack: &[u8], needle: &[u8]) -> Option<usize>
{
    return haystack.windows(needle.len()).position(|window| return window == needle);
}


/// Scans one `line`, advancing `state` in place and returning its code bytes — the single
/// call site [`Code_Only`] has, so its own result never needs unpacking anywhere else.
pub(crate) fn Advance_Literal_State(line: &str, state: &mut RustLiteralState) -> Vec<u8>
{
    let result = Code_Only(line, *state);
    *state = result.next_state;
    return result.code;
}


/// `line`'s own code, with every string literal and every `//`/`///` line comment blanked
/// out — the bytes [`Has_Single_Line_Function_Body`] actually judges, so a `"fn a() {}"`
/// fixture string or a doc comment quoting the same shape is never mistaken for a real
/// declaration. Carries the literal state `line` leaves open for whichever line comes next,
/// which is how a raw string or a backslash-continued plain string spanning several physical
/// lines is still read as one literal rather than several unrelated ones.
fn Code_Only(line: &str, entering: RustLiteralState) -> CodeOnlyResult
{
    let bytes = line.as_bytes();
    let mut state = entering;
    let mut index = 0usize;
    let mut code = Vec::with_capacity(line.len());

    while let Some(&byte) = bytes.get(index)
    {
        match state
        {
            RustLiteralState::Double => index = Step_Inside_Double_Quote(index, byte, &mut state),
            RustLiteralState::Raw(hashes) => index = Step_Inside_Raw_String(bytes, index, hashes, &mut state),
            RustLiteralState::None => match Step_Outside_Literal(bytes, index, &mut code, &mut state)
            {
                Some(next_index) => index = next_index,
                None => return CodeOnlyResult { code, next_state: state },
            },
        }
    }

    return CodeOnlyResult { code, next_state: state };
}


/// Advances past one byte read while [`Code_Only`] is inside a `"..."` literal, closing it on
/// an unescaped `"`. Returns the next index; updates `*state` in place when the literal closes.
fn Step_Inside_Double_Quote(index: usize, byte: u8, state: &mut RustLiteralState) -> usize
{
    const ESCAPE_SEQUENCE_LENGTH: usize = 2; // the backslash plus the one character it escapes

    if byte == b'\\'
    {
        return index.saturating_add(ESCAPE_SEQUENCE_LENGTH); // the escaped character cannot close anything
    }
    if byte == b'"'
    {
        *state = RustLiteralState::None;
    }

    return index.saturating_add(1);
}


/// Advances past one byte read while [`Code_Only`] is inside a raw string opened with
/// `hashes` `#`s, closing it on a `"` followed by that many `#`s. Returns the next index;
/// updates `*state` in place when the literal closes.
fn Step_Inside_Raw_String(bytes: &[u8], index: usize, hashes: usize, state: &mut RustLiteralState) -> usize
{
    if Is_Quote_At(bytes, index) && Is_Closing_A_Raw_String(bytes, index, hashes)
    {
        *state = RustLiteralState::None;
        return index.saturating_add(1).saturating_add(hashes);
    }

    return index.saturating_add(1);
}


/// Whether `bytes[index]` is a `"`, read defensively since a caller only ever asks at an
/// index it already knows is in bounds.
fn Is_Quote_At(bytes: &[u8], index: usize) -> bool
{
    return bytes.get(index).copied() == Some(b'"');
}

// `'a'`: quote, one plain character, quote.
const PLAIN_CHAR_LITERAL_LENGTH: usize = 3;

// From the `x` marker in `'\xNN'` to the position right after its two hex digits.
const HEX_ESCAPE_MARKER_SPAN: usize = 3;

// `'\xNN'`: quote, backslash, `x`, two hex digits, quote.
const HEX_CHAR_LITERAL_LENGTH: usize = 6;

// `'\n'`-shaped: quote, backslash, one escape character, quote.
const ESCAPED_CHAR_LITERAL_LENGTH: usize = 4;


/// Whether the `"` at `bytes[index]` closes a raw string that opened with `hashes` `#`s —
/// the next `hashes` bytes must all be `#`.
fn Is_Closing_A_Raw_String(bytes: &[u8], index: usize, hashes: usize) -> bool
{
    return (0..hashes).all(|offset| return bytes.get(index.saturating_add(1).saturating_add(offset)) == Some(&b'#'));
}


/// Advances past one byte read while [`Code_Only`] is in [`RustLiteralState::None`] — the
/// four shapes that arm used to decide inline: a line comment, a char literal, a raw-string
/// opener, and a plain string opener, falling through to ordinary code otherwise. Extracted
/// so the dispatch on `state` in [`Code_Only`] stays a table of cases rather than a
/// procedure: what happens in [`RustLiteralState::None`] is now this one named function.
/// `code` collects the bytes that are not part of a comment or an opening literal. Returns
/// the next index to resume from, updating `*state` in place when a literal opens; `None`
/// means a `//` line comment began, so nothing after it on this line is code.
fn Step_Outside_Literal(bytes: &[u8], index: usize, code: &mut Vec<u8>, state: &mut RustLiteralState) -> Option<usize>
{
    let byte = bytes.get(index).copied().unwrap_or_default();
    if byte == b'/' && bytes.get(index.saturating_add(1)) == Some(&b'/')
    {
        return None;
    }
    if let Some(length) = Char_Literal_Length(bytes, index)
    {
        return Some(index.saturating_add(length)); // e.g. `'"'`: its own `"` opens nothing
    }
    if let Some(next_index) = Raw_String_Opening_Step(bytes, index, state)
    {
        return Some(next_index);
    }
    if byte == b'"'
    {
        *state = RustLiteralState::Double;
        return Some(index.saturating_add(1));
    }

    code.push(byte);
    return Some(index.saturating_add(1));
}


/// The byte length of the char (or byte-char) literal at `bytes[index]` — `'a'`, `'"'`,
/// `'\n'`, `'\''`, `'\x41'` — or `None` if `bytes[index]` is not `'` or what follows is not
/// one of those shapes. This crate's own `Quote_State_After` (`script_discipline.rs`) writes
/// `b'"'` to name the double-quote byte, and without this a scan in [`RustLiteralState::
/// None`] reads that `"` as opening a real string — the reason this exists at all.
///
/// Deliberately not a lifetime reader: `'a`, `'static` and `'_` all fail every branch below
/// (nothing closes them with a second `'`) and fall through as an ordinary byte, which is
/// the correct outcome — a lifetime's own apostrophe carries no quoting meaning to skip past.
/// A `\u{...}` escape is the one real char-literal shape this does not recognize, since its
/// length varies; nothing in this workspace's own measured false positives has that shape,
/// so the gap is left silent rather than guessed at.
fn Char_Literal_Length(bytes: &[u8], index: usize) -> Option<usize>
{
    if bytes.get(index) != Some(&b'\'')
    {
        return None;
    }

    let escape_or_char = index.saturating_add(1);
    if bytes.get(escape_or_char) != Some(&b'\\')
    {
        let closing = escape_or_char.saturating_add(1);
        return (bytes.get(escape_or_char).is_some() && bytes.get(closing) == Some(&b'\'')).then_some(PLAIN_CHAR_LITERAL_LENGTH);
    }

    let escaped = escape_or_char.saturating_add(1);
    if bytes.get(escaped) == Some(&b'x')
    {
        let closing = escaped.saturating_add(HEX_ESCAPE_MARKER_SPAN);
        return (bytes.get(closing) == Some(&b'\'')).then_some(HEX_CHAR_LITERAL_LENGTH);
    }

    let closing = escaped.saturating_add(1);
    return (bytes.get(escaped).is_some() && bytes.get(closing) == Some(&b'\'')).then_some(ESCAPED_CHAR_LITERAL_LENGTH);
}


/// The next index [`Step_Outside_Literal`] should resume from when `bytes[index]` opens a raw
/// string, updating `*state` to the state it opens in — `None` if it does not open one there.
fn Raw_String_Opening_Step(bytes: &[u8], index: usize, state: &mut RustLiteralState) -> Option<usize>
{
    const RAW_STRING_PREFIX_LENGTH: usize = 2; // `r` plus the opening `"`, before any `#`s

    let hashes = Raw_String_Opener_Hashes(bytes, index)?;
    *state = RustLiteralState::Raw(hashes);

    return Some(index.saturating_add(RAW_STRING_PREFIX_LENGTH).saturating_add(hashes));
}


/// The number of `#` between `bytes[index]` and the `"` that opens a raw string there
/// (`r"`, `r#"`, `r##"`, ...), or `None` if `bytes[index]` does not begin one. `r` must
/// start a word — otherwise an identifier merely ending in `r` would be misread as one.
fn Raw_String_Opener_Hashes(bytes: &[u8], index: usize) -> Option<usize>
{
    if bytes.get(index) != Some(&b'r')
    {
        return None;
    }
    if Is_Preceded_By_An_Identifier_Byte(bytes, index)
    {
        return None;
    }

    let hashes = Leading_Hash_Count(bytes, index.saturating_add(1));
    let quote_index = index.saturating_add(1).saturating_add(hashes);

    return (bytes.get(quote_index) == Some(&b'"')).then_some(hashes);
}


/// Whether `bytes[index]` is immediately preceded by an identifier byte — an `r` ending an
/// identifier (`foobar`) does not start a raw string, only one starting a word does.
fn Is_Preceded_By_An_Identifier_Byte(bytes: &[u8], index: usize) -> bool
{
    let previous_is_identifier_byte = bytes.get(index.saturating_sub(1)).is_some_and(|&before| return Is_Identifier_Byte(before));

    return index > 0 && previous_is_identifier_byte;
}


fn Is_Identifier_Byte(byte: u8) -> bool
{
    return byte.is_ascii_alphanumeric() || byte == b'_';
}


/// The number of consecutive `#` bytes starting at `start`.
fn Leading_Hash_Count(bytes: &[u8], start: usize) -> usize
{
    let mut hashes = 0usize;
    let mut cursor = start;
    while bytes.get(cursor) == Some(&b'#')
    {
        hashes = hashes.saturating_add(1);
        cursor = cursor.saturating_add(1);
    }

    return hashes;
}

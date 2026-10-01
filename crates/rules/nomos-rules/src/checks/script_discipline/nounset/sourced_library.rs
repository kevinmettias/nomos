//! Whether a shebang file is a sourced library -- one whose top-level statements only define
//! and never do -- with the quote-state tracking that keeps a multi-line literal from reading
//! as a statement.

/// A file meant to be *sourced* rather than executed: one whose body only defines —
/// functions and constants — and never *does*. The signal is the absence of any top-level
/// (column-zero) statement that runs something; the first one flips the file from library
/// to executed and ends the scan early.
///
/// `pub(super)` because the rule that reads it lives in `script_discipline/nounset.rs`, one
/// level up: the decision needs the whole line grammar below, and the rule does not.
pub(super) fn Is_Sourced_Library(source: &str) -> bool
{
    let mut saw_definition = false;
    let mut inside = QuoteState::None;

    for raw in source.split('\n')
    {
        let continuation = inside != QuoteState::None;
        inside = Quote_State_After(raw, inside);
        if continuation
        {
            // Inside a literal that opened on an earlier line: never a top-level statement.
            continue;
        }

        match Top_Level_Line_Verdict(raw)
        {
            TopLevelLine::NotStatement => continue,
            TopLevelLine::Definition => saw_definition = true,
            TopLevelLine::Statement => return false,
        }
    }

    return saw_definition;
}

/// Which kind of string literal, if any, is still open at the end of a line. A closed set
/// rather than two bools, because "inside single" and "inside double" are mutually
/// exclusive and two bools admit a fourth state that cannot exist.
#[derive(Clone, Copy, PartialEq, Eq)]
enum QuoteState
{
    /// No literal is open; the next line begins in code.
    None,
    /// A `'...'` literal is open. Shell gives single quotes no escape at all, so only the
    /// next `'` can close one.
    Single,
    /// A `"..."` literal is open. Backslash escapes inside these, so a `\"` does not close it.
    Double,
}

/// A `\x` escape consumes the backslash and the character after it -- two bytes, neither of
/// which can close or open a literal.
const ESCAPE_SEQUENCE_BYTES: usize = 2;

/// One step of [`Quote_State_After`]'s scan: either keep going from a new state and index, or
/// the line's unquoted `#` comment marker was found and the state at that point is final.
enum StepOutcome
{
    Continue { state: QuoteState, next_index: usize },
    StopAt(QuoteState),
}

/// Which literal, if any, is still open once `line` has been read, given whichever was open
/// when it began. Stops at an unquoted `#`, because the rest of the line is a comment and a
/// comment's contents are not code — without that, an ordinary apostrophe in prose
/// (`# don't do this`) would open a single-quoted string that never closes.
fn Quote_State_After(line: &str, opening: QuoteState) -> QuoteState
{
    let bytes = line.as_bytes();
    let mut state = opening;
    let mut index = 0usize;

    while let Some(&character) = bytes.get(index)
    {
        match Step_In_State(line, character, index, state)
        {
            StepOutcome::Continue { state: next_state, next_index } =>
            {
                state = next_state;
                index = next_index;
            }
            StepOutcome::StopAt(final_state) => return final_state,
        }
    }

    return state;
}

fn Step_In_State(line: &str, character: u8, index: usize, state: QuoteState) -> StepOutcome
{
    return match state
    {
        QuoteState::Single => Step_In_Single(character, index),
        QuoteState::Double => Step_In_Double(character, index),
        QuoteState::None => Step_In_Code(line, character, index),
    };
}

fn Step_In_Single(character: u8, index: usize) -> StepOutcome
{
    let next_state = if character == b'\'' { QuoteState::None } else { QuoteState::Single };
    return StepOutcome::Continue { state: next_state, next_index: index.saturating_add(1) };
}

fn Step_In_Double(character: u8, index: usize) -> StepOutcome
{
    if character == b'\\'
    {
        // the escaped character cannot close anything
        return StepOutcome::Continue { state: QuoteState::Double, next_index: index.saturating_add(ESCAPE_SEQUENCE_BYTES) };
    }

    let next_state = if character == b'"' { QuoteState::None } else { QuoteState::Double };
    return StepOutcome::Continue { state: next_state, next_index: index.saturating_add(1) };
}

fn Step_In_Code(line: &str, character: u8, index: usize) -> StepOutcome
{
    if character == b'\\'
    {
        // escapes the next character, quote or not
        return StepOutcome::Continue { state: QuoteState::None, next_index: index.saturating_add(ESCAPE_SEQUENCE_BYTES) };
    }
    if character == b'#' && Is_Starting_A_Word(line, index)
    {
        // a comment: nothing after it is code
        return StepOutcome::StopAt(QuoteState::None);
    }

    let next_state = match character
    {
        b'\'' => QuoteState::Single,
        b'"' => QuoteState::Double,
        _ => QuoteState::None,
    };
    return StepOutcome::Continue { state: next_state, next_index: index.saturating_add(1) };
}

/// Whether the byte at `index` starts a word — it is first on the line, or the character
/// before it is a space or a tab. Shell only treats `#` as a comment there.
fn Is_Starting_A_Word(line: &str, index: usize) -> bool
{
    if index == 0
    {
        return true;
    }
    return line
        .as_bytes()
        .get(index.saturating_sub(1))
        .is_some_and(|&byte| return byte == b' ' || byte == b'\t');
}

/// What a candidate top-level (column-zero, non-continuation) line amounts to: nothing worth
/// judging, a definition, or a real statement that flips the file from library to executed.
enum TopLevelLine
{
    NotStatement,
    Definition,
    Statement,
}

fn Top_Level_Line_Verdict(raw: &str) -> TopLevelLine
{
    use super::Code_On;

    if raw.starts_with(' ') || raw.starts_with('\t')
    {
        // Indented: a function body, a heredoc, or a nested block.
        return TopLevelLine::NotStatement;
    }

    let code = Code_On(raw).trim();
    return Code_Verdict(code);
}

/// What a top-level line's code (comment stripped, whitespace trimmed) amounts to, once the
/// caller already knows the line was not indented.
fn Code_Verdict(code: &str) -> TopLevelLine
{
    if code.is_empty()
    {
        return TopLevelLine::NotStatement;
    }

    let is_standalone_brace_or_paren = code == "{" || code == "}" || code == "(" || code == ")";
    if is_standalone_brace_or_paren
    {
        // A function body's braces, or a multi-line array's parentheses, standing alone.
        return TopLevelLine::NotStatement;
    }

    if Is_Function_Header(code) || Is_Declaration(code)
    {
        return TopLevelLine::Definition;
    }

    return TopLevelLine::Statement;
}

/// A top-level line that defines a function rather than runs a command: `name() {`, the
/// bare `name()`, or the `function name` spelling.
fn Is_Function_Header(code: &str) -> bool
{
    if code.starts_with("function ")
    {
        return true;
    }

    return Is_Bare_Function_Header(code);
}

/// `name() {` or the bare `name()`: no `function` keyword, just an identifier immediately
/// followed by an empty parameter list.
fn Is_Bare_Function_Header(code: &str) -> bool
{
    let Some(paren) = code.find('(')
    else
    {
        return false;
    };
    if paren == 0
    {
        return false;
    }

    let name = &code[..paren];
    if !Is_Valid_Function_Name(name)
    {
        return false;
    }

    return code[paren..].starts_with("()");
}

/// `name` carries no whitespace before `(` (a call or a subshell would) and is made only of
/// the characters a shell function name allows.
fn Is_Valid_Function_Name(name: &str) -> bool
{
    if name.trim() != name
    {
        // Whitespace before `(`: a call or a subshell, not a definition header.
        return false;
    }

    return name.chars().all(|symbol| return symbol.is_ascii_alphanumeric() || symbol == '_' || symbol == '-');
}

/// A top-level line that introduces a name without running anything: a `readonly`,
/// `declare`, `typeset`, `export` or `local` keyword, or a bare `NAME=value` (including
/// `NAME+=` and `NAME[i]=`).
fn Is_Declaration(code: &str) -> bool
{
    const KEYWORDS: [&str; 5] = ["readonly ", "declare ", "typeset ", "export ", "local "];
    if KEYWORDS.iter().any(|keyword| return code.starts_with(*keyword))
    {
        return true;
    }

    return Is_Bare_Assignment(code);
}

/// `NAME=value` (including `NAME+=` and `NAME[i]=`), with no keyword in front of it.
fn Is_Bare_Assignment(code: &str) -> bool
{
    let Some(equals) = code.find('=')
    else
    {
        return false;
    };
    if equals == 0
    {
        return false;
    }

    let name = Assigned_Name(&code[..equals]);
    if name.is_empty()
    {
        return false;
    }

    return Is_Valid_Variable_Name(name);
}

/// The bare variable name an assignment's left side names, once a trailing `+` (`NAME+=`)
/// and an index or key (`NAME[i]=`) are stripped off.
fn Assigned_Name(before_equals: &str) -> &str
{
    let mut name = before_equals.strip_suffix('+').unwrap_or(before_equals);
    if let Some(index) = name.find('[')
    {
        name = &name[..index];
    }
    return name;
}

/// A shell variable name: a leading letter or underscore, then letters, digits or
/// underscores.
fn Is_Valid_Variable_Name(name: &str) -> bool
{
    for (position, symbol) in name.chars().enumerate()
    {
        let letter = symbol.is_ascii_alphabetic();
        let is_invalid_leading_symbol = position == 0 && !(letter || symbol == '_');
        if is_invalid_leading_symbol
        {
            return false;
        }

        let is_invalid_symbol = !(letter || symbol == '_' || symbol.is_ascii_digit());
        if is_invalid_symbol
        {
            return false;
        }
    }

    return true;
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The decision `check-test-coverage` asks this file to name, and the one case
    /// `sourced_library.rs`'s own reading turns on: a literal's continuation line is not a
    /// top-level statement, while a real call at column zero is.
    #[test]
    fn Test_Is_Sourced_Library_Should_Track_A_Literal_Across_Lines()
    {
        let quoted = "describe() {\n printf '%s\\n' \"vendored grammar\"\n}\n";
        let called = "main() {\n exec ./bin/gate \"$@\"\n}\nmain \"$@\"\n";

        assert!(Is_Sourced_Library(quoted), "a body of definitions only is a library");
        assert!(!Is_Sourced_Library(called), "one top-level call makes it a program");
    }
}

//! The text-scanning kit `check-constant-scope` and `check-tuple-variant` both drive a
//! brace-delimited declaration scan with.
//!
//! Both rules read a file line by line: each looks for a declaration's own header, remembers
//! that header's name until the line that opens its block arrives, and then judges the lines
//! inside that block. Two things differ between them -- which keyword starts a header, and
//! what they judge once one is open -- and neither of those is a line predicate. Everything
//! that *is* one was written out twice before this file existed, and
//! `check-interfile-duplication` reported the pair.
//!
//! Held here rather than in either rule because neither owns these. A leaf reaching into a
//! sibling leaf for one would make the borrower structurally downstream of the lender over a
//! fact neither one decides -- the reason [`crate::checks::Relay_Findings`] and
//! [`crate::checks::Is_Test_Or_Example_Source`] sit at this level, and the same judgment
//! [`crate::checks::finding_shape`] records for the shape a rule's own verdict takes.
//!
//! What is deliberately *not* here is either rule's own header matcher. A `const`/`func`
//! header and an `enum` header are different shapes over different keywords, and each is
//! reached by exactly one scan.

/// A declaration block tracked line by line: the depth the scan has reached, the name of a
/// header whose opening brace it is still waiting for, and the depth that header's own block
/// turned out to open at.
///
/// `GoConstantScan` and `EnumScan` were this same three-field record with its middle field
/// spelled differently, and the function advancing each was this same function with that
/// spelling swapped -- which is the duplication, not the three field names. The Go rule's own
/// `in_const_block` deliberately stays with the Go rule: it is a question about `const`
/// blocks, which this type knows nothing about.
pub(in crate::checks) struct DeclarationBlock
{
    /// The brace depth the scan has reached.
    pub(in crate::checks) depth: usize,
    /// `(depth_at_open, name)` for the header whose block the scan is inside of, if any.
    pub(in crate::checks) open: Option<(usize, String)>,
    /// A header seen on a line that carried no `{` yet.
    pub(in crate::checks) pending: Option<String>,
}

impl DeclarationBlock
{
    /// A scan that has not started: no depth, no block open, nothing pending.
    pub(in crate::checks) fn New() -> Self
    {
        return Self { depth: 0, open: None, pending: None };
    }

    /// Advances the scan past `line`, answering whether the open block closed on it.
    ///
    /// A pending header becomes the open block only on the first line carrying a `{` after
    /// it, which is what makes `fn Name(\n  arg: u8,\n) {` and `fn Name() {` scan alike, and
    /// what keeps a `{` inside a signature's own generic list from opening a block early. The
    /// block closes on the first line whose depth has fallen back to the depth it opened at.
    /// A line with no block open does nothing and answers `false`.
    pub(in crate::checks) fn Advance(&mut self, line: &str) -> bool
    {
        let (opened, closed) = Brace_Delta(line);

        if let Some(name) = self.pending.take_if(|_| return opened > 0)
        {
            self.open = Some((self.depth, name));
        }

        self.depth = self.depth.saturating_add(opened);
        self.depth = self.depth.saturating_sub(closed);

        if self.open.as_ref().is_some_and(|(open_depth, _)| return self.depth <= *open_depth)
        {
            self.open = None;
            return true;
        }

        return false;
    }
}

/// A one-based line number for a zero-based line index.
pub(in crate::checks) fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// How far `line`'s own braces move the block depth, as the pair `(opened, closed)`.
///
/// Both rules count a line's braces rather than tracking them character by character, so a
/// brace inside a string literal or a character literal moves the depth the same way a real
/// one does. That is the reading each rule was ported with, and matching it is what keeps
/// this file from quietly changing where either rule thinks a block ends.
pub(in crate::checks) fn Brace_Delta(line: &str) -> (usize, usize)
{
    return (line.matches('{').count(), line.matches('}').count());
}

/// The identifier right after a candidate keyword occurrence at `[start, end)`, if `start`
/// sits at a word boundary and at least one whitespace character separates the keyword from
/// a non-empty run of identifier characters.
///
/// Borrows from `line` rather than owning it, so a caller that keeps the name for the rest of
/// its scan copies once, deliberately, at the point it decides to keep it.
pub(in crate::checks) fn Name_After_Keyword<'line>(
    line: &'line str,
    bytes: &[u8],
    start: usize,
    end: usize,
) -> Option<&'line str>
{
    if !Has_Left_Boundary(bytes, start)
    {
        return None;
    }

    let after = line.get(end..)?;
    if !after.starts_with(char::is_whitespace)
    {
        return None;
    }

    let trimmed = after.trim_start();
    let name_len = trimmed.find(|character: char| return !Is_Ident_Char(character)).unwrap_or(trimmed.len());
    if name_len == 0
    {
        return None;
    }

    return trimmed.get(..name_len);
}

/// Whether `start` sits at the left edge of a word: either nothing precedes it, or the byte
/// that does is not part of an identifier.
fn Has_Left_Boundary(bytes: &[u8], start: usize) -> bool
{
    return start
        .checked_sub(1)
        .and_then(|previous| return bytes.get(previous))
        .is_none_or(|&byte| return !Is_Ident_Byte(byte));
}

/// Whether `byte` can continue a Rust or Go identifier.
fn Is_Ident_Byte(byte: u8) -> bool
{
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// Whether `character` can continue a Rust or Go identifier.
pub(in crate::checks) fn Is_Ident_Char(character: char) -> bool
{
    return character.is_alphanumeric() || character == '_';
}

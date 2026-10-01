//! Where a C# line starts: in code, where a `#` opens a directive, or inside something that
//! spans lines, where it does not.
//!
//! A pre-processing directive is a line whose first non-blank character is `#` -- unless that
//! line sits inside a delimited comment, a verbatim string, a raw string, or an interpolation hole
//! that began on an earlier line, where the same characters are comment or string text. So the
//! compiled parts of a file have to be lexed far enough to know which of those is open at each
//! line's start, and no further: this lexer tracks exactly the constructs that can span a line and
//! the ones that can hide their openers (a `/*` inside a string opens nothing), and nothing else.
//!
//! Skipped sections are never fed to it. The compiler does not lex them -- inside a skipped
//! section any line starting with `#` is a directive, whatever text surrounds it -- and a lexer
//! that read them would let an apostrophe in skipped prose hide the `#endif` that ends it.

/// One construct open at a point in the text, innermost last on [`LineLexer`]'s stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Frame
{
    /// Code inside an interpolation hole: `depth` counts the braces the code itself has opened,
    /// and `closers` is how many `}` in a row close the hole -- one, or a raw string's `$` count.
    Hole { depth: usize, closers: usize },
    /// `"..."` or `$"..."`, which cannot span a line on its own.
    Regular { interpolated: bool },
    /// `@"..."` or `$@"..."`, which can.
    Verbatim { interpolated: bool },
    /// `"""..."""` with `quotes` quotes, interpolated with `dollars` braces when `dollars > 0`.
    Raw { quotes: usize, dollars: usize },
    /// `'...'`.
    Character,
    /// `/* ... */`.
    Comment,
}

/// The multi-line lexical state of a file's compiled text, advanced one compiled line at a time.
#[derive(Clone, Debug, Default)]
pub(crate) struct LineLexer
{
    stack: Vec<Frame>,
    /// Whether any token has been read yet -- after one, `#define` and `#undef` are errors.
    pub(crate) seen_token: bool,
}

impl LineLexer
{
    /// Whether a line starting now starts in code, where a leading `#` is a directive.
    pub(crate) fn Is_At_Code(&self) -> bool
    {
        return matches!(self.stack.last(), None | Some(Frame::Hole { .. }));
    }

    /// Advances over one compiled line that is not a directive.
    pub(crate) fn Read_Line(&mut self, line: &str)
    {
        let characters: Vec<char> = line.chars().collect();
        let mut at = 0usize;
        while at < characters.len()
        {
            at = self.Step(&characters, at);
        }

        // Neither a regular string nor a character literal spans a line; one left open is an
        // error the compiler reports, and the next line starts outside it.
        while matches!(self.stack.last(), Some(Frame::Regular { .. } | Frame::Character))
        {
            self.stack.pop();
        }
    }

    /// Reads from `at` and returns where the next step starts.
    fn Step(&mut self, characters: &[char], at: usize) -> usize
    {
        return match self.stack.last().copied()
        {
            None => self.Step_Code(characters, at, None),
            Some(Frame::Hole { depth, closers }) => self.Step_Code(characters, at, Some((depth, closers))),
            Some(Frame::Regular { interpolated }) => self.Step_Regular(characters, at, interpolated),
            Some(Frame::Verbatim { interpolated }) => self.Step_Verbatim(characters, at, interpolated),
            Some(Frame::Raw { quotes, dollars }) => self.Step_Raw(characters, at, quotes, dollars),
            Some(Frame::Character) => self.Step_Character(characters, at),
            Some(Frame::Comment) => self.Step_Comment(characters, at),
        };
    }

    fn Step_Code(&mut self, characters: &[char], at: usize, hole: Option<(usize, usize)>) -> usize
    {
        let current = Char_At(characters, at);
        let next = Char_At(characters, at.saturating_add(1));
        if current.is_whitespace()
        {
            return at.saturating_add(1);
        }
        if current == '/' && next == '/'
        {
            return characters.len();
        }
        if current == '/' && next == '*'
        {
            self.stack.push(Frame::Comment);
            return at.saturating_add(2);
        }

        self.seen_token = true;
        if let Some(after) = self.Open_String(characters, at)
        {
            return after;
        }
        if current == '\''
        {
            self.stack.push(Frame::Character);
            return at.saturating_add(1);
        }

        return self.Step_Brace(characters, at, hole);
    }

    /// Tracks a hole's own braces, and closes the hole on the brace run that ends it.
    fn Step_Brace(&mut self, characters: &[char], at: usize, hole: Option<(usize, usize)>) -> usize
    {
        let Some((depth, closers)) = hole
        else
        {
            return at.saturating_add(1);
        };

        match Char_At(characters, at)
        {
            '{' => self.Replace_Top(Frame::Hole { depth: depth.saturating_add(1), closers }),
            '}' if depth > 0 => self.Replace_Top(Frame::Hole { depth: depth.saturating_sub(1), closers }),
            '}' =>
            {
                self.stack.pop();
                return at.saturating_add(Run_Length(characters, at, '}').min(closers));
            }
            _ => {}
        }

        return at.saturating_add(1);
    }

    /// Opens a string literal starting at `at`, if one does: its `$` and `@` prefix, then its
    /// quotes. Returns where reading continues, or `None` when `at` starts no string.
    fn Open_String(&mut self, characters: &[char], at: usize) -> Option<usize>
    {
        let dollars = Run_Length(characters, at, '$');
        let mut cursor = at.saturating_add(dollars);
        let mut verbatim = false;
        if Char_At(characters, cursor) == '@'
        {
            verbatim = true;
            cursor = cursor.saturating_add(1);
        }
        let dollars = dollars.saturating_add(if verbatim { Run_Length(characters, cursor, '$') } else { 0 });
        cursor = at.saturating_add(dollars).saturating_add(usize::from(verbatim));
        if Char_At(characters, cursor) != '"'
        {
            return None;
        }

        let quotes = Run_Length(characters, cursor, '"');
        let interpolated = dollars > 0;
        if verbatim
        {
            self.stack.push(Frame::Verbatim { interpolated });
            return Some(cursor.saturating_add(1));
        }
        if quotes >= 3
        {
            self.stack.push(Frame::Raw { quotes, dollars });
            return Some(cursor.saturating_add(quotes));
        }
        if quotes == 2
        {
            return Some(cursor.saturating_add(2));
        }

        self.stack.push(Frame::Regular { interpolated });
        return Some(cursor.saturating_add(1));
    }

    fn Step_Regular(&mut self, characters: &[char], at: usize, interpolated: bool) -> usize
    {
        return match Char_At(characters, at)
        {
            '\\' => at.saturating_add(2),
            '"' =>
            {
                self.stack.pop();
                at.saturating_add(1)
            }
            '{' | '}' if interpolated => self.Step_Interpolation_Brace(characters, at, 1),
            _ => at.saturating_add(1),
        };
    }

    fn Step_Verbatim(&mut self, characters: &[char], at: usize, interpolated: bool) -> usize
    {
        return match Char_At(characters, at)
        {
            '"' if Char_At(characters, at.saturating_add(1)) == '"' => at.saturating_add(2),
            '"' =>
            {
                self.stack.pop();
                at.saturating_add(1)
            }
            '{' | '}' if interpolated => self.Step_Interpolation_Brace(characters, at, 1),
            _ => at.saturating_add(1),
        };
    }

    fn Step_Raw(&mut self, characters: &[char], at: usize, quotes: usize, dollars: usize) -> usize
    {
        let current = Char_At(characters, at);
        if current == '"'
        {
            let run = Run_Length(characters, at, '"');
            if run >= quotes
            {
                self.stack.pop();
            }
            return at.saturating_add(run);
        }
        if current == '{' && dollars > 0
        {
            return self.Step_Interpolation_Brace(characters, at, dollars);
        }

        return at.saturating_add(1);
    }

    /// A brace run inside an interpolated string: `openers` braces in a row open a hole, and any
    /// fewer are text. In a raw string the run's last `openers` braces open it.
    fn Step_Interpolation_Brace(&mut self, characters: &[char], at: usize, openers: usize) -> usize
    {
        let brace = Char_At(characters, at);
        let run = Run_Length(characters, at, brace);
        if brace == '{' && openers == 1 && run == 1
        {
            self.stack.push(Frame::Hole { depth: 0, closers: 1 });
            return at.saturating_add(1);
        }
        if brace == '{' && openers == 1
        {
            // `{{` is an escaped brace in a one-`$` string: two characters of text.
            return at.saturating_add(2.min(run));
        }
        if brace == '{' && run >= openers
        {
            self.stack.push(Frame::Hole { depth: 0, closers: openers });
            return at.saturating_add(run);
        }

        return at.saturating_add(run.max(1));
    }

    fn Step_Character(&mut self, characters: &[char], at: usize) -> usize
    {
        return match Char_At(characters, at)
        {
            '\\' => at.saturating_add(2),
            '\'' =>
            {
                self.stack.pop();
                at.saturating_add(1)
            }
            _ => at.saturating_add(1),
        };
    }

    fn Step_Comment(&mut self, characters: &[char], at: usize) -> usize
    {
        if Char_At(characters, at) == '*' && Char_At(characters, at.saturating_add(1)) == '/'
        {
            self.stack.pop();
            return at.saturating_add(2);
        }

        return at.saturating_add(1);
    }

    fn Replace_Top(&mut self, frame: Frame)
    {
        if let Some(top) = self.stack.last_mut()
        {
            *top = frame;
        }
    }
}

/// The character at `at`, or a NUL past the end -- which no rule above matches.
fn Char_At(characters: &[char], at: usize) -> char
{
    return characters.get(at).copied().unwrap_or('\0');
}

/// How many `wanted` characters run from `at`.
fn Run_Length(characters: &[char], at: usize, wanted: char) -> usize
{
    return characters.iter().skip(at).take_while(|character| return **character == wanted).count();
}

#[cfg(test)]
mod tests;

//! Whether a line carries a phrase or a ticket key, in `OD-POLICY-002` decision 2's vocabulary.
//!
//! A line is compared as characters, folded to lower case one character for one character so
//! that positions stay aligned with the original. There is no pattern language: a phrase is
//! its words in order, with any run of spaces, dots, underscores or hyphens between them, or
//! none at all.

use crate::party_policy::{Boundary, Is_Separator};

/// The smallest number of digits after a ticket key's hyphen.
const TICKET_DIGITS_AT_LEAST: usize = 2;

/// A line prepared once for every rule that reads it.
pub(crate) struct Line
{
    exact: Vec<char>,
    folded: Vec<char>,
}

impl Line
{
    pub(crate) fn Of(text: &str) -> Self
    {
        let exact: Vec<char> = text.chars().collect();
        let folded = exact.iter().map(|character| return Fold(*character)).collect();
        return Self { exact, folded };
    }

    /// Whether `words` occur, compared against the folded line when `ignore_case`.
    pub(crate) fn Has_Phrase(&self, words: &[Vec<char>], boundary: Boundary, ignore_case: bool) -> bool
    {
        let line = if ignore_case { &self.folded } else { &self.exact };
        return (0..line.len()).any(|start| return Phrase_At(line, start, words, boundary));
    }

    /// Whether `key-NN` occurs, bounded on both sides and case-sensitive.
    pub(crate) fn Has_Ticket(&self, key: &[char]) -> bool
    {
        return (0..self.exact.len()).any(|start| return Ticket_At(&self.exact, start, key));
    }
}

/// One character to lower case, keeping one character so positions do not shift.
pub(crate) fn Fold(character: char) -> char
{
    return character.to_lowercase().next().unwrap_or(character);
}

fn Phrase_At(line: &[char], start: usize, words: &[Vec<char>], boundary: Boundary) -> bool
{
    if boundary != Boundary::Anywhere && !Bounded_Before(line, start)
    {
        return false;
    }
    let mut position = start;
    for (index, word) in words.iter().enumerate()
    {
        if index > 0
        {
            position = Past_Separators(line, position);
        }
        match Past_Word(line, position, word)
        {
            Some(next) => position = next,
            None => return false,
        }
    }

    return boundary != Boundary::Word || Bounded_At(line, position);
}

fn Ticket_At(line: &[char], start: usize, key: &[char]) -> bool
{
    if !Bounded_Before(line, start)
    {
        return false;
    }
    let Some(after_key) = Past_Word(line, start, key) else { return false };
    if line.get(after_key) != Some(&'-')
    {
        return false;
    }
    let Some(first_digit) = after_key.checked_add(1) else { return false };
    let digits = line.get(first_digit..).map_or(0, |rest| return rest.iter().take_while(|character| return character.is_ascii_digit()).count());
    let Some(end) = first_digit.checked_add(digits) else { return false };

    return digits >= TICKET_DIGITS_AT_LEAST && Bounded_At(line, end);
}

/// The position after `word` if it starts at `position`.
fn Past_Word(line: &[char], position: usize, word: &[char]) -> Option<usize>
{
    let end = position.checked_add(word.len())?;
    return (line.get(position..end) == Some(word)).then_some(end);
}

fn Past_Separators(line: &[char], position: usize) -> usize
{
    let run = line.get(position..).map_or(0, |rest| return rest.iter().take_while(|character| return Is_Separator(**character)).count());
    return position.saturating_add(run);
}

/// Whether nothing alphanumeric sits just before `position`.
fn Bounded_Before(line: &[char], position: usize) -> bool
{
    return position.checked_sub(1).and_then(|before| return line.get(before)).is_none_or(|character| return !character.is_alphanumeric());
}

/// Whether nothing alphanumeric sits at `position`.
fn Bounded_At(line: &[char], position: usize) -> bool
{
    return line.get(position).is_none_or(|character| return !character.is_alphanumeric());
}

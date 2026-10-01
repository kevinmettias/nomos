//! What the work verbs promise, exercised.
//!
//! Split by the group each test drives -- the command line each verb parses, what the `--`
//! separator carries behind it, the labels `list` and `show` print, the two closed
//! vocabularies the printed usage text has to agree with, and the exit codes it documents --
//! because what a reader comes here to find is the group rather than the assertion. The two
//! translations more than one group needs, a command line into arguments and a printed
//! `a|b|c` run into words, live here beside the `mod` list.

use super::*;

mod exit_codes;
mod listing;
mod parsing;
mod separator;
mod vocabulary;

/// Splits a command line into the arguments a caller would hand the parser -- the one
/// translation every parsing test needs, so each case states its own command line as text.
fn Arguments(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}

/// The item an `add` line declares, with the parser's `Add` arm already unpacked. Two of
/// the groups below call it, so it lives here rather than with either of them.
fn Added(text: &str) -> LedgerItem
{
    return match Work_Command_From_String_Arguments(&Arguments(text))
        .expect("every caller hands this an add line, so the parser returns an Add command")
    {
        WorkCommand::Add { item, .. } => *item,
        // Every call site hands this an `add` line, so the variant is fixed by construction
        // and the only thing a fallible signature would buy is an `unwrap` at each of them.
        // The other variant is printed because the useful failure is which command the
        // parser chose instead — that names the flag it started reading differently.
        other => panic!("expected an add, got {other:?}"),
    };
}

/// A help text, and the `--flag` whose alternatives are wanted out of it.
///
/// One type rather than two adjacent `&str` positions: the text and the flag are both strings,
/// so a caller could transpose them and the search would look for a usage line inside a flag
/// name, find none, and return the empty list — which reads as "the text lists no
/// alternatives" rather than as the mistake it is.
#[derive(Clone, Copy)]
struct FlagAlternatives<'a>
{
    usage: &'a str,
    flag: &'a str,
}

/// The alternatives a `--flag a|b|c` segment of the help text lists.
///
/// `OD-AGENT-004` version 2 asks a printed vocabulary to be compared against its authority in
/// both directions. That comparison needs the printed side as data, and the printed side is a
/// pipe-joined run of words ending at the first whitespace — `[--state a|b]` closes with a
/// bracket, which comes off here so a caller compares words rather than punctuation.
///
/// The first occurrence wins, which is what the callers want: every flag below is spelled bare
/// in its own usage line and backticked (`` `--kind` ``) in the prose underneath, and the bare
/// one is the line a reader copies from.
fn Alternatives_After(question: FlagAlternatives<'_>) -> Vec<String>
{
    let Some((_, rest)) = question.usage.split_once(&format!("{} ", question.flag))
    else
    {
        return Vec::new();
    };
    let listed = rest.split_whitespace().next().unwrap_or_default().trim_end_matches(']');

    return listed.split('|').map(str::to_owned).collect();
}

/// A list of words in ascending order, so that two of them can be compared as sets.
fn Sorted_Words(words: impl Iterator<Item = String>) -> Vec<String>
{
    let mut sorted: Vec<String> = words.collect();
    sorted.sort();
    sorted.dedup();

    return sorted;
}

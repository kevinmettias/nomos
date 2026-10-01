//! The one vocabulary `--effort` prints and takes, judged from both ends: the usage text
//! against the spelling list, and the spelling list against an exhaustive match.

use super::super::Command_From_String_Arguments;
use super::execute_parsing::EFFORT_SPELLINGS;

/// The spelling `--effort` takes for a level, as an exhaustive match.
///
/// This is what closes [`EFFORT_SPELLINGS`] against its own authority. That array is six pairs
/// a person wrote, and `Test_An_Execute_Command_Parses_Every_Effort_Spelling` proves the parser
/// accepts all six -- neither says a seventh level would be noticed. A variant added to
/// `EffortLevel` fails this match to compile, which is the forcing function that brings an
/// author to the array beside it.
fn Spelled_Effort(effort: nomos_model_package::EffortLevel) -> &'static str
{
    return match effort
    {
        nomos_model_package::EffortLevel::BackendDefault => "backend-default",
        nomos_model_package::EffortLevel::Minimal => "minimal",
        nomos_model_package::EffortLevel::Low => "low",
        nomos_model_package::EffortLevel::Medium => "medium",
        nomos_model_package::EffortLevel::High => "high",
        nomos_model_package::EffortLevel::Maximum => "maximum",
    };
}

/// A list of words in ascending order, so that two of them can be compared as sets.
fn Sorted_Words(words: impl Iterator<Item = String>) -> Vec<String>
{
    let mut sorted: Vec<String> = words.collect();
    sorted.sort();
    sorted.dedup();

    return sorted;
}

/// The effort levels the usage text names, in the order it names them.
///
/// Read as prose rather than as a `a|b|c` run, because that is how this group writes it: a
/// comma-separated sentence ending in "or maximum", inside a paragraph. The words are located
/// by searching the whole text for each one rather than by parsing the sentence apart, so a
/// rewording of the sentence does not fail the test below for a reason that is not about the
/// vocabulary. What that costs is the reverse direction -- a stale word left in the prose would
/// still be found -- so the reverse direction is asserted separately, against
/// `UNRECOGNIZED_EFFORTS`'s own discipline: every level the text names must be one the command
/// line accepts.
///
/// The text is read through the refusal a verbless invocation gives rather than out of
/// `USAGE_TEXT`, which is private to `agent::parsing`: the text a user is actually shown is the
/// subject, and reaching it this way needs no widening of what this test can see.
fn Effort_Levels_Named_In_Usage() -> Vec<String>
{
    let usage = Command_From_String_Arguments(&[]).expect_err("no verb prints the usage text");
    let (_, spelled) = usage
        .split_once("--effort takes ")
        .expect("the usage text names what --effort takes");
    let sentence = spelled.split(" -- ").next().unwrap_or_default();
    let named: Vec<String> = sentence
        .split(|character: char| return character == ',' || character.is_whitespace())
        .map(str::trim)
        .filter(|word| return !word.is_empty() && *word != "or")
        .map(str::to_owned)
        .collect();

    assert!(
        !named.is_empty(),
        "no effort level was parsed out of the usage text, so this compared nothing: {sentence}"
    );

    return named;
}

/// The effort levels `--effort` prints are the levels it takes, both directions.
///
/// `OD-AGENT-004` version 2: a help text may enumerate a compiled vocabulary only where a test
/// compares that enumeration against the vocabulary's own authority. The help text calls this
/// one `MODEL-ROUTE-004`'s closed vocabulary in its own words and then lists it, and until this
/// test nothing compared the two.
#[test]
fn Test_The_Named_Effort_Levels_Should_Be_Every_Level_The_Command_Line_Takes()
{
    let named = Effort_Levels_Named_In_Usage();

    assert_eq!(
        Sorted_Words(named.into_iter()),
        Sorted_Words(EFFORT_SPELLINGS.iter().map(|(spelling, _)| return (*spelling).to_owned())),
        "the usage text and EFFORT_SPELLINGS disagree about what --effort takes"
    );

    for (spelling, level) in EFFORT_SPELLINGS
    {
        assert_eq!(
            Spelled_Effort(level),
            spelling,
            "EFFORT_SPELLINGS and the exhaustive match disagree about how {level:?} is spelled"
        );
    }
}

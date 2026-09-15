//! [`super::Work_Command_From_String_Arguments`] and [`super::Parse_Duration`], exercised.
//!
//! Split from `parse.rs` itself once that file passed the ~500-line review trigger --
//! `parse.rs` is the parsing logic, this is its own coverage, the same split this
//! workspace already keeps between `spec.rs` and `spec/tests.rs`.

use super::*;

#[test]
fn Test_Work_Command_From_String_Arguments_Should_Refuse_An_Unknown_Verb()
{
    let error = Work_Command_From_String_Arguments(&Arguments("frobnicate")).unwrap_err();

    assert!(error.contains("frobnicate"));
    assert!(error.contains("usage"));
}

fn Arguments(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}

/// The three leases the assertion below spells as text, in the seconds [`Parse_Duration`]
/// has to turn each into -- named so the test is about the conversion rather than about
/// the arithmetic a second time.
const TWO_HOURS_IN_SECONDS: u64 = 7_200;

/// Thirty minutes, which is `30m` below.
const THIRTY_MINUTES_IN_SECONDS: u64 = 1_800;

/// Forty-five seconds, which is `45s` below -- and the only unit the parse leaves as it is.
const FORTY_FIVE_SECONDS: u64 = 45;

#[test]
fn Test_Parse_Duration_Should_Convert_Hour_Minute_And_Second_Units()
{
    assert_eq!(Parse_Duration("2h").unwrap(), Duration::from_secs(TWO_HOURS_IN_SECONDS));
    assert_eq!(Parse_Duration("30m").unwrap(), Duration::from_secs(THIRTY_MINUTES_IN_SECONDS));
    assert_eq!(Parse_Duration("45s").unwrap(), Duration::from_secs(FORTY_FIVE_SECONDS));
}

/// The usage text is what an agent reads at exit 2, so a verb missing from it is a verb
/// that does not exist as far as the next session is concerned.
///
/// This used to iterate `Every_Verb`, a hand-written list of eleven names, asserting that
/// the usage text named each and that the parser did not reject it as unknown. Both of
/// those are real, and they are two directions between the *text* and the *parser* —
/// neither is tied to [`WorkCommand`], which is the thing that actually changes when a
/// verb is added. A verb reaching the dispatch match above and not that list was named by
/// nobody, and the suite stayed green. `OD-AGENT-004` version 2 states the condition this
/// now meets: enumerate a compiled vocabulary only where a test compares the enumeration
/// against its authority.
///
/// The two original claims are kept and a third is added. Verbs are read out of the usage
/// text rather than listed here, so the list *is* the text; each is driven through the real
/// parser; and the command each builds is named by [`Named`]'s wildcard-free match, so a
/// variant added to `WorkCommand` stops this file compiling.
#[test]
fn Test_Usage_Text_Should_Name_Every_Verb_It_Accepts()
{
    let named = Named_Verbs();

    assert!(
        !named.is_empty(),
        "no verb was parsed out of the usage text, so this compared nothing: {}",
        Usage_Text()
    );

    let mut built = Vec::new();
    for verb in &named
    {
        built.push(Built_From(verb));
    }

    built.sort_unstable();
    built.dedup();

    assert_eq!(built.len(), named.len(), "two verbs built the same command: {named:?}");
}

/// One verb from the usage text, driven through the real parser and named back.
///
/// The two failures are named separately because they are different defects: a verb the
/// text advertises with no fixture here, and a verb the parser will not accept. Both are
/// the usage text having grown ahead of the code under it.
fn Built_From(verb: &str) -> &'static str
{
    let Some(line) = Minimal_Line(verb)
    else
    {
        panic!("the usage text names `{verb}` and no minimal line is written for it here");
    };
    let command = Work_Command_From_String_Arguments(&Arguments(line))
        .unwrap_or_else(|error| panic!("the usage text names `{verb}` and the parser refuses it: {error}"));
    let spelled = Named(&command);

    assert_eq!(spelled, verb, "`{verb}` builds a different command");

    return spelled;
}

/// The verb each line of the usage text's verb block opens with.
///
/// A verb line carries exactly two leading spaces; every continuation line under one is
/// indented further, and the notes below the block are not indented at all. So the shape of
/// the block is what selects the verbs, and no second list of what counts as a verb line is
/// needed.
fn Named_Verbs() -> Vec<String>
{
    return Usage_Text()
        .lines()
        .filter(|line| return line.starts_with("  ") && !line.starts_with("   "))
        .filter_map(|line| return line.split_whitespace().next())
        .map(str::to_owned)
        .collect();
}

/// The command a verb builds, as an exhaustive match, so that adding one stops the build
/// here.
///
/// Matched on the variant rather than on the spelling, because the two are not one-to-one:
/// `takeover` is spelled without the capital [`WorkCommand::TakeOver`] carries, and `claim`,
/// `renew` and `takeover` share a parse arm while being three variants. A table keyed on
/// spellings would have to write that down twice.
fn Named(command: &WorkCommand) -> &'static str
{
    return match *command
    {
        WorkCommand::List { .. } => "list",
        WorkCommand::Show { .. } => "show",
        WorkCommand::Add { .. } => "add",
        WorkCommand::Finish { .. } => "finish",
        WorkCommand::Claim(_) => "claim",
        WorkCommand::Renew(_) => "renew",
        WorkCommand::TakeOver(_) => "takeover",
        WorkCommand::Abandon(_) => "abandon",
        WorkCommand::Decline(_) => "decline",
        WorkCommand::Widen { .. } => "widen",
        WorkCommand::Validate => "validate",
        WorkCommand::Audit => "audit",
    };
}

/// The shortest argument list each verb accepts, so the parser can be asked what it builds.
///
/// [`None`] for a verb with no line rather than a panic here, so the test reports *which*
/// verb the usage text grew without a fixture — the failure a new verb actually causes.
fn Minimal_Line(verb: &str) -> Option<&'static str>
{
    return match verb
    {
        "list" => Some("list"),
        "show" => Some("show --item T-1"),
        "add" => Some(
            "add --item T-1 --title t --why w --done-when d --kind correction \
                 --origin proposed --territory src/a.rs",
        ),
        "claim" => Some("claim --item T-1 --holder h"),
        "renew" => Some("renew --item T-1 --holder h"),
        "takeover" => Some("takeover --item T-1 --holder h"),
        "finish" => Some("finish --item T-1 --holder h"),
        "abandon" => Some("abandon --item T-1 --holder h --reason r"),
        "decline" => Some("decline --item T-1 --holder h --reason r"),
        "widen" => Some("widen --item T-1 --holder h --territory a.rs"),
        "validate" => Some("validate"),
        "audit" => Some("audit"),
        _ => None,
    };
}

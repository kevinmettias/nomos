//! What `nomos request` promises, exercised.

use super::*;
use nomos_spec_model::{Severity, SubmissionKind, SubmissionState};

#[test]
fn Test_Command_From_String_Arguments_Should_Parse_Its_Fields_And_Default_State_And_Version()
{
    let arguments = Arguments_From_Text(
        "submit --kind feature-request --id FR-100 --by kevin \
         --field title=t --field goal=g",
    );

    let Command::Submit(request) = Command_From_String_Arguments(&arguments)
        .expect("the arguments above are a complete submit line");

    assert_eq!(request.kind, SubmissionKind::FeatureRequest);
    assert_eq!(request.id, "FR-100");
    assert_eq!(request.by, "kevin");
    assert_eq!(request.state, SubmissionState::Draft);
    assert_eq!(request.contract_version, 1);
    assert_eq!(
        request.fields,
        vec![("title".to_owned(), "t".to_owned()), ("goal".to_owned(), "g".to_owned())]
    );
}

/// Every `--field` command line this crate refuses as a usage error -- a named provider so
/// another malformed `--field` scenario is an entry here, not a second copy of the test
/// below.
fn Malformed_Field_Command_Lines() -> Vec<&'static str>
{
    return vec!["submit --kind feature-request --id FR-101 --by kevin --field oops"];
}

#[test]
fn Test_A_Field_With_No_Equals_Should_Be_A_Usage_Error()
{
    for text in Malformed_Field_Command_Lines()
    {
        let arguments = Arguments_From_Text(text);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--field"), "{error}");
    }
}

/// Every `--kind` command line this crate refuses as a usage error -- a named provider so
/// another unrecognised `--kind` scenario is an entry here, not a second copy of the test
/// below.
fn Unrecognised_Kind_Command_Lines() -> Vec<&'static str>
{
    return vec!["submit --kind nonsense --id FR-102 --by kevin"];
}

#[test]
fn Test_Usage_Text_Should_Be_Appended_To_An_Unrecognised_Kinds_Refusal()
{
    for text in Unrecognised_Kind_Command_Lines()
    {
        let arguments = Arguments_From_Text(text);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--kind"), "{error}");
    }
}

#[test]
fn Test_A_Gap_Should_Parse_Its_Blocked_Fields_And_Severity()
{
    let arguments = Arguments_From_Text(
        "submit --kind feature-request --id FR-103 --by kevin \
         --gap which-substrate|behaviour,goal|blocking",
    );

    let Command::Submit(request) = Command_From_String_Arguments(&arguments)
        .expect("the arguments above are a complete submit line");

    assert_eq!(request.gaps.len(), 1);
    let gap = request.gaps.first().expect("the command line above carries one --gap");
    assert_eq!(gap.question, "which-substrate");
    assert_eq!(gap.blocks, vec!["behaviour".to_owned(), "goal".to_owned()]);
    assert_eq!(gap.severity, Severity::Blocking);
    assert!(gap.closed_by.is_none());
}

#[test]
fn Test_A_Parsed_Submission_Should_Carry_This_Transport_Name()
{
    let arguments = Arguments_From_Text(
        "submit --kind feature-request --id FR-104 --by kevin --field title=t",
    );

    let Command::Submit(request) = Command_From_String_Arguments(&arguments)
        .expect("the arguments above are a complete submit line");

    assert_eq!(request.submitted_through, "cli");
}

fn Arguments_From_Text(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}

/// A help text, and the `--flag` whose alternatives are wanted out of it.
///
/// One type rather than two adjacent `&str` positions: the text and the flag are both
/// strings, so a caller could transpose them and the search would look for a usage line
/// inside a flag name, find none, and return the empty list -- which reads as "the text lists
/// no alternatives" rather than as the mistake it is.
struct FlagAlternatives<'a>
{
    usage: &'a str,
    flag: &'a str,
}

/// The alternatives a `--flag a|b|c` segment of the help text lists.
///
/// The brackets and angle brackets a usage line wraps its alternatives in come off, so a caller
/// compares words rather than punctuation -- this group spells one of its two as
/// `<a|b|c>` and the other as `[--state a|b]`, and neither shape is about the vocabulary.
fn Alternatives_After(question: FlagAlternatives<'_>) -> Vec<String>
{
    let Some((_, rest)) = question.usage.split_once(&format!("{} ", question.flag))
    else
    {
        return Vec::new();
    };
    let listed = rest
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|character| return matches!(character, '[' | ']' | '<' | '>'));

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

/// The alternatives the help text lists for `flag`, refused when it lists none.
///
/// A text that spells no alternative for the flag would have the comparison below pass
/// against an empty set, and an empty set comparing equal to an empty set is what agreement
/// looks like -- so the emptiness is a failure here rather than a silent pass there.
fn Listed_Alternatives(flag: &str) -> Vec<String>
{
    let listed = Alternatives_After(FlagAlternatives { usage: &super::parsing::Usage_Text(), flag });

    assert!(
        !listed.is_empty(),
        "no {flag} alternative was parsed out of the usage text, so this compared nothing: {}",
        super::parsing::Usage_Text()
    );

    return listed;
}

/// Every kind a submission can declare.
///
/// `nomos_spec_model::submission::Kind`'s own file is not this item's territory, so the census
/// is here, paired with [`Spelled_Kind`]'s wildcard-free match: a variant added to
/// [`SubmissionKind`] fails this file to compile rather than to pass.
fn Every_Submission_Kind() -> &'static [SubmissionKind]
{
    return &[
        SubmissionKind::FeatureRequest,
        SubmissionKind::DesignSpec,
        SubmissionKind::FeatureResult,
    ];
}

/// The spelling `request submit --kind` takes for a kind, as an exhaustive match.
///
/// Not trusted from here: the test below parses each spelling back through the real command
/// line and asserts it produces the variant named beside it.
fn Spelled_Kind(kind: SubmissionKind) -> &'static str
{
    return match kind
    {
        SubmissionKind::FeatureRequest => "feature-request",
        SubmissionKind::DesignSpec => "design-spec",
        SubmissionKind::FeatureResult => "feature-result",
    };
}

/// Every state a submission can be in. [`Every_Submission_Kind`]'s reasoning, one set over.
fn Every_Submission_State() -> &'static [SubmissionState]
{
    return &[SubmissionState::Draft, SubmissionState::Accepted];
}

/// The spelling `request submit --state` takes for a state, as an exhaustive match.
fn Spelled_State(state: SubmissionState) -> &'static str
{
    return match state
    {
        SubmissionState::Draft => "draft",
        SubmissionState::Accepted => "accepted",
    };
}

/// A `--kind` spelling taken from the usage text, not from [`Spelled_Kind`]'s match.
///
/// Its own type so that [`Submit_Line`] cannot be handed the two spellings the wrong way
/// round: both are strings, a transposed line still parses, and the failure would surface as
/// an assertion about a different field than the one the swap happened in.
struct KindSpelling<'a>(&'a str);

/// A `--state` spelling, [`KindSpelling`]'s reasoning over the other position.
struct StateSpelling<'a>(&'a str);

/// A `submit` line carrying one `--kind` and one `--state`, so a spelling can be parsed back.
fn Submit_Line(kind: KindSpelling<'_>, state: StateSpelling<'_>) -> Vec<String>
{
    return Arguments_From_Text(&format!(
        "submit --kind {} --id FR-1 --by kevin --state {} --field title=t --field goal=g",
        kind.0, state.0
    ));
}

/// The submission the command line built.
///
/// An irrefutable binding rather than a match with a fallback arm: [`Command`] has exactly one
/// variant today, so a fallback would be unreachable code the compiler rejects. A second verb
/// added to this group turns this line into a compile error, which is the right place to be
/// told.
fn Submitted(arguments: &[String]) -> nomos_spec_orchestration::SubmitRequest
{
    let Command::Submit(request) = Command_From_String_Arguments(arguments)
        .expect("Submit_Line builds a complete submit line for every caller");

    return request;
}

/// The kinds `request submit` prints are the kinds a submission can declare, both directions.
///
/// `OD-AGENT-004` version 2: a help text may enumerate a compiled vocabulary only where a test
/// compares that enumeration against the vocabulary's own authority. The set comparison is one
/// direction each; the parse loop underneath is what makes the spellings in [`Spelled_Kind`]
/// trustworthy rather than self-agreeing, since a word that drifted from `parsing.rs`'s own
/// match would be refused there.
#[test]
fn Test_The_Listed_Submission_Kinds_Should_Be_Every_Kind_A_Submission_Can_Declare()
{
    let listed = Listed_Alternatives("--kind");

    assert_eq!(
        Sorted_Words(listed.iter().cloned()),
        Sorted_Words(Every_Submission_Kind().iter().map(|kind| return Spelled_Kind(*kind).to_owned())),
        "the usage text and SubmissionKind disagree about what --kind takes"
    );

    for spelling in &listed
    {
        let line = Submit_Line(KindSpelling(spelling), StateSpelling("draft"));
        let submitted = Submitted(&line);
        let named = Every_Submission_Kind()
            .iter()
            .find(|kind| return Spelled_Kind(**kind) == spelling.as_str())
            .expect("the set comparison above already established the spelling is one of these");

        assert_eq!(submitted.kind, *named, "--kind {spelling} does not parse to the kind spelled for it here");
    }
}

/// The states `request submit` prints are the states a submission can be in, both directions.
/// [`Test_The_Listed_Submission_Kinds_Should_Be_Every_Kind_A_Submission_Can_Declare`]'s
/// reasoning, over this group's other closed set.
#[test]
fn Test_The_Listed_Submission_States_Should_Be_Every_State_A_Submission_Can_Be_In()
{
    let listed = Listed_Alternatives("--state");

    assert_eq!(
        Sorted_Words(listed.iter().cloned()),
        Sorted_Words(Every_Submission_State().iter().map(|state| return Spelled_State(*state).to_owned())),
        "the usage text and SubmissionState disagree about what --state takes"
    );

    for spelling in &listed
    {
        let line = Submit_Line(KindSpelling("feature-request"), StateSpelling(spelling));
        let submitted = Submitted(&line);
        let named = Every_Submission_State()
            .iter()
            .find(|state| return Spelled_State(**state) == spelling.as_str())
            .expect("the set comparison above already established the spelling is one of these");

        assert_eq!(submitted.state, *named, "--state {spelling} does not parse to the state spelled for it here");
    }
}

/// Every code this group can leave the process with.
///
/// `request::ExitCode` carries no census of its own the way `check::ExitCode::All()` does,
/// and its declaration is not this item's territory, so the census is here. [`Labelled`]'s
/// match has no wildcard arm, so a variant added to [`ExitCode`] fails this file to
/// *compile* rather than to pass — that is the forcing function that brings an author to
/// this array in the same edit. It is honestly weaker than a census living beside the
/// declaration, and it is what a test file can do without editing what it judges.
fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[
        ExitCode::Ok,
        ExitCode::Usage,
        ExitCode::StoreError,
        ExitCode::Unwritable,
        ExitCode::Refused,
    ];
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Usage => "Usage",
        ExitCode::StoreError => "StoreError",
        ExitCode::Unwritable => "Unwritable",
        ExitCode::Refused => "Refused",
    };
}

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// The codes this group's help text documents are the codes this group can exit with.
///
/// The same comparison `check` and `gate` have each carried for a while, against this
/// group's own enum. Eight groups print an exit-code list and only those two mirrored it;
/// the other six were correct rather than guarded, which is a different thing, and
/// `OD-AGENT-004`'s amendment says a printed vocabulary is admissible only where a test
/// compares it against its authority. The usage text is prose a person reads and
/// [`ExitCode`] is what the process returns, the two were written separately, and a code
/// added or renumbered in one of them and not the other is the failure that actually
/// happens.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let usage = Usage_Text_Of_This_Group();

    let (_, spelled) = usage
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok()));
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| return code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );
    assert_eq!(
        documented,
        implemented,
        "the usage text and ExitCode disagree about what this command can exit with; the \
         enum declares {:?}",
        Every_Exit_Code().iter().map(|code| return Labelled(*code)).collect::<Vec<_>>()
    );
}

/// This group's own help text, refused when it turns out to be some other group's.
///
/// Eight groups print a usage text and this file guards one of them. A `Usage_Text()` that
/// returned another group's prose would have every comparison below pass against the wrong
/// authority -- a failure that reads exactly like success -- so the identity of the text is
/// asserted before anything is read out of it.
fn Usage_Text_Of_This_Group() -> String
{
    let usage = super::parsing::Usage_Text();

    assert!(
        usage.starts_with("usage: nomos request"),
        "this compared some other group's help text: {usage}"
    );

    return usage;
}

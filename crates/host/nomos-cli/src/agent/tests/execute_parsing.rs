//! `execute`'s own command line: every spelling each of its flags takes, and every word
//! each of them refuses.

use super::super::{Backend, Command, Command_From_String_Arguments};
use super::Arguments;

#[test]
fn Test_An_Execute_Command_Should_Parse_Its_Goal()
{
    let arguments = Arguments("execute --goal hello");

    let Command::Execute { goal, effort, preferred } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(goal, "hello");
    assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
    assert_eq!(preferred, None, "no backend flag was given, so nothing is preferred");
}

/// The default is `BackendDefault`, the one value `Effort_Flag` maps to "omit the flag
/// entirely" -- an `execute` call with no `--effort` must reach the subprocess exactly
/// as it did before this flag existed.
#[test]
fn Test_An_Execute_Command_With_No_Effort_Defaults_To_Backend_Default()
{
    let arguments = Arguments("execute --goal hello");

    let Command::Execute { effort, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
}

#[test]
fn Test_An_Execute_Command_Should_Parse_Its_Effort()
{
    let arguments = Arguments("execute --goal hello --effort high");

    let Command::Execute { effort, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(effort, nomos_model_package::EffortLevel::High);
}

/// `MODEL-ROUTE-004`'s six spellings, named once so a case is a diff to this list and not
/// to the test that reads it. `agent::tests::effort_usage` reads the same list, because the
/// usage text has to name exactly these -- one authority, two judgments about it.
pub(super) const EFFORT_SPELLINGS: [(&str, nomos_model_package::EffortLevel); 6] = [
    ("backend-default", nomos_model_package::EffortLevel::BackendDefault),
    ("minimal", nomos_model_package::EffortLevel::Minimal),
    ("low", nomos_model_package::EffortLevel::Low),
    ("medium", nomos_model_package::EffortLevel::Medium),
    ("high", nomos_model_package::EffortLevel::High),
    ("maximum", nomos_model_package::EffortLevel::Maximum),
];

/// All six of `MODEL-ROUTE-004`'s spellings, not just one -- the same universe-closing
/// discipline `nomos-model-package::effort_level`'s own `Test_Every_Value_Is_In_The_Tested_Universe`
/// holds itself to.
#[test]
fn Test_An_Execute_Command_Parses_Every_Effort_Spelling()
{
    for (spelling, expected) in EFFORT_SPELLINGS
    {
        let arguments = Arguments(&format!("execute --goal hello --effort {spelling}"));

        let Command::Execute { effort, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

        assert_eq!(effort, expected, "spelling {spelling}");
    }
}

/// Spellings `--effort` refuses -- an unknown word, and a real value cased wrong, since a
/// closed vocabulary that matched case-insensitively would silently accept a spelling
/// `Usage_Text` never documents.
const UNRECOGNIZED_EFFORTS: [&str; 2] = ["superhuman", "LOW"];

#[test]
fn Test_An_Unrecognized_Effort_Should_Be_A_Usage_Error()
{
    for effort in UNRECOGNIZED_EFFORTS
    {
        let arguments = Arguments(&format!("execute --goal hello --effort {effort}"));

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--effort"), "{effort}: {error}");
        assert!(error.contains(effort), "{effort}: {error}");
    }
}

/// With neither flag, the command carries no preference at all.
///
/// It used to carry `Backend::ClaudeCode`, and that constant was the defect
/// `OD-PACKAGE-016` decision 9 is about: a dispatch target chosen by this parser rather than
/// resolved against anything, so a build whose declared set had dropped Claude Code would
/// still have dispatched to it. `None` here is not a missing value -- it is the absence of a
/// preference, which is what lets the declared profile resolve and answer for a reason.
///
/// Where the backend now comes from is asserted in `nomos-agent-orchestration`, by
/// `Test_A_Profile_With_No_Preference_Should_Still_Reach_A_Backend`. This end only has to
/// stop choosing.
// test-data: allow this pins the single, fixed scenario of omitting both flags; there is
// no second "no backend given" input to tabulate against it.
#[test]
fn Test_An_Execute_Command_With_No_Flag_Should_Carry_No_Preference()
{
    let arguments = Arguments("execute --goal hello");

    let Command::Execute { preferred, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(preferred, None, "this parser must not choose a backend");
}

// test-data: allow `claude-code` is the only spelling `--executor` accepts today --
// `OD-EXECUTOR-005` found only one real `AgentExecutor` exists in this workspace, so a
// second case cannot exist until a second one does.
#[test]
fn Test_An_Execute_Command_Parses_Executor_Claude_Code()
{
    let arguments = Arguments("execute --goal hello --executor claude-code");

    let Command::Execute { preferred, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(preferred.as_deref(), Some("claude-code"));
}

#[test]
fn Test_An_Execute_Command_Parses_Model_Backend_Ollama()
{
    let arguments = Arguments("execute --goal hello --model-backend ollama");

    let Command::Execute { preferred, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(preferred.as_deref(), Some("ollama"));
}

/// Values `--executor` refuses -- a name with no meaning at all, and `ollama`, which is a
/// real spelling but belongs to `--model-backend`, not this flag.
const UNRECOGNIZED_EXECUTORS: [&str; 2] = ["gpt5", "ollama"];

#[test]
fn Test_An_Unrecognized_Executor_Should_Be_A_Usage_Error()
{
    for executor in UNRECOGNIZED_EXECUTORS
    {
        let arguments = Arguments(&format!("execute --goal hello --executor {executor}"));

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--executor"), "{executor}: {error}");
        assert!(error.contains(executor), "{executor}: {error}");
    }
}

/// Values `--model-backend` refuses -- a name with no meaning at all, and `claude-code`,
/// which is a real spelling but belongs to `--executor`, not this flag.
const UNRECOGNIZED_MODEL_BACKENDS: [&str; 2] = ["gpt5", "claude-code"];

#[test]
fn Test_An_Unrecognized_Model_Backend_Should_Be_A_Usage_Error()
{
    for model_backend in UNRECOGNIZED_MODEL_BACKENDS
    {
        let arguments = Arguments(&format!("execute --goal hello --model-backend {model_backend}"));

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--model-backend"), "{model_backend}: {error}");
        assert!(error.contains(model_backend), "{model_backend}: {error}");
    }
}

/// `--executor` naming an `AgentExecutor` and `--model-backend` naming a `ModelBackend`
/// at once is not a request either flag alone could satisfy -- a call dispatches to
/// exactly one backend, so both present is refused rather than one silently winning.
// test-data: allow the refusal fires on both flags being present at all, regardless of
// which values they carry, so a second case would vary nothing this assertion depends on.
#[test]
fn Test_Both_Executor_And_Model_Backend_Together_Should_Be_A_Usage_Error()
{
    let arguments = Arguments("execute --goal hello --executor claude-code --model-backend ollama");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--executor"), "{error}");
    assert!(error.contains("--model-backend"), "{error}");
}

/// Arguments to `execute` that never supply `--goal` -- bare, and with an unrelated flag
/// present, so the check does not depend on `--goal` being the only thing missing.
const MISSING_GOAL_ARGUMENTS: [&str; 2] = ["execute", "execute --effort high"];

#[test]
fn Test_A_Missing_Goal_Should_Be_A_Usage_Error()
{
    for text in MISSING_GOAL_ARGUMENTS
    {
        let arguments = Arguments(text);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--goal"), "{text}: {error}");
    }
}

/// Verbs `Command_From_String_Arguments`'s own top-level `match` on the first argument
/// names neither.
const UNKNOWN_VERBS: [&str; 2] = ["dance", "build"];

#[test]
fn Test_Command_From_String_Arguments_Should_Refuse_An_Unknown_Verb()
{
    for verb in UNKNOWN_VERBS
    {
        let arguments = Arguments(verb);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains(verb), "{verb}: {error}");
    }
}

#[test]
fn Test_No_Verb_At_All_Should_Be_A_Usage_Error()
{
    let error = Command_From_String_Arguments(&[]).expect_err("must refuse");

    assert!(error.contains("usage"));
}

/// The one place `--goal ""` reaches this crate's tests without spending a real
/// invocation: parsing accepts an empty string exactly as it accepts any other, and
/// whatever `nomos-agent-executor-claude-code` or the real CLI does with it is that crate's own
/// concern, verified there, not re-verified through this transport.
// test-data: allow this pins the single scenario where --goal is the last token with
// nothing following it; a value placed after it is read as that value, not as a second
// case of "no value", and the "no --goal at all" cases are already tabulated in
// Test_A_Missing_Goal_Should_Be_A_Usage_Error.
#[test]
fn Test_An_Empty_Goal_Still_Parses()
{
    let arguments = Arguments("execute --goal");
    let error = Command_From_String_Arguments(&arguments).expect_err("a flag with nothing after it has no value");

    assert!(error.contains("--goal"));
}

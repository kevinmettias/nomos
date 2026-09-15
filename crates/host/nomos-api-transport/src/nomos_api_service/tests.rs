//! What this crate still decides, now that the envelope is the engine's.
//!
//! # Why the envelope is no longer asserted here
//!
//! It is not this crate's any more. That a malformed line is a parse error, that
//! another protocol version is refused, that an id comes back untouched, that a
//! blank line is skipped, that a real request crosses a real socket -- all of it
//! moved down with the transport and is asserted in `xvpe-remote-call-backend-json`'s own
//! suite. A second copy here would be two suites drifting over one behaviour,
//! and the one further from the code would be the one that lied.
//!
//! What remains is the half that is genuinely this workspace's: which verbs are
//! served, which are refused, and what each one makes of its arguments.

use super::*;
use crate::test_support::{At, Count_At};
use serde_json::Value;

/// An answer this service produced, parsed.
///
/// Reached through the strategy surface rather than through a wire line: the
/// line is the engine's now, and a test that built one here would be exercising
/// the engine's parser to reach this crate's dispatch.
fn Answered(method: ServedMethod, parameters: &str) -> RemoteCallOutcome
{
    return NomosApiService.Answer(method.Name(), parameters);
}

/// The document an answered outcome carries.
fn Document(outcome: &RemoteCallOutcome) -> Value
{
    let RemoteCallOutcome::Answered(ref result) = *outcome
    else
    {
        panic!("this operation answers rather than refusing");
    };

    return serde_json::from_str(result).expect("an answer is a JSON document");
}

/// The code a refused outcome carries.
fn Refusal_Code(outcome: &RemoteCallOutcome) -> i32
{
    let RemoteCallOutcome::Refused(ref refusal) = *outcome
    else
    {
        panic!("this call is refused rather than answered");
    };

    return refusal.code;
}

/// What is served is exactly what `ServedMethod::REGISTRY` admits -- every entry served, and
/// no more than the entries -- whatever length that list happens to be.
///
/// The assertion is deliberately against the registry's own length rather than a literal, so
/// adding a verb there is enough to extend what this test demands. Naming a number here would
/// pin the one thing the code was written not to depend on, and would go stale silently while
/// the assertion stayed correct: it already did once, when `P101` added a fifth verb for gate
/// compare and only the label was wrong.
#[test]
fn Test_The_Served_Methods_Should_Be_Exactly_The_Admitted_Registry()
{
    let served = NomosApiService.Served_Methods();

    assert_eq!(served.len(), ServedMethod::REGISTRY.len(), "{served:?}");
    for method in ServedMethod::REGISTRY
    {
        assert!(served.contains(&method.Name()), "{served:?}");
    }
}

/// Every repo-tooling handler `nomos-api` exports is outside the registry, so the
/// engine refuses it before this crate is ever asked.
///
/// The subject is the registry rather than a wire answer: what `OD-HOST-007`
/// decided is what this surface *claims to serve*, and that claim is now one
/// list in one place.
#[test]
fn Test_No_Repo_Tooling_Operation_Should_Be_In_The_Registry()
{
    let served = NomosApiService.Served_Methods();

    for method in ["nomos.work.finish", "nomos.spec.commit", "nomos.work.list"]
    {
        assert!(!served.contains(&method), "{method} must not be served");
    }
}

/// Arguments of the wrong shape are refused as invalid parameters, which is a
/// different answer from an unknown operation and from malformed JSON.
#[test]
fn Test_Arguments_Of_The_Wrong_Shape_Should_Be_Invalid_Parameters()
{
    let outcome = Answered(ServedMethod::GateRun, r#"{"root":[]}"#);

    assert_eq!(Refusal_Code(&outcome), RemoteCallRefusal::INVALID_PARAMETERS);
}

/// An explain naming no finding is refused before anything is judged -- the
/// required arguments are the parameter type's own, so the refusal costs no walk.
#[test]
fn Test_An_Explain_Naming_No_Finding_Should_Be_Invalid_Parameters()
{
    let outcome = Answered(ServedMethod::GateExplain, "{}");

    assert_eq!(Refusal_Code(&outcome), RemoteCallRefusal::INVALID_PARAMETERS);
}

/// A real call with no arguments at all reaches a real answer: the engine hands
/// an empty object to an operation the caller sent nothing for, and
/// `nomos.gate.plan` takes none anyway.
#[test]
fn Test_A_Plan_With_No_Arguments_Should_Reach_A_Real_Registry()
{
    let outcome = Answered(ServedMethod::GatePlan, "{}");

    let result = Document(&outcome);
    assert_eq!(At(&result, "/outcome"), "planned", "{result}");
    assert!(Count_At(&result, "/rules") > 0, "{result}");
}

/// A real correction run over a fixture tree with no blocking claim reaches a
/// real `clean` answer -- proving this service, not only `nomos-api` directly,
/// can reach `Handle_Correction_Run`.
#[test]
fn Test_A_Correction_Run_Over_A_Clean_Tree_Should_Reach_A_Real_Clean_Answer()
{
    let root = std::env::temp_dir().join("nomos-api-transport-correction-run-clean");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("creates a fresh directory");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n")
        .expect("create_dir_all above made this directory on an empty path");

    let parameters = serde_json::json!({ "root": root.display().to_string() }).to_string();
    let outcome = Answered(ServedMethod::Correction, &parameters);

    let _ignored = std::fs::remove_dir_all(&root);
    let result = Document(&outcome);
    assert_eq!(At(&result, "/outcome"), "clean", "{result}");
}

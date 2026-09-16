//! What this crate still decides, now that the handshake is the engine's.
//!
//! # Why the protocol is no longer asserted here
//!
//! It is not this crate's any more. That `initialize` declares one capability
//! and no others, that a notification receives no answer, that an unknown tool
//! is refused while a failing tool answers with `isError`, that a listing
//! carries a schema as a document and not a string, that a blank line is
//! skipped -- all of it moved down with the handshake and is asserted in
//! `xvpe-remote-call-backend-json`'s own suite. A second copy here would be two suites
//! drifting over one behaviour.
//!
//! What remains is the half that is genuinely this workspace's: which tools
//! exist, and where a call to one actually lands.

use super::*;
use crate::test_support::Field_At;

/// A call must reach the real registry rather than a description of it.
const CALL_REACHES_THE_REGISTRY: &str = "a call reaches the handler behind the tool";
/// A tool that refuses must come back as a failure, not as a produced answer.
const REFUSAL_BECOMES_A_FAILURE: &str = "a refused call comes back as a failure";
/// The catalogue is exactly the four served verbs.
const CATALOGUE_IS_THE_REGISTRY: &str =
    "the catalogue offers exactly the operations the transport serves";

#[test]
fn Test_The_Catalogue_Should_Offer_Exactly_The_Served_Operations()
{
    let offered = NomosToolCatalog.Tools();
    let served = NomosApiDispatch.Served_Methods();

    assert_eq!(offered.len(), served.len(), "{CATALOGUE_IS_THE_REGISTRY}");
    for tool in &offered
    {
        // A tool name *is* a served method name, which is what keeps this crate
        // from being able to reach anything that crate does not already serve.
        assert!(served.contains(&tool.name.as_str()), "{CATALOGUE_IS_THE_REGISTRY}");
    }
}

#[test]
fn Test_This_Server_Should_Name_Itself_And_Its_Own_Package_Version()
{
    let identity = NomosToolCatalog.Identity();

    assert_eq!(identity.name, SERVER_NAME);
    assert_eq!(identity.version, env!("CARGO_PKG_VERSION"));
    assert!(!identity.version.is_empty());
}

#[test]
fn Test_A_Call_Should_Reach_The_Real_Registry()
{
    let answer = NomosToolCatalog.Call_With_Json_Arguments(ServedTool::REGISTRY[0].Name(), "{}");

    assert!(!answer.Is_A_Failure(), "{CALL_REACHES_THE_REGISTRY}");
    let document: serde_json::Value =
        serde_json::from_str(answer.Text()).expect("a produced answer is a JSON document");
    assert_eq!(Field_At(&document, "/outcome"), "planned", "{CALL_REACHES_THE_REGISTRY}");
}

#[test]
fn Test_A_Call_With_Unusable_Arguments_Should_Come_Back_As_A_Failure()
{
    // The schema for this tool requires two fields. Sending neither is a real
    // refusal from the handler's own parameter type, and it must arrive as a
    // failed tool rather than as a successfully produced answer.
    let answer = NomosToolCatalog.Call_With_Json_Arguments("nomos.gate.explain", "{}");

    assert!(answer.Is_A_Failure(), "{REFUSAL_BECOMES_A_FAILURE}");
    assert!(!answer.Text().is_empty(), "{REFUSAL_BECOMES_A_FAILURE}");
}

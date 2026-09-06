//! One line in, one answer out -- and the four ways a line does not become a call.

use crate::{CorrectionParameters, FindingParameters, GateParameters, ServedMethod, WireError, WireRequest, WireResponse};
use serde::Serialize;
use serde_json::Value;

/// The answer to one JSON-RPC line.
///
/// Total by construction: every failure between here and a handler becomes a [`WireResponse`]
/// carrying a reserved code, never a panic and never a dropped connection. A transport that
/// closes the socket on a malformed request tells a client nothing it can act on, and this
/// crate is reached by clients it did not compile against.
#[must_use]
pub fn Answer(line: &str) -> WireResponse
{
    let request: WireRequest = match serde_json::from_str(line)
    {
        Ok(request) => request,
        // The id is not recoverable from a line that did not parse, so the answer carries
        // null -- which is what the specification says a parse error's id is.
        Err(error) => return WireResponse::Refusing(Value::Null, WireError::New(WireError::PARSE_ERROR, error.to_string())),
    };

    if request.jsonrpc != WireResponse::VERSION
    {
        let claimed = request.jsonrpc.clone();
        return WireResponse::Refusing(request.id, WireError::New(WireError::INVALID_REQUEST, format!("this transport speaks JSON-RPC {}, not `{claimed}`", WireResponse::VERSION)));
    }

    let Some(method) = ServedMethod::Named(&request.method)
    else
    {
        let unknown = request.method.clone();
        return WireResponse::Refusing(request.id, WireError::New(WireError::METHOD_NOT_FOUND, format!("`{unknown}` is not an operation this transport serves; it serves {}", Served_Names())));
    };

    return Answered(method, request);
}

/// The registry as a caller-readable list, for the one message that has to name it.
fn Served_Names() -> String
{
    let names: Vec<&str> = ServedMethod::REGISTRY.iter().map(|method| return method.Name()).collect();

    return names.join(", ");
}

/// The operation `method` names, called with the arguments `request` carried.
fn Answered(method: ServedMethod, request: WireRequest) -> WireResponse
{
    let WireRequest { id, parameters, .. } = request;

    return match method
    {
        // `Handle_Gate_Plan` takes no argument, by its own doc: `Plan` reports what the rule
        // registry holds and does not walk `root`, so there is nothing here for a caller's
        // own arguments to supply. Arguments are ignored rather than refused for that reason -- a caller
        // sending the same object it sends to `nomos.gate.run` is not making a mistake this
        // transport should fail.
        ServedMethod::GatePlan => Serialized(id, &nomos_api::Handle_Gate_Plan()),
        ServedMethod::GateRun => match Parsed::<GateParameters>(parameters)
        {
            Ok(parameters) => Serialized(id, &nomos_api::Handle_Gate_Run(&parameters.Command())),
            Err(error) => WireResponse::Refusing(id, error),
        },
        ServedMethod::GateExplain => match Parsed::<FindingParameters>(parameters)
        {
            Ok(parameters) => Serialized(id, &nomos_api::Handle_Gate_Explain(&parameters.Root(), &parameters.Query())),
            Err(error) => WireResponse::Refusing(id, error),
        },
        ServedMethod::Correction => match Parsed::<CorrectionParameters>(parameters)
        {
            Ok(parameters) => Serialized(id, &nomos_api::Handle_Correction_Run(&parameters.Command())),
            Err(error) => WireResponse::Refusing(id, error),
        },
    };
}

/// A request's arguments as the parameter type an operation takes.
///
/// Absent, they are null, and null is read as an empty object so that an operation whose
/// arguments are all optional can be called with none at all.
fn Parsed<T: serde::de::DeserializeOwned>(parameters: Value) -> Result<T, WireError>
{
    let parameters = if parameters.is_null() { Value::Object(serde_json::Map::new()) } else { parameters };

    return serde_json::from_value(parameters).map_err(|error| return WireError::New(WireError::INVALID_PARAMETERS, error.to_string()));
}

/// An operation's own response, rendered into the wire answer.
fn Serialized<T: Serialize>(id: Value, response: &T) -> WireResponse
{
    return match serde_json::to_value(response)
    {
        Ok(result) => WireResponse::Answering(id, result),
        // Not reachable over the response types `nomos-api` exports, which are plain derived
        // `Serialize` implementations over owned data. It is answered rather than asserted
        // because the alternative is a panic inside a request handler, which would take the
        // whole connection down for a caller who did nothing wrong.
        Err(error) => WireResponse::Refusing(id, WireError::New(WireError::INTERNAL_ERROR, error.to_string())),
    };
}

#[cfg(test)]
mod tests
{
    use super::Answer;
    use crate::test_support::{At, Count_At};
    use crate::{ServedMethod, WireError};

    /// A line that is not JSON is refused as a parse error, with the null id the
    /// specification gives a request that could not be read far enough to have one.
    #[test]
    fn Test_A_Line_That_Is_Not_Json_Should_Be_A_Parse_Error()
    {
        let response = Answer("not json at all");

        assert_eq!(response.error.map(|error| return error.code), Some(WireError::PARSE_ERROR));
        assert!(response.id.is_null());
    }

    /// A request claiming another protocol version is refused, rather than served as if it
    /// had claimed this one.
    #[test]
    fn Test_A_Request_Claiming_Another_Version_Should_Be_Refused()
    {
        let response = Answer(r#"{"jsonrpc":"1.0","id":7,"method":"nomos.gate.plan"}"#);

        assert_eq!(response.error.map(|error| return error.code), Some(WireError::INVALID_REQUEST));
        assert_eq!(response.id, 7);
    }

    /// Every repo-tooling handler `nomos-api` exports is unreachable here, and the refusal
    /// names the registry rather than merely saying no.
    #[test]
    fn Test_A_Repo_Tooling_Operation_Should_Be_Method_Not_Found()
    {
        for method in ["nomos.work.finish", "nomos.spec.commit", "Handle_Work_List"]
        {
            let response = Answer(&format!(r#"{{"jsonrpc":"2.0","id":1,"method":"{method}"}}"#));

            let error = response.error.expect("an unserved operation is refused");
            assert_eq!(error.code, WireError::METHOD_NOT_FOUND, "{method}");
            assert!(error.message.contains(ServedMethod::GateRun.Name()), "{}", error.message);
        }
    }

    /// Arguments of the wrong shape are refused as invalid parameters, which is a different
    /// answer from an unknown method and from malformed JSON.
    #[test]
    fn Test_Arguments_Of_The_Wrong_Shape_Should_Be_Invalid_Parameters()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":1,"method":"nomos.gate.run","params":{"root":[]}}"#);

        assert_eq!(response.error.map(|error| return error.code), Some(WireError::INVALID_PARAMETERS));
    }

    /// An explain naming no finding is refused before anything is judged -- the required
    /// arguments are the parameter type's own, so the refusal costs no walk.
    #[test]
    fn Test_An_Explain_Naming_No_Finding_Should_Be_Invalid_Parameters()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":1,"method":"nomos.gate.explain","params":{}}"#);

        assert_eq!(response.error.map(|error| return error.code), Some(WireError::INVALID_PARAMETERS));
    }

    /// A real call with no arguments at all reaches a real answer: an absent argument object
    /// is read as an empty one, and `nomos.gate.plan` takes none anyway.
    #[test]
    fn Test_A_Plan_With_No_Arguments_Should_Reach_A_Real_Registry()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":"a","method":"nomos.gate.plan"}"#);

        let result = response.result.expect("a plan over this workspace's own registry answers");
        assert_eq!(At(&result, "/outcome"), "planned", "{result}");
        assert!(Count_At(&result, "/rules") > 0, "{result}");
    }

    /// A real correction run over a fixture tree with no blocking claim reaches a real
    /// `clean` answer -- proving this transport, not only `nomos-api` directly, can reach
    /// `Handle_Correction_Run`.
    #[test]
    fn Test_A_Correction_Run_Over_A_Clean_Tree_Should_Reach_A_Real_Clean_Answer()
    {
        let root = std::env::temp_dir().join("nomos-api-transport-correction-run-clean");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a fresh directory");
        std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");

        let body = format!(r#"{{"jsonrpc":"2.0","id":1,"method":"nomos.correction.run","params":{{"root":{:?}}}}}"#, root.display().to_string());
        let response = Answer(&body);

        let _ignored = std::fs::remove_dir_all(&root);
        let result = response.result.expect("a correction run over a clean tree answers");
        assert_eq!(At(&result, "/outcome"), "clean", "{result}");
    }
}

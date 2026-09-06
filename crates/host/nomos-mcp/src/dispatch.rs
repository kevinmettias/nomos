//! One MCP request in, one answer out -- `initialize`, `tools/list`, `tools/call`, and every
//! refusal `nomos-api-transport::Answer` already knows how to give a caller reaching for it.

use crate::ServedTool;
use nomos_api_transport::{Answer as Transport_Answer, WireError, WireRequest, WireResponse};
use serde::Deserialize;
use serde_json::{json, Value};

/// The MCP protocol revision this server declares in `initialize`.
///
/// The first published stable revision, chosen because this server implements a fixed,
/// narrow subset of it -- one capability (`tools`), no resources, no prompts, no sampling,
/// no notifications beyond ignoring the ones a client sends -- and claiming a later
/// revision this server does not otherwise distinguish itself from would promise a client
/// more negotiation than [`Answered`] below actually does.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// This server's own name and version, echoed in `initialize`'s `serverInfo`.
const SERVER_NAME: &str = "nomos-mcp";

/// One line in, one answer out -- `None` for a notification, which JSON-RPC and MCP both
/// forbid answering: a notification carries no `id` a response could echo, and a client
/// reading an unsolicited line back would have no request to match it against.
///
/// Total by construction the same way [`nomos_api_transport::dispatch::Answer`] is: nothing
/// between a malformed line and a served operation panics or drops the connection, because
/// this crate is reached by an MCP client it did not compile against.
#[must_use]
pub fn Answer(line: &str) -> Option<WireResponse>
{
    let request: WireRequest = match serde_json::from_str(line)
    {
        Ok(request) => request,
        // The id is not recoverable from a line that did not parse, so the answer carries
        // null -- the same reading `nomos_api_transport::dispatch::Answer` gives this case,
        // and what the JSON-RPC specification says a parse error's id is.
        Err(error) => return Some(WireResponse::Refusing(Value::Null, WireError::New(WireError::PARSE_ERROR, error.to_string()))),
    };

    if Is_Notification(&request.method)
    {
        return None;
    }

    return Some(Answered(request));
}

/// Whether `method` is one of MCP's own notifications -- `notifications/initialized` is the
/// one every client sends after `initialize`, and this server has nothing to do in response
/// to it or to any other `notifications/*` message, since it keeps no per-session state that
/// a notification would change.
fn Is_Notification(method: &str) -> bool
{
    return method.starts_with("notifications/");
}

/// The MCP method `request.method` names, called with the arguments it carried.
fn Answered(request: WireRequest) -> WireResponse
{
    let WireRequest { id, method, parameters, .. } = request;

    return match method.as_str()
    {
        "initialize" => WireResponse::Answering(id, Initialize_Result()),
        "tools/list" => WireResponse::Answering(id, Tools_List_Result()),
        "tools/call" => Tools_Call_Result(id, parameters),
        // A client is entitled to probe liveness before it has anything else to ask; an
        // empty object is the whole of what `ping` promises back.
        "ping" => WireResponse::Answering(id, json!({})),
        _ => WireResponse::Refusing(
            id,
            WireError::New(WireError::METHOD_NOT_FOUND, format!("`{method}` is not a method this server serves; it serves initialize, tools/list, tools/call and ping")),
        ),
    };
}

/// `initialize`'s own answer: the revision this server speaks, the one capability it has,
/// and what to call it.
fn Initialize_Result() -> Value
{
    return json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
    });
}

/// `tools/list`'s own answer: every tool in [`ServedTool::REGISTRY`], in registry order.
fn Tools_List_Result() -> Value
{
    let tools: Vec<Value> = ServedTool::REGISTRY.iter().map(|tool| return tool.Listing()).collect();

    return json!({ "tools": tools });
}

/// A `tools/call` request's own arguments: which tool, and what to call it with.
#[derive(Deserialize)]
struct ToolCall
{
    name: String,
    #[serde(default)]
    arguments: Value,
}

/// `tools/call`'s own answer, for the request `id` identified carrying `parameters`.
///
/// Refused at the JSON-RPC level only for a malformed call envelope or an unknown tool name
/// -- both are the caller misusing this protocol rather than the tool itself failing. A tool
/// that ran and refused (an invalid argument, an unreadable tree) answers successfully at
/// the JSON-RPC level with `isError: true` in its own result, which is MCP's own distinction
/// between "this call could not be made" and "this call was made and failed": a client
/// reading `isError` does not have to tell the two apart by inspecting a JSON-RPC error code
/// this server never had a reason to invent one for.
fn Tools_Call_Result(id: Value, parameters: Value) -> WireResponse
{
    let call: ToolCall = match serde_json::from_value(parameters)
    {
        Ok(call) => call,
        Err(error) => return WireResponse::Refusing(id, WireError::New(WireError::INVALID_PARAMETERS, error.to_string())),
    };

    if ServedTool::Named(&call.name).is_none()
    {
        return WireResponse::Refusing(id, WireError::New(WireError::METHOD_NOT_FOUND, format!("`{}` is not a tool this server serves; it serves {}", call.name, Served_Names())));
    }

    let line = Synthetic_Line(&call.name, &call.arguments);
    let answer = Transport_Answer(&line);

    return WireResponse::Answering(id, Tool_Result_Of(answer));
}

/// The registry as a caller-readable list, for the one message that has to name it.
fn Served_Names() -> String
{
    let names: Vec<&str> = ServedTool::REGISTRY.iter().map(|tool| return tool.Name()).collect();

    return names.join(", ");
}

/// `name` and `arguments`, as the one JSON-RPC line `nomos_api_transport::Answer` reads --
/// the same envelope shape [`nomos_api_transport::WireRequest`] deserializes, built rather
/// than reused as a value so this crate never constructs that type directly and has nothing
/// to keep in step if a field is added to it. The id inside is never read back by anything
/// outside this function: [`Tools_Call_Result`] answers the *outer* MCP request under the
/// id it was given, whichever way the inner call comes back.
fn Synthetic_Line(name: &str, arguments: &Value) -> String
{
    return json!({ "jsonrpc": "2.0", "id": 0, "method": name, "params": arguments }).to_string();
}

/// `answer`, as the `{content, isError}` object MCP's `tools/call` result carries.
fn Tool_Result_Of(answer: WireResponse) -> Value
{
    if let Some(result) = answer.result
    {
        return json!({ "content": [{ "type": "text", "text": result.to_string() }], "isError": false });
    }

    let message = answer.error.map_or_else(|| return "the operation refused to answer".to_owned(), |error| return error.message);

    return json!({ "content": [{ "type": "text", "text": message }], "isError": true });
}

#[cfg(test)]
mod tests
{
    use super::{Answer, PROTOCOL_VERSION};
    use crate::test_support::At;
    use nomos_api_transport::WireError;
    use serde_json::Value;

    /// A line that is not JSON is refused as a parse error, with the null id the
    /// specification gives a request that could not be read far enough to have one.
    #[test]
    fn Test_A_Line_That_Is_Not_Json_Should_Be_A_Parse_Error()
    {
        let response = Answer("not json at all").expect("a malformed line is a request, not a notification");

        assert_eq!(response.error.map(|error| return error.code), Some(WireError::PARSE_ERROR));
        assert!(response.id.is_null());
    }

    /// A notification is answered with nothing at all -- not even a response carrying no
    /// result, since MCP forbids replying to one.
    #[test]
    fn Test_A_Notification_Should_Receive_No_Answer()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);

        assert!(response.is_none());
    }

    /// `initialize` reports the protocol revision this server declares and the one
    /// capability it has.
    #[test]
    fn Test_Initialize_Should_Report_The_Declared_Revision_And_The_Tools_Capability()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#).expect("initialize is a request");
        let result = response.result.expect("initialize always answers");

        assert_eq!(At(&result, "/protocolVersion"), PROTOCOL_VERSION, "{result}");
        assert!(At(&result, "/capabilities/tools").is_object(), "{result}");
        assert_eq!(At(&result, "/serverInfo/name"), "nomos-mcp", "{result}");
    }

    /// `tools/list` names Gate's three verbs plus Correction's one, and no other tool.
    #[test]
    fn Test_Tools_List_Should_Name_Exactly_The_Four_Admitted_Verbs()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#).expect("tools/list is a request");
        let result = response.result.expect("tools/list always answers");
        let tools = At(&result, "/tools");
        let listed = tools.as_array().expect("tools/list answers with an array");
        let names: Vec<Value> = (0..listed.len()).map(|index| return At(&tools, &format!("/{index}/name"))).collect();

        assert_eq!(names, vec!["nomos.gate.plan", "nomos.gate.run", "nomos.gate.explain", "nomos.correction.run"], "{result}");
    }

    /// A real `tools/call` for `nomos.gate.plan` reaches a real answer over this
    /// workspace's own real rule registry, through `nomos-api-transport`'s own dispatch --
    /// this crate never calls `nomos_api::Handle_Gate_Plan` itself.
    #[test]
    fn Test_A_Real_Tool_Call_Should_Reach_A_Real_Registry()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":"a","method":"tools/call","params":{"name":"nomos.gate.plan","arguments":{}}}"#)
            .expect("tools/call is a request");
        let result = response.result.expect("a real registry answers");

        assert_eq!(At(&result, "/isError"), false, "{result}");
        let text = At(&result, "/content/0/text");
        let text = text.as_str().expect("a text content block");
        let inner: Value = serde_json::from_str(text).expect("the tool's own answer is JSON");
        assert_eq!(At(&inner, "/outcome"), "planned", "{inner}");
    }

    /// A `tools/call` naming a tool outside the registry is refused at the JSON-RPC level,
    /// the same code an unknown method gets -- the caller asked for something this server
    /// does not have, not something a tool tried and failed.
    #[test]
    fn Test_A_Tool_Call_For_An_Unknown_Tool_Should_Be_Method_Not_Found()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nomos.work.list","arguments":{}}}"#)
            .expect("tools/call is a request");

        let error = response.error.expect("an unregistered tool is refused");
        assert_eq!(error.code, WireError::METHOD_NOT_FOUND, "{}", error.message);
    }

    /// A tool called with arguments its own schema refuses answers successfully at the
    /// JSON-RPC level, with `isError: true` -- the call was made; the tool is what failed.
    #[test]
    fn Test_A_Tool_Refusing_Its_Own_Arguments_Should_Answer_With_Is_Error_True()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nomos.gate.explain","arguments":{}}}"#)
            .expect("tools/call is a request");
        let result = response.result.expect("a refused tool call still answers the outer request");

        assert_eq!(At(&result, "/isError"), true, "{result}");
        let text = At(&result, "/content/0/text");
        assert!(!text.as_str().expect("a text content block").is_empty());
    }

    /// An unknown top-level method is refused, naming what this server does serve.
    #[test]
    fn Test_An_Unknown_Method_Should_Be_Refused_Naming_What_Is_Served()
    {
        let response = Answer(r#"{"jsonrpc":"2.0","id":1,"method":"resources/list"}"#).expect("resources/list is a request, not a notification");

        let error = response.error.expect("an unserved method is refused");
        assert_eq!(error.code, WireError::METHOD_NOT_FOUND, "{}", error.message);
        assert!(error.message.contains("tools/list"), "{}", error.message);
    }
}

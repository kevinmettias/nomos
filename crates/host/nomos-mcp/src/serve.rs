//! Serving MCP requests over a byte stream, line by line -- the framing
//! `nomos-api-transport`'s own module doc already names as what MCP's own stdio transport
//! uses, so this crate spends no design budget picking a second one.

use crate::dispatch::Answer;
use nomos_api_transport::WireResponse;
use std::io::{BufRead, Write};

/// What a response that could not be rendered says, so a client always gets a line back --
/// the identical fallback [`nomos_api_transport::serve`] carries, for the identical reason:
/// the failure it covers is serialization itself.
const UNRENDERABLE: &str = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"this answer could not be rendered"}}"#;

/// Serves every request `input` carries, one line at a time, until it is exhausted.
///
/// Generic over [`BufRead`] and [`Write`] rather than fixed to stdio, so a test drives this
/// over an in-memory buffer and `src/main.rs` drives it over this process's own stdin and
/// stdout -- the same split [`nomos_api_transport::Serve_Connection`] draws between a real
/// stream and the framing run over it. A blank line is skipped rather than answered, and a
/// notification is answered with nothing at all: both leave the caller with fewer lines out
/// than lines in, which is the correct shape for a framing where every line in is not
/// guaranteed a line back.
///
/// # Errors
///
/// Returns the first read or write failure. Reaching the end of the stream is not one: a
/// client closing its side of the pipe is how a session ends, the same reading
/// `nomos_api_transport::Serve_Connection`'s own doc gives a peer hanging up.
pub fn Serve(input: impl BufRead, mut output: impl Write) -> std::io::Result<()>
{
    for line in input.lines()
    {
        let line = line?;
        if line.trim().is_empty()
        {
            continue;
        }

        let Some(response) = Answer(&line)
        else
        {
            continue;
        };

        let body = Rendered(&response);
        writeln!(output, "{body}")?;
        output.flush()?;
    }

    return Ok(());
}

/// `response` as the line a client reads.
fn Rendered(response: &WireResponse) -> String
{
    return serde_json::to_string(response).unwrap_or_else(|_| return UNRENDERABLE.to_owned());
}

#[cfg(test)]
mod tests
{
    use super::{Serve, UNRENDERABLE};
    use crate::test_support::At;
    use serde_json::Value;
    use std::io::Cursor;

    /// A real conversation -- `initialize`, then a `tools/call` -- reaches real answers, one
    /// line per request, with a blank line and a notification both producing no line back.
    #[test]
    fn Test_A_Real_Conversation_Should_Answer_Every_Request_On_Its_Own_Line()
    {
        let input = [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            "",
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"nomos.gate.plan","arguments":{}}}"#,
        ]
        .join("\n");

        let lines = Served(&input);
        let first = lines.first().expect("the assertion below proves there are two lines");
        let second = lines.get(1).expect("the assertion below proves there are two lines");

        assert_eq!(lines.len(), 2, "{lines:?}");
        assert_eq!(At(first, "/id"), 1, "{first}");
        assert!(At(first, "/result/protocolVersion").is_string(), "{first}");
        assert_eq!(At(second, "/id"), 2, "{second}");
        assert_eq!(At(second, "/result/isError"), false, "{second}");
    }

    /// The fallback line is itself a valid JSON-RPC error object, so the one path that
    /// cannot render an answer still cannot hand a client something unparseable.
    #[test]
    fn Test_The_Unrenderable_Fallback_Should_Itself_Be_A_Valid_Answer()
    {
        let fallback: Value = serde_json::from_str(UNRENDERABLE).expect("the fallback is JSON");

        assert_eq!(At(&fallback, "/jsonrpc"), "2.0");
        assert_eq!(At(&fallback, "/error/code"), -32603);
    }

    /// `input`, served over an in-memory buffer, as the lines it answered.
    fn Served(input: &str) -> Vec<Value>
    {
        let mut output = Vec::new();
        Serve(Cursor::new(input), &mut output).expect("an in-memory buffer never fails to read or write");

        let text = String::from_utf8(output).expect("every answer is written as UTF-8 text");

        return text
            .lines()
            .map(|line| return serde_json::from_str(line).unwrap_or(Value::Null))
            .collect();
    }
}

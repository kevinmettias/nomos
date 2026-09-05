//! The socket, the framing, and why both are the plainest thing that could work.

use crate::{Answer, WireResponse};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

/// What a response that could not be rendered says, so a client always gets a line back.
///
/// A constant rather than a second serialization, because the failure it covers is
/// serialization itself.
const UNRENDERABLE: &str = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"this answer could not be rendered"}}"#;

/// Serves every connection `listener` accepts, one at a time, until accepting fails.
///
/// One connection at a time, and the choice is deliberate rather than provisional. A thread
/// per connection, a pool, or an async runtime are three different answers to a question no
/// caller has asked yet: nothing in this workspace connects to this transport concurrently,
/// and a concurrency model chosen ahead of a real second client is the premature surface
/// `OD-PACKAGE-006` and `OD-PACKAGE-008` refuse. It is also not free to reverse quietly --
/// `nomos_api::Handle_Gate_Run` walks and judges a whole tree, so two of them at once is a
/// resource decision rather than a scheduling detail.
///
/// # Errors
///
/// Returns the first failure to accept a connection. A failure *within* one connection ends
/// that connection and no other: a client hanging up mid-request is the ordinary case, not a
/// reason to stop serving.
pub fn Serve(listener: &TcpListener) -> std::io::Result<()>
{
    loop
    {
        let (stream, _peer) = listener.accept()?;
        let _ignored = Serve_Connection(&stream);
    }
}

/// Serves every request on one already-accepted connection, until the peer closes it.
///
/// The framing is one JSON object per line, in both directions, which is the framing MCP's
/// own stdio transport uses. `Content-Length` headers -- LSP's framing, and MCP's over other
/// carriers -- would be the other defensible choice; this one is picked because a line is
/// self-delimiting, so a truncated request cannot be read as a complete shorter one, and
/// because a person can drive this transport from a terminal to see what it does. A blank
/// line is skipped rather than answered, so a client whose writes end with a newline is not
/// told it sent a parse error.
///
/// # Errors
///
/// Returns the first read or write failure on the connection. Reaching the end of the stream
/// is not one: a peer closing the connection is how a session ends.
pub fn Serve_Connection(stream: &TcpStream) -> std::io::Result<()>
{
    let reader = BufReader::new(stream);
    let mut writer = stream;

    for line in reader.lines()
    {
        let line = line?;
        if line.trim().is_empty()
        {
            continue;
        }

        let response = Answer(&line);
        let body = Rendered(&response);
        writeln!(writer, "{body}")?;
        writer.flush()?;
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
    use super::{Serve_Connection, UNRENDERABLE};
    use crate::test_support::{At, Count_At};
    use crate::{WireError, WireResponse};
    use serde_json::Value;
    use std::io::{BufRead, BufReader, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::PathBuf;

    /// A real request over a real socket reaches a real answer.
    ///
    /// This is the assertion `P40-API-TRANSPORT-2` exists for, and what makes it different
    /// from every test in `nomos-api` is that nothing here calls a handler: the request is
    /// bytes written to a TCP stream by one thread and read off another, and the answer is
    /// bytes read back. A direct call to `Answer` would prove the dispatch and prove nothing
    /// about the transport.
    #[test]
    fn Test_A_Real_Request_Over_A_Real_Socket_Should_Reach_A_Real_Registry()
    {
        let answered = Over_A_Socket(&[r#"{"jsonrpc":"2.0","id":1,"method":"nomos.gate.plan"}"#]);

        let answer = answered.first().expect("one request is answered once");
        assert_eq!(At(answer, "/jsonrpc"), "2.0", "{answer}");
        assert_eq!(At(answer, "/id"), 1, "{answer}");
        assert_eq!(At(answer, "/result/outcome"), "planned", "{answer}");
        assert!(Count_At(answer, "/result/rules") > 0, "{answer}");
    }

    /// Arguments cross the wire and reach the handler: a run naming a tree answers about that
    /// tree, not about the serving process's own working directory.
    #[test]
    fn Test_Arguments_Should_Cross_The_Wire_And_Reach_The_Handler()
    {
        let root = Fresh_Root("nomos-api-transport-serve-run-root");
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "run",
            "method": "nomos.gate.run",
            "params": { "root": root },
        })
        .to_string();

        let answered = Over_A_Socket(&[&body]);

        let _ignored = std::fs::remove_dir_all(&root);
        let answer = answered.first().expect("one request is answered once");
        assert_eq!(At(answer, "/id"), "run", "{answer}");
        assert_eq!(At(answer, "/result/root"), serde_json::json!(root), "{answer}");
    }

    /// One connection carries many requests, and each answer is its own line -- so a client
    /// reading line by line never has to find a boundary inside one.
    #[test]
    fn Test_One_Connection_Should_Answer_Every_Request_On_Its_Own_Line()
    {
        let answered = Over_A_Socket(&[
            r#"{"jsonrpc":"2.0","id":1,"method":"nomos.gate.plan"}"#,
            "",
            r#"{"jsonrpc":"2.0","id":2,"method":"nomos.work.list"}"#,
        ]);

        assert_eq!(answered.len(), 2, "{answered:?}");
        assert_eq!(answered.first().map(|answer| return At(answer, "/id")), Some(Value::from(1)));
        assert_eq!(
            answered.get(1).map(|answer| return At(answer, "/error/code")),
            Some(Value::from(WireError::METHOD_NOT_FOUND))
        );
    }

    /// The fallback line is itself a valid JSON-RPC error object, so the one path that cannot
    /// render an answer still cannot hand a client something unparseable.
    #[test]
    fn Test_The_Unrenderable_Fallback_Should_Itself_Be_A_Valid_Answer()
    {
        let fallback: Value = serde_json::from_str(UNRENDERABLE).expect("the fallback is JSON");

        assert_eq!(At(&fallback, "/jsonrpc"), WireResponse::VERSION);
        assert_eq!(At(&fallback, "/error/code"), Value::from(WireError::INTERNAL_ERROR));
    }

    /// Writes `requests` to a real loopback socket a real server thread is serving, and reads
    /// back every line it answered.
    fn Over_A_Socket(requests: &[&str]) -> Vec<Value>
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port is bindable");
        let address = listener.local_addr().expect("a bound listener has an address");
        let server = std::thread::spawn(move || {
            let (stream, _peer) = listener.accept().expect("the client below connects");
            let _ignored = Serve_Connection(&stream);
        });

        let client = TcpStream::connect(address).expect("the listener above is accepting");
        let mut writer = &client;
        for request in requests
        {
            writeln!(writer, "{request}").expect("the connection is writable");
        }
        writer.flush().expect("the connection is flushable");
        client
            .shutdown(std::net::Shutdown::Write)
            .expect("half-closing tells the server its requests are done");

        let answered = BufReader::new(&client)
            .lines()
            .map_while(Result::ok)
            .map(|line| return serde_json::from_str(&line).unwrap_or(Value::Null))
            .collect();

        server.join().expect("the server thread does not panic");

        return answered;
    }

    /// Removes and recreates `root` under the system temp directory, so a test starts from a
    /// clean, empty tree regardless of what an earlier run left behind.
    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");

        return root;
    }
}

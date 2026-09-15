//! A real client talking to a real subprocess -- what makes this different from every test
//! in `src/`, the same distinction `nomos-api-transport`'s own socket test draws against a
//! direct call to `Answer`: "A direct call to `Answer` would prove the dispatch and prove
//! nothing about the transport." Here the transport is a spawned process's stdin and
//! stdout, not a library call, and no request-generating code in this file has ever seen
//! `nomos-mcp`'s own source.

use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// The value at `pointer` inside `value`, or null if nothing is there -- this workspace
/// denies `clippy::indexing_slicing`, and a JSON pointer asks the same question without
/// panicking on an absent field. Duplicated from `src/test_support.rs` rather than shared:
/// that module is `#[cfg(test)]`-private to the library crate, and this file is a separate
/// compilation unit that never links against the library's own test code.
fn At(value: &Value, pointer: &str) -> Value
{
    return value.pointer(pointer).cloned().unwrap_or(Value::Null);
}

/// The id the second request carries, and the id its answer must echo back untouched -- the
/// one number in this conversation the engine takes from the request rather than choosing.
const CLIENT_REQUEST_ID: u64 = 2;

/// A real `initialize` and a real `tools/call`, written to a real child process's stdin and
/// read back from its stdout -- the same conversation an MCP client has with this server
/// when it launches it as a subprocess, which is the only way this binary is ever run.
#[test]
fn Test_A_Real_Subprocess_Should_Answer_A_Real_Conversation_Over_Its_Own_Stdio()
{
    let mut conversation = Spawned_Server();

    let initialized = Asked(&mut conversation, r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#);
    assert_eq!(At(&initialized, "/result/serverInfo/name"), "nomos-mcp", "{initialized}");

    let call = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"nomos.gate.plan","arguments":{}}}"#;
    let called = Asked(&mut conversation, call);
    assert_eq!(At(&called, "/id"), CLIENT_REQUEST_ID, "{called}");
    assert_eq!(At(&called, "/result/isError"), false, "{called}");

    drop(conversation.stdin);
    let status = conversation.child.wait().expect("the child exits once its stdin closes");
    assert!(status.success(), "{status:?}");
}

/// A spawned server this test owns both pipes of, and the line reader over its stdout.
struct Conversation
{
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    answers: std::io::Lines<BufReader<std::process::ChildStdout>>,
}

/// The compiled server, spawned with both pipes requested -- nothing is written to it yet.
fn Spawned_Server() -> Conversation
{
    let mut child = Command::new(env!("CARGO_BIN_EXE_nomos-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the built nomos-mcp binary is spawnable");

    let stdin = child.stdin.take().expect("stdin was requested as piped");
    let stdout = child.stdout.take().expect("stdout was requested as piped");

    return Conversation { child, stdin, answers: BufReader::new(stdout).lines() };
}

/// One request written to `conversation`'s stdin, and the answer it wrote back.
fn Asked(conversation: &mut Conversation, request: &str) -> Value
{
    writeln!(conversation.stdin, "{request}").expect("the child's stdin accepts a line");

    return Next_Answer(&mut conversation.answers);
}

/// The next line the child wrote, parsed as JSON.
fn Next_Answer(lines: &mut std::io::Lines<BufReader<std::process::ChildStdout>>) -> Value
{
    let line = lines
        .next()
        .expect("the child has not exited")
        .expect("the child's stdout is readable");

    return serde_json::from_str(&line).unwrap_or_else(|error| panic!("{line} did not parse: {error}"));
}

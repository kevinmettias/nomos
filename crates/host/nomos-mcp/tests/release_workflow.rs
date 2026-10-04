//! A coding client completes the correction lifecycle through the built MCP server.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;
use std::time::Duration;

const RULE: &str = "completeness-mirror";
const BEFORE: &str = "/// A list.\n/// Mirrored by `Test_Release_Ghost`.\npub const TABLES: &[&str] = &[];\n";
const AFTER: &str = "/// A list.\npub const TABLES: &[&str] = &[];\n";
const ANSWER_LIMIT: Duration = Duration::from_secs(30);

struct Fixture
{
    root: PathBuf,
}

impl Fixture
{
    fn Of(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-release-mcp-{}-{name}", std::process::id()));
        std::fs::create_dir(&root).expect("an unused temporary directory");
        std::fs::write(root.join("a.rs"), BEFORE).expect("a writable fixture source");
        return Self { root };
    }

    fn Contents(&self) -> String
    {
        return std::fs::read_to_string(self.root.join("a.rs")).expect("the source should remain readable");
    }
}

impl Drop for Fixture
{
    fn drop(&mut self)
    {
        std::fs::remove_dir_all(&self.root).expect("a removable temporary fixture");
    }
}

struct Conversation
{
    child: Child,
    stdin: Option<ChildStdin>,
    answers: Receiver<std::io::Result<String>>,
    reader: Option<JoinHandle<()>>,
    next_id: u64,
}

impl Conversation
{
    fn Started() -> Self
    {
        let mut child = Command::new(env!("CARGO_BIN_EXE_nomos-mcp")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("the built MCP server should start");
        let stdin = child.stdin.take().expect("stdin was requested");
        let stdout = child.stdout.take().expect("stdout was requested");
        let (sender, answers) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines()
            {
                if sender.send(line).is_err()
                {
                    return;
                }
            }
        });
        return Self { child, stdin: Some(stdin), answers, reader: Some(reader), next_id: 1 };
    }

    fn Sent(&mut self, text: &str) -> Value
    {
        writeln!(self.stdin.as_mut().expect("an open conversation"), "{text}").expect("the server should accept a request");
        let line = self.answers.recv_timeout(ANSWER_LIMIT).expect("the server must answer within its bound").expect("a readable response");
        return serde_json::from_str(&line).expect("the server should answer valid JSON");
    }

    fn Requested(&mut self, method: &str, parameters: Value) -> Value
    {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let response = self.Sent(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": parameters}).to_string());
        assert_eq!(response.pointer("/id"), Some(&json!(id)), "correlation must survive the conversation");
        return response;
    }

    fn Called(&mut self, name: &str, arguments: Value) -> Value
    {
        let response = self.Requested("tools/call", json!({"name": name, "arguments": arguments}));
        assert_eq!(response.pointer("/result/isError"), Some(&json!(false)), "{response}");
        let text = response.pointer("/result/content/0/text").and_then(Value::as_str).expect("a successful tool should carry its result");
        return serde_json::from_str(text).expect("Nomos tools return a JSON document");
    }

    fn Initialized(&mut self)
    {
        let response = self.Requested("initialize", json!({}));
        assert_eq!(response.pointer("/result/serverInfo/name"), Some(&json!("nomos-mcp")));
        let listed = self.Requested("tools/list", json!({}));
        let tools = listed.pointer("/result/tools").and_then(Value::as_array).expect("a real tool catalogue");
        assert!(tools.iter().any(|tool| return tool.pointer("/name") == Some(&json!("nomos.correction.run"))));
    }
}

impl Drop for Conversation
{
    fn drop(&mut self)
    {
        drop(self.stdin.take());
        let _ignored = self.child.kill();
        let _ignored = self.child.wait();
        if let Some(reader) = self.reader.take()
        {
            let _ignored = reader.join();
        }
    }
}

fn Judged(client: &mut Conversation, fixture: &Fixture) -> Value
{
    return client.Called("nomos.gate.run", json!({"root": fixture.root, "rules": [RULE]}));
}

fn Assert_Evidence(document: &Value)
{
    let findings = document.pointer("/findings/blocking_findings").and_then(Value::as_array).expect("a findings collection");
    let finding = findings.iter().find(|finding| return finding.pointer("/rule") == Some(&json!(RULE))).expect("the deliberately false claim should be found");
    assert!(finding.pointer("/summary").and_then(Value::as_str).is_some_and(|summary| return summary.contains("Test_Release_Ghost")));
    assert!(finding.pointer("/locations").and_then(Value::as_array).is_some_and(|locations| return locations.contains(&json!("a.rs"))));
    assert!(finding.pointer("/applicability").is_some_and(|value| return !value.is_null()));
    assert!(finding.pointer("/evidence").is_some_and(|value| return !value.is_null()));
}

#[test]
fn Test_An_Mcp_Client_Should_Inspect_Decline_Commit_And_Verify_A_Real_Correction()
{
    let fixture = Fixture::Of("workflow");
    let mut client = Conversation::Started();
    client.Initialized();
    let before = Judged(&mut client, &fixture);
    Assert_Evidence(&before);
    let explanation = client.Called("nomos.gate.explain", json!({"root": fixture.root, "rule": RULE, "location": "a.rs"}));
    assert_eq!(explanation.pointer("/outcome"), Some(&json!("found")));
    assert_eq!(explanation.pointer("/would_block"), Some(&json!(true)));
    let staged = client.Called("nomos.correction.run", json!({"root": fixture.root}));
    assert_eq!(staged.pointer("/outcome"), Some(&json!("staged")));
    assert!(staged.pointer("/preview").and_then(Value::as_str).is_some_and(|preview| return preview.contains("Test_Release_Ghost")));
    assert_eq!(fixture.Contents(), BEFORE, "a preview wrote the source");
    Assert_Evidence(&Judged(&mut client, &fixture));
    assert_eq!(fixture.Contents(), BEFORE, "declining by withholding commit changed the source");
    let committed = client.Called("nomos.correction.run", json!({"root": fixture.root, "commit": true}));
    assert_eq!(committed.pointer("/outcome"), Some(&json!("committed")));
    assert_ne!(committed.pointer("/base"), committed.pointer("/after"));
    assert_eq!(fixture.Contents(), AFTER);
    let after = Judged(&mut client, &fixture);
    assert_eq!(after.pointer("/findings/blocking_findings"), Some(&json!([])), "the correction did not clear its finding");
}

#[test]
fn Test_A_Refused_Mcp_Request_Should_Not_Write_And_Should_Leave_The_Server_Usable()
{
    let fixture = Fixture::Of("refusals");
    let mut client = Conversation::Started();
    client.Initialized();
    let malformed = client.Sent("{not-json");
    assert!(malformed.pointer("/error").is_some(), "{malformed}");
    for (name, arguments) in [("nomos.no_such_tool", json!({"root": fixture.root, "commit": true})), ("nomos.correction.run", json!({"root": fixture.root, "commit": "yes"}))]
    {
        let refused = client.Requested("tools/call", json!({"name": name, "arguments": arguments}));
        assert!(refused.pointer("/error").is_some() || refused.pointer("/result/isError") == Some(&json!(true)), "{refused}");
        assert_eq!(fixture.Contents(), BEFORE, "a refused request wrote the source");
    }
    Assert_Evidence(&Judged(&mut client, &fixture));
}

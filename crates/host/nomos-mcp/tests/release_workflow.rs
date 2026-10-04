//! A coding client completes the correction lifecycle through the built MCP server.

use nomos_platform::ProgramLauncher;
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
    source: &'static str,
}

impl Fixture
{
    fn Of(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-release-mcp-{}-{name}", std::process::id()));
        std::fs::create_dir(&root).expect("an unused temporary directory");
        std::fs::write(root.join("a.rs"), BEFORE).expect("a writable fixture source");
        return Self { root, source: "a.rs" };
    }

    fn Contents(&self) -> String
    {
        return std::fs::read_to_string(self.root.join(self.source)).expect("the source should remain readable");
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
        return Self::Started_Using(None);
    }

    fn Started_Using(root: Option<&std::path::Path>) -> Self
    {
        let mut command = Command::new(env!("CARGO_BIN_EXE_nomos-mcp"));
        if let Some(root) = root
        {
            command.env("CARGO_TARGET_DIR", root.join("target"));
        }
        let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("the built MCP server should start");
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

fn Assert_Evidence(document: &Value, path: &str)
{
    let findings = document.pointer("/findings/blocking_findings").and_then(Value::as_array).expect("a findings collection");
    let finding = findings.iter().find(|finding| return finding.pointer("/rule") == Some(&json!(RULE))).expect("the deliberately false claim should be found");
    assert!(finding.pointer("/summary").and_then(Value::as_str).is_some_and(|summary| return summary.contains("Test_Release_Ghost")));
    assert!(finding.pointer("/locations").and_then(Value::as_array).is_some_and(|locations| return locations.contains(&json!(path))));
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
    Assert_Evidence(&before, fixture.source);
    let explanation = client.Called("nomos.gate.explain", json!({"root": fixture.root, "rule": RULE, "location": "a.rs"}));
    assert_eq!(explanation.pointer("/outcome"), Some(&json!("found")));
    assert_eq!(explanation.pointer("/would_block"), Some(&json!(true)));
    let staged = client.Called("nomos.correction.run", json!({"root": fixture.root}));
    assert_eq!(staged.pointer("/outcome"), Some(&json!("staged")));
    assert!(staged.pointer("/preview").and_then(Value::as_str).is_some_and(|preview| return preview.contains("Test_Release_Ghost")));
    assert_eq!(fixture.Contents(), BEFORE, "a preview wrote the source");
    Assert_Evidence(&Judged(&mut client, &fixture), fixture.source);
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
    Assert_Evidence(&Judged(&mut client, &fixture), fixture.source);
}

const EXTERNAL_SOURCE: &str = "src/lib.rs";
const VALIDATION_LIMIT: Duration = Duration::from_secs(120);

/// Copy the full source population. Build products and VCS state are not source inputs.
fn Copied_Tree(source: &std::path::Path, destination: &std::path::Path)
{
    std::fs::create_dir_all(destination).expect("a writable scratch directory");
    for entry in std::fs::read_dir(source).expect("readable external source directory")
    {
        let entry = entry.expect("a readable directory entry");
        if entry.file_name() == "target" || entry.file_name() == ".git"
        {
            continue;
        }
        let kind = entry.file_type().expect("readable entry metadata");
        assert!(!kind.is_symlink(), "the acceptance copy must not follow a link outside its source tree");
        let target = destination.join(entry.file_name());
        if kind.is_dir()
        {
            Copied_Tree(&entry.path(), &target);
        }
        else
        {
            std::fs::copy(entry.path(), target).expect("copy every source file without trimming it");
        }
    }
}

impl Fixture
{
    fn External(source: &std::path::Path, name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-release-mcp-{}-{name}", std::process::id()));
        assert!(!root.exists(), "the scratch root must be unique");
        Copied_Tree(source, &root);
        return Self { root, source: EXTERNAL_SOURCE };
    }

    fn Write(&self, content: &str)
    {
        std::fs::write(self.root.join(self.source), content).expect("a writable scratch library");
    }
}

fn Cargo_Validation(root: &std::path::Path) -> nomos_platform::ExitOutcome
{
    let mut command = nomos_platform::Command::From_String_Arguments(
        vec!["cargo".to_owned(), "test".to_owned(), "--offline".to_owned(), "--quiet".to_owned(),
            "--target-dir".to_owned(), root.join("target").to_str().expect("a UTF-8 scratch path").to_owned()],
        VALIDATION_LIMIT,
    );
    command.working_directory = Some(root.to_owned());
    let started = std::time::Instant::now();
    let output = nomos_platform_std::StdProgramLauncher.Run(&command).expect("launch real offline Cargo validation");
    std::fs::write(root.join("validation.stdout"), output.stdout).expect("capture compiler output");
    std::fs::write(root.join("validation.stderr"), output.stderr).expect("capture compiler diagnostics");
    println!("external Cargo validation: {:?}, {} ms", output.outcome, started.elapsed().as_millis());
    return output.outcome;
}
fn Assert_Cargo_Passes(fixture: &Fixture)
{
    assert!(Cargo_Validation(&fixture.root).Is_Successful(), "{}", std::fs::read_to_string(fixture.root.join("validation.stderr")).expect("captured compiler diagnostics"));
    let output = std::fs::read_to_string(fixture.root.join("validation.stdout")).expect("captured test output");
    assert!(Has_Executed_Tests(&output), "validation must execute nonzero tests, not just launch Cargo: {output}");
    println!("{output}");
}

fn Actual_Proposed_Bytes(staged: &Value, before: &str) -> String
{
    assert_eq!(staged.pointer("/outcome"), Some(&json!("staged")));
    let preview = staged.pointer("/preview").and_then(Value::as_str).expect("a concrete proposed edit");
    let marker = format!("edit\t{EXTERNAL_SOURCE}\t{before}\t");
    let (_, after) = preview.split_once(&marker).expect("the edit's before bytes must exactly match the observed source");
    return after.strip_suffix('\n').expect("one complete rendered edit").to_owned();
}

fn Declined_After_Failed_Validation(client: &mut Conversation, fixture: &Fixture, proposed: &str)
{
    let invalid = Fixture::External(&fixture.root, "external-invalid-validation");
    invalid.Write(&format!("{proposed}\nthis deliberately cannot parse as Rust\n"));
    let status = Cargo_Validation(&invalid.root);
    assert_eq!(status, nomos_platform::ExitOutcome::Exited { code: 101 }, "the malformed verification tree must fail real compilation");
    let diagnostics = std::fs::read_to_string(invalid.root.join("validation.stderr")).expect("compiler diagnostics");
    assert!(diagnostics.contains("error:"), "the failed validation must be a compiler failure: {diagnostics}");
    Assert_Evidence(&Judged(client, fixture), fixture.source);
    println!("failed external validation: commit withheld, original scratch source and finding preserved");
}

#[test]
#[ignore = "requires NOMOS_RELEASE_EXTERNAL_REPO naming a complete independently authored Rust package"]
fn Test_External_Source_Should_Be_Validated_Before_An_Mcp_Client_Commits_Its_Correction()
{
    let external = std::env::var_os("NOMOS_RELEASE_EXTERNAL_REPO").filter(|value| return !value.is_empty()).map(PathBuf::from)
        .expect("set NOMOS_RELEASE_EXTERNAL_REPO; an absent corpus must never pass acceptance");
    assert!(external.join("Cargo.toml").is_file() && external.join(EXTERNAL_SOURCE).is_file(), "the corpus must be a complete Rust source package");
    let original = std::fs::read_to_string(external.join(EXTERNAL_SOURCE)).expect("read the unchanged upstream library");
    let started = std::time::Instant::now();
    let fixture = Fixture::External(&external, "external-working-copy");
    let mut client = Conversation::Started_Using(Some(&fixture.root));
    client.Initialized();
    assert_eq!(Judged(&mut client, &fixture).pointer("/findings/blocking_findings"), Some(&json!([])), "the untouched source must have no blocking mirror finding");
    let before = format!("{original}\n{BEFORE}");
    fixture.Write(&before);
    Assert_Cargo_Passes(&fixture);
    Assert_Evidence(&Judged(&mut client, &fixture), fixture.source);
    println!("compiler-only comparison: tests pass while the injected false enforcement claim remains");
    let staged = client.Called("nomos.correction.run", json!({"root": fixture.root}));
    let proposed = Actual_Proposed_Bytes(&staged, &before);
    assert_eq!(proposed, format!("{original}\n{AFTER}"));
    assert_eq!(fixture.Contents(), before, "staging must not write the active source");
    Declined_After_Failed_Validation(&mut client, &fixture, &proposed);
    assert_eq!(fixture.Contents(), before, "declining after failed validation must preserve bytes");
    let valid = Fixture::External(&fixture.root, "external-valid-validation");
    valid.Write(&proposed);
    Assert_Cargo_Passes(&valid);
    let committed = client.Called("nomos.correction.run", json!({"root": fixture.root, "commit": true}));
    assert_eq!(committed.pointer("/outcome"), Some(&json!("committed")), "{committed}");
    assert_ne!(committed.pointer("/base"), committed.pointer("/after"));
    assert_eq!(fixture.Contents(), proposed, "the committed bytes must be the exact externally validated proposal");
    assert_eq!(Judged(&mut client, &fixture).pointer("/findings/blocking_findings"), Some(&json!([])));
    Assert_Cargo_Passes(&fixture);
    assert_eq!(std::fs::read_to_string(external.join(EXTERNAL_SOURCE)).expect("the upstream source remains readable"), original);
    println!("external MCP correction acceptance: {}, {} ms", external.display(), started.elapsed().as_millis());
}

fn Has_Executed_Tests(output: &str) -> bool
{
    return output.lines().filter_map(|line| return line.strip_prefix("running "))
        .filter_map(|line| return line.split_whitespace().next())
        .filter_map(|count| return count.parse::<usize>().ok()).any(|count| return count > 0);
}

#[test]
fn Test_Zero_External_Tests_Should_Not_Count_As_Relevant_Validation()
{
    assert!(!Has_Executed_Tests("running 0 tests\ntest result: ok. 0 passed; 0 failed;\n"));
    assert!(!Has_Executed_Tests("Finished dev profile\n"));
    assert!(Has_Executed_Tests("running 2 tests\ntest result: ok. 2 passed; 0 failed;\n"));
}

#[test]
#[should_panic(expected = "validation must execute nonzero tests")]
fn Test_A_Compilable_Package_With_No_Tests_Should_Fail_The_Validation_Acceptance_Check()
{
    let fixture = Fixture::Of("external-no-tests");
    let manifest = "[package]\nname = \"nomos-no-tests-validation-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[lib]\npath = \"a.rs\"\n[workspace]\n";
    std::fs::write(fixture.root.join("Cargo.toml"), manifest).expect("a standalone package with no tests");
    Assert_Cargo_Passes(&fixture);
}

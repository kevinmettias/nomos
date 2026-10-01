//! Unit tests over an invented party. Nothing here names a real one (`OD-POLICY-002`).

use std::path::Path;

use nomos_platform::{Command, DeterminismStrength, ExitOutcome, ProgramLauncher, ProgramOutput, ReproducibilityScope, Strategy, TraceEquivalence};

use crate::unified_diff::Judge_Diff;
use crate::*;

const POLICY: &str = r#"{
  "party": "Northwind",
  "identities": { "email_domains": ["northwind.example.invalid"] },
  "rules": [
    { "id": "party-name", "why": "names the party", "phrases": ["northwind"], "boundary": "anywhere" },
    { "id": "product", "why": "names its product", "phrases": ["Gale Force"] },
    { "id": "acronym", "why": "its acronym", "phrases": ["NWX"], "case": "exact" },
    { "id": "prefix", "why": "its driver prefix", "phrases": ["nwd"], "boundary": "word-start" },
    { "id": "ticket", "why": "its tracker", "tickets": ["NW"] }
  ],
  "own_repositories": { "remotes": ["https://git.example.invalid/northwind/"], "paths": ["C:/work/northwind"] },
  "exceptions": [
    { "rule": "product", "path": "docs/weather.md", "reason": "a weather term, not the product" },
    { "rule": "acronym", "path": "(message)", "reason": "a vetted message" },
    { "rule": "identity", "path": "(message)", "reason": "no exception can cover an identity" },
    { "rule": "party-name", "path": "docs/any.md", "reason": "" }
  ]
}"#;

fn Policy() -> PartyPolicy
{
    return Party_Policy_From_Json(POLICY).expect("the test policy is valid");
}

fn Rules_Hit(judge: &Judge<'_>, text: &str) -> Vec<String>
{
    let place = Place::Line { commit: None, path: "src/a.rs".to_owned(), line: 1 };
    return judge.Text(&place, text).into_iter().map(|refusal| return refusal.rule).collect();
}

#[test]
fn Test_Each_Form_Should_Match_Its_Target_And_Not_Its_Lookalikes()
{
    let policy = Policy();
    let judge = Judge::New(&policy);
    let cases: [(&str, &[&str], &[&str]); 5] = [
        ("party-name", &["https://northwind.example.invalid/x", "using Northwind.Billing;", "NORTHWIND"], &["north wind", "northward"]),
        ("product", &["the gale-force driver", "GaleForce", "gale.force 2", "gale_force"], &["gale forecast", "megale force"]),
        ("acronym", &["the NWX feed", "(NWX)"], &["nwx", "NWXY", "ANWX"]),
        ("prefix", &["nwd-serial", "nwdriver", "NWD_PORT"], &["anwd", "a nw d"]),
        ("ticket", &["fixes NW-12", "NW-9001."], &["NW-1", "XNW-12", "NW-12a", "nw-12", "NW 12"]),
    ];
    for (rule, hits, misses) in cases
    {
        for text in hits
        {
            assert!(Rules_Hit(&judge, text).iter().any(|hit| return hit == rule), "{rule} should refuse {text:?}");
        }
        for text in misses
        {
            assert!(!Rules_Hit(&judge, text).iter().any(|hit| return hit == rule), "{rule} should admit {text:?}");
        }
    }
}

#[test]
fn Test_A_Party_Address_Should_Be_Refused_By_Its_Domain_Not_Its_Local_Part()
{
    let policy = Policy();
    let judge = Judge::New(&policy);
    let place = Place::Identity { commit: None, role: "author" };
    assert!(judge.Identity(&place, "dev@Northwind.Example.Invalid").is_some(), "a party domain in any case is refused");
    assert!(judge.Identity(&place, "northwind-fan@home.example.invalid").is_none(), "a personal address is not refused for its local part");
}

#[test]
fn Test_An_Exception_Should_Cover_One_Rule_At_One_Path_And_Never_An_Identity()
{
    let policy = Policy();
    let judge = Judge::New(&policy);
    let at = |path: &str| return Place::Line { commit: None, path: path.to_owned(), line: 1 };
    let excepted = judge.Unexcepted(judge.Text(&at("docs/weather.md"), "gale force winds"));
    assert!(excepted.is_empty(), "the vetted path is excepted");
    assert_eq!(judge.Unexcepted(judge.Text(&at("docs/other.md"), "gale force winds")).len(), 1, "another path is not");
    let message = Place::Message { commit: None };
    assert!(judge.Unexcepted(judge.Text(&message, "the NWX feed")).is_empty(), "a message exception covers a message");
    let identity = judge.Identity(&Place::Identity { commit: None, role: "author" }, "a@northwind.example.invalid").into_iter().collect();
    assert_eq!(judge.Unexcepted(identity).len(), 1, "no exception covers an identity");
    assert_eq!(judge.Unexcepted(judge.Text(&at("docs/any.md"), "northwind")).len(), 1, "an exception with no reason was dropped");
}

#[test]
fn Test_The_Partys_Own_Repository_Should_Be_Recognised_By_Remote_Or_Path()
{
    let policy = Policy();
    let judge = Judge::New(&policy);
    assert!(judge.Is_Own_Repository(&["HTTPS://git.example.invalid/Northwind/app.git".to_owned()], "D:/elsewhere"));
    assert!(judge.Is_Own_Repository(&[], r"c:\Work\Northwind\app"));
    assert!(!judge.Is_Own_Repository(&["https://git.example.invalid/me/app.git".to_owned()], "C:/work/northwind-fan"));
}

#[test]
fn Test_A_Policy_That_Cannot_Be_Acted_On_Should_Be_Refused_When_Read()
{
    let refused = [
        r#"{ "party": "N", "rules": [ { "id": "a", "why": "w", "phrase": ["x"] } ] }"#,
        r#"{ "party": "N", "rules": [ { "id": "a", "why": "w" } ] }"#,
        r#"{ "party": "N", "rules": [ { "id": "a", "why": "w", "phrases": ["x"], "tickets": ["X"] } ] }"#,
        r#"{ "party": "N", "rules": [ { "id": "a", "why": "w", "phrases": ["x"] }, { "id": "a", "why": "w", "phrases": ["y"] } ] }"#,
        r#"{ "party": "N", "rules": [ { "id": "a", "why": "w", "phrases": ["x"], "boundary": "somewhere" } ] }"#,
        r#"{ "party": "N", "rules": [ { "id": "a", "why": "w", "phrases": [" - "] } ] }"#,
        r#"{ "party": "", "rules": [ { "id": "a", "why": "w", "phrases": ["x"] } ] }"#,
        r#"{ "party": "N", "rules": [] }"#,
        "not json",
    ];
    for text in refused
    {
        assert!(Party_Policy_From_Json(text).is_err(), "should be refused: {text}");
    }
}

#[test]
fn Test_A_Diff_Should_Be_Judged_Only_For_What_It_Adds()
{
    let policy = Policy();
    let judge = Judge::New(&policy);
    let diff = "diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -3,0 +4,2 @@\n+clean\n+ported from northwind\n\
                diff --git a/old.rs b/old.rs\n--- a/old.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-northwind was here\n\
                diff --git a/logo.png b/logo.png\nBinary files /dev/null and b/logo.png differ\n\
                diff --git a/nwd/x.rs b/nwd/x.rs\n--- /dev/null\n+++ b/nwd/x.rs\n@@ -0,0 +1 @@\n+fine\n";
    let places: Vec<String> = Judge_Diff(&judge, None, diff).into_iter().map(|refusal| return refusal.place.to_string()).collect();
    assert_eq!(places, ["src/a.rs:5 (the commit being made)", "file name nwd/x.rs (the commit being made)"]);
}

#[test]
fn Test_A_Message_Should_Skip_Git_Comments_And_Everything_Below_The_Scissors()
{
    let policy = Policy();
    let judge = Judge::New(&policy);
    let launcher = Scripted { remote_listing: String::new(), top_level: "C:/work/mine".to_owned() };
    let git = GitReader::New(&launcher, Path::new("."));
    let clean = "tidy\n# northwind in a comment is git's\n# ------------------------ >8 ------------------------\n+northwind";
    assert_eq!(Judge_Message(&judge, &git, clean).expect("scripted git answers"), Verdict::Clean);
    let refused = Judge_Message(&judge, &git, "fix NW-12\n").expect("scripted git answers");
    assert!(matches!(refused, Verdict::Refused(ref found) if found.len() == 1));
}

#[test]
fn Test_Every_Hook_Script_Should_Hand_Over_And_Only_The_Judged_Ones_Should_Judge()
{
    let scripts = Hook_Scripts("C:/bin/nomos.exe", "C:/Users/o'neil/party.json");
    let named = |name: &str| return scripts.iter().find(|script| return script.name == name).map(|script| return script.text.clone()).unwrap_or_default();
    for script in &scripts
    {
        assert!(script.text.contains(&format!("/hooks/{}\"", script.name)), "{} hands over to the repository's own hook", script.name);
    }
    assert!(named("pre-merge-commit").contains("guard pre-commit --policy 'C:/Users/o'\\''neil/party.json'"), "a quote in a path is escaped");
    assert!(named("pre-push").contains("input=$(cat"), "pre-push keeps its standard input for both readers");
    assert!(!named("post-checkout").contains("--policy"), "a handed-over hook judges nothing");
}

/// A launcher that answers the two questions standing aside asks, and nothing else.
struct Scripted
{
    remote_listing: String,
    top_level: String,
}

impl Strategy for Scripted
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl ProgramLauncher for Scripted
{
    fn Run(&self, command: &Command) -> Result<ProgramOutput, String>
    {
        let asks = |word: &str| return command.argv.iter().any(|argument| return argument == word);
        let stdout = if asks("remote") { self.remote_listing.clone() } else if asks("rev-parse") { self.top_level.clone() } else { return Err("not scripted".to_owned()) };
        return Ok(ProgramOutput { outcome: ExitOutcome::Exited { code: 0 }, stdout, stderr: String::new() });
    }
}

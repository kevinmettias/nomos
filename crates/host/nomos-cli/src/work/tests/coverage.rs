//! `nomos work add` and `nomos work widen` judged against the coverage declaration at the
//! repository's root, end to end: the composition root reads the file beside the board, and
//! the code an agent branches on is the one `report` documents. `OD-GATE-036`.

use super::super::{ClaimRequest, ExitCode, Run, WorkCommand, Work_Command_From_String_Arguments};
use super::Arguments;

/// The declaration a repository root carries in the tests that want one.
const DECLARATION: &str = r#"{
    "rules": [
        {
            "record": "OD-EXAMPLE-001",
            "paths": ["crates/rules"],
            "requires": ["nomos-cli", "nomos-integration-tests"],
            "satisfied_by": ["--workspace"]
        }
    ]
}"#;

/// A scratch repository root holding a `work/` board, and the declaration if one was asked for.
///
/// Its own directory under the temporary root rather than a board directly in it: the board's
/// parent is where the declaration is read from, and a board straight under the temporary root
/// would read whatever some other program left there.
struct Repository
{
    root: std::path::PathBuf,
}

impl Repository
{
    fn Named(name: &str, declaration: Option<&str>) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-cli-work-coverage-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("work")).expect("a scratch board directory under the temporary root");
        if let Some(text) = declaration
        {
            std::fs::write(root.join(nomos_ledger::PREDICATE_COVERAGE), text).expect("the scratch root is writable");
        }

        return Self { root };
    }

    /// Runs one `work` command line against this repository's board, and what it printed.
    fn Ran(&self, line: &str) -> (ExitCode, String)
    {
        let command = Work_Command_From_String_Arguments(&Arguments(line)).expect("every line here parses");
        let mut output = Vec::new();

        let code = Run(&command, &self.root.join("work"), &mut output);

        return (code, String::from_utf8(output).expect("work writes only str into the buffer"));
    }

    /// Claims `item` as `agent-a`, so a widening can be asked of it.
    fn Claimed(&self, item: &str)
    {
        let request = ClaimRequest {
            item: nomos_ledger::ItemId::New(item),
            holder: "agent-a".to_owned(),
            lease: std::time::Duration::from_secs(600),
        };

        assert_eq!(Run(&WorkCommand::Claim(request), &self.root.join("work"), &mut Vec::new()), ExitCode::Ok);
    }
}

impl Drop for Repository
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// An add line for `item` reserving `territory`, verified by `predicate`.
fn Add_Line(item: &str, territory: &str, predicate: &str) -> String
{
    return format!(
        "add --item {item} --title t --why w --done-when d --kind capability --origin proposed --territory {territory} -- {predicate}"
    );
}

#[test]
fn Test_Add_Should_Refuse_A_Short_Predicate_With_A_Validation_Error_Naming_What_Is_Missing()
{
    let repository = Repository::Named("add-refused", Some(DECLARATION));

    let (code, said) = repository.Ran(&Add_Line("T-1", "crates/rules/a.rs", "cargo test -p nomos-rules -p nomos-cli"));
    let (_, listed) = repository.Ran("list --all");

    assert_eq!(code, ExitCode::ValidationError, "{said}");
    assert!(said.contains("nomos-integration-tests") && said.contains("OD-EXAMPLE-001"), "{said}");
    assert!(!said.contains("nomos-cli,"), "only the missing argument is named: {said}");
    assert!(!listed.contains("T-1"), "the refused item reached the board: {listed}");
}

#[test]
fn Test_Add_Should_Accept_The_Required_Arguments_Or_One_That_Suffices_Alone()
{
    let repository = Repository::Named("add-accepted", Some(DECLARATION));

    let (both, said_both) = repository.Ran(&Add_Line(
        "T-1",
        "crates/rules/a.rs",
        "cargo test -p nomos-rules -p nomos-cli -p nomos-integration-tests",
    ));
    let (workspace, said_workspace) = repository.Ran(&Add_Line("T-2", "crates/rules/b.rs", "cargo test --workspace"));

    assert_eq!(both, ExitCode::Ok, "{said_both}");
    assert_eq!(workspace, ExitCode::Ok, "{said_workspace}");
}

#[test]
fn Test_Add_Should_Accept_A_Short_Predicate_Reaching_No_Declared_Path_Or_In_A_Repository_Without_A_Declaration()
{
    let declaring = Repository::Named("add-elsewhere", Some(DECLARATION));
    let undeclared = Repository::Named("add-undeclared", None);

    let (elsewhere, said_elsewhere) = declaring.Ran(&Add_Line("T-1", "crates/substrate/a.rs", "cargo test -p nomos-ledger"));
    let (without, said_without) = undeclared.Ran(&Add_Line("T-1", "crates/rules/a.rs", "cargo test -p nomos-rules"));

    assert_eq!(elsewhere, ExitCode::Ok, "{said_elsewhere}");
    assert_eq!(without, ExitCode::Ok, "a repository with no declaration is unaffected: {said_without}");
}

#[test]
fn Test_Widen_Should_Refuse_A_Widening_Into_A_Declared_Path_With_The_Adds_Code()
{
    let repository = Repository::Named("widen-refused", Some(DECLARATION));
    let (added, said_added) = repository.Ran(&Add_Line("T-1", "crates/substrate/a.rs", "cargo test -p nomos-ledger"));
    repository.Claimed("T-1");

    let (code, said) = repository.Ran("widen --item T-1 --holder agent-a --territory crates/rules/b.rs");
    let (elsewhere, said_elsewhere) = repository.Ran("widen --item T-1 --holder agent-a --territory crates/substrate/b.rs");

    assert_eq!(added, ExitCode::Ok, "{said_added}");
    assert_eq!(code, ExitCode::ValidationError, "{said}");
    assert!(said.contains("T-1 cannot widen there") && said.contains("OD-EXAMPLE-001"), "{said}");
    assert_eq!(elsewhere, ExitCode::Ok, "a widening reaching nothing declared is granted: {said_elsewhere}");
}

#[test]
fn Test_Add_Should_Refuse_Under_A_Declaration_That_Does_Not_Parse_As_A_Conflict()
{
    let repository = Repository::Named("add-unreadable", Some("{ \"rules\": ["));

    let (code, said) = repository.Ran(&Add_Line("T-1", "docs/a.md", "cargo test --workspace"));

    assert_eq!(code, ExitCode::Conflict, "{said}");
    assert!(said.contains(nomos_ledger::PREDICATE_COVERAGE), "the refusal names the file to repair: {said}");
}

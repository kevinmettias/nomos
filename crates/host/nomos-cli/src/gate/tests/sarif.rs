//! `run --sarif`: the gate's judgment written as a SARIF log beside the human report, from the
//! command line a CI job actually runs.

use super::super::{ExitCode, Gate_Invocation_From_String_Arguments, GateCommand, Invocation, Run, SarifDestination};
use nomos_contracts::RuleId;
use nomos_gate_orchestration::RuleSelector;
use std::path::PathBuf;

/// A file whose one declared list names a mirror that does not exist -- a real blocking
/// finding, so the log below has something to carry and the run a code worth comparing.
const STALE_MIRROR: &str = "/// A list.\n/// Mirrored by `Test_Sarif_Ghost`.\npub const TABLES: &[&str] = &[];\n";

/// A fresh temporary tree named `case`, holding one file with [`STALE_MIRROR`] in it.
fn Tree_With_A_Stale_Mirror(case: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(case);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(root.join("a.rs"), STALE_MIRROR).expect("the root was created just above, so a new file lands inside it");
    return root;
}

/// A run over `root` scoped to the completeness-mirror rule the fixture is written against, so
/// the run judges what the assertions are about.
fn Scoped_Run(root: &std::path::Path) -> GateCommand
{
    return GateCommand {
        root: root.to_path_buf(),
        rules: RuleSelector { include: vec![RuleId::New(nomos_rules::COMPLETENESS_MIRROR)] },
        ..GateCommand::default()
    };
}

/// `invocation`'s code and both streams.
fn Ran(invocation: &Invocation) -> (ExitCode, Vec<u8>, Vec<u8>)
{
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Run(invocation, &mut stdout, &mut stderr);
    return (code, stdout, stderr);
}

/// `run --sarif <path>` parses to a run carrying its log's destination, `run --sarif` to one
/// writing it to standard output, and every other verb refuses the flag rather than accepting
/// a destination it has no judgment to write to.
#[test]
fn Test_The_Sarif_Flag_Should_Parse_For_Run_And_Be_Refused_By_Every_Other_Verb()
{
    let words = |line: &[&str]| return line.iter().map(|word| return (*word).to_owned()).collect::<Vec<String>>();

    let to_file = Gate_Invocation_From_String_Arguments(&words(&["run", "--sarif", "gate.sarif"])).expect("run takes --sarif");
    let to_stdout = Gate_Invocation_From_String_Arguments(&words(&["run", "--sarif"])).expect("--sarif's path is optional");

    assert!(matches!(to_file, Invocation::RunWithLog { log: SarifDestination::File(ref path), .. } if *path == PathBuf::from("gate.sarif")), "{to_file:?}");
    assert!(matches!(to_stdout, Invocation::RunWithLog { log: SarifDestination::StandardOutput, .. }), "{to_stdout:?}");
    for verb in ["plan", "policy", "steps"]
    {
        let refused = Gate_Invocation_From_String_Arguments(&words(&[verb, "--sarif"])).expect_err("only run reaches a judgment to write");
        assert!(refused.contains("--sarif"), "{verb}: {refused}");
    }
}

/// With no path, standard output is one parseable SARIF 2.1.0 document whose results name the
/// judged file, each result carrying the bucket the gate placed it in; the human report is on
/// standard error, and the exit code is the plain run's.
#[test]
fn Test_A_Gate_Log_On_Standard_Output_Should_Parse_As_One_Document_Naming_The_Judged_File()
{
    let root = Tree_With_A_Stale_Mirror("nomos-cli-gate-sarif-stdout");

    let (plain_code, plain_stdout, _plain_stderr) = Ran(&Invocation::Run(Scoped_Run(&root)));
    let (code, stdout, stderr) = Ran(&Invocation::RunWithLog { command: Scoped_Run(&root), log: SarifDestination::StandardOutput });
    let _ignored = std::fs::remove_dir_all(&root);

    let log: serde_json::Value = serde_json::from_slice(&stdout).expect("standard output carries one SARIF document and nothing else");
    assert_eq!(log.pointer("/version").and_then(serde_json::Value::as_str), Some("2.1.0"), "{log}");
    let results = log.pointer("/runs/0/results").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
    let judged = results
        .iter()
        .find(|result| return result.pointer("/locations/0/physicalLocation/artifactLocation/uri").and_then(serde_json::Value::as_str) == Some("a.rs"))
        .unwrap_or_else(|| panic!("the judged file reaches the log: {log}"));
    assert_eq!(judged.pointer("/properties/bucket").and_then(serde_json::Value::as_str), Some("blocking"), "{judged}");
    assert_eq!(code, plain_code, "the flag changes no exit code");
    // Every run mints a fresh run id and the report prints it first, so two runs of one tree
    // differ in that line alone; the rest of the report moved whole.
    let without_run_id = |text: &[u8]| {
        return String::from_utf8_lossy(text).lines().filter(|line| return !line.starts_with("run: ")).collect::<Vec<&str>>().join("\n");
    };
    assert!(without_run_id(&stderr).contains(&without_run_id(&plain_stdout)), "the report moved to standard error: {}", String::from_utf8_lossy(&stderr));
}

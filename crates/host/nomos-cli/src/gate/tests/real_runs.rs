//! The real runs: each test below drives `Run` over this workspace's own tree, because what
//! this module promises end to end is what the shipped binary does when it is actually
//! called rather than what a mock returns.

use super::super::{ExitCode, Gate_Invocation_From_String_Arguments, GateCommand, Invocation, Run};
use std::path::{Path, PathBuf};

/// `compare` runs two real walks of this workspace's own tree and reports a difference.
///
/// The two sides name the same root deliberately: a tree compared with itself has a known
/// answer -- nothing moved -- which is the one assertion about `compare`'s arithmetic that
/// does not depend on what this workspace's rules happen to find today. That it reaches
/// `Ok` is the other half, and it is the claim `OD-GATE-022`'s Status made about a first
/// caller: two same-process walks, no store, no new serializable type.
#[test]
fn Test_Compare_Should_Report_No_Difference_Between_A_Tree_And_Itself()
{
    let root = Repository_Root();
    let invocation = Gate_Invocation_From_String_Arguments(&Compare_Arguments(&root)).expect("compare parses");

    let summary = Run_Over_This_Tree(invocation);

    assert_eq!(summary.code, ExitCode::Ok, "stderr: {}", summary.diagnostics);
    assert!(summary.output.contains("0 added, 0 removed, 0 changed disposition"), "{}", summary.output);
    // Two executions, not one: a RunId distinguishes them even over one unchanged tree.
    assert!(summary.output.contains("baseline:"), "{}", summary.output);
    assert!(summary.output.contains("candidate:"), "{}", summary.output);
}

/// A side that was never judged is not a difference of zero. `compare` must say which side,
/// and leave the code `run` already gives that reason, rather than rendering an empty
/// difference as though the trees agreed.
#[test]
fn Test_Compare_Should_Refuse_A_Side_That_Was_Never_Judged()
{
    let empty = Repository_Root().join("target").join("nomos-gate-compare-empty-side");
    std::fs::create_dir_all(&empty).expect("the fixture directory");
    let invocation = Gate_Invocation_From_String_Arguments(&Compare_Arguments(&empty)).expect("compare parses");

    let summary = Run_Over_This_Tree(invocation);

    assert_eq!(summary.code, ExitCode::Vacuous, "stderr: {}", summary.diagnostics);
    assert!(summary.diagnostics.contains("--against"), "{}", summary.diagnostics);
    std::fs::remove_dir_all(&empty).ok();
}

/// The `policy` verb over a real tree reports the file that decided a field, through the
/// same composition root `run` walks with.
///
/// The other half of the end-to-end claim `report/tests/policy.rs` makes: that one resolves
/// three declared contributions and renders them, because no composition root can state one
/// field at three layers today; this one proves the two layers that *do* have a source reach
/// a host's report from a real `Run_Gate` rather than from a result a test built.
///
/// Over a scratch tree with no source rather than this workspace's own: what the walk finds
/// does not reach the resolution at all, and judging the whole repository to read back one
/// line of policy would spend thirteen seconds proving nothing extra.
#[test]
fn Test_The_Policy_Verb_Should_Name_The_File_That_Decided_A_Field_On_A_Real_Run()
{
    let root = Repository_Root().join("target").join("nomos-gate-policy-verb");
    std::fs::create_dir_all(&root).expect("the fixture directory");
    std::fs::write(root.join("nomos-gate.json"), r#"{ "coverage": "require-completeness" }"#).expect("the fixture policy");
    let arguments = vec!["policy".to_owned(), "--root".to_owned(), root.display().to_string()];
    let invocation = Gate_Invocation_From_String_Arguments(&arguments).expect("policy parses");

    let summary = Run_Over_This_Tree(invocation);

    std::fs::remove_dir_all(&root).ok();
    assert_eq!(summary.code, ExitCode::Ok, "stderr: {}", summary.diagnostics);
    assert!(summary.output.contains("coverage:"), "{}", summary.output);
    assert!(summary.output.contains("Repository"), "{}", summary.output);
    assert!(summary.output.contains("nomos-gate.json"), "{}", summary.output);
}

/// This repository's own root, three levels above `crates/host/nomos-cli` -- the same
/// derivation `check_command.rs`'s and `agent/tests.rs`'s own copies of this helper use.
///
/// Not `PathBuf::from(".")`. `cargo test` sets a test binary's working directory to its
/// own crate's manifest directory, not the workspace root, so a bare relative "." resolves
/// to `crates/host/nomos-cli` here -- a real, smaller tree that happens to compile and walk
/// without error, which is what makes the mistake quiet. `env!("CARGO_MANIFEST_DIR")` is
/// fixed at compile time and immune to the difference.
fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(PathBuf::from)
        .expect("this crate sits three levels below the workspace root");
}

/// A `compare` command line whose baseline is this workspace's own root and whose candidate
/// is `candidate` -- the two tests above differ only in which tree that is.
fn Compare_Arguments(candidate: &Path) -> Vec<String>
{
    let root = Repository_Root();

    return vec![
        "compare".to_owned(),
        "--root".to_owned(),
        root.display().to_string(),
        "--against".to_owned(),
        candidate.display().to_string(),
    ];
}

/// The code one real run of [`Run`] exited with, and the two streams it wrote.
///
/// A named result rather than a tuple, because `output` and `diagnostics` are both
/// `String`: a caller reading the pair in the wrong order would assert against the wrong
/// stream, and the compiler would not object.
struct RunSummary
{
    code: ExitCode,
    output: String,
    diagnostics: String,
}

/// Runs `invocation` over this workspace's own tree and captures stdout/stderr as owned
/// strings -- the "real command over the real tree" setup
/// `Test_Render_Plan_Should_Report_All_Four_Shipped_Rules` and
/// `Test_Host_Variant_Should_Compose_Into_A_Real_Run_That_Judges_This_Workspaces_Own_Tree`
/// both need, differing only in which verb's [`Invocation`] they build.
fn Run_Over_This_Tree(invocation: Invocation) -> RunSummary
{
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&invocation, &mut stdout, &mut stderr);

    return RunSummary {
        code,
        output: String::from_utf8_lossy(&stdout).into_owned(),
        diagnostics: String::from_utf8_lossy(&stderr).into_owned(),
    };
}

/// A real run over this workspace's own shipped rules reports every one of them, and exits
/// clean.
///
/// End to end, the way the shipped binary is actually called -- `nomos_gate_orchestration
/// ::Registered` composes the offers `P13-GATE-ORCHESTRATION-1`'s own crate test already
/// checks; this is the assertion that the CLI seam renders what came back rather than
/// trusting the crate boundary silently.
///
/// Against [`nomos_check_orchestration::Composed_Rules`] rather than a written-out list of
/// eight names and a literal `rules: 8`, which is what this asserted while the registry was
/// eight rules behind the run and `P35-GATE-020-REGISTRY-WHOLE` closed. Both halves matter:
/// the count catches a rule offered and not rendered, and naming each one catches a count
/// that is right for the wrong reasons.
#[test]
fn Test_Render_Plan_Should_Report_Every_Rule_A_Check_Run_Composes()
{
    let command = GateCommand { root: PathBuf::from("."), ..Default::default() };
    let summary = Run_Over_This_Tree(Invocation::Plan(command));
    let composed = nomos_check_orchestration::Composed_Rules();

    assert_eq!(summary.code, ExitCode::Ok, "{}", summary.output);
    assert!(summary.output.contains(&format!("rules: {}", composed.len())), "{}", summary.output);
    for rule in &composed
    {
        assert!(
            summary.output.contains(rule.As_Str()),
            "{rule} is composed into a check run but the plan does not name it: {}",
            summary.output
        );
    }
    assert!(summary.diagnostics.is_empty());
}

/// The `[Blocking]` findings this workspace's own tree carries today, and why each is
/// accepted rather than fixed.
///
/// Empty today, and that is the assertion rather than the absence of one: with no entry,
/// [`Has_No_Unaccepted_Blocking_Findings`] means this workspace's own tree carries no
/// `[Blocking]` finding at all, which is strictly stronger than the two-named version it
/// replaces.
///
/// Both former entries were against the same file,
/// `tests/integration/fixtures/third-party/hex-0.4.3/lib.rs`, and they left for different
/// reasons. `single-letter-names: T` was a real gap and was fixed: `OD-CAPABILITY-014` put an
/// `impl` block's own generic parameters in the syntax payload, and the rule now exempts an
/// `Implementation` item whose own name is one of them (`P96`, `c278d896`). `abbreviations:
/// val` was *not* fixed and never will be -- this list's own previous text called it a
/// permanent, deliberate true positive, because `hex`'s author really did choose that name --
/// but that reasoning was always about the finding and never about whether this repository
/// walks the file. `P96` (`c3ff169e`) stopped the shared walk descending into a directory
/// carrying its own `standards.json`, so the vendored fixture is no longer judged from this
/// root at all, and `tests/integration/tests/calibration.rs` still judges it from its own.
///
/// A named allowlist rather than a bare count: a finding accepted the same deliberate way
/// must be added here explicitly, and anything not named here fails these tests -- neither
/// `nomos_gate_orchestration::Suppression` nor `RuleCalibration` is wired to a real config
/// file yet, so this allowlist is what stands in for that mechanism.
const ACCEPTED_BLOCKING_FINDINGS: &[&str] = &[];

/// Whether `rendered` carries no `[Blocking]` line other than the ones
/// [`ACCEPTED_BLOCKING_FINDINGS`] names.
fn Has_No_Unaccepted_Blocking_Findings(rendered: &str) -> bool
{
    return rendered
        .lines()
        .filter(|line| return line.starts_with("[Blocking]"))
        .all(|line| return ACCEPTED_BLOCKING_FINDINGS.iter().any(|accepted| return line.starts_with(accepted)));
}

/// A real `run` over this workspace's own tree, end to end -- the same "real run over the
/// real tree" discipline the `plan` test above already uses. This repository's own `Rules`
/// step runs `gate run` over this same tree, so this test, exercising the same command,
/// must agree with itself end to end: `Ok`, carrying no `[Blocking]` finding that
/// [`ACCEPTED_BLOCKING_FINDINGS`] does not name, and the same rule names `plan` already
/// reports must be nameable in the rendered findings' rule ids where any exist.
///
/// `Ok` is a real assertion and not a weaker one. `Test_Walked_Sources_Should_Not_Report_Ok_
/// Over_An_Empty_Tree` below is what keeps it from being satisfied by a run that judged
/// nothing: a walk finding no source reports `NothingJudged`, not this.
#[test]
fn Test_Host_Variant_Should_Compose_Into_A_Real_Run_That_Judges_This_Workspaces_Own_Tree()
{
    let command = GateCommand { root: Repository_Root(), ..Default::default() };
    let summary = Run_Over_This_Tree(Invocation::Run(command));

    assert_eq!(
        summary.code,
        ExitCode::Ok,
        "this workspace's own tree carries nothing that can fail a build today, and \
         ACCEPTED_BLOCKING_FINDINGS is empty to say so; anything but Ok here is either a real \
         regression or a finding somebody meant to accept without naming it: {}",
        summary.output
    );
    assert!(
        Has_No_Unaccepted_Blocking_Findings(&summary.output),
        "a Blocking finding exists that ACCEPTED_BLOCKING_FINDINGS does not name -- a real, \
         new regression, or an accepted finding whose exact rendered text drifted: {}",
        summary.output
    );
    assert!(summary.diagnostics.is_empty());
}

/// A real `run`'s rendered report names the `RunId` `gate.rs` already computes via
/// `Fresh_Run_Id` and threads through `Run_Gate` -- `OD-WORKFLOW-002`'s one real, mechanical
/// gap, closed by `P13-GATE-REPORT-RUNID`. `RunId` renders as 32 lowercase hex characters
/// (`Digest128`'s own `Display`), so this asserts the shape rather than a fixed value: two
/// runs never share an id, the way `Fresh_Run_Id`'s own tests already guarantee.
#[test]
fn Test_Read_Source_Should_Underlie_A_Real_Runs_RunId_Report()
{
    let command = GateCommand { root: Repository_Root(), ..Default::default() };
    let summary = Run_Over_This_Tree(Invocation::Run(command));

    assert_eq!(summary.code, ExitCode::Ok, "{}", summary.output);
    assert!(Has_No_Unaccepted_Blocking_Findings(&summary.output), "{}", summary.output);
    let run_line = summary.output.lines().find(|line| line.starts_with("run: ")).unwrap_or_else(|| panic!("no `run: ` line in: {}", summary.output));
    let hex = run_line.trim_start_matches("run: ");
    assert_eq!(
        hex.len(),
        nomos_contracts::Digest128::HEX_LENGTH,
        "RunId should render as {} hex characters: {run_line}",
        nomos_contracts::Digest128::HEX_LENGTH
    );
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()), "RunId should be lowercase hex: {run_line}");
    assert!(summary.diagnostics.is_empty());
}

/// `run` over an empty tree must not report the same code as a clean run -- `OD-GATE-003`,
/// `Test_Check_Should_Refuse_Ok_Over_An_Empty_Tree`'s own reasoning, now checked at this
/// seam too.
#[test]
fn Test_Walked_Sources_Should_Not_Report_Ok_Over_An_Empty_Tree()
{
    let empty = std::env::temp_dir().join("nomos-cli-gate-run-empty-tree");
    let _ignored = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).expect("creates an empty directory");

    let invocation = Invocation::Run(GateCommand { root: empty.clone(), ..Default::default() });
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&invocation, &mut stdout, &mut stderr);

    let _ignored = std::fs::remove_dir_all(&empty);

    assert_eq!(code, ExitCode::Vacuous, "an empty tree must not report Ok");
}

/// A real `run` scoped to a file that does not exist judges nothing, the same `Vacuous`
/// exit an empty tree already gets -- "scoped to nothing" and "found nothing" are the same
/// claim to a caller.
#[test]
fn Test_Run_Should_Not_Report_Ok_When_Scoped_To_Nothing()
{
    let invocation = Invocation::Run(GateCommand {
        root: PathBuf::from("."),
        scope: nomos_gate_orchestration::ScopeSelector {
            include: vec!["does/not/exist.rs".to_owned()],
            exclude: Vec::new(),
        },
        rules: nomos_gate_orchestration::RuleSelector::default(),
        suppressions: nomos_gate_orchestration::SuppressionPolicy::default(),
        baseline: nomos_gate_orchestration::BaselinePolicy::default(),
        adoption: nomos_gate_orchestration::AdoptionPolicy::default(),
        coverage: nomos_gate_orchestration::CoveragePolicy::default(),
        model: None,
        phases: Vec::new(),
        approvals: Vec::new(),
    });
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&invocation, &mut stdout, &mut stderr);

    assert_eq!(code, ExitCode::Vacuous, "a scope matching nothing must not report Ok");
}

/// A real `explain` over this workspace's own clean tree, for a query naming a finding
/// that does not exist, answers `not found` and exits clean -- end to end, the same "real
/// run over the real tree" discipline `run`'s and `plan`'s own tests already use.
#[test]
fn Test_Relative_Path_Should_Underlie_A_Real_Explains_Search_Over_A_Clean_Tree()
{
    let invocation = Invocation::Explain {
        command: GateCommand { root: PathBuf::from("."), ..Default::default() },
        query: nomos_gate_orchestration::FindingQuery {
            rule: nomos_contracts::RuleId::New("naming-convention"),
            location: "does/not/exist.rs".to_owned(),
        },
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&invocation, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();

    assert_eq!(code, ExitCode::Ok, "{rendered}");
    assert!(rendered.contains("not found"), "{rendered}");
    assert!(String::from_utf8_lossy(&stderr).is_empty());
}

/// The three outcomes, rendered and exited, end to end through this module's own `Run`.
///
/// # Why the exit codes pair the way they do
///
/// `Permitted` and `NotJudged` both exit `Ok` and only `Refused` exits `Violations`. The
/// pairing is the one place this verb could mislead a script: failing on `NotJudged` would
/// stop a build over a crate this workspace has no opinion about, and succeeding on `Refused`
/// would let the edge through. So the assertion below pins the codes *and* the words, because
/// the words are what tell a person "yes" from "no answer" when the code cannot.
#[test]
fn Test_Admits_Should_Report_Each_Outcome_With_Its_Own_Code_And_Words()
{
    for (depending, depended, code, expected) in [
        ("nomos-rules", "nomos-contracts", ExitCode::Ok, "permitted"),
        ("nomos-contracts", "nomos-rules", ExitCode::Violations, "refused"),
        ("serde_json", "nomos-contracts", ExitCode::Ok, "not judged"),
    ]
    {
        let invocation = Invocation::Admits {
            depending: depending.to_owned(),
            depended: depended.to_owned(),
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let exited = Run(&invocation, &mut stdout, &mut stderr);

        let reported = String::from_utf8_lossy(&stdout).into_owned();
        assert_eq!(exited, code, "{depending} -> {depended}: {reported}");
        assert!(reported.contains(expected), "{depending} -> {depended}: {reported}");
        assert!(stderr.is_empty(), "an answer is not a diagnostic: {}", String::from_utf8_lossy(&stderr));
    }
}

/// `admits` walks no tree, which is what makes it answerable before the edge is written, and
/// it takes no `--root` for the same reason. It does read one file: the architecture it judges
/// against is the repository's own declaration, found by searching upward from the working
/// directory the way `cargo` finds a manifest -- so the verb answers from any subdirectory,
/// which is what this asserts. `cargo test` runs this binary with the crate directory as its
/// working directory, four levels below the declaration, so a version that read only `.` fails
/// here.
///
/// # What this deliberately no longer does
///
/// It used to `set_current_dir` to the platform temporary directory and require a `permitted`
/// answer from outside any repository, on the premise that `admits` reads nothing from disk.
/// That premise was true while the architecture was compiled into `nomos-rules` and is not any
/// more: outside a repository there is no architecture to answer from, and `not judged` is the
/// honest outcome.
///
/// The property is still asserted -- in `nomos_gate_orchestration::admissibility`'s own
/// `Test_A_Repository_That_Declared_Nothing_Should_Not_Be_Judged`, where a declaration that
/// declares nothing is a value rather than a directory. That is the better home for it and not
/// merely an available one: `set_current_dir` mutates process-global state, the Rust harness
/// runs tests as threads of one process, and every other test resolving a relative path raced
/// this one for as long as it stood here.
#[test]
fn Test_Admits_Should_Answer_From_A_Subdirectory_Of_The_Declaring_Repository()
{
    let invocation = Invocation::Admits {
        depending: "nomos-rules".to_owned(),
        depended: "nomos-contracts".to_owned(),
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exited = Run(&invocation, &mut stdout, &mut stderr);

    assert_eq!(exited, ExitCode::Ok);
    let reported = String::from_utf8_lossy(&stdout).into_owned();
    assert!(reported.contains("permitted"), "{reported}");
    assert!(stderr.is_empty(), "an answer is not a diagnostic: {}", String::from_utf8_lossy(&stderr));
}

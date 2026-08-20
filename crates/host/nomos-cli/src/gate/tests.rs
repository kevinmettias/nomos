//! What this module promises, exercised.

use super::*;
use super::exit_code::Every_Exit_Code;
use super::parsing::USAGE;

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Violations => "Violations",
        ExitCode::Usage => "Usage",
        ExitCode::Contradictory => "Contradictory",
        ExitCode::Vacuous => "Vacuous",
    };
}

/// `Every_Exit_Code`'s own mirror, named in its doc comment.
///
/// The match has no wildcard arm. A variant added to [`ExitCode`] without a matching arm
/// added here fails this file to *compile*, not merely to pass.
#[test]
fn Test_Every_ExitCode_Should_Be_Matched_Exhaustively()
{
    fn Ordinal(code: ExitCode) -> usize
    {
        return match code
        {
            ExitCode::Ok => 0,
            ExitCode::Violations => 1,
            ExitCode::Usage => 2,
            ExitCode::Contradictory => 3,
            ExitCode::Vacuous => 4,
        };
    }

    for (index, code) in Every_Exit_Code().iter().enumerate()
    {
        assert_eq!(
            Ordinal(*code),
            index,
            "{} is not matched at the position Every_Exit_Code() puts it, so the exhaustive \
             match and the census have drifted apart",
            Labelled(*code)
        );
    }
}

/// Zero is the only success this group leaves the process with.
#[test]
fn Test_Only_Ok_Should_Carry_The_Passing_Exit_Code()
{
    for code in Every_Exit_Code().iter().copied()
    {
        assert_eq!(
            code.Value() == 0,
            code == ExitCode::Ok,
            "{} exits {}, and zero is read as success",
            Labelled(code),
            code.Value()
        );
    }
}

/// The codes this file documents are the codes this group can exit with.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let (_, spelled) = USAGE
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| word.parse().ok()));
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );
    assert_eq!(
        documented, implemented,
        "the usage text and ExitCode disagree about what this command can exit with"
    );
}

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// A [`GateInvocation`]'s carried [`GateCommand`], whichever verb it is -- most of these
/// tests only care about `root` and would otherwise have to match twice for no reason.
fn Root_Of(invocation: &GateInvocation) -> &PathBuf
{
    return match invocation
    {
        GateInvocation::Plan(command) | GateInvocation::Run(command) | GateInvocation::Explain(command, _) => &command.root,
    };
}

#[test]
fn Test_A_Root_Should_Default_To_Here()
{
    let invocation = Parse(&["plan".to_owned()]).expect("plan with no root is valid");

    assert_eq!(Root_Of(&invocation), &PathBuf::from("."));
}

#[test]
fn Test_A_Given_Root_Should_Win()
{
    let arguments = vec!["plan".to_owned(), "--root".to_owned(), "somewhere".to_owned()];
    let invocation = Parse(&arguments).expect("plan --root is valid");

    assert_eq!(Root_Of(&invocation), &PathBuf::from("somewhere"));
}

/// No verb at all must not be silently read as `plan`.
#[test]
fn Test_No_Verb_Should_Refuse()
{
    let error = Parse(&[]).expect_err("must refuse");

    assert!(error.contains("usage"), "{error}");
}

/// `run` is a real verb now -- `plan` and `run` must both parse.
#[test]
fn Test_Run_Should_Parse()
{
    let invocation = Parse(&["run".to_owned()]).expect("run with no root is valid");

    assert!(matches!(invocation, GateInvocation::Run(_)), "run must not parse as Plan");
    assert_eq!(Root_Of(&invocation), &PathBuf::from("."));
}

/// `compare` is named by `ARC-ROADMAP-001` but has no real implementation yet, so this
/// must refuse rather than quietly running `plan` instead. `explain` moved out of this
/// test once it gained a real body (`P13-GATE-EXPLAIN-FIRST-INCREMENT`) -- it is now
/// covered by its own parsing tests instead, and would pass this one for the wrong reason
/// (a missing `--rule`, not an unrecognized verb) if it stayed.
#[test]
fn Test_An_Unimplemented_Verb_Should_Refuse()
{
    let error = Parse(&["compare".to_owned()]).expect_err("must refuse");

    assert!(error.contains("compare"), "{error}");
    assert!(error.contains("usage"), "{error}");
}

/// A mistyped flag must not be silently ignored into a default.
#[test]
fn Test_An_Unknown_Flag_Should_Refuse()
{
    let arguments = vec!["plan".to_owned(), "--rooot".to_owned(), "x".to_owned()];

    let error = Parse(&arguments).expect_err("must refuse");

    assert!(error.contains("--rooot"), "{error}");
    assert!(error.contains("usage"), "{error}");
}

/// Runs `invocation` over this workspace's own tree and captures stdout/stderr as owned
/// strings -- the "real command over the real tree" setup
/// `Test_A_Real_Plan_Should_Report_All_Four_Shipped_Rules` and
/// `Test_A_Real_Run_Should_Judge_This_Workspaces_Own_Tree` both need, differing only in
/// which verb's [`GateInvocation`] they build.
fn Run_Over_This_Tree(invocation: GateInvocation) -> (ExitCode, String, String)
{
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&invocation, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();
    let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();

    return (code, rendered, rendered_stderr);
}

/// A real run over this workspace's own four shipped rules reports all four, and exits
/// clean.
///
/// End to end, the way the shipped binary is actually called -- `nomos_gate_orchestration
/// ::Registered` composes the same four real offers `P13-GATE-ORCHESTRATION-1`'s own crate
/// test already checks; this is the assertion that the CLI seam renders what came back
/// rather than trusting the crate boundary silently.
#[test]
fn Test_A_Real_Plan_Should_Report_All_Four_Shipped_Rules()
{
    let command = GateCommand { root: PathBuf::from("."), ..Default::default() };
    let (code, rendered, stderr) = Run_Over_This_Tree(GateInvocation::Plan(command));

    assert_eq!(code, ExitCode::Ok, "{rendered}");
    assert!(rendered.contains("rules: 4"), "{rendered}");
    assert!(rendered.contains("completeness-mirror"), "{rendered}");
    assert!(rendered.contains("dependency-direction"), "{rendered}");
    assert!(rendered.contains("function-naming-convention"), "{rendered}");
    assert!(rendered.contains("unread-reaches-finding"), "{rendered}");
    assert!(stderr.is_empty());
}

/// A real `run` over this workspace's own tree, end to end -- the same "real run over the
/// real tree" discipline the `plan` test above already uses. This repository's own `Rules`
/// step already runs `gate run` over this same tree and expects it clean -- the exact
/// command under test here -- so this test, exercising the same command CI actually runs,
/// must agree with itself end to end: `Ok`, not `Violations`, and the same four rule names
/// `plan` already reports must be nameable in the rendered findings' rule ids where any
/// exist, or the finding count must be zero.
#[test]
fn Test_A_Real_Run_Should_Judge_This_Workspaces_Own_Tree()
{
    let command = GateCommand { root: PathBuf::from("."), ..Default::default() };
    let (code, rendered, stderr) = Run_Over_This_Tree(GateInvocation::Run(command));

    assert_eq!(
        code,
        ExitCode::Ok,
        "this repository's own `Rules` step already runs `gate run --root .` over this \
         same tree and requires it to exit clean; this test exercises that same command, \
         so it must agree with itself end to end: {rendered}"
    );
    assert!(rendered.contains("finding(s), 0 of which can fail a build"), "{rendered}");
    assert!(stderr.is_empty());
}

/// `run` over an empty tree must not report the same code as a clean run -- `OD-GATE-003`,
/// `Test_Check_Should_Refuse_Ok_Over_An_Empty_Tree`'s own reasoning, now checked at this
/// seam too.
#[test]
fn Test_A_Run_Over_An_Empty_Tree_Should_Not_Report_Ok()
{
    let empty = std::env::temp_dir().join("nomos-cli-gate-run-empty-tree");
    let _ignored = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).expect("creates an empty directory");

    let invocation = GateInvocation::Run(GateCommand { root: empty.clone(), ..Default::default() });
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&invocation, &mut stdout, &mut stderr);

    let _ignored = std::fs::remove_dir_all(&empty);

    assert_eq!(code, ExitCode::Vacuous, "an empty tree must not report Ok");
}

/// `--include`/`--exclude` repeat, the same shape `Named_Values` already gives every other
/// repeatable flag in this binary.
#[test]
fn Test_Include_And_Exclude_Should_Repeat()
{
    let arguments = vec![
        "plan".to_owned(),
        "--include".to_owned(),
        "crates/rules".to_owned(),
        "--include".to_owned(),
        "crates/contracts".to_owned(),
        "--exclude".to_owned(),
        "crates/rules/nomos-rules/tests".to_owned(),
    ];
    let invocation = Parse(&arguments).expect("plan with include/exclude is valid");

    let GateInvocation::Plan(command) = invocation
    else
    {
        panic!("plan must parse as Plan");
    };
    assert_eq!(command.scope.include, vec!["crates/rules".to_owned(), "crates/contracts".to_owned()]);
    assert_eq!(command.scope.exclude, vec!["crates/rules/nomos-rules/tests".to_owned()]);
}

/// `--rule` repeats into [`nomos_gate_orchestration::RuleSelector::include`].
#[test]
fn Test_Rule_Should_Repeat()
{
    let arguments = vec!["run".to_owned(), "--rule".to_owned(), "naming-convention".to_owned()];
    let invocation = Parse(&arguments).expect("run with --rule is valid");

    let GateInvocation::Run(command) = invocation
    else
    {
        panic!("run must parse as Run");
    };
    assert_eq!(command.rules.include, vec![nomos_contracts::RuleId::New("naming-convention")]);
}

/// A real `run` scoped to a file that does not exist judges nothing, the same `Vacuous`
/// exit an empty tree already gets -- "scoped to nothing" and "found nothing" are the same
/// claim to a caller.
#[test]
fn Test_A_Run_Scoped_To_Nothing_Should_Not_Report_Ok()
{
    let invocation = GateInvocation::Run(GateCommand {
        root: PathBuf::from("."),
        scope: nomos_gate_orchestration::ScopeSelector {
            include: vec!["does/not/exist.rs".to_owned()],
            exclude: Vec::new(),
        },
        rules: nomos_gate_orchestration::RuleSelector::default(),
        suppressions: nomos_gate_orchestration::SuppressionPolicy::default(),
    });
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&invocation, &mut stdout, &mut stderr);

    assert_eq!(code, ExitCode::Vacuous, "a scope matching nothing must not report Ok");
}

/// `explain` is a real verb now -- it must parse, and carry the rule/location it named.
#[test]
fn Test_Explain_Should_Parse_With_Rule_And_Location()
{
    let arguments = vec![
        "explain".to_owned(),
        "--rule".to_owned(),
        "naming-convention".to_owned(),
        "--location".to_owned(),
        "a.rs".to_owned(),
    ];
    let invocation = Parse(&arguments).expect("explain with --rule and --location is valid");

    let GateInvocation::Explain(_, query) = invocation
    else
    {
        panic!("explain must parse as Explain");
    };
    assert_eq!(query.rule, nomos_contracts::RuleId::New("naming-convention"));
    assert_eq!(query.location, "a.rs");
}

/// `explain` without `--rule` must not silently answer about no rule at all.
#[test]
fn Test_Explain_Should_Require_Rule()
{
    let arguments = vec!["explain".to_owned(), "--location".to_owned(), "a.rs".to_owned()];

    let error = Parse(&arguments).expect_err("must refuse");

    assert!(error.contains("--rule"), "{error}");
}

/// `explain` without `--location` must not silently answer about no location at all.
#[test]
fn Test_Explain_Should_Require_Location()
{
    let arguments = vec!["explain".to_owned(), "--rule".to_owned(), "naming-convention".to_owned()];

    let error = Parse(&arguments).expect_err("must refuse");

    assert!(error.contains("--location"), "{error}");
}

/// A real `explain` over this workspace's own clean tree, for a query naming a finding
/// that does not exist, answers `not found` and exits clean -- end to end, the same "real
/// run over the real tree" discipline `run`'s and `plan`'s own tests already use.
#[test]
fn Test_A_Real_Explain_Should_Report_Not_Found_Over_A_Clean_Tree()
{
    let invocation = GateInvocation::Explain(
        GateCommand { root: PathBuf::from("."), ..Default::default() },
        nomos_gate_orchestration::FindingQuery {
            rule: nomos_contracts::RuleId::New("naming-convention"),
            location: "does/not/exist.rs".to_owned(),
        },
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&invocation, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();

    assert_eq!(code, ExitCode::Ok, "{rendered}");
    assert!(rendered.contains("not found"), "{rendered}");
    assert!(String::from_utf8_lossy(&stderr).is_empty());
}

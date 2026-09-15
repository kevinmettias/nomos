//! The command line `gate` accepts: every verb and flag it parses, and every word it refuses.

use super::super::{Gate_Invocation_From_String_Arguments, Invocation};
use std::path::PathBuf;

/// An [`Invocation`]'s carried [`GateCommand`], whichever verb it is -- most of these
/// tests only care about `root` and would otherwise have to match twice for no reason.
fn Root_Of(invocation: &Invocation) -> &PathBuf
{
    return match invocation
    {
        Invocation::Plan(command) | Invocation::Run(command) | Invocation::Explain { command, .. } => &command.root,
        // The baseline's, because `--root` spells it for every verb including this one;
        // `--against`'s is reached through the variant itself where a test needs both.
        Invocation::Compare { baseline, .. } => &baseline.root,
        // `admits` carries no `GateCommand` and no root, deliberately: it reads nothing from
        // disk, which is `OD-GATE-026`'s reason it can answer before the edge exists. A test
        // asking this one for a root has misunderstood the verb rather than found a gap, so
        // it says so here rather than inventing `.` and answering about the wrong thing.
        Invocation::Admits { .. } =>
        {
            panic!("`admits` has no root; it judges a crate pair rather than a tree")
        }
    };
}

#[test]
fn Test_A_Root_Should_Default_To_Here()
{
    let invocation = Gate_Invocation_From_String_Arguments(&["plan".to_owned()]).expect("plan with no root is valid");

    assert_eq!(Root_Of(&invocation), &PathBuf::from("."));
}

#[test]
fn Test_A_Given_Root_Should_Win()
{
    let arguments = vec!["plan".to_owned(), "--root".to_owned(), "somewhere".to_owned()];
    let invocation = Gate_Invocation_From_String_Arguments(&arguments).expect("plan --root is valid");

    assert_eq!(Root_Of(&invocation), &PathBuf::from("somewhere"));
}

/// No verb at all must not be silently read as `plan`.
#[test]
fn Test_No_Verb_Should_Refuse()
{
    let error = Gate_Invocation_From_String_Arguments(&[]).expect_err("must refuse");

    assert!(error.contains("usage"), "{error}");
}

/// `run` is a real verb now -- `plan` and `run` must both parse.
#[test]
fn Test_Gate_Invocation_From_String_Arguments_Should_Parse_Run_As_Run_Not_Plan()
{
    let invocation = Gate_Invocation_From_String_Arguments(&["run".to_owned()]).expect("run with no root is valid");

    assert!(matches!(invocation, Invocation::Run(_)), "run must not parse as Plan");
    assert_eq!(Root_Of(&invocation), &PathBuf::from("."));
}

/// A verb this group does not implement must refuse rather than quietly running `plan`
/// instead. `explain` moved out of this test once it gained a real body
/// (`P13-GATE-EXPLAIN-FIRST-INCREMENT`) and `compare` when it gained one
/// (`P73-GATE-COMPARE-HAS-NO-CALLER`) -- each is covered by its own tests now, and either
/// would pass this one for the wrong reason (a missing required flag, not an unrecognized
/// verb) if it stayed. `diff` is a name a person might plausibly reach for and this group
/// does not answer to.
#[test]
fn Test_An_Unimplemented_Verb_Should_Refuse()
{
    let error = Gate_Invocation_From_String_Arguments(&["diff".to_owned()]).expect_err("must refuse");

    assert!(error.contains("diff"), "{error}");
    assert!(error.contains("usage"), "{error}");
}

/// A mistyped flag must not be silently ignored into a default.
#[test]
fn Test_An_Unknown_Flag_Should_Refuse()
{
    let arguments = vec!["plan".to_owned(), "--rooot".to_owned(), "x".to_owned()];

    let error = Gate_Invocation_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--rooot"), "{error}");
    assert!(error.contains("usage"), "{error}");
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
    let invocation = Gate_Invocation_From_String_Arguments(&arguments).expect("plan with include/exclude is valid");

    let Invocation::Plan(command) = invocation
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
    let invocation = Gate_Invocation_From_String_Arguments(&arguments).expect("run with --rule is valid");

    let Invocation::Run(command) = invocation
    else
    {
        panic!("run must parse as Run");
    };
    assert_eq!(command.rules.include, vec![nomos_contracts::RuleId::New("naming-convention")]);
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
    let invocation = Gate_Invocation_From_String_Arguments(&arguments).expect("explain with --rule and --location is valid");

    let Invocation::Explain { query, .. } = invocation
    else
    {
        panic!("explain must parse as Explain");
    };
    assert_eq!(query.rule, nomos_contracts::RuleId::New("naming-convention"));
    assert_eq!(query.location, "a.rs");
}

/// Every argument list `explain` refuses for a missing `--rule` -- a named provider so
/// another missing-`--rule` scenario is an entry here, not a second copy of the test below.
fn Explain_Arguments_Missing_Rule() -> Vec<Vec<String>>
{
    return vec![vec!["explain".to_owned(), "--location".to_owned(), "a.rs".to_owned()]];
}

/// `explain` without `--rule` must not silently answer about no rule at all.
#[test]
fn Test_Explain_Should_Require_Rule()
{
    for arguments in Explain_Arguments_Missing_Rule()
    {
        let error = Gate_Invocation_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--rule"), "{error}");
    }
}

/// Every argument list `explain` refuses for a missing `--location` -- a named provider so
/// another missing-`--location` scenario is an entry here, not a second copy of the test
/// below.
fn Explain_Arguments_Missing_Location() -> Vec<Vec<String>>
{
    return vec![vec!["explain".to_owned(), "--rule".to_owned(), "naming-convention".to_owned()]];
}

/// `explain` without `--location` must not silently answer about no location at all.
#[test]
fn Test_Explain_Should_Require_Location()
{
    for arguments in Explain_Arguments_Missing_Location()
    {
        let error = Gate_Invocation_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--location"), "{error}");
    }
}

/// `admits` parses into its own variant and carries both crate names.
#[test]
fn Test_Admits_Should_Parse_Into_Its_Own_Variant()
{
    let arguments = vec![
        "admits".to_owned(),
        "--from".to_owned(),
        "nomos-rules".to_owned(),
        "--to".to_owned(),
        "nomos-contracts".to_owned(),
    ];

    let invocation = Gate_Invocation_From_String_Arguments(&arguments).expect("both flags are given");

    let Invocation::Admits { depending, depended } = invocation
    else
    {
        panic!("admits parsed as another verb");
    };
    assert_eq!(depending, "nomos-rules");
    assert_eq!(depended, "nomos-contracts");
}

/// Both flags are required, and the refusal names the missing one.
///
/// Each direction separately: a parser reading one flag and defaulting the other would pass
/// a single test and answer about a pair nobody asked about.
#[test]
fn Test_Admits_Should_Refuse_A_Missing_Crate_By_Name()
{
    let missing_to = Gate_Invocation_From_String_Arguments(&[
        "admits".to_owned(),
        "--from".to_owned(),
        "nomos-rules".to_owned(),
    ])
    .expect_err("--to is required");
    let missing_from = Gate_Invocation_From_String_Arguments(&[
        "admits".to_owned(),
        "--to".to_owned(),
        "nomos-contracts".to_owned(),
    ])
    .expect_err("--from is required");

    assert!(missing_to.contains("--to"), "{missing_to}");
    assert!(missing_from.contains("--from"), "{missing_from}");
}

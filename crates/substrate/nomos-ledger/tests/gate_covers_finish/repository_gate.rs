//! The repository's own gate is still derivable.


/// A guard on the real workflow rather than a fixture. If the repository's gate is
/// renamed or rewritten as a script, finishing anything stops working — and this test is
/// what says so, instead of the next author discovering it mid-finish.
#[test]
fn Test_This_Repository_Gate_Should_Still_Yield_A_Lint_Step()
{
    use std::path::Path;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let workflow = std::fs::read_to_string(nomos_ledger::Workflow_Path(&root))
        .expect("this repository has a gate workflow");

    let argv = nomos_ledger::Derive_Step(&workflow, nomos_ledger::LINT_STEP)
        .expect("the repository's gate must still yield a lint step");

    assert_eq!(argv.first().map(String::as_str), Some("cargo"));
    assert!(argv.iter().any(|argument| return argument == "clippy"));
    assert!(
        argv.iter().any(|argument| return argument == "-D"),
        "the lint step must still deny warnings, got {argv:?}"
    );
}

/// `Duration` is used by the predicate's timeout; this keeps the import honest if the
/// fixture above ever stops constructing one.
#[test]
fn Test_A_Predicate_Should_Carry_A_Timeout()
{
    use nomos_ledger::VerificationPredicate;
    use std::time::Duration;

    let predicate = VerificationPredicate::New(vec!["cargo".to_owned()]);

    assert!(Duration::from_secs(predicate.timeout_seconds) > Duration::ZERO);
}

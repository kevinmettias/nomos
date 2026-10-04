//! The `nomos.cap.syntax.sites` family through a real run: the composed Rust offer materializes
//! each source's sites, and the rule over the labeled-jump kind judges them.

use nomos_analysis::MemoryFactStore;
use nomos_contracts::{Applicability, Finding, RuleId};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::SourceFile;

use crate::composed_providers::ComposedProviders;
use crate::{CheckOutcome, Run, RunContext};

use super::{Bounded_Providers, Scratch_Directory, Source_File, SourceText, Test_Variant};

/// One `Run` selecting only the labeled-jump rule over `sources` through `providers`, reduced to
/// its findings.
fn Labeled_Jump_Findings(sources: &[SourceFile], name: &str, providers: &ComposedProviders<StdProgramLauncher, StdFileSystem, StdEnvironment>) -> Vec<Finding>
{
    let root = Scratch_Directory(name);
    let outcome = Run(
        sources,
        RunContext { variant: Test_Variant(), root: &root, launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New(), providers },
        &[RuleId::New(nomos_rules::A_LABELED_JUMP_LEAVES_ONE_LOOP)],
    );

    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("a tree of readable sources must be judged: {outcome:?}");
    };

    return findings;
}

/// A label on the loop a `match` arm jumps out of buys nothing in Rust, and the doubly-nested
/// search's label is what a label is for: one finding, on the noise label's own line, through the
/// composed provider and the materialization a selected rule demands.
#[test]
fn Test_Run_Should_Report_A_Rust_Label_No_Jump_Needs_And_Keep_One_That_Is_Needed()
{
    let noise = "fn drain(kinds: &[i32]) {\n    'loop_kinds: for kind in kinds {\n        match kind {\n            0 => break 'loop_kinds,\n            _ => continue,\n        }\n    }\n}\n";
    let needed = "fn search(grid: &[Vec<i32>], needle: i32) -> bool {\n    'outer: for row in grid {\n        for cell in row {\n            if *cell == needle {\n                break 'outer;\n            }\n        }\n    }\n    false\n}\n";

    let findings = Labeled_Jump_Findings(
        &[Source_File("drain.rs", SourceText(noise)), Source_File("search.rs", SourceText(needed))],
        "labeled-jump-rust",
        &Bounded_Providers(),
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert_eq!(found.subject_name, "drain.rs:2");
    assert!(found.summary.contains("`'loop_kinds`"), "{}", found.summary);
}

/// A recognized source no composed sites row answers for is never a clean pass: it has no sites,
/// so the rule reports it unjudged. Proven over a table with the sites rows taken out, so the
/// proof does not depend on which languages happen to offer the family today.
#[test]
fn Test_Run_Should_Not_Pass_A_Source_No_Sites_Provider_Answers_For()
{
    let without_sites = ComposedProviders { sites: Vec::new(), ..Bounded_Providers() };

    let findings = Labeled_Jump_Findings(&[Source_File("drain.rs", SourceText("fn drain() { 'l: loop { break 'l; } }\n"))], "labeled-jump-unanswered", &without_sites);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert_ne!(found.applicability, Applicability::Supported, "{found:?}");
    assert!(found.summary.contains("were not judged"), "{}", found.summary);
}

/// The corpus check's own Go positive, `Test_A_Go_Label_Nothing_Needs_Is_Reported`, through the
/// composed Go offer: one finding on the label's line, saying to delete it.
#[test]
fn Test_Run_Should_Report_A_Go_Label_Nothing_Needs()
{
    let first = "package p\n\nfunc first(values []int) int {\nLoop:\n    for _, value := range values {\n        if value > 0 {\n            break Loop\n        }\n    }\n    return 0\n}\n";

    let findings = Labeled_Jump_Findings(&[Source_File("first.go", SourceText(first))], "labeled-jump-go", &Bounded_Providers());

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert_eq!(found.subject_name, "first.go:4", "{found:?}");
    assert!(found.summary.contains("Delete the label"), "{}", found.summary);
}

/// The corpus check's `Test_A_Go_Break_Out_Of_A_Switch_Is_Not_Reported`: Go's bare `break` is caught
/// by the `switch`, so the label naming the innermost loop is the only way out, and nothing is
/// reported -- the shape a check reasoning from depth alone would wrongly convict.
#[test]
fn Test_Run_Should_Not_Report_A_Go_Break_Out_Of_A_Switch()
{
    let drain = "package p\n\nfunc drain(kinds []int) {\nLoop:\n    for _, kind := range kinds {\n        switch kind {\n        case 0:\n            break Loop\n        }\n    }\n}\n";

    let findings = Labeled_Jump_Findings(&[Source_File("drain.go", SourceText(drain))], "labeled-jump-go-switch", &Bounded_Providers());

    assert_eq!(findings, Vec::new());
}

/// A C# file is `NotApplicable`, with C#'s reason, and never a clean pass or a gap: the composed C#
/// offer declines the kind.
#[test]
fn Test_Run_Should_Report_A_Csharp_File_As_Not_Applicable()
{
    let csharp = "class C\n{\n    void F(int[] xs)\n    {\n        foreach (var x in xs) { if (x == 0) { break; } }\n    }\n}\n";

    let findings = Labeled_Jump_Findings(&[Source_File("C.cs", SourceText(csharp))], "labeled-jump-csharp", &Bounded_Providers());

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert_eq!(found.applicability, Applicability::NotApplicable, "{found:?}");
    assert!(found.summary.contains("cannot name a loop"), "{}", found.summary);
}

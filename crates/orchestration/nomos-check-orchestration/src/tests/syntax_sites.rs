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

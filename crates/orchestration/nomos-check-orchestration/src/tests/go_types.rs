//! Go's discarded values through [`crate::Run`], over temporary trees: an error thrown away with
//! `_` is reported by the rule that judges it, a string thrown away is not, and a module on a host
//! with no Go is reported rather than judged clean.
//!
//! The first test runs the real Go toolchain -- `go` on the path, or `GOROOT` naming one -- and fails
//! rather than passes without it, for the reason `nomos-lang-go-types`' own guarantee test gives.

use nomos_analysis::MemoryFactStore;
use nomos_contracts::{Applicability, Finding, RuleId};
use nomos_platform::Environment;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use std::path::{Path, PathBuf};

use crate::{CheckOutcome, Run, RunContext};

use super::go_lint::MissingGo;
use super::{Bounded_Providers, Scratch_Directory, Source_File, SourceText, Test_Variant};

/// Four discards: an unexplained error from a call in another package's standard library at line
/// 10, a string at line 11 -- which a reading of the text alone reported as an error -- an error
/// hidden in a tuple at line 12, which that reading never saw, and an explained error at line 14.
const MAIN: &str = "package main

import (
\t\"fmt\"
\t\"os\"
\t\"strconv\"
)

func main() {
\t_ = os.Remove(\"a\")
\t_ = fmt.Sprintf(\"%d\", 1)
\tn, _ := strconv.Atoi(\"1\")
\t// a missing file is already the state this wants
\t_ = os.Remove(\"b\")
\tfmt.Println(n)
}
";

fn Go_Tree(name: &str) -> PathBuf
{
    let root = Scratch_Directory(name);
    std::fs::write(root.join("go.mod"), "module example.com/app\n\ngo 1.22\n").expect("a scratch module");
    std::fs::write(root.join("main.go"), MAIN).expect("a scratch source");
    return root;
}

/// The discarded-error rule's findings over `sources` under `root`.
fn Discard_Findings<Env: Environment>(root: &Path, sources: &[nomos_rules::SourceFile], environment: &Env) -> Vec<Finding>
{
    let providers = Bounded_Providers();
    let context = RunContext {
        variant: Test_Variant(),
        root,
        launcher: &StdProgramLauncher,
        filesystem: &StdFileSystem,
        environment,
        workspace: &mut None,
        store: &mut MemoryFactStore::New(),
        providers: &providers,
    };

    let CheckOutcome::Judged { findings, .. } = Run(sources, context, &[RuleId::New(nomos_rules::A_DISCARDED_ERROR_IS_EXPLAINED)])
    else
    {
        panic!("a tree whose sources the syntax providers read must be judged");
    };
    return findings;
}

/// Exactly the two unexplained errors are reported, each at its line: the string is not an error,
/// and the explained one is explained by a comment the rule reads from the walked file's text.
#[test]
fn Test_Only_An_Unexplained_Discarded_Error_Should_Be_Reported()
{
    let root = Go_Tree("go-types-discards");

    let findings = Discard_Findings(&root, &[Source_File("main.go", SourceText(MAIN))], &StdEnvironment);

    let judged: Vec<&str> = findings.iter().filter(|finding| return finding.applicability == Applicability::Supported).map(|finding| return finding.subject_name.as_str()).collect();
    assert_eq!(judged, ["main.go:10", "main.go:12"], "this test runs the real Go toolchain (go on the path, or GOROOT naming one) and cannot pass without one: {findings:?}");
    assert_eq!(findings.len(), 2, "{findings:?}");
    let _ignored = std::fs::remove_dir_all(&root);
}

/// No Go on the host: the module is reported `ProviderUnavailable`, naming its `go.mod`, and nothing
/// is judged -- never a file judged clean because nothing could look, and never judged from its
/// text instead.
#[test]
fn Test_A_Go_Module_On_A_Host_Without_Go_Should_Be_Reported_Unavailable()
{
    let root = Go_Tree("go-types-no-go");

    let findings = Discard_Findings(&root, &[Source_File("main.go", SourceText(MAIN))], &MissingGo);

    let module: Vec<&Finding> = findings.iter().filter(|finding| return finding.subject_name == "go.mod").collect();
    assert_eq!(module.len(), 1, "{findings:?}");
    assert_eq!(module.first().map(|finding| return finding.applicability), Some(Applicability::ProviderUnavailable));
    assert!(!findings.iter().any(|finding| return finding.applicability == Applicability::Supported), "{findings:?}");
    let _ignored = std::fs::remove_dir_all(&root);
}

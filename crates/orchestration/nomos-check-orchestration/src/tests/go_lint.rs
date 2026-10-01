//! `go vet` through [`crate::Run`], over temporary trees: a Go module with a finding reports it
//! under the rule that already judges lint, and a Go module on a host with no Go is reported rather
//! than judged clean.
//!
//! The first test runs the real Go toolchain -- `go` on the path, or `GOROOT` naming one -- and fails
//! rather than passes without it, for the reason `nomos-lang-go-lint`'s own guarantee test gives.

use nomos_analysis::MemoryFactStore;
use nomos_contracts::{Applicability, DeterminismStrength, Finding, ReproducibilityScope, RuleId, Strategy, TraceEquivalence};
use nomos_platform::{Environment, EnvironmentError};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{CheckOutcome, Run, RunContext};

use super::{Bounded_Providers, Scratch_Directory, Source_File, SourceText, Test_Variant};

/// A `printf` verb that disagrees with its argument's declared type -- a finding only a type-checked
/// reading reaches -- at line 7.
const MAIN: &str = "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tvar count string = \"many\"\n\tfmt.Printf(\"%d items\\n\", count)\n}\n";

fn Go_Tree(name: &str) -> PathBuf
{
    let root = Scratch_Directory(name);
    std::fs::write(root.join("go.mod"), "module example.com/app\n\ngo 1.22\n").expect("a scratch module");
    std::fs::write(root.join("main.go"), MAIN).expect("a scratch source");
    return root;
}

/// The lint rule's findings over `sources` under `root`.
fn Lint_Findings<Env: Environment>(root: &Path, sources: &[nomos_rules::SourceFile], environment: &Env) -> Vec<Finding>
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

    let CheckOutcome::Judged { findings, .. } = Run(sources, context, &[RuleId::New(nomos_rules::LINT_DIAGNOSTICS)])
    else
    {
        panic!("a tree whose sources the syntax providers read must be judged");
    };
    return findings;
}

/// The Go module's diagnostic reaches the rule that has always judged lint, relayed at its file and
/// line with the full applicability a real fact carries -- read under `go vet`'s own offer, beside
/// clippy's, which answers nothing in a tree with no Cargo workspace.
#[test]
fn Test_A_Go_Module_With_A_Finding_Should_Report_It_Through_The_Lint_Rule()
{
    let root = Go_Tree("go-lint-finding");

    let findings = Lint_Findings(&root, &[Source_File("main.go", SourceText(MAIN))], &StdEnvironment);

    let relayed: Vec<&Finding> = findings.iter().filter(|finding| return finding.summary.contains("vet::printf")).collect();
    assert_eq!(
        relayed.len(),
        1,
        "this test runs the real Go toolchain (go on the path, or GOROOT naming one) and cannot pass without one: {findings:?}"
    );
    let found = relayed.first().expect("asserted one above");
    assert_eq!(found.applicability, Applicability::Supported);
    assert_eq!(found.locations, ["main.go:7"]);
    let _ignored = std::fs::remove_dir_all(&root);
}

/// A tree holding a Cargo package and a Go module is two members answered by two offers, and each
/// is read under the offer that filed it: clippy's warning and `go vet`'s both reach the rule. The
/// registry, which never sees a subject, would resolve one offer for both -- so this is the run in
/// which dropping either side's narrowing loses that side's finding, which is what makes the
/// narrowing load-bearing and not decoration.
#[test]
fn Test_A_Cargo_Package_And_A_Go_Module_Should_Each_Be_Read_Under_Its_Own_Offer()
{
    let root = Scratch_Directory("go-lint-mixed");
    std::fs::write(root.join("Cargo.toml"), "[package]
name = \"mixed\"
version = \"0.1.0\"
edition = \"2021\"

[workspace]
").expect("a scratch manifest");
    std::fs::create_dir_all(root.join("src")).expect("a scratch source directory");
    std::fs::write(root.join("src/lib.rs"), RUST).expect("a scratch source");
    std::fs::create_dir_all(root.join("svc")).expect("a scratch module directory");
    std::fs::write(root.join("svc/go.mod"), "module example.com/svc

go 1.22
").expect("a scratch module");
    std::fs::write(root.join("svc/main.go"), MAIN).expect("a scratch source");

    let sources = [Source_File("src/lib.rs", SourceText(RUST)), Source_File("svc/main.go", SourceText(MAIN))];
    let findings = Lint_Findings(&root, &sources, &StdEnvironment);

    let relayed = |lint: &str| return findings.iter().filter(|finding| return finding.summary.contains(lint) && finding.applicability == Applicability::Supported).count();
    assert_eq!(relayed("clippy::needless_return"), 1, "{findings:?}");
    assert_eq!(relayed("vet::printf"), 1, "this test runs the real Go toolchain and cannot pass without one: {findings:?}");
    let _ignored = std::fs::remove_dir_all(&root);
}

/// A needless `return`, which clippy warns about by default, at line 1.
const RUST: &str = "pub fn one() -> i32 { return 1; }
";

/// No Go on the host: the module is reported `ProviderUnavailable`, naming its `go.mod`, and no
/// diagnostic is relayed -- never a module judged clean because nothing could look.
#[test]
fn Test_A_Go_Module_On_A_Host_Without_Go_Should_Be_Reported_Unavailable()
{
    let root = Go_Tree("go-lint-no-go");

    let findings = Lint_Findings(&root, &[Source_File("main.go", SourceText(MAIN))], &MissingGo);

    let module: Vec<&Finding> = findings.iter().filter(|finding| return finding.subject_name == "go.mod").collect();
    assert_eq!(module.len(), 1, "{findings:?}");
    assert_eq!(module.first().map(|finding| return finding.applicability), Some(Applicability::ProviderUnavailable));
    assert!(!findings.iter().any(|finding| return finding.summary.contains("vet::")), "{findings:?}");
    let _ignored = std::fs::remove_dir_all(&root);
}

/// A tree with no Go source: `go` is never launched, so a host without it reports nothing about Go --
/// the missing toolchain above is never charged to a repository that asked about no Go.
#[test]
fn Test_A_Tree_Without_Go_Should_Report_Nothing_About_Go()
{
    let root = Scratch_Directory("go-lint-none");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("a scratch source");

    let findings = Lint_Findings(&root, &[Source_File("a.rs", SourceText("pub fn Ok() {}\n"))], &MissingGo);

    assert!(!findings.iter().any(|finding| return finding.summary.contains("go vet") || finding.subject_name.ends_with("go.mod")), "{findings:?}");
    let _ignored = std::fs::remove_dir_all(&root);
}

/// An environment naming a Go root that does not exist, and the real working directory -- shared
/// with `go_types`, whose provider finds `go` the same way.
pub(super) struct MissingGo;

/// Answers from fixed data, so every read is the same.
impl Strategy for MissingGo
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for MissingGo
{
    fn Variable(&self, name: &str) -> Option<OsString>
    {
        return (name == "GOROOT").then(|| return OsString::from("nomos-no-such-go-root"));
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        return std::env::current_dir().map_err(|error| return EnvironmentError::WorkingDirectoryUnreadable { cause: error.to_string() });
    }
}

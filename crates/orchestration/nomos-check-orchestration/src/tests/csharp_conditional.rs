//! The C# conditional-compilation family through [`crate::Run`], over real trees a test wrote:
//! the builds a repository declares decide which branch is dead, a repository declaring none gets
//! no guessed build, and a host with no SDK is never judged as though nothing were defined.
//!
//! The first test asks the real .NET SDK, and fails rather than passes on a host without one,
//! for the reason `nomos-lang-csharp-compiler`'s own guarantee test gives: a proof that passes
//! where it could not look is the vacuous one. The gate's hosted runners carry an SDK.

use nomos_analysis::MemoryFactStore;
use nomos_contracts::{Applicability, DeterminismStrength, Finding, GateCategory, ReproducibilityScope, RuleId, Strategy, TraceEquivalence};
use nomos_platform::{Environment, EnvironmentError};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{CheckOutcome, Run, RunContext};

use super::{Bounded_Providers, Scratch_Directory, Source_File, SourceText, Test_Variant};

const PROJECT: &str = "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <TargetFramework>net8.0</TargetFramework>\n  </PropertyGroup>\n</Project>\n";

/// `#else` of `#if DEBUG` at line 5, compiled by a Release build and by no Debug one; `#if NET48`
/// at line 8, compiled by neither, since the project targets `net8.0` alone.
const WIDGET: &str = "namespace App\n{\n#if DEBUG\n    class DebugOnly {}\n#else\n    class ReleaseOnly {}\n#endif\n#if NET48\n    class Legacy {}\n#endif\n}\n";

fn Tree(name: &str, declaration: Option<&str>) -> PathBuf
{
    let root = Scratch_Directory(name);
    std::fs::write(root.join("App.csproj"), PROJECT).expect("a scratch project");
    std::fs::write(root.join("Widget.cs"), WIDGET).expect("a scratch source");
    if let Some(declaration) = declaration
    {
        std::fs::write(root.join("nomos-csharp-builds.json"), declaration).expect("a scratch declaration");
    }
    return root;
}

/// The rule's findings over `root`'s one C# source, through the real composition.
fn Findings<Env: Environment>(root: &Path, environment: &Env) -> Vec<Finding>
{
    let sources = vec![Source_File("Widget.cs", SourceText(WIDGET))];
    let selected = [RuleId::New(nomos_rules::UNCOMPILED_CONDITIONAL_BRANCH)];
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

    let CheckOutcome::Judged { findings, .. } = Run(&sources, context, &selected)
    else
    {
        panic!("a tree whose C# the syntax provider reads must be judged");
    };
    return findings;
}

fn Names(findings: &[Finding]) -> Vec<&str>
{
    return findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
}

/// Declared Debug alone, both branches are dead; declaring Release as well revives the `#else`,
/// which that build compiles, and leaves `#if NET48`, which nothing declared compiles.
#[test]
fn Test_The_Declared_Builds_Should_Decide_Which_Branch_Is_Compiled_By_None()
{
    let debug = Tree("csharp-debug", Some(r#"{"builds":[{"project":"App.csproj","configuration":"Debug"}]}"#));
    let both = Tree("csharp-both", Some(r#"{"builds":[{"project":"App.csproj","configuration":"Debug"},{"project":"App.csproj","configuration":"Release"}]}"#));

    let under_debug = Findings(&debug, &StdEnvironment);
    let under_both = Findings(&both, &StdEnvironment);

    assert!(
        under_debug.iter().all(|finding| return finding.applicability == Applicability::Supported),
        "this test asks the real .NET SDK and cannot pass without one: {under_debug:?}"
    );
    assert_eq!(Names(&under_debug), ["Widget.cs:5", "Widget.cs:8"], "{under_debug:?}");
    assert_eq!(Names(&under_both), ["Widget.cs:8"], "{under_both:?}");
    let _ignored = (std::fs::remove_dir_all(&debug), std::fs::remove_dir_all(&both));
}

/// A reassessing run over a store and cache kept from the call before re-judges the rule when a
/// declared build is dropped, though nothing is written: the surviving build's fact is byte-for-byte
/// the one the store holds, so no family reads as changed by its write counter. Reused, the rule
/// would still report only the branch neither build compiled; re-judged, it reports the `#else` the
/// dropped build was the only one compiling. The slice's population is what moved.
#[test]
fn Test_A_Dropped_Build_Should_Re_Judge_The_Rule_Though_Nothing_Is_Written()
{
    let both = r#"{"builds":[{"project":"App.csproj","configuration":"Debug"},{"project":"App.csproj","configuration":"Release"}]}"#;
    let root = Tree("csharp-reassess", Some(both));
    let sources = vec![Source_File("Widget.cs", SourceText(WIDGET))];
    let selected = [RuleId::New(nomos_rules::UNCOMPILED_CONDITIONAL_BRANCH)];
    let providers = Bounded_Providers();
    let mut store = MemoryFactStore::New();
    let mut workspace = None;
    let mut cache = crate::RuleReassessmentCache::New();
    let mut reassess = |root: &Path, cache: &mut crate::RuleReassessmentCache| {
        let context = RunContext {
            variant: Test_Variant(),
            root,
            launcher: &StdProgramLauncher,
            filesystem: &StdFileSystem,
            environment: &StdEnvironment,
            workspace: &mut workspace,
            store: &mut store,
            providers: &providers,
        };
        let CheckOutcome::Judged { findings, .. } = crate::Run_Reassessing(&sources, context, &selected, cache)
        else
        {
            panic!("a tree whose C# the syntax provider reads must be judged");
        };
        return findings;
    };

    let under_both = reassess(&root, &mut cache);
    std::fs::write(root.join("nomos-csharp-builds.json"), r#"{"builds":[{"project":"App.csproj","configuration":"Debug"}]}"#).expect("the declaration, narrowed");
    let under_debug = reassess(&root, &mut cache);
    let recorded = cache.Recorded();
    let again = reassess(&root, &mut cache);

    assert_eq!(Names(&under_both), ["Widget.cs:8"], "this test asks the real .NET SDK: {under_both:?}");
    assert_eq!(Names(&under_debug), ["Widget.cs:5", "Widget.cs:8"], "a reused rule would still say only line 8: {under_debug:?}");
    assert_eq!(cache.Recorded(), recorded, "nothing moved on the third call, so the rule is reused rather than run");
    assert_eq!(again, under_debug);
    let _ignored = std::fs::remove_dir_all(&root);
}

/// C# and no declaration: one finding saying so, not applicable and advisory, and no branch
/// judged against a build nobody declared.
#[test]
fn Test_A_Repository_Declaring_No_Build_Should_Be_Told_So_And_Judged_Under_None()
{
    let root = Tree("csharp-undeclared", None);

    let findings = Findings(&root, &StdEnvironment);

    assert_eq!(Names(&findings), ["nomos-csharp-builds.json"], "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert_eq!(found.applicability, Applicability::NotApplicable);
    assert_eq!(found.gate, GateCategory::Advisory);
    let _ignored = std::fs::remove_dir_all(&root);
}

/// A declaration naming a key it does not have is one that will not be applied, so it blocks, and
/// nothing is judged until it is corrected.
#[test]
fn Test_A_Declaration_That_Will_Not_Be_Applied_Should_Block()
{
    let root = Tree("csharp-misspelled", Some(r#"{"builds":[{"project":"App.csproj","configuraton":"Debug"}]}"#));

    let findings = Findings(&root, &StdEnvironment);

    assert_eq!(Names(&findings), ["nomos-csharp-builds.json"], "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert_eq!(found.gate, GateCategory::Blocking);
    assert!(found.summary.contains("`configuraton`"), "{}", found.summary);
    let _ignored = std::fs::remove_dir_all(&root);
}

/// A host with no SDK is `DependencyUnavailable` for the declared build, and no branch is called
/// dead -- which a run that read "no answer" as "no symbols defined" would do to every `#if`.
#[test]
fn Test_A_Host_Without_The_Sdk_Should_Be_Reported_Dependency_Unavailable_And_Judge_Nothing()
{
    let root = Tree("csharp-no-sdk", Some(r#"{"builds":[{"project":"App.csproj","configuration":"Debug"}]}"#));

    let findings = Findings(&root, &MissingDotnet);

    assert_eq!(Names(&findings), ["App.csproj"], "{findings:?}");
    assert_eq!(findings.first().map(|finding| return finding.applicability), Some(Applicability::DependencyUnavailable));
    let _ignored = std::fs::remove_dir_all(&root);
}

/// A repository with no C# source has nothing for a build to decide: nothing is reported, whether
/// or not a declaration exists.
#[test]
fn Test_A_Repository_Without_Csharp_Should_Report_Nothing()
{
    let root = Scratch_Directory("csharp-none");
    let sources = vec![Source_File("a.rs", SourceText("pub fn Ok() {}\n"))];
    let providers = Bounded_Providers();
    let context = RunContext {
        variant: Test_Variant(),
        root: &root,
        launcher: &StdProgramLauncher,
        filesystem: &StdFileSystem,
        environment: &MissingDotnet,
        workspace: &mut None,
        store: &mut MemoryFactStore::New(),
        providers: &providers,
    };

    let CheckOutcome::Judged { findings, .. } = Run(&sources, context, &[RuleId::New(nomos_rules::UNCOMPILED_CONDITIONAL_BRANCH)])
    else
    {
        panic!("a Rust tree must be judged");
    };

    assert!(findings.is_empty(), "{findings:?}");
    let _ignored = std::fs::remove_dir_all(&root);
}

/// An environment naming a `dotnet` host that does not exist.
struct MissingDotnet;

/// Answers from a fixed value, so every read is the same.
impl Strategy for MissingDotnet
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for MissingDotnet
{
    fn Variable(&self, name: &str) -> Option<OsString>
    {
        return (name == "DOTNET_HOST_PATH").then(|| return OsString::from("nomos-no-such-dotnet-host"));
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        return Ok(PathBuf::from("."));
    }
}

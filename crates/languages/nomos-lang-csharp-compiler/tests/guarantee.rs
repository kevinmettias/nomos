//! Every axis of `Declared_Guarantee` exercised against what the provider emits -- the upward
//! direction `OD-CAPABILITY-016` requires -- and, for soundness and completeness, against the C#
//! compiler itself.
//!
//! The compiler-backed test needs a .NET SDK on the host, and fails rather than passes when there
//! is none: a guarantee test that passes where it could not look is the vacuous proof this file
//! exists to avoid. The gate's hosted runners carry one.

use nomos_analysis::MaterializedFact;
use nomos_cap_csharp_semantics::{BranchState, BuildSelection, ConditionalRegion, Parse_Payload};
use nomos_contracts::{Applicability, BuildVariantId, ConfigurationId, DeterminismStrength, Digest128, FactVariant, GenerationId, ReproducibilityScope, SnapshotId, Strategy, SubjectId, TraceEquivalence};
use nomos_lang_csharp_compiler::{
    Build_Subject, ConditionalReading, Declared_Guarantee, DefinitionSet, Evaluate_Build, EvaluationRequest, FactContext, Materialization, Materialize_Conditional_Fact, Read_Conditionals,
};
use nomos_model::Content_Digest;
use nomos_platform::{Command, Environment, EnvironmentError, ExitOutcome, ProgramLauncher};
use nomos_platform_std::{StdEnvironment, StdProgramLauncher};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The fixture the compiler judges. Every branch's first line is a placeholder, so a branch's body
/// can be rewritten without moving any directive; directive-shaped lines inside a verbatim string, a
/// raw string and a delimited comment are there to be recognized as text.
const FIXTURE: &str = "#define LOCAL
#undef TRACE
namespace Oracle
{
#if DEBUG && !RELEASE
    //@@BODY
#elif RELEASE
    //@@BODY
#else
    //@@BODY
#endif
#if NET8_0_OR_GREATER || NETSTANDARD2_0
    //@@BODY
#endif
#if TRACE
    //@@BODY
#else
    //@@BODY
#endif
#if LOCAL == true && (FEATURE_X != false)
    //@@BODY
#if NESTED_NEVER
    //@@BODY
#else
    //@@BODY
#endif
#endif
#if false
    //@@BODY
#if true
    //@@BODY
#endif
#endif
    static class Texts
    {
        static readonly string Verbatim = @\"
#if VERBATIM_NOT_A_DIRECTIVE
\";
        static readonly string Raw = \"\"\"
#endif
\"\"\";
        /*
#else
        */
    }
//@@PROBE
}
";

const PROJECT: &str = "<Project Sdk=\"Microsoft.NET.Sdk\">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <DefineConstants>$(DefineConstants);FEATURE_X</DefineConstants>
    <ImplicitUsings>disable</ImplicitUsings>
    <Nullable>disable</Nullable>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup>
    <Compile Include=\"Oracle.cs\" />
  </ItemGroup>
</Project>
";

const PLACEHOLDER: &str = "//@@BODY";

/// Seeds distinct enough that no two of the context's digests compare equal.
const SNAPSHOT_SEED: u8 = 1;
const VARIANT_SEED: u8 = 2;
const CONFIGURATION_SEED: u8 = 3;

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([SNAPSHOT_SEED; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_SEED; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_SEED; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

fn Set(configuration: &str, symbols: &[&str]) -> DefinitionSet
{
    let selection = BuildSelection { project: "Oracle.csproj".to_owned(), configuration: configuration.to_owned(), target_framework: "net8.0".to_owned() };
    return DefinitionSet::New(selection, symbols.iter().map(|symbol| return (*symbol).to_owned()).collect());
}

fn Regions(source: &str, symbols: &BTreeSet<String>) -> Vec<ConditionalRegion>
{
    return match Read_Conditionals(source, symbols)
    {
        ConditionalReading::Read { regions, .. } => regions,
        ConditionalReading::Refused(failure) => panic!("the fixture's directives nest: {failure}"),
    };
}

fn File_Subject(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
}

fn Fact(source: &str, set: &DefinitionSet, path: &str) -> MaterializedFact
{
    return match Materialize_Conditional_Fact(File_Subject(path), source, set, Context())
    {
        Materialization::Materialized(fact) => *fact,
        // Every fixture here is directives written to nest; a refusal is a broken fixture, and the
        // comparisons below would otherwise have nothing on one side and read as agreement.
        Materialization::Refused(failure) => panic!("expected a fact: {failure}"),
    };
}

/// `SemanticallyResolved`: the same text under two builds' definition sets is two answers. What
/// decides a branch is the build's set, which no reading of the file holds.
#[test]
fn Test_The_Variant_Should_Be_Resolved_Because_The_Answer_Turns_On_The_Builds_Symbols()
{
    let debug = Regions(FIXTURE, &Set("Debug", &["DEBUG", "TRACE", "NET8_0_OR_GREATER"]).symbols);
    let release = Regions(FIXTURE, &Set("Release", &["RELEASE", "TRACE", "NET8_0_OR_GREATER"]).symbols);

    assert_eq!(Declared_Guarantee().variant, FactVariant::SemanticallyResolved);
    assert_eq!(debug.first().map(|region| return region.state), Some(BranchState::Compiled));
    assert_eq!(release.first().map(|region| return region.state), Some(BranchState::Skipped));
}

/// File: one file's text and one set are the whole input -- the same pair for two files is one
/// answer, filed under each file's own build subject, and nothing read for one file reaches another.
#[test]
fn Test_The_Granularity_Should_Be_File_Because_A_Reading_Carries_Nothing_Across()
{
    let debug = Set("Debug", &["DEBUG"]);
    let first = Fact(FIXTURE, &debug, "a.cs");
    let second = Fact(FIXTURE, &debug, "b.cs");

    assert_eq!(first.payload.Digest(), second.payload.Digest());
    assert_ne!(first.Key().subject, second.Key().subject);
}

/// One file under two builds is two facts under two subjects, whatever subject the caller passed;
/// the same build answering with moved symbols is one subject answering differently, so its new
/// fact replaces the old one rather than standing beside it.
#[test]
fn Test_A_Fact_Should_Be_Filed_Under_Its_Builds_Subject_And_Never_Under_The_Files_Own()
{
    let debug = Set("Debug", &["DEBUG"]);
    let first = Fact(FIXTURE, &debug, "a.cs");
    let release = Fact(FIXTURE, &Set("Release", &["RELEASE"]), "a.cs");
    let moved = Fact(FIXTURE, &Set("Debug", &["DEBUG", "FEATURE_X"]), "a.cs");

    assert_eq!(first.Key().subject, Build_Subject(File_Subject("a.cs"), &debug.selection));
    assert_ne!(first.Key().subject, File_Subject("a.cs"));
    assert_ne!(first.Key().subject, release.Key().subject);
    assert_eq!(first.Key().subject, moved.Key().subject);
    assert_ne!(first.payload.Digest(), moved.payload.Digest());
}

/// The fact carries the build it was judged under and decodes under the schema's own reader.
#[test]
fn Test_The_Fact_Should_Decode_Under_The_Schemas_Own_Reader_And_Name_Its_Build()
{
    let set = Set("Debug", &["DEBUG"]);

    let payload = Parse_Payload(&Fact(FIXTURE, &set, "a.cs").payload.bytes).expect("this provider writes nomos.csharp.conditional_compilation.v1");

    assert_eq!(payload.selection, set.selection);
    assert_eq!(payload.symbols, ["DEBUG"]);
    assert_eq!(payload.definitions.len(), 2, "the fixture's #define and #undef");
}

/// A host with no SDK is `DependencyUnavailable`, never an answer computed from no symbols --
/// shown with the real launcher and a `dotnet` that does not exist.
#[test]
fn Test_A_Host_Without_The_Sdk_Should_Be_Reported_Dependency_Unavailable()
{
    let request = EvaluationRequest { root: PathBuf::from("."), project: "Oracle.csproj".to_owned(), configuration: "Debug".to_owned(), target_framework: None };

    let error = Evaluate_Build(&request, &StdProgramLauncher, &MissingDotnet).expect_err("no such program");

    assert_eq!(error.failure.Applicability(), Applicability::DependencyUnavailable, "{error}");
}

/// Soundness and completeness, judged by the compiler: every branch the provider calls skipped is
/// made invalid C#, every branch it calls compiled declares a type the file then requires, and the
/// build must succeed. Flipping any one state -- in either direction -- must make it fail, which is
/// what shows the success is the compiler agreeing rather than the build not looking.
///
/// The same evaluation says which files the build compiles, and the project turns its default
/// items off and names one, beside a second `.cs` file it does not name: a directory would call
/// both compiled, and `MSBuild` must list only the one.
#[test]
fn Test_Every_Branch_State_Should_Be_The_One_The_Compiler_Takes()
{
    let root = Oracle_Directory("states");
    std::fs::write(root.join("Oracle.csproj"), PROJECT).expect("a scratch project");
    std::fs::write(root.join("Stray.cs"), "this file is not compiled, or the build fails").expect("a file the project does not name");
    let request = EvaluationRequest { root: root.clone(), project: "Oracle.csproj".to_owned(), configuration: "Debug".to_owned(), target_framework: None };
    let evaluation = Evaluate_Build(&request, &StdProgramLauncher, &StdEnvironment)
        .unwrap_or_else(|error| panic!("this test needs a .NET SDK on the host to ask ({error}); it proves the guarantee against the real compiler and cannot pass without one"));
    assert_eq!(evaluation.compiled.iter().map(String::as_str).collect::<Vec<_>>(), ["Oracle.cs"], "MSBuild's Compile items, not the directory's files");
    let set = evaluation.definitions;
    for expected in ["DEBUG", "FEATURE_X", "NET8_0_OR_GREATER", "TRACE"]
    {
        assert!(set.symbols.contains(expected), "MSBuild's answer lacks {expected}: {:?}", set.symbols);
    }

    let regions = Regions(FIXTURE, &set.symbols);
    Assert_Every_Placeholder_Belongs_To_Exactly_One_Branch(&regions);
    assert!(regions.iter().all(|region| return region.state != BranchState::Unevaluated), "{regions:?}");

    assert!(Builds(&root, &Rendered(&regions, None)), "the faithful rendering must build");
    let compiled = regions.iter().position(|region| return region.state == BranchState::Compiled).expect("the fixture has a compiled branch");
    let skipped = regions.iter().position(|region| return region.state == BranchState::Skipped).expect("the fixture has a skipped branch");
    assert!(!Builds(&root, &Rendered(&regions, Some(compiled))), "a compiled branch made invalid must fail the build");
    assert!(!Builds(&root, &Rendered(&regions, Some(skipped))), "a skipped branch made necessary must fail the build");

    let _ = std::fs::remove_dir_all(&root);
}

/// Completeness and soundness of the directive scan, before the compiler: each placeholder follows
/// exactly one reported branch's directive, and each reported branch's directive is followed by one.
/// A missed branch leaves a placeholder unclaimed; an invented one claims a line that is not.
fn Assert_Every_Placeholder_Belongs_To_Exactly_One_Branch(regions: &[ConditionalRegion])
{
    let lines: Vec<&str> = FIXTURE.lines().collect();
    let placeholders: BTreeSet<usize> = lines.iter().enumerate().filter(|(_, line)| return line.trim() == PLACEHOLDER).map(|(index, _)| return index.saturating_add(1)).collect();
    let claimed: BTreeSet<usize> = regions.iter().map(|region| return region.line.saturating_add(1)).collect();

    assert_eq!(claimed, placeholders, "{regions:?}");
    assert_eq!(regions.len(), placeholders.len(), "{regions:?}");
}

/// The fixture with each branch's placeholder replaced by what its state requires, and `flipped`'s
/// replaced by what the opposite state would.
fn Rendered(regions: &[ConditionalRegion], flipped: Option<usize>) -> String
{
    let mut lines: Vec<String> = FIXTURE.lines().map(str::to_owned).collect();
    let mut required = Vec::new();
    for (index, region) in regions.iter().enumerate()
    {
        let compiled = (region.state == BranchState::Compiled) != (flipped == Some(index));
        let body = if compiled { format!("    class Branch_{} {{}}", region.line) } else { format!("    this is not C# at all {}", region.line) };
        if compiled
        {
            required.push(format!("typeof(Branch_{})", region.line));
        }
        if let Some(line) = lines.get_mut(region.line)
        {
            *line = body;
        }
    }

    let probe = format!("    static class Probe {{ static readonly System.Type[] Required = {{ {} }}; }}", required.join(", "));
    return lines.into_iter().map(|line| return if line == "//@@PROBE" { probe.clone() } else { line }).collect::<Vec<String>>().join("\n");
}

fn Builds(root: &Path, source: &str) -> bool
{
    std::fs::write(root.join("Oracle.cs"), source).expect("the rendered fixture");
    // No node reuse and no shared compilation, so no MSBuild node or compiler server outlives the
    // build and holds the scratch directory open after the test.
    let argv = ["dotnet", "build", "Oracle.csproj", "-nologo", "-nodeReuse:false", "-p:UseSharedCompilation=false", "-v:q", "-c", "Debug"].map(str::to_owned).to_vec();
    let mut command = Command::From_String_Arguments(argv, Duration::from_secs(300));
    command.working_directory = Some(root.to_path_buf());

    let output = StdProgramLauncher.Run(&command).expect("dotnet was found once already, by the evaluation");
    return matches!(output.outcome, ExitOutcome::Exited { code: 0 });
}

fn Oracle_Directory(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-csharp-oracle-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch directory");
    return root;
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

//! The provider against the real Go toolchain, over a temporary tree: what it files, what it
//! refuses, and the axes of `Declared_Guarantee` it can show.
//!
//! The real-toolchain tests need a Go the environment can reach -- `go` on the path, or a `GOROOT`
//! naming one -- and fail rather than pass without it: a lint proof that passes where it could not
//! look is the vacuous one. The gate's hosted runners carry Go. On a host whose first `go` is a
//! trimmed build, export `GOROOT` to the real toolchain before running them.

use nomos_cap_lint::{Parse_Payload, Payload_Schema};
use nomos_contracts::{Applicability, BuildVariantId, ConfigurationId, DeterminismStrength, Digest128, FactVariant, GenerationId, ReproducibilityScope, SnapshotId, Strategy, TraceEquivalence};
use nomos_lang_go_lint::{Declared_Guarantee, FactContext, LintAnswer, LintPorts, Materialize_Modules, Unlinted, VetFailure};
use nomos_platform::{Environment, EnvironmentError};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

/// A tree with two modules and one stray source: `app`, whose `main` package has a `printf` verb
/// that disagrees with its argument's declared type and whose `util` package is clean; `broken`,
/// which does not type-check; and `stray.go`, under no `go.mod`.
fn Tree(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-go-lint-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let write = |relative: &str, text: &str| {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("a file has a directory")).expect("a scratch directory");
        std::fs::write(path, text).expect("a scratch file");
    };
    write("app/go.mod", "module example.com/app\n\ngo 1.22\n");
    write("app/main.go", "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tvar count string = \"many\"\n\tfmt.Printf(\"%d items\\n\", count)\n}\n");
    write("app/util/util.go", "package util\n\n// Twice doubles n.\nfunc Twice(n int) int {\n\treturn n * 2\n}\n");
    write("broken/go.mod", "module example.com/broken\n\ngo 1.22\n");
    write("broken/main.go", "package main\n\nfunc main() {\n\tundefinedThing()\n}\n");
    write("stray.go", "package stray\n");
    return root;
}

const FILES: [&str; 4] = ["app/main.go", "app/util/util.go", "broken/main.go", "stray.go"];

fn Linted<Env: Environment>(root: &Path, environment: &Env) -> LintAnswer
{
    let ports = LintPorts { launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment };
    return Materialize_Modules(root, &FILES, Context(), &ports);
}

/// Soundness and the resolved variant, shown by the real toolchain: the one module that type-checks
/// is filed with exactly the diagnostic `go vet` reports for it -- a verb judged against a declared
/// type, which no reading of the text alone could reach -- the one that does not type-check is a
/// failure and never an empty fact, and the stray source is reported rather than dropped.
#[test]
fn Test_A_Real_Tree_Should_Be_Filed_Module_By_Module_With_Each_Gap_Reported()
{
    let root = Tree("real");

    let answer = Linted(&root, &StdEnvironment);

    let app = answer.facts.iter().find(|fact| return fact.path == "app").unwrap_or_else(|| {
        panic!("this test needs a Go toolchain the environment can reach (go on the path, or GOROOT naming one) and cannot pass without one: {:?}", answer.unlinted)
    });
    assert_eq!(app.fact.payload.schema, Payload_Schema());
    let payload = Parse_Payload(&app.fact.payload.bytes).expect("the provider writes the lint schema");
    assert_eq!(payload.package, "example.com/app");
    let found: Vec<(&str, u32, Option<&str>)> = payload.diagnostics.iter().map(|diagnostic| return (diagnostic.file.as_str(), diagnostic.line, diagnostic.lint.as_deref())).collect();
    assert_eq!(found, [("app/main.go", 7, Some("vet::printf"))], "{payload:?}");
    assert_eq!(app.fact.guarantee.variant, FactVariant::SemanticallyResolved);
    assert_eq!(app.fact.guarantee, Declared_Guarantee());

    assert_eq!(answer.facts.len(), 1, "only the module that type-checks has a fact: {:?}", answer.facts);
    assert!(
        answer.unlinted.iter().any(|unlinted| return matches!(unlinted, Unlinted::Module { path, failure: VetFailure::Failed, .. } if path == "broken")),
        "{:?}",
        answer.unlinted
    );
    assert!(answer.unlinted.contains(&Unlinted::Outside { files: vec!["stray.go".to_owned()] }), "{:?}", answer.unlinted);
    let _ = std::fs::remove_dir_all(&root);
}

/// A host whose named toolchain does not exist is `ProviderUnavailable` for every module, never a
/// clean result.
#[test]
fn Test_A_Host_Without_Go_Should_Be_Reported_Unavailable_And_Never_Clean()
{
    let root = Tree("absent");

    let answer = Linted(&root, &MissingGo);

    assert!(answer.facts.is_empty(), "{:?}", answer.facts);
    let failures: Vec<VetFailure> = answer.unlinted.iter().filter_map(|unlinted| return if let Unlinted::Module { failure, .. } = unlinted { Some(*failure) } else { None }).collect();
    assert_eq!(failures, [VetFailure::GoUnavailable, VetFailure::GoUnavailable]);
    assert_eq!(VetFailure::GoUnavailable.Applicability(), Applicability::ProviderUnavailable);
    let _ = std::fs::remove_dir_all(&root);
}

/// No Go source, nothing launched: the missing toolchain above is never reported for a repository
/// that asked about no Go at all.
#[test]
fn Test_No_Go_Source_Should_Launch_Nothing_And_Report_Nothing()
{
    let ports = LintPorts { launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &MissingGo };

    let answer = Materialize_Modules(Path::new("."), &[], Context(), &ports);

    assert_eq!(answer, LintAnswer::default());
}

/// An environment naming a Go root that does not exist.
struct MissingGo;

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

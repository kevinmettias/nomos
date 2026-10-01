//! The provider against the real Go toolchain, over a temporary tree: what it files, what it
//! reports unanswered, and the axes of `Declared_Guarantee` it can show.
//!
//! The real-toolchain tests need a Go the environment can reach -- `go` on the path, or a `GOROOT`
//! naming one -- at 1.23 or later, since the tree ranges over a function, and fail rather than pass
//! without it: a typing proof that passes where it could not look is the vacuous one. On a host
//! whose first `go` is a trimmed build, export `GOROOT` to the real toolchain before running them.

use nomos_cap_go_types::{DiscardedValue, Parse_Payload, Payload_Schema};
use nomos_contracts::{Applicability, BuildVariantId, ConfigurationId, DeterminismStrength, Digest128, FactVariant, GenerationId, ReproducibilityScope, SnapshotId, Strategy, TraceEquivalence};
use nomos_lang_go_types::{Declared_Guarantee, FactContext, Materialize_Files, TypesAnswer, TypesFailure, TypesPorts, Untyped};
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

/// The main file of the `app` module: one value reaching `_` by each form Go has -- the last a
/// `const` line repeating the one above it -- the first call declared in another package, and values
/// that are not errors beside the ones that are.
const APP_MAIN: &str = "package main

import (
\t\"fmt\"
\t\"os\"
\t\"strconv\"

\t\"example.com/app/util\"
)

var _ = os.Chdir(\".\")

func Pairs[S ~[]E, E any](s S, seq func(yield func(int, error) bool)) {
\tfor _, _ = range seq {
\t}
\tfor _, e := range s {
\t\tfmt.Println(e)
\t}
}

func main() {
\t_ = util.Do()
\tn, _ := strconv.Atoi(\"1\")
\t_, _ = fmt.Println(n)
\t_ = fmt.Sprintf(\"%d\", n)
}

const (
\tFirst = iota
\t_
)
";

/// A tree with two modules and one stray source: `app`, which type-checks and holds a test and a
/// file only another platform compiles; `broken`, whose one package does not type-check; and
/// `stray.go`, under no `go.mod`.
fn Tree(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-go-types-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let write = |relative: &str, text: &str| {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("a file has a directory")).expect("a scratch directory");
        std::fs::write(path, text).expect("a scratch file");
    };
    write("app/go.mod", "module example.com/app\n\ngo 1.23\n");
    write("app/main.go", APP_MAIN);
    write("app/main_test.go", "package main\n\nimport (\n\t\"os\"\n\t\"testing\"\n)\n\nfunc TestMain(t *testing.T) {\n\t_ = os.Remove(\"t\")\n}\n");
    write("app/util/util.go", "package util\n\n// Do does nothing, and says so.\nfunc Do() error {\n\treturn nil\n}\n");
    write("app/only_plan9.go", "//go:build plan9\n\npackage main\n");
    write("broken/go.mod", "module example.com/broken\n\ngo 1.23\n");
    write("broken/main.go", "package main\n\nfunc main() {\n\tundefinedThing()\n}\n");
    write("stray.go", "package stray\n");
    return root;
}

const FILES: [&str; 7] = ["app/main.go", "app/main_test.go", "app/util/util.go", "app/only_plan9.go", "broken/main.go", "stray.go", "app/missing.go"];

fn Typed<Env: Environment>(root: &Path, environment: &Env) -> TypesAnswer
{
    let ports = TypesPorts { launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment };
    return Materialize_Files(root, &FILES, Context(), &ports);
}

fn Value(line: u32, column: u32, is_error: bool, type_name: &str) -> DiscardedValue
{
    return DiscardedValue { line, column, is_error, type_name: type_name.to_owned() };
}

fn Values_Of(answer: &TypesAnswer, path: &str) -> Vec<DiscardedValue>
{
    let fact = answer.facts.iter().find(|fact| return fact.path == path).unwrap_or_else(|| {
        panic!("this test needs a Go toolchain the environment can reach (go on the path, or GOROOT naming one) and cannot pass without one: {:?}", answer.untyped)
    });
    assert_eq!(fact.fact.payload.schema, Payload_Schema());
    assert_eq!(fact.fact.guarantee, Declared_Guarantee());
    assert_eq!(fact.fact.guarantee.variant, FactVariant::SemanticallyResolved);
    return Parse_Payload(&fact.fact.payload.bytes).expect("the provider writes the schema").values;
}

/// Soundness, completeness and the resolved variant, shown by the real toolchain: every form a
/// value reaches `_` by is typed -- a call declared in another package, a tuple's parts, a range
/// over a type parameter and over a function -- and only the errors among them are errors; a
/// test file is checked with its package, and a file that discards nothing is an empty answer.
#[test]
fn Test_Every_Blank_Assignment_Should_Be_Typed_By_The_Real_Toolchain()
{
    let root = Tree("real");

    let answer = Typed(&root, &StdEnvironment);

    assert_eq!(
        Values_Of(&answer, "app/main.go"),
        [
            Value(11, 5, true, "error"),
            Value(14, 6, false, "int"),
            Value(14, 9, true, "error"),
            Value(16, 6, false, "int"),
            Value(22, 2, true, "error"),
            Value(23, 5, true, "error"),
            Value(24, 2, false, "int"),
            Value(24, 5, true, "error"),
            Value(25, 2, false, "string"),
            Value(30, 2, false, "untyped int"),
        ]
    );
    assert_eq!(Values_Of(&answer, "app/main_test.go"), [Value(9, 2, true, "error")]);
    assert_eq!(Values_Of(&answer, "app/util/util.go"), []);
    assert_eq!(answer.facts.len(), 3, "only the files a package that type-checked compiles have a fact: {:?}", answer.facts);
    let _ = std::fs::remove_dir_all(&root);
}

/// No file nobody checked is filed: the package that does not type-check, the file only another
/// platform compiles, a file the walk named that is not there, and the stray source are each
/// reported, and none of them is an empty fact.
#[test]
fn Test_Every_File_Not_Checked_Should_Be_Reported_And_Never_Filed_Empty()
{
    let root = Tree("gaps");

    let answer = Typed(&root, &StdEnvironment);

    let broken = answer.untyped.iter().find_map(|untyped| return if let Untyped::Package { package, files, reason } = untyped { Some((package, files, reason)) } else { None });
    let (package, files, reason) = broken.unwrap_or_else(|| panic!("the package that does not type-check is reported: {:?}", answer.untyped));
    assert_eq!(package, "example.com/broken");
    assert_eq!(files, &["broken/main.go"]);
    assert!(reason.contains("undefinedThing"), "{reason}");
    assert!(
        answer.untyped.contains(&Untyped::Unchecked { module: "app".to_owned(), files: vec!["app/only_plan9.go".to_owned(), "app/missing.go".to_owned()] }),
        "{:?}",
        answer.untyped
    );
    assert!(answer.untyped.contains(&Untyped::Outside { files: vec!["stray.go".to_owned()] }), "{:?}", answer.untyped);
    assert!(!answer.facts.iter().any(|fact| return ["broken/main.go", "app/only_plan9.go", "app/missing.go", "stray.go"].contains(&fact.path.as_str())));
    let _ = std::fs::remove_dir_all(&root);
}

/// A host whose named toolchain does not exist is `ProviderUnavailable` for every module, never a
/// clean result.
#[test]
fn Test_A_Host_Without_Go_Should_Be_Reported_Unavailable_And_Never_Clean()
{
    let root = Tree("absent");

    let answer = Typed(&root, &MissingGo);

    assert!(answer.facts.is_empty(), "{:?}", answer.facts);
    let failures: Vec<TypesFailure> = answer.untyped.iter().filter_map(|untyped| return if let Untyped::Module { failure, .. } = untyped { Some(*failure) } else { None }).collect();
    assert_eq!(failures, [TypesFailure::GoUnavailable, TypesFailure::GoUnavailable]);
    assert_eq!(TypesFailure::GoUnavailable.Applicability(), Applicability::ProviderUnavailable);
    let _ = std::fs::remove_dir_all(&root);
}

/// No Go source, nothing written and nothing launched: the missing toolchain above is never
/// reported for a repository that asked about no Go at all.
#[test]
fn Test_No_Go_Source_Should_Launch_Nothing_And_Report_Nothing()
{
    let ports = TypesPorts { launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &MissingGo };

    let answer = Materialize_Files(Path::new("."), &[], Context(), &ports);

    assert_eq!(answer, TypesAnswer::default());
}

/// The host's own environment, except that it names a Go root that does not exist.
struct MissingGo;

/// Answers the one variable it replaces from fixed data, and passes the rest through unchanged.
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
        if name == "GOROOT"
        {
            return Some(OsString::from("nomos-no-such-go-root"));
        }
        return StdEnvironment.Variable(name);
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        return StdEnvironment.Working_Directory();
    }
}

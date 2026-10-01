//! `nomos-lang-go-lint`'s production, over a scripted `go vet` and an in-memory module, so what
//! the declaration covers -- reading and filing, given what `go vet` printed -- is what is measured.

use nomos_contracts::{BuildVariantId, ConfigurationId, DeterminismStrength, GenerationId, ReproducibilityScope, SnapshotId, Strategy, TraceEquivalence};
use nomos_lang_go_lint::{FactContext, LintPorts, Materialize_Modules};
use nomos_platform::{Command, Environment, EnvironmentError, ExitOutcome, FileSystem, FileSystemError, ProgramLauncher, ProgramOutput};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What `go vet -json` printed for the fixture, with `ROOT` standing for the fixture's root and `SEP`
/// for this platform's separator, each as a JSON string spells it: two packages' diagnostics, one reported twice and all out of order,
/// beside a clean package's `{}`.
const VET_ANSWER: &str = r#"{"example.com/fixture/pkg": {"unusedresult": [{"posn": "ROOTSEPpkgSEPutil.go:9:2", "message": "result of fmt.Sprintf call not used"}]}}
{}
{"example.com/fixture": {"printf": [{"posn": "ROOTSEPmain.go:7:2", "message": "fmt.Printf format %d has arg count of wrong type string"}], "assign": [{"posn": "ROOTSEPmain.go:4:2", "message": "self-assignment of x"}, {"posn": "ROOTSEPmain.go:4:2", "message": "self-assignment of x"}]}}
"#;

/// One module at the root, vetted from [`VET_ANSWER`], so the bytes carry the sort and the dedup the
/// declaration relies on.
pub(crate) fn Go_Lint_Production() -> Vec<u8>
{
    let root = PathBuf::from(Root());
    let context = FactContext {
        snapshot: SnapshotId::From_Digest(nomos_model::Content_Digest(b"go-lint-snapshot")),
        variant: BuildVariantId::From_Digest(nomos_model::Content_Digest(b"go-lint-variant")),
        configuration: ConfigurationId::From_Digest(nomos_model::Content_Digest(b"go-lint-configuration")),
        generation: GenerationId::INITIAL,
    };
    let ports = LintPorts { launcher: &ScriptedVet, filesystem: &OneModule, environment: &AtRoot };

    let answer = Materialize_Modules(&root, &["main.go", "pkg/util.go"], context, &ports);
    assert!(answer.unlinted.is_empty(), "the scripted module is vetted: {:?}", answer.unlinted);
    assert_eq!(answer.facts.len(), 1, "one module, one fact: {:?}", answer.facts);

    let mut rendered = Vec::new();
    for module in answer.facts
    {
        rendered.extend_from_slice(format!("module\t{}\n", module.path).as_bytes());
        rendered.extend_from_slice(format!("key\t{}\n", module.fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&module.fact.payload.bytes);
    }
    return rendered;
}

fn Root() -> &'static str
{
    return if cfg!(windows) { "C:\\fixture" } else { "/fixture" };
}

/// Prints [`VET_ANSWER`] for the fixture, whatever it is asked -- each position under the root as
/// this platform spells it, escaped as JSON escapes a backslash.
struct ScriptedVet;

/// Answers from fixed data, so every run is the same.
impl Strategy for ScriptedVet
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for ScriptedVet
{
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        let separator = if cfg!(windows) { "\\\\" } else { "/" };
        let stdout = VET_ANSWER.replace("ROOT", &Root().replace('\\', "\\\\")).replace("SEP", separator);
        return Ok(ProgramOutput { outcome: ExitOutcome::Exited { code: 0 }, stdout, stderr: String::new() });
    }
}

/// A filesystem holding one `go.mod`, at the root.
struct OneModule;

/// Answers from fixed data, so every read is the same.
impl Strategy for OneModule
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl FileSystem for OneModule
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        if self.Exists(path)
        {
            return Ok("module example.com/fixture\n\ngo 1.22\n".to_owned());
        }
        return Err(FileSystemError::NotFound { path: path.to_string_lossy().into_owned() });
    }

    fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
    {
        unimplemented!("linting never writes")
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return path == Path::new(Root()).join("go.mod");
    }
}

/// No variable is set, and the working directory is the fixture's root.
struct AtRoot;

/// Answers from fixed data, so every read is the same.
impl Strategy for AtRoot
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for AtRoot
{
    fn Variable(&self, _name: &str) -> Option<OsString>
    {
        return None;
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        return Ok(PathBuf::from(Root()));
    }
}

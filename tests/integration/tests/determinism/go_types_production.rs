//! `nomos-lang-go-types`' production, over a scripted helper and an in-memory module, so what the
//! declaration covers -- reading and filing, given what the helper printed -- is what is measured.

use nomos_contracts::{BuildVariantId, ConfigurationId, DeterminismStrength, GenerationId, ReproducibilityScope, SnapshotId, Strategy, TraceEquivalence};
use nomos_lang_go_types::{FactContext, Materialize_Files, TypesPorts};
use nomos_platform::{Command, Environment, EnvironmentError, ExitOutcome, FileSystem, FileSystemError, ProgramLauncher, ProgramOutput};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What the helper printed for the fixture, with `ROOT` standing for the fixture's root and `SEP` for
/// this platform's separator, each as a JSON string spells it: two checked files, one of them
/// discarding nothing, and three values for the other that arrive out of position order.
const HELPER_ANSWER: &str = r#"{"file":"ROOTSEPmain.go","kind":"checked"}
{"file":"ROOTSEPpkgSEPutil.go","kind":"checked"}
{"column":5,"file":"ROOTSEPmain.go","is_error":true,"kind":"value","line":12,"type":"error"}
{"column":2,"file":"ROOTSEPmain.go","is_error":false,"kind":"value","line":14,"type":"string"}
{"column":2,"file":"ROOTSEPmain.go","is_error":false,"kind":"value","line":12,"type":"int"}
"#;

/// One module at the root, typed from [`HELPER_ANSWER`], so the bytes carry the position order the
/// declaration relies on.
pub(crate) fn Go_Types_Production() -> Vec<u8>
{
    let root = PathBuf::from(Root());
    let context = FactContext {
        snapshot: SnapshotId::From_Digest(nomos_model::Content_Digest(b"go-types-snapshot")),
        variant: BuildVariantId::From_Digest(nomos_model::Content_Digest(b"go-types-variant")),
        configuration: ConfigurationId::From_Digest(nomos_model::Content_Digest(b"go-types-configuration")),
        generation: GenerationId::INITIAL,
    };
    let ports = TypesPorts { launcher: &ScriptedHelper, filesystem: &OneModule, environment: &AtRoot };

    let answer = Materialize_Files(&root, &["main.go", "pkg/util.go"], context, &ports);
    assert!(answer.untyped.is_empty(), "the scripted module is typed: {:?}", answer.untyped);
    assert_eq!(answer.facts.len(), 2, "two checked files, two facts: {:?}", answer.facts);

    let mut rendered = Vec::new();
    for file in answer.facts
    {
        rendered.extend_from_slice(format!("file\t{}\n", file.path).as_bytes());
        rendered.extend_from_slice(format!("key\t{}\n", file.fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&file.fact.payload.bytes);
    }
    return rendered;
}

fn Root() -> &'static str
{
    return if cfg!(windows) { "C:\\fixture" } else { "/fixture" };
}

/// Prints [`HELPER_ANSWER`] for the fixture, whatever it is asked -- each path under the root as this
/// platform spells it, escaped as JSON escapes a backslash.
struct ScriptedHelper;

/// Answers from fixed data, so every run is the same.
impl Strategy for ScriptedHelper
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for ScriptedHelper
{
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        let separator = if cfg!(windows) { "\\\\" } else { "/" };
        let stdout = HELPER_ANSWER.replace("ROOT", &Root().replace('\\', "\\\\")).replace("SEP", separator);
        return Ok(ProgramOutput { outcome: ExitOutcome::Exited { code: 0 }, stdout, stderr: String::new() });
    }
}

/// A filesystem holding one `go.mod`, at the root, that accepts the helper's two files and keeps
/// neither.
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
        return Err(FileSystemError::NotFound { path: path.to_string_lossy().into_owned() });
    }

    fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
    {
        return Ok(());
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return path == Path::new(Root()).join("go.mod");
    }
}

/// A temporary directory under the fixture's root, and the root as the working directory.
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
    fn Variable(&self, name: &str) -> Option<OsString>
    {
        return (name == "TMPDIR").then(|| return PathBuf::from(Root()).join("tmp").into_os_string());
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        return Ok(PathBuf::from(Root()));
    }
}

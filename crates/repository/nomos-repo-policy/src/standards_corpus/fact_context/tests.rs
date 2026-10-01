//! What a repository's declared corpus becomes once it is wrapped into a fact.

use super::*;
use crate::standards_corpus::STANDARDS_CORPUS_JSON;
use nomos_platform::{DeterminismStrength, FileSystemError, ReproducibilityScope, Strategy, TraceEquivalence};
use std::path::PathBuf;

/// A rule document declaring the whole of what the corpus's schema requires of one.
const A_RULE: &str = "kind: rule\nid: naming\ntitle: Naming\nseverity: MUST NOT\ngate: review\nenforced_by: [review]\n";

/// A [`FileSystem`] holding one declared root with one rule document under it, so that the
/// payload a reader produces can be followed all the way to the bytes a fact carries.
struct FakeCorpus
{
    declaration: String,
}

impl FakeCorpus
{
    fn Declaring(declaration: serde_json::Value) -> Self
    {
        return Self {
            declaration: declaration.to_string(),
        };
    }
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for FakeCorpus
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl FileSystem for FakeCorpus
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        if path.file_name().and_then(|name| return name.to_str()) == Some(STANDARDS_CORPUS_JSON)
        {
            return Ok(self.declaration.clone());
        }

        return Ok(format!("---\n{A_RULE}---\n\n# Prose\n\nA body this reader does not read at all.\n"));
    }

    fn Read_Directory(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>
    {
        if path.ends_with("docs/standards")
        {
            return Ok(vec![PathBuf::from(".").join("docs/standards/naming.md")]);
        }

        return Err(FileSystemError::NotFound { path: path.display().to_string() });
    }

    fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
    {
        unimplemented!("this reader never writes")
    }

    fn Exists(&self, _path: &Path) -> bool
    {
        return true;
    }
}

#[test]
fn Test_Materialize_Workspace_Should_Materialize_A_Declared_Rule()
{
    let filesystem = FakeCorpus::Declaring(serde_json::json!({ "corpus_roots": ["docs/standards"] }));

    let PolicyFact { subject, fact } = Materialize_Workspace(Path::new("."), Context(), &filesystem).expect("well-formed JSON");

    let decoded = nomos_cap_standards_corpus::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
    assert_eq!(decoded.roots, vec!["docs/standards".to_owned()]);
    assert_eq!(decoded.rules.len(), 1, "{decoded:?}");
    assert_eq!(decoded.rules.first().expect("asserted a length of one above").id, "naming");

    scaffolding::test_support::Assert_Carries_Its_Declaration(&subject, &fact, Declared_Guarantee(), &Payload_Schema());
}

/// A repository declaring nothing still gets a fact, carrying an empty population. This is the
/// `done_when`'s own requirement — "a repository that declares no corpus judges exactly as it
/// does today" — and an absent fact would instead make a rule report the capability's absence.
#[test]
fn Test_Materialize_Workspace_Should_Materialize_An_Empty_Declaration()
{
    let filesystem = FakeCorpus::Declaring(serde_json::json!({}));

    let PolicyFact { fact, .. } = Materialize_Workspace(Path::new("."), Context(), &filesystem).expect("well-formed JSON");

    let decoded = nomos_cap_standards_corpus::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
    assert_eq!(decoded, nomos_cap_standards_corpus::StandardsCorpusPayload::default());
    assert!(decoded.roots.is_empty(), "a repository that declared no corpus is not a declared empty corpus");
}

#[test]
fn Test_A_Fact_Key_Should_Depend_On_The_Guarantee()
{
    let identity = scaffolding::Declared_Identity(Capability(), CONTRACT_VERSION, PROVIDER, CONTRACT_VERSION);

    scaffolding::test_support::Assert_Keys_File_Apart_By_Guarantee(&identity, Declared_Guarantee());
}

/// This test's own fixture, shared with its six siblings — see
/// `crate::scaffolding::test_support::Sample_Context`.
fn Context() -> FactContext
{
    return scaffolding::test_support::Sample_Context();
}

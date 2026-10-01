//! What the reader makes of a declaration and of the documents it points at, and what it
//! refuses.

use super::*;
use nomos_cap_standards_corpus::{Encode_Payload, Parse_Payload};
use nomos_contracts::GateCategory;
use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform_std::StdFileSystem;

/// A rule document declaring the whole of what the corpus's schema requires of one.
const A_RULE: &str = "kind: rule\nid: naming\ntitle: Naming\nseverity: MUST NOT\ngate: blocking\nenforced_by: [review]\n";

/// A [`FileSystem`] holding a fixed tree instead of a real one, so that a reader whose whole job
/// is walking a corpus can be measured against corpora a test wrote — including the corpora a
/// repository would have to get wrong for the reader's refusals to be exercised at all.
///
/// This is the one double in this crate's `reading` modules that answers [`FileSystem::Read_Directory`]:
/// every sibling's fake hands back one document's text, because every sibling reads one file and
/// never descends. Here a directory is derived from the tree — a path is a directory exactly when
/// some document sits under it — so no test has to declare its own directories and none can
/// declare a directory the walk would not reach anyway.
struct FakeTree
{
    declaration: String,
    files: Vec<(PathBuf, String)>,
}

impl FakeTree
{
    /// A tree whose only file is the repository's own declaration.
    fn Declaring(declaration: serde_json::Value) -> Self
    {
        return Self {
            declaration: declaration.to_string(),
            files: Vec::new(),
        };
    }

    /// The same tree with `path` holding `text`, replacing a document already at that path —
    /// one path holds one document, so a test that perturbs a key is writing the same corpus
    /// with one document changed rather than a corpus with two documents in one place.
    fn With(mut self, path: &str, text: &str) -> Self
    {
        let path = PathBuf::from(".").join(path);
        self.files.retain(|(known, _)| return *known != path);
        self.files.push((path, text.to_owned()));

        return self;
    }

    /// Every path in this tree: each document, and every directory each one sits under.
    fn Nodes(&self) -> Vec<PathBuf>
    {
        let mut nodes = Vec::new();
        for (file, _) in &self.files
        {
            let mut current = Some(file.as_path());
            while let Some(path) = current
            {
                if !nodes.iter().any(|known| return known == path)
                {
                    nodes.push(path.to_path_buf());
                }
                current = path.parent().filter(|parent| return !parent.as_os_str().is_empty());
            }
        }

        return nodes;
    }
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for FakeTree
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl FileSystem for FakeTree
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        if path.file_name().and_then(|name| return name.to_str()) == Some(STANDARDS_CORPUS_JSON)
        {
            return Ok(self.declaration.clone());
        }

        return self
            .files
            .iter()
            .find(|(known, _)| return known == path)
            .map(|(_, text)| return text.clone())
            .ok_or_else(|| FileSystemError::NotFound { path: path.display().to_string() });
    }

    fn Read_Directory(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>
    {
        let entries: Vec<PathBuf> = self.Nodes().into_iter().filter(|node| return node.parent() == Some(path)).collect();
        if entries.is_empty()
        {
            // A file, or a path this tree holds nothing under. Either way the reader's own
            // descent treats it as "not a directory", which is the question being asked.
            return Err(FileSystemError::NotFound { path: path.display().to_string() });
        }

        return Ok(entries);
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

/// A document carrying `lines` between the declaration fences, with prose below that the reader
/// must not read.
fn Document(lines: &str) -> String
{
    return format!("---\n{lines}---\n\n# Prose\n\nA body this reader does not read at all.\n");
}

/// A corpus holding one well-formed rule, which the tests below perturb one key at a time.
fn Corpus() -> FakeTree
{
    return FakeTree::Declaring(serde_json::json!({ "corpus_roots": ["docs/standards"] }))
        .With("docs/standards/naming.md", &Document(A_RULE));
}

/// The one issue `payload` carries, or a panic naming what it carried instead.
fn Sole_Issue(payload: &StandardsCorpusPayload) -> DeclarationIssue
{
    assert_eq!(payload.issues.len(), 1, "expected exactly one issue: {payload:?}");

    return payload.issues.first().expect("asserted a length of one above").clone();
}

#[test]
fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_Missing_File()
{
    let payload = Discover_Workspace(Path::new("a/path/that/does/not/exist"), &StdFileSystem)
        .expect("a missing declaration file declares nothing rather than failing");

    assert_eq!(payload, StandardsCorpusPayload::default());
}

/// A file that parses and names no `corpus_roots` key is a repository that wrote the file and
/// not filled it in, which reads as declaring nothing rather than as a refusal.
#[test]
fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_File_With_No_Root_Key()
{
    let payload = Discover_Workspace(Path::new("."), &FakeTree::Declaring(serde_json::json!({}))).expect("well-formed JSON");

    assert_eq!(payload, StandardsCorpusPayload::default());
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_Invalid_Json()
{
    let filesystem = FakeTree {
        declaration: "not json at all".to_owned(),
        files: Vec::new(),
    };

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

    assert!(error.reason.contains("not valid JSON"), "{}", error.reason);
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Roots_Value_That_Is_Not_An_Array()
{
    let filesystem = FakeTree::Declaring(serde_json::json!({ "corpus_roots": "docs/standards" }));

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-array corpus_roots must be refused");

    assert!(error.reason.contains("is not an array"), "{}", error.reason);
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Non_String_Root()
{
    let filesystem = FakeTree::Declaring(serde_json::json!({ "corpus_roots": [7] }));

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-string root must be refused");

    assert!(error.reason.contains("non-string entry"), "{}", error.reason);
}

/// The order of a JSON array is not the repository's own statement about anything, so the reader
/// imposes one. Without it the encoded bytes would differ between two reads of the same file.
#[test]
fn Test_Discover_Workspace_Should_Order_Roots_Independently_Of_The_Declared_Order()
{
    let forwards = FakeTree::Declaring(serde_json::json!({ "corpus_roots": ["alpha", "zulu"] }));
    let backwards = FakeTree::Declaring(serde_json::json!({ "corpus_roots": ["zulu", "alpha"] }));

    let first = Discover_Workspace(Path::new("."), &forwards).expect("well-formed JSON");
    let second = Discover_Workspace(Path::new("."), &backwards).expect("well-formed JSON");

    assert_eq!(first, second);
}

/// A repository that writes its root the way a path is usually written still gets the
/// repository-relative prefix the population is reported under.
#[test]
fn Test_Discover_Workspace_Should_Normalize_A_Declared_Root()
{
    let filesystem = FakeTree::Declaring(serde_json::json!({ "corpus_roots": ["/docs/standards/"] }));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

    assert_eq!(payload.roots, vec!["docs/standards".to_owned()]);
}

#[test]
fn Test_A_Document_Declaring_A_Rule_Should_Be_Read_As_One()
{
    let payload = Discover_Workspace(Path::new("."), &Corpus()).expect("well-formed JSON");

    assert_eq!(
        payload.rules,
        vec![DeclaredRule {
            path: "docs/standards/naming.md".to_owned(),
            id: "naming".to_owned(),
            title: "Naming".to_owned(),
            severity: RuleSeverity::MustNot,
            gate: GateCategory::Blocking,
            enforced_by: vec!["review".to_owned()],
        }]
    );
    assert!(payload.issues.is_empty(), "{payload:?}");
}

/// The population is every document on disk, not every *rule*: a document that declares nothing,
/// one that declares an index, and one whose kind this workspace cannot read are all documents,
/// and a file that is not markdown is not one.
#[test]
fn Test_The_Population_Should_Hold_Every_Markdown_Document_And_No_Other_File()
{
    let filesystem = Corpus()
        .With("docs/standards/README.md", "# A document that declares nothing\n")
        .With("docs/standards/index.md", &Document("kind: index\ntitle: Index\n"))
        .With("docs/standards/style.md", &Document("kind: standard\ntitle: Style\n"))
        .With("docs/standards/notes.txt", "not a standards document\n");

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");
    let paths: Vec<&str> = payload.documents.iter().map(|document| return document.path.as_str()).collect();

    assert_eq!(
        paths,
        vec![
            "docs/standards/README.md",
            "docs/standards/index.md",
            "docs/standards/naming.md",
            "docs/standards/style.md",
        ]
    );
    assert_eq!(payload.rules.len(), 1, "only naming.md declared itself a rule: {payload:?}");
}

/// [`DeclarationKind::Undeclared`] covers two documents the corpus's schema treats alike and this
/// reader must not: one that wrote nothing, and one whose declaration did not parse. The
/// difference between them is the issue, and losing it would lose the defect.
#[test]
fn Test_A_Document_That_Declared_Nothing_Should_Be_Undeclared_And_Not_An_Issue()
{
    let filesystem = Corpus().With("docs/standards/README.md", "# A document that declares nothing\n");

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

    assert!(payload.issues.is_empty(), "a document that declared nothing has nothing to refuse: {payload:?}");
    assert_eq!(payload.documents.len(), 2, "{payload:?}");
}

#[test]
fn Test_A_Declaration_Whose_Fence_Never_Closes_Should_Be_An_Issue()
{
    let filesystem = FakeTree::Declaring(serde_json::json!({ "corpus_roots": ["docs/standards"] }))
        .With("docs/standards/naming.md", &format!("---\n{A_RULE}"));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

    assert!(payload.rules.is_empty(), "an unclosed declaration is not a rule: {payload:?}");
    assert!(Sole_Issue(&payload).reason.contains("fence never closes"));
}

/// The defect measured in the `code-standards` corpus: forty rule documents declare no `gate` at
/// all. A reader that supplied a default would report a declaration nobody wrote.
#[test]
fn Test_A_Rule_Naming_No_Gate_Should_Be_An_Issue_Rather_Than_A_Defaulted_Gate()
{
    let declared = "kind: rule\nid: naming\ntitle: Naming\nseverity: MUST NOT\nenforced_by: [review]\n";

    let payload = Discover_Workspace(Path::new("."), &Corpus().With("docs/standards/naming.md", &Document(declared)))
        .expect("well-formed JSON");

    assert!(payload.rules.is_empty(), "a rule that declared no gate is not a read rule: {payload:?}");
    assert!(Sole_Issue(&payload).reason.contains("no gate"), "{payload:?}");
}

/// The other defect measured there: thirteen rule documents declare `gate: permissive`, which is
/// outside the schema's enum. The vocabulary is closed, so this is an issue rather than a rule
/// filed under a category the document never named.
#[test]
fn Test_A_Gate_Outside_The_Vocabulary_Should_Be_An_Issue()
{
    let declared = "kind: rule\nid: naming\ntitle: Naming\nseverity: MUST NOT\ngate: permissive\nenforced_by: [review]\n";

    let payload = Discover_Workspace(Path::new("."), &Corpus().With("docs/standards/naming.md", &Document(declared)))
        .expect("well-formed JSON");

    assert!(payload.rules.is_empty(), "{payload:?}");
    assert!(Sole_Issue(&payload).reason.contains("gate is not in the corpus's vocabulary"), "{payload:?}");
}

#[test]
fn Test_A_Severity_Outside_The_Vocabulary_Should_Be_An_Issue()
{
    let declared = "kind: rule\nid: naming\ntitle: Naming\nseverity: SHALL\ngate: review\nenforced_by: [review]\n";

    let payload = Discover_Workspace(Path::new("."), &Corpus().With("docs/standards/naming.md", &Document(declared)))
        .expect("well-formed JSON");

    assert!(payload.rules.is_empty(), "{payload:?}");
    assert!(Sole_Issue(&payload).reason.contains("severity is not in the corpus's vocabulary"), "{payload:?}");
}

/// The corpus's schema requires at least one enforcer, and the payload carries no row for a rule
/// whose ownership the document left unwritten: an ownerless rule would encode to an empty field
/// its own reader refuses.
#[test]
fn Test_A_Rule_Naming_No_Enforcer_Should_Be_An_Issue()
{
    let declared = "kind: rule\nid: naming\ntitle: Naming\nseverity: MUST NOT\ngate: review\nenforced_by: []\n";

    let payload = Discover_Workspace(Path::new("."), &Corpus().With("docs/standards/naming.md", &Document(declared)))
        .expect("well-formed JSON");
    let encoded = Encode_Payload(&payload);

    assert!(Sole_Issue(&payload).reason.contains("requires at least one"), "{payload:?}");
    assert!(Parse_Payload(&encoded).is_ok(), "an issue is the shape that survives this payload's own encoding");
}

/// The corpus's schema declares no `applies_to`, and the sibling `code-standards` corpus added
/// keys of its own. A reader that refused a document for carrying one would report a defect where
/// a document is simply richer than this reader.
#[test]
fn Test_An_Unknown_Front_Matter_Key_Should_Be_Ignored()
{
    let declared = format!("{A_RULE}applies_to: [rust]\ncategory: style\ncanonical: true\n");

    let payload = Discover_Workspace(Path::new("."), &Corpus().With("docs/standards/naming.md", &Document(&declared)))
        .expect("well-formed JSON");

    assert_eq!(payload.rules.len(), 1, "an unread key is not a defect: {payload:?}");
    assert!(payload.issues.is_empty(), "{payload:?}");
}

#[test]
fn Test_A_Declared_Root_That_Is_Not_A_Directory_Should_Be_An_Issue()
{
    let filesystem = FakeTree::Declaring(serde_json::json!({ "corpus_roots": ["docs/absent"] }));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

    assert_eq!(payload.roots, vec!["docs/absent".to_owned()]);
    assert!(Sole_Issue(&payload).reason.contains("could not be listed"), "{payload:?}");
}

/// A corpus is a tree, not a directory: the sibling `xvpe` corpus keeps its documents one level
/// down, and a reader that read only the top level would report a population of zero there.
#[test]
fn Test_A_Nested_Directory_Should_Be_Walked()
{
    let filesystem = Corpus().With("docs/standards/rust/naming.md", &Document(A_RULE));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");
    let paths: Vec<&str> = payload.rules.iter().map(|rule| return rule.path.as_str()).collect();

    assert_eq!(paths, vec!["docs/standards/naming.md", "docs/standards/rust/naming.md"]);
}

/// A declaration may name overlapping roots, and a document under two of them is one document.
/// The deduplication is load-bearing rather than tidy: a duplicate path is exactly what
/// [`Parse_Payload`] refuses, so a reader without it could not encode its own payload.
#[test]
fn Test_Overlapping_Roots_Should_Count_A_Document_Once()
{
    let filesystem = FakeTree::Declaring(serde_json::json!({ "corpus_roots": ["docs", "docs/standards"] }))
        .With("docs/standards/naming.md", &Document(A_RULE))
        .With("docs/README.md", "# Another document, under the wider root only\n");

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");
    let encoded = Encode_Payload(&payload);

    assert_eq!(payload.roots.len(), 2, "both roots are declared: {payload:?}");
    assert_eq!(payload.documents.len(), 2, "{payload:?}");
    assert_eq!(payload.rules.len(), 1, "{payload:?}");
    assert!(Parse_Payload(&encoded).is_ok(), "the population a reader produces must survive its own encoding");
}

/// The property `Canonical` exists for, measured against a population richer than the happy path:
/// a rule, an index, a document that declared nothing and a document whose declaration has a
/// defect. The reader's own payload must be exactly what its bytes decode to, so a caller that
/// reads a fact back out of an encoding sees the corpus rather than a lossy paraphrase of it.
#[test]
fn Test_The_Readers_Payload_Should_Round_Trip_Through_Its_Own_Encoding()
{
    let filesystem = Corpus()
        .With("docs/standards/README.md", "# A document that declares nothing\n")
        .With("docs/standards/index.md", &Document("kind: index\ntitle: Index\n"))
        .With("docs/standards/broken.md", &Document("kind: rule\nseverity: MUST\n"));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");
    let decoded = Parse_Payload(&Encode_Payload(&payload)).expect("this reader's own encoding of its own population");

    assert_eq!(decoded, payload);
    assert_eq!(payload.documents.len(), 4, "{payload:?}");
    assert_eq!(payload.rules.len(), 1, "{payload:?}");
    assert_eq!(payload.issues.len(), 1, "{payload:?}");
}

#[test]
fn Test_Discover_Workspace_Should_Read_This_Repositorys_Own_Declaration()
{
    let payload = Discover_Workspace(&Repository_Root(), &StdFileSystem)
        .expect("this repository's own nomos-standards-corpus.json is real, or absent, and either answers");

    assert!(
        payload.roots.is_empty(),
        "this repository declares no standards corpus of its own; the corpora it is measured \
         against live outside it and arrive through the three corpus variables: {payload:?}"
    );
}

fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    return manifest
        .ancestors()
        .nth(3)
        .expect("this crate sits three levels below the workspace root")
        .to_path_buf();
}

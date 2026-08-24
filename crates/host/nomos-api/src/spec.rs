//! A third real caller of `nomos_spec_orchestration` verbs -- `Profiles`, the simplest of
//! its nine (no root, no corpus, no platform, not even an already-assembled store),
//! `Sources`, the next-simplest: a unit `SpecCommand` variant needing no field of its own,
//! but the first here to go through `nomos_spec_orchestration::corpus::Assemble` at all --
//! and now `Record`, the first verb here carrying a request payload of its own rather than a
//! unit variant.

use nomos_platform_std::StdFileSystem;
use nomos_spec_orchestration::corpus::{Absence, CorpusRequest, DEFAULT_REVISION};
use nomos_spec_orchestration::{RecordAnswer, RecordRefusal, RecordRequest, SpecCommand};
use nomos_spec_project::Profile;
use nomos_spec_store::{DocumentSource, NodeSummary, StoreError};
use serde::Serialize;
use std::path::PathBuf;

/// Which environment variable names the corpus root -- the same constant `nomos-cli`'s own
/// `main.rs` keeps privately, ported here rather than shared for the reason every other
/// composition-root value in this crate is: `OD-HOST-002` treats it as this root's own
/// choice, not a shared dependency.
const CORPUS_VARIABLE: &str = "NOMOS_V14_CORPUS";

/// Lists every shipped projection profile, exactly as `nomos spec profiles` would, and
/// hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Spec_Profiles() -> ProfilesResponse
{
    return ProfilesResponse::From(nomos_spec_orchestration::Profiles());
}

/// What a real `nomos spec profiles` produced, in a shape `serde_json` can hand across a
/// wire.
///
/// A tagged enum, the same shape `WorkListResponse` already uses: `nomos_spec_project::
/// ProjectError` does not derive `Serialize` -- nothing needed a wire format for it before
/// this crate existed -- so the refusal case is named rather than collapsed into an empty
/// success. `Profiles`' own doc calls that refusal "a defect in this build, not in
/// anything the caller did": real, but not expected to fire in practice, the same
/// never-actually-taken arm `nomos-cli`'s own `gate/tests.rs` already exhausts other exit
/// codes for.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ProfilesResponse
{
    /// The embedded profile catalogue parsed.
    Listed
    {
        /// Every shipped projection profile.
        profiles: Vec<Profile>,
    },
    /// The embedded profile catalogue itself failed to parse.
    Unreadable
    {
        /// What went wrong, as `ProjectError`'s own `Display` renders it.
        cause: String,
    },
}

impl ProfilesResponse
{
    fn From(profiles: Result<Vec<Profile>, nomos_spec_project::ProjectError>) -> Self
    {
        return match profiles
        {
            Ok(profiles) => Self::Listed { profiles },
            Err(error) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

/// Reports what this store was assembled from, and what it is missing, exactly as `nomos
/// spec sources` would, and hands back a JSON-serializable response.
///
/// Builds its own `CorpusRequest` from the environment alone -- the same fields `nomos-cli`'s
/// own `main.rs::Corpus_Request` builds from `--corpus`/`--corpus-revision` or the
/// environment, minus the two argv overrides this crate has no argv to read. `CorpusRequest`
/// is configuration (which corpus, which revision), not a per-call identifier the way
/// `FindingQuery` or `ItemId` are, so this function takes no parameter for it -- the same
/// "no invented shape ahead of a real body" choice [`crate::Handle_Gate_Run`] already makes
/// for `GateCommand`'s own scope/rules selectors.
#[must_use]
pub fn Handle_Spec_Sources() -> SpecSourcesResponse
{
    let request = CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from),
        revision: DEFAULT_REVISION.to_owned(),
    };

    let outcome = nomos_spec_orchestration::Run(&SpecCommand::Sources, &request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Sources(sourced) = outcome
    else
    {
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecSourcesResponse::From(sourced);
}

/// What a real `nomos spec sources` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecSourcesResponse
{
    /// The store assembled -- possibly with real absences, which is itself the answer this
    /// verb exists to give, not a failure.
    Assembled
    {
        /// One line per input the assembly read, in the order it was read.
        read: Vec<String>,
        /// Every input the assembly expected and did not find.
        absent: Vec<AbsenceResponse>,
    },
    /// The store itself could not be assembled at all.
    Unreadable
    {
        /// What went wrong, as `StoreError`'s own `Display` renders it.
        cause: String,
    },
}

impl SpecSourcesResponse
{
    fn From(result: Result<nomos_spec_orchestration::SourcesAnswer, StoreError>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Assembled {
                read: answer.read,
                absent: answer.absent.into_iter().map(AbsenceResponse::From).collect(),
            },
            Err(error) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

/// A serializable twin of [`nomos_spec_orchestration::corpus::Absence`], which does not
/// derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct AbsenceResponse
{
    /// What is missing, as a person would name it.
    pub subject: String,
    /// Where it was looked for, concretely.
    pub expected: String,
    /// Why it is not here.
    pub cause: String,
    /// What is therefore not in this store.
    pub cost: String,
}

impl AbsenceResponse
{
    fn From(absence: Absence) -> Self
    {
        return Self {
            subject: absence.subject,
            expected: absence.expected,
            cause: absence.cause,
            cost: absence.cost,
        };
    }
}

/// Resolves one record by identifier, exactly as `nomos spec record` would, and hands back a
/// JSON-serializable response.
///
/// Builds its own `CorpusRequest` from the environment, the same composition
/// [`Handle_Spec_Sources`] already uses -- an API handler holds no pre-assembled `Assembly`
/// the way `nomos-cli`'s own multi-verb dispatch does, so it pays for one assembly per call.
/// `run::record::Record` itself never touches a `FileSystem`, but the shared entry point it
/// is dispatched through, `nomos_spec_orchestration::Run`, is generic over one for every verb
/// regardless -- see [`Handle_Spec_Sources`]'s own documentation for why.
#[must_use]
pub fn Handle_Spec_Record(request: &RecordRequest) -> SpecRecordResponse
{
    let corpus_request = CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from),
        revision: DEFAULT_REVISION.to_owned(),
    };

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Record(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Record(result) = outcome
    else
    {
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecRecordResponse::From(result);
}

/// What a real `nomos spec record` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecRecordResponse
{
    /// The one document behind the identifier, resolved.
    Resolved
    {
        /// The identifier that was asked about.
        id: String,
        /// The document it resolved to.
        document: DocumentSourceResponse,
    },
    /// No document backs this identifier at the requested revision.
    ///
    /// `node` is the graph's own summary of the identifier, when the identifier is known at
    /// all -- a node the store holds with no source document recorded against it is a
    /// different situation from an identifier nothing in the store recognizes.
    NotFound
    {
        id: String,
        revision: Option<String>,
        node: Option<NodeSummaryResponse>,
    },
    /// The identifier is held at more than one revision, so resolving one of them without a
    /// narrower request would be a guess.
    Ambiguous
    {
        id: String,
        documents: Vec<DocumentSourceResponse>,
    },
    /// The store could not be read at all.
    Unreadable
    {
        /// What went wrong, as `StoreError`'s own `Display` renders it.
        cause: String,
    },
}

impl SpecRecordResponse
{
    fn From(result: Result<RecordAnswer, RecordRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Resolved { id: answer.id, document: DocumentSourceResponse::From(answer.document) },
            Err(RecordRefusal::NotFound { id, revision, node }) =>
            {
                Self::NotFound { id, revision, node: node.map(NodeSummaryResponse::From) }
            }
            Err(RecordRefusal::Ambiguous { id, documents }) => Self::Ambiguous {
                id,
                documents: documents.into_iter().map(DocumentSourceResponse::From).collect(),
            },
            Err(RecordRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

/// A serializable twin of [`nomos_spec_store::DocumentSource`], which does not derive
/// `Serialize`. Omits `uid`: that field's own doc comment says it is "never exported and
/// never printed."
#[derive(Debug, Serialize)]
pub struct DocumentSourceResponse
{
    pub path: String,
    pub revision: String,
    /// The document's content address.
    pub content_hash: String,
    /// The document exactly as it was ingested, byte for byte.
    pub text: String,
}

impl DocumentSourceResponse
{
    fn From(document: DocumentSource) -> Self
    {
        return Self {
            path: document.path,
            revision: document.revision,
            content_hash: document.content_hash,
            text: document.text,
        };
    }
}

/// A serializable twin of [`nomos_spec_store::NodeSummary`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct NodeSummaryResponse
{
    pub node_id: String,
    pub kind: String,
    pub authority: String,
    pub representation: String,
    pub title: String,
}

impl NodeSummaryResponse
{
    fn From(node: NodeSummary) -> Self
    {
        return Self {
            node_id: node.node_id,
            kind: node.kind,
            authority: node.authority,
            representation: node.representation,
            title: node.title,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The catalogue is embedded in the binary, so this is a real end-to-end exercise of
    /// this workspace's own shipped profiles -- no fixture, no scratch directory, the same
    /// zero-setup shape `Profiles()` itself has.
    #[test]
    fn Test_This_Workspaces_Own_Shipped_Profiles_Should_List()
    {
        let response = Handle_Spec_Profiles();

        let ProfilesResponse::Listed { profiles } = response
        else
        {
            panic!("this workspace's own embedded catalogue parses");
        };
        assert!(!profiles.is_empty());
    }

    #[test]
    fn Test_A_Real_Profiles_Response_Should_Round_Trip_As_Json()
    {
        let response = Handle_Spec_Profiles();

        let json = serde_json::to_string(&response).expect("a ProfilesResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized ProfilesResponse always has this field");

        assert_eq!(outcome, "listed", "{json}");
    }

    /// A real end-to-end call, over whatever this session's own `NOMOS_V14_CORPUS` state
    /// happens to be. `Assemble`'s own contract is that neither the variable nor the corpus
    /// it names being present is required -- an absent corpus reports real absences rather
    /// than refusing -- so this asserts only the outcome variant, never a specific
    /// read/absent count a local environment would make flaky.
    #[test]
    fn Test_A_Real_Call_Should_Assemble_Not_Refuse()
    {
        let response = Handle_Spec_Sources();

        assert!(matches!(response, SpecSourcesResponse::Assembled { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Assembled_Response_Should_Round_Trip_As_Json()
    {
        let response = Handle_Spec_Sources();

        let json = serde_json::to_string(&response).expect("a SpecSourcesResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecSourcesResponse always has this field");

        assert_eq!(outcome, "assembled", "{json}");
    }

    /// `D-132` is a real, embedded governing record -- `nomos_spec_orchestration`'s own
    /// `tests.rs` already resolves it with no corpus present -- so this needs no scratch
    /// directory and no `NOMOS_V14_CORPUS` to be set, the same zero-setup shape every other
    /// test in this file already has.
    #[test]
    fn Test_A_Real_Governing_Record_Should_Resolve()
    {
        let request = RecordRequest { id: "D-132".to_owned(), revision: None };

        let response = Handle_Spec_Record(&request);

        let SpecRecordResponse::Resolved { id, document } = response
        else
        {
            panic!("D-132 is a governing record, embedded even with no corpus: {response:?}");
        };
        assert_eq!(id, "D-132");
        assert!(document.text.starts_with("---\nid: D-132\n"), "{}", document.text);
    }

    #[test]
    fn Test_An_Unknown_Identifier_Should_Report_Not_Found_With_No_Node()
    {
        let request = RecordRequest { id: "D-9999999-DOES-NOT-EXIST".to_owned(), revision: None };

        let response = Handle_Spec_Record(&request);

        assert!(matches!(response, SpecRecordResponse::NotFound { node: None, .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Resolved_Response_Should_Round_Trip_As_Json()
    {
        let request = RecordRequest { id: "D-132".to_owned(), revision: None };

        let response = Handle_Spec_Record(&request);

        let json = serde_json::to_string(&response).expect("a SpecRecordResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecRecordResponse always has this field");

        assert_eq!(outcome, "resolved", "{json}");
    }
}

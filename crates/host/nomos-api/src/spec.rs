//! A third real caller of `nomos_spec_orchestration` verbs -- `Profiles`, the simplest of
//! its nine (no root, no corpus, no platform, not even an already-assembled store),
//! `Sources`, the next-simplest: a unit `SpecCommand` variant needing no field of its own,
//! but the first here to go through `nomos_spec_orchestration::corpus::Assemble` at all --
//! and now `Record`, the first verb here carrying a request payload of its own rather than a
//! unit variant.

use nomos_platform_std::StdFileSystem;
use nomos_spec_orchestration::corpus::{Absence, CorpusRequest, DEFAULT_REVISION};
use nomos_spec_orchestration::{
    EditRequest, FreshnessAnswer, FreshnessRefusal, FreshnessRequest, PreviewRefusal, ProfileOutcome, RecordAnswer,
    RecordRefusal, RecordRequest, RenderAnswer, RenderRefusal, RenderRequest, SpecCommand, TableAnswer, TableRefusal,
    TableRequest, Verdict,
};
use nomos_spec_project::{Freshness, Profile, Stamp};
use nomos_spec_store::{
    BlockChange, DocumentSource, EditError, EditPreview, IdentityChange, NodeSummary, NormativeMovement,
    NormativeOutcome, PathMatch, RecordProjection, RowCensus, StoreError, TableLine,
};
use nomos_spec_model::RecordRelation;
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

/// Selects table rows exactly as `nomos spec table` would, and hands back a
/// JSON-serializable response.
///
/// Follows [`Handle_Spec_Record`]'s own composition: builds a `CorpusRequest` from the
/// environment, dispatches through the shared `Run` entry point, matches `SpecOutcome::
/// Table`. `run::table::Table` itself never touches a `FileSystem`, the same as `Record`.
#[must_use]
pub fn Handle_Spec_Table(request: &TableRequest) -> SpecTableResponse
{
    let corpus_request = CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from),
        revision: DEFAULT_REVISION.to_owned(),
    };

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Table(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Table(result) = outcome
    else
    {
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecTableResponse::From(result);
}

/// What a real `nomos spec table` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecTableResponse
{
    /// The rows the request selected, and the document and census they came from.
    Selected
    {
        document: DocumentSourceResponse,
        tier: PathMatchResponse,
        census: RowCensusResponse,
        lines: Vec<TableLineResponse>,
    },
    /// No document in the store matches the address.
    NoSuchDocument,
    /// The address matches more than one document.
    AmbiguousDocument
    {
        matched: usize,
        tier: PathMatchResponse,
    },
    /// The document was found and read, and the request's own narrowing selected no rows.
    NoRows
    {
        document: DocumentSourceResponse,
        tier: PathMatchResponse,
        census: RowCensusResponse,
    },
    /// The store could not be read at all.
    Unreadable
    {
        /// What went wrong, as `StoreError`'s own `Display` renders it.
        cause: String,
    },
}

impl SpecTableResponse
{
    fn From(result: Result<TableAnswer, TableRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Selected {
                document: DocumentSourceResponse::From(answer.document),
                tier: PathMatchResponse::From(answer.tier),
                census: RowCensusResponse::From(answer.census),
                lines: answer.lines.into_iter().map(TableLineResponse::From).collect(),
            },
            Err(TableRefusal::NoSuchDocument) => Self::NoSuchDocument,
            Err(TableRefusal::AmbiguousDocument { matched, tier }) =>
            {
                Self::AmbiguousDocument { matched, tier: PathMatchResponse::From(tier) }
            }
            Err(TableRefusal::NoRows { document, tier, census }) => Self::NoRows {
                document: DocumentSourceResponse::From(document),
                tier: PathMatchResponse::From(tier),
                census: RowCensusResponse::From(census),
            },
            Err(TableRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

/// A serializable twin of [`nomos_spec_store::PathMatch`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathMatchResponse
{
    /// The path was given in full.
    Exact,
    /// The last segment of the path was given.
    FileName,
    /// The text appears somewhere in the path.
    Fragment,
}

impl PathMatchResponse
{
    fn From(tier: PathMatch) -> Self
    {
        return match tier
        {
            PathMatch::Exact => Self::Exact,
            PathMatch::FileName => Self::FileName,
            PathMatch::Fragment => Self::Fragment,
        };
    }
}

/// A serializable twin of [`nomos_spec_store::RowCensus`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct RowCensusResponse
{
    /// Every pipe line, whatever it turned out to be.
    pub lines: u32,
    pub header: u32,
    pub content: u32,
    pub separator: u32,
    /// Authored lines: header and content together.
    pub non_separator: u32,
}

impl RowCensusResponse
{
    fn From(census: RowCensus) -> Self
    {
        return Self {
            lines: census.lines,
            header: census.header,
            content: census.content,
            separator: census.separator,
            non_separator: census.non_separator,
        };
    }
}

/// A serializable twin of [`nomos_spec_store::TableLine`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct TableLineResponse
{
    /// Which block of the document carries it.
    pub block_ordinal: u32,
    /// Which table within that block.
    pub table_ordinal: u32,
    /// Which line within the block, 1-based.
    pub row_ordinal: u32,
    /// `header`, `content` or `separator`.
    pub kind: String,
    pub cells: Vec<String>,
    /// The line as authored.
    pub text: String,
    pub content_hash: String,
}

impl TableLineResponse
{
    fn From(line: TableLine) -> Self
    {
        return Self {
            block_ordinal: line.block_ordinal,
            table_ordinal: line.table_ordinal,
            row_ordinal: line.row_ordinal,
            kind: line.kind,
            cells: line.cells,
            text: line.text,
            content_hash: line.content_hash,
        };
    }
}

/// Renders one record's markdown from the store's own rows, exactly as `nomos spec markdown`
/// would, and hands back a JSON-serializable response.
///
/// Follows [`Handle_Spec_Record`]'s own composition, reusing the same [`RecordRequest`]:
/// builds a `CorpusRequest` from the environment, dispatches through the shared `Run` entry
/// point, matches `SpecOutcome::Markdown`. `run::markdown::Markdown` itself never touches a
/// `FileSystem`, the same as `Record` and `Table`.
#[must_use]
pub fn Handle_Spec_Markdown(request: &RecordRequest) -> SpecMarkdownResponse
{
    let corpus_request = CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from),
        revision: DEFAULT_REVISION.to_owned(),
    };

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Markdown(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Markdown(result) = outcome
    else
    {
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecMarkdownResponse::From(result);
}

/// What a real `nomos spec markdown` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecMarkdownResponse
{
    /// The record, rendered from the store's own rows.
    Resolved
    {
        node_id: String,
        path: String,
        revision: String,
        /// The markdown, rendered from the store's rows.
        markdown: String,
        /// What the ingested bytes hash to.
        source_hash: String,
        /// What this projection hashes to.
        projected_hash: String,
        /// Whether the projection is the ingested bytes -- `source_hash == projected_hash`.
        matches_source: bool,
    },
    /// The identifier did not resolve to one record's markdown.
    Refused
    {
        /// What went wrong, as `EditError`'s own `Display` renders it.
        cause: String,
    },
}

impl SpecMarkdownResponse
{
    fn From(result: Result<RecordProjection, EditError>) -> Self
    {
        return match result
        {
            Ok(projection) =>
            {
                let matches_source = projection.Matches_Source();

                Self::Resolved {
                    node_id: projection.node_id,
                    path: projection.path,
                    revision: projection.revision,
                    markdown: projection.markdown,
                    source_hash: projection.source_hash,
                    projected_hash: projection.projected_hash,
                    matches_source,
                }
            }
            Err(error) => Self::Refused { cause: error.to_string() },
        };
    }
}

/// Compares every shipped profile's build root against the store, exactly as `nomos spec
/// freshness` would, and hands back a JSON-serializable response.
///
/// Follows [`Handle_Spec_Record`]'s own composition. `run::freshness::Freshness` is generic
/// over `FileSystem` (it reads a rendered body and its sidecar at `request.into`, through
/// `nomos_platform::FileSystem::Read_To_String`), but never writes -- unlike `Render`,
/// `Preview` and `Commit`, exposing it carries none of the "does a wire call write to this
/// host's disk" hazard those three do, since `StdFileSystem` here only ever reads paths the
/// caller already named.
#[must_use]
pub fn Handle_Spec_Freshness(request: &FreshnessRequest) -> SpecFreshnessResponse
{
    let corpus_request = CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from),
        revision: DEFAULT_REVISION.to_owned(),
    };

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Freshness(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Freshness(result) = outcome
    else
    {
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecFreshnessResponse::From(result);
}

/// What a real `nomos spec freshness` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecFreshnessResponse
{
    /// Every profile this run looked at, and what it required.
    Examined
    {
        examined: Vec<ProfileOutcomeResponse>,
        required: Vec<String>,
    },
    /// `--profile` or a `--require` names an identifier the catalogue does not carry.
    NoSuchProfile
    {
        requested: String,
        known: Vec<String>,
    },
    /// `--require` names a profile that `--profile` narrowed this run away from.
    RequirementUnexamined
    {
        requested: String,
        only: Option<String>,
    },
    /// The embedded catalogue or the store could not be read at all.
    Unreadable
    {
        /// What went wrong, as the underlying error's own `Display` renders it.
        cause: String,
    },
}

impl SpecFreshnessResponse
{
    fn From(result: Result<FreshnessAnswer, FreshnessRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Examined {
                examined: answer.examined.into_iter().map(ProfileOutcomeResponse::From).collect(),
                required: answer.required,
            },
            Err(FreshnessRefusal::NoSuchProfile { requested, known }) => Self::NoSuchProfile { requested, known },
            Err(FreshnessRefusal::RequirementUnexamined { requested, only }) =>
            {
                Self::RequirementUnexamined { requested, only }
            }
            Err(FreshnessRefusal::Project(error)) => Self::Unreadable { cause: error.to_string() },
            Err(FreshnessRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

/// A serializable twin of [`nomos_spec_orchestration::ProfileOutcome`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct ProfileOutcomeResponse
{
    /// The profile examined, resolved and (if subject-addressed) already narrowed. Already
    /// `Serialize` -- reused directly.
    pub profile: Profile,
    pub verdict: VerdictResponse,
}

impl ProfileOutcomeResponse
{
    fn From(outcome: ProfileOutcome) -> Self
    {
        return Self { profile: outcome.profile, verdict: VerdictResponse::From(outcome.verdict) };
    }
}

/// A serializable twin of [`nomos_spec_orchestration::Verdict`], which does not derive
/// `Serialize`. `Verdict::Compared`'s own `Result<Freshness, ProjectError>` is split into two
/// variants here rather than nested, so a wire caller can match on `kind` alone.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VerdictResponse
{
    /// Neither the body nor its sidecar is on disk.
    Absent,
    /// A body is there and no sidecar beside it.
    Unstamped,
    /// A sidecar is there and no body beside it.
    Unbodied,
    /// Both halves are there, compared against what the store would produce now.
    Compared
    {
        /// Whether every comparison below found nothing to report.
        fresh: bool,
        stale: Option<(String, String)>,
        edited: Option<(String, String)>,
        diverged: Option<(String, String)>,
    },
    /// Rebuilding the profile to compare against failed.
    BuildFailed
    {
        /// What went wrong, as `ProjectError`'s own `Display` renders it.
        cause: String,
    },
}

impl VerdictResponse
{
    fn From(verdict: Verdict) -> Self
    {
        return match verdict
        {
            Verdict::Absent => Self::Absent,
            Verdict::Unstamped => Self::Unstamped,
            Verdict::Unbodied => Self::Unbodied,
            Verdict::Compared(Ok(freshness)) => Self::From_Freshness(&freshness),
            Verdict::Compared(Err(error)) => Self::BuildFailed { cause: error.to_string() },
        };
    }

    fn From_Freshness(freshness: &Freshness) -> Self
    {
        return Self::Compared {
            fresh: freshness.Is_Fresh(),
            stale: freshness.stale.clone(),
            edited: freshness.edited.clone(),
            diverged: freshness.diverged.clone(),
        };
    }
}

/// Says what committing a staged edit would change, without writing anything, exactly as
/// `nomos spec preview` would, and hands back a JSON-serializable response.
///
/// Follows [`Handle_Spec_Record`]'s own composition. `run::preview::Preview` is generic over
/// `FileSystem` (it reads `request.from` through `nomos_platform::FileSystem::
/// Read_To_String`), but that is its only contact with a filesystem -- everything else is an
/// in-memory `assembly.store` operation. Unlike `Render` and `Commit`, which place or
/// overwrite files at a caller-known output path, `Preview` never writes, so exposing it
/// over nomos-api carries none of the "does a wire call write to this host's disk" hazard
/// those two do -- the same reasoning [`Handle_Spec_Freshness`] already gives for its own
/// `FileSystem` parameter.
#[must_use]
pub fn Handle_Spec_Preview(request: &EditRequest) -> SpecPreviewResponse
{
    let corpus_request = CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from),
        revision: DEFAULT_REVISION.to_owned(),
    };

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Preview(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Preview(result) = outcome
    else
    {
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecPreviewResponse::From(result);
}

/// What a real `nomos spec preview` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecPreviewResponse
{
    /// What committing the staged edit would change.
    Previewed
    {
        node_id: String,
        path: String,
        /// The paths before and after, when this edit is also a rename.
        rename: Option<(String, String)>,
        /// The staged bytes.
        markdown: String,
        blocks: Vec<BlockChangeResponse>,
        identity: Vec<IdentityChangeResponse>,
        relations_added: Vec<RecordRelationResponse>,
        relations_removed: Vec<RecordRelationResponse>,
        statements: Vec<NormativeMovementResponse>,
        /// The question `D-129` calls mandatory: does this edit move wording that was there?
        wording_moved: bool,
        /// Whether this edit changes anything at all.
        changes_nothing: bool,
    },
    /// `--from` could not be read.
    Unreadable
    {
        path: String,
        /// What went wrong, as `FileSystemError`'s own `Display` renders it.
        cause: String,
    },
    /// The store refused the staged edit.
    Refused
    {
        /// What went wrong, as `EditError`'s own `Display` renders it.
        cause: String,
    },
}

impl SpecPreviewResponse
{
    fn From(result: Result<EditPreview, PreviewRefusal>) -> Self
    {
        return match result
        {
            Ok(preview) => Self::From_Preview(&preview),
            Err(PreviewRefusal::Unreadable { path, error }) =>
            {
                Self::Unreadable { path: path.display().to_string(), cause: error.to_string() }
            }
            Err(PreviewRefusal::Edit(error)) => Self::Refused { cause: error.to_string() },
        };
    }

    fn From_Preview(preview: &EditPreview) -> Self
    {
        return Self::Previewed {
            node_id: preview.Node_Id().to_owned(),
            path: preview.Path().to_owned(),
            rename: preview.Rename().map(|(before, after)| return (before.to_owned(), after.to_owned())),
            markdown: preview.Markdown().to_owned(),
            blocks: preview.Blocks().iter().cloned().map(BlockChangeResponse::From).collect(),
            identity: preview.Identity().iter().cloned().map(IdentityChangeResponse::From).collect(),
            relations_added: preview.Relations_Added().iter().cloned().map(RecordRelationResponse::From).collect(),
            relations_removed: preview
                .Relations_Removed()
                .iter()
                .cloned()
                .map(RecordRelationResponse::From)
                .collect(),
            statements: preview.Statements().iter().cloned().map(NormativeMovementResponse::From).collect(),
            wording_moved: preview.Wording_Moved(),
            changes_nothing: preview.Changes_Nothing(),
        };
    }
}

/// A serializable twin of [`nomos_spec_store::BlockChange`], which does not derive
/// `Serialize`. Tagged `change` rather than the more usual `kind`, since `Added` and
/// `Removed` already carry a field of their own named `kind`.
#[derive(Debug, Serialize)]
#[serde(tag = "change", rename_all = "snake_case")]
pub enum BlockChangeResponse
{
    Added
    {
        ordinal: u32, kind: String
    },
    Removed
    {
        ordinal: u32, kind: String
    },
    /// Same position, different wording.
    Reworded
    {
        ordinal: u32, before: String, after: String
    },
    /// Same wording, different position.
    Moved
    {
        from: u32, to: u32
    },
    /// Same wording, different whitespace.
    Reflowed
    {
        ordinal: u32
    },
}

impl BlockChangeResponse
{
    fn From(change: BlockChange) -> Self
    {
        return match change
        {
            BlockChange::Added { ordinal, kind } => Self::Added { ordinal, kind },
            BlockChange::Removed { ordinal, kind } => Self::Removed { ordinal, kind },
            BlockChange::Reworded { ordinal, before, after } => Self::Reworded { ordinal, before, after },
            BlockChange::Moved { from, to } => Self::Moved { from, to },
            BlockChange::Reflowed { ordinal } => Self::Reflowed { ordinal },
        };
    }
}

/// A serializable twin of [`nomos_spec_store::IdentityChange`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct IdentityChangeResponse
{
    pub field: String,
    pub before: String,
    pub after: String,
}

impl IdentityChangeResponse
{
    fn From(change: IdentityChange) -> Self
    {
        return Self { field: change.field, before: change.before, after: change.after };
    }
}

/// A serializable twin of [`nomos_spec_model::RecordRelation`], which derives `Deserialize`
/// but not `Serialize`.
#[derive(Debug, Serialize)]
pub struct RecordRelationResponse
{
    pub target: String,
    pub relation: String,
}

impl RecordRelationResponse
{
    fn From(relation: RecordRelation) -> Self
    {
        return Self { target: relation.target, relation: relation.relation };
    }
}

/// A serializable twin of [`nomos_spec_store::NormativeMovement`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct NormativeMovementResponse
{
    pub statement_id: String,
    pub canonical_hash: String,
    pub outcome: NormativeOutcomeResponse,
}

impl NormativeMovementResponse
{
    fn From(movement: NormativeMovement) -> Self
    {
        return Self {
            statement_id: movement.statement_id,
            canonical_hash: movement.canonical_hash,
            outcome: NormativeOutcomeResponse::From(movement.outcome),
        };
    }
}

/// A serializable twin of [`nomos_spec_store::NormativeOutcome`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NormativeOutcomeResponse
{
    /// Still in the block it was in.
    Held
    {
        block: u32
    },
    /// Still present, in a different block.
    Moved
    {
        from: u32, to: u32
    },
    /// Was in the record and is not in the staged text.
    Gone
    {
        from: u32
    },
    /// Recorded against the record and not found in it before the edit either.
    Unlocatable,
}

impl NormativeOutcomeResponse
{
    fn From(outcome: NormativeOutcome) -> Self
    {
        return match outcome
        {
            NormativeOutcome::Held { block } => Self::Held { block },
            NormativeOutcome::Moved { from, to } => Self::Moved { from, to },
            NormativeOutcome::Gone { from } => Self::Gone { from },
            NormativeOutcome::Unlocatable => Self::Unlocatable,
        };
    }
}

/// Builds a projection profile and writes both halves of it under `request.into`, exactly as
/// `nomos spec render` would, and hands back a JSON-serializable response.
///
/// Follows [`Handle_Spec_Record`]'s own composition. Unlike `Record`, `Table`, `Markdown`,
/// `Freshness` and `Preview`, this verb does write: `run::render::Render` places a built
/// projection's body and its sidecar under `request.into` through
/// `nomos_platform::FileSystem::Replace_Atomically`, unconditionally overwriting whatever
/// was there. This crate already performs real, unauthenticated `StdFileSystem`-backed
/// writes over a wire call today -- [`Handle_Work_Claim`](crate::Handle_Work_Claim) writes a
/// real ledger file at a caller-named directory -- and `request.into` here plays the same
/// role `directory` does there: a caller-named root, with the leaf path underneath it chosen
/// by this crate's own store/profile machinery rather than by the caller. The one thing
/// `Render`'s write is not is destructive in the way `Commit`'s vacate step can be: a
/// rendered projection is a derived, regenerable artifact, not the governing record itself.
#[must_use]
pub fn Handle_Spec_Render(request: &RenderRequest) -> SpecRenderResponse
{
    let corpus_request = CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from),
        revision: DEFAULT_REVISION.to_owned(),
    };

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Render(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Render(result) = outcome
    else
    {
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecRenderResponse::From(result);
}

/// What a real `nomos spec render` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecRenderResponse
{
    /// Both halves of the built projection, placed where the run asked for them.
    Placed
    {
        /// The profile identifier that was built.
        id: String,
        /// Where the body landed.
        body: PathBuf,
        /// Where its sidecar landed.
        sidecar: PathBuf,
        /// What the build selected, and what it hashes to. Already `Serialize`, reused
        /// directly.
        stamp: Stamp,
    },
    /// The catalogue does not carry a profile by this name.
    NoSuchProfile
    {
        requested: String,
        known: Vec<String>,
    },
    /// The built projection could not be written where it was asked to go.
    Unwritable
    {
        path: String,
        /// What went wrong, as `FileSystemError`'s own `Display` renders it.
        cause: String,
    },
    /// The catalogue could not be read, resolving the subject failed, building the
    /// projection failed, or the store could not be assembled at all.
    Unreadable
    {
        /// What went wrong, as the underlying error's own `Display` renders it.
        cause: String,
    },
}

impl SpecRenderResponse
{
    fn From(result: Result<RenderAnswer, RenderRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Placed { id: answer.id, body: answer.body, sidecar: answer.sidecar, stamp: answer.stamp },
            Err(RenderRefusal::NoSuchProfile { requested, known }) => Self::NoSuchProfile { requested, known },
            Err(RenderRefusal::Unwritable { path, error }) =>
            {
                Self::Unwritable { path: path.display().to_string(), cause: error.to_string() }
            }
            Err(RenderRefusal::Project(error)) => Self::Unreadable { cause: error.to_string() },
            Err(RenderRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
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

    /// An empty, unique scratch directory means nothing has ever been rendered there, so
    /// every shipped profile examines as `Absent` regardless of this session's own
    /// `NOMOS_V14_CORPUS` state -- the same zero-setup determinism every other test in this
    /// file already relies on.
    fn Unique_Scratch_Directory(label: &str) -> std::path::PathBuf
    {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        let directory = std::env::temp_dir().join(format!(
            "nomos-api-spec-freshness-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("a fresh scratch directory can always be created");

        return directory;
    }

    #[test]
    fn Test_A_Real_Call_Over_An_Empty_Root_Should_Examine_Every_Profile_As_Absent()
    {
        let request =
            FreshnessRequest { into: Unique_Scratch_Directory("empty-root"), profile: None, require: Vec::new() };

        let response = Handle_Spec_Freshness(&request);

        let SpecFreshnessResponse::Examined { examined, .. } = response
        else
        {
            panic!("an empty scratch root examines cleanly: {response:?}");
        };
        assert!(!examined.is_empty(), "the embedded catalogue always ships at least one profile");
        assert!(
            examined.iter().all(|outcome| matches!(outcome.verdict, VerdictResponse::Absent)),
            "{examined:?}"
        );
    }

    #[test]
    fn Test_An_Unknown_Required_Profile_Should_Report_No_Such_Profile()
    {
        let request = FreshnessRequest {
            into: Unique_Scratch_Directory("unknown-required"),
            profile: None,
            require: vec!["definitely-not-a-real-profile".to_owned()],
        };

        let response = Handle_Spec_Freshness(&request);

        assert!(matches!(response, SpecFreshnessResponse::NoSuchProfile { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Examined_Response_Should_Round_Trip_As_Json()
    {
        let request =
            FreshnessRequest { into: Unique_Scratch_Directory("round-trip"), profile: None, require: Vec::new() };

        let response = Handle_Spec_Freshness(&request);

        let json = serde_json::to_string(&response).expect("a SpecFreshnessResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecFreshnessResponse always has this field");

        assert_eq!(outcome, "examined", "{json}");
    }

    /// No document under any corpus state can match this name, regardless of whether this
    /// session's own `NOMOS_V14_CORPUS` happens to be set -- `Assemble`'s own contract is
    /// that an absent corpus reports real absences rather than refusing.
    #[test]
    fn Test_A_Real_Call_For_An_Unknown_Document_Should_Report_No_Such_Document()
    {
        let request = TableRequest {
            document: "definitely-nonexistent-table-document-xyz".to_owned(),
            block: None,
            table: None,
            revision: None,
        };

        let response = Handle_Spec_Table(&request);

        assert!(matches!(response, SpecTableResponse::NoSuchDocument), "{response:?}");
    }

    #[test]
    fn Test_A_No_Such_Document_Response_Should_Round_Trip_As_Json()
    {
        let request = TableRequest {
            document: "definitely-nonexistent-table-document-xyz".to_owned(),
            block: None,
            table: None,
            revision: None,
        };

        let response = Handle_Spec_Table(&request);

        let json = serde_json::to_string(&response).expect("a SpecTableResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecTableResponse always has this field");

        assert_eq!(outcome, "no_such_document", "{json}");
    }

    /// `D-132` is a real, embedded governing record with declared front matter --
    /// `nomos_spec_orchestration`'s own `tests.rs` already renders it with no corpus present.
    #[test]
    fn Test_A_Real_Governing_Record_Should_Render_As_Markdown()
    {
        let request = RecordRequest { id: "D-132".to_owned(), revision: None };

        let response = Handle_Spec_Markdown(&request);

        let SpecMarkdownResponse::Resolved { node_id, markdown, .. } = response
        else
        {
            panic!("D-132 is a governing record with declared front matter: {response:?}");
        };
        assert_eq!(node_id, "D-132");
        assert!(markdown.starts_with("---\nid: D-132\n"), "{markdown}");
    }

    #[test]
    fn Test_An_Unknown_Identifier_Should_Report_Refused()
    {
        let request = RecordRequest { id: "D-9999999-DOES-NOT-EXIST".to_owned(), revision: None };

        let response = Handle_Spec_Markdown(&request);

        assert!(matches!(response, SpecMarkdownResponse::Refused { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Resolved_Markdown_Response_Should_Round_Trip_As_Json()
    {
        let request = RecordRequest { id: "D-132".to_owned(), revision: None };

        let response = Handle_Spec_Markdown(&request);

        let json = serde_json::to_string(&response).expect("a SpecMarkdownResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecMarkdownResponse always has this field");

        assert_eq!(outcome, "resolved", "{json}");
    }

    /// Stages a canonical heading rename against `id`'s own real, embedded markdown (read
    /// through this crate's own `Handle_Spec_Markdown`, so this test needs no corpus and
    /// touches no file this repository tracks), writes it to `staged.md` under `into`, and
    /// hands back the path it was written to. Mirrors
    /// `nomos_spec_orchestration`'s own `Staged_Heading_Rename`.
    fn Staged_Heading_Rename(id: &str, into: &std::path::Path) -> std::path::PathBuf
    {
        let request = RecordRequest { id: id.to_owned(), revision: None };
        let SpecMarkdownResponse::Resolved { markdown, .. } = Handle_Spec_Markdown(&request)
        else
        {
            panic!("{id} is a governing record, embedded even with no corpus");
        };
        let edited = markdown.replace("## Decision", "## The decision");
        let staged = into.join("staged.md");
        std::fs::write(&staged, &edited).expect("writes the staged edit");

        return staged;
    }

    #[test]
    fn Test_A_Real_Heading_Rename_Should_Preview_Wording_Moved()
    {
        let staged = Staged_Heading_Rename("D-132", &Unique_Scratch_Directory("preview"));
        let request = EditRequest { id: "D-132".to_owned(), from: staged, rename: None };

        let response = Handle_Spec_Preview(&request);

        let SpecPreviewResponse::Previewed { wording_moved, .. } = response
        else
        {
            panic!("a canonical heading rename previews cleanly: {response:?}");
        };
        assert!(wording_moved, "a heading rename must count as wording moved");
    }

    #[test]
    fn Test_A_Missing_Staged_File_Should_Report_Unreadable()
    {
        let request = EditRequest {
            id: "D-132".to_owned(),
            from: std::path::PathBuf::from("no-such-staged-file-anywhere.md"),
            rename: None,
        };

        let response = Handle_Spec_Preview(&request);

        assert!(matches!(response, SpecPreviewResponse::Unreadable { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Previewed_Response_Should_Round_Trip_As_Json()
    {
        let staged = Staged_Heading_Rename("D-132", &Unique_Scratch_Directory("preview-json"));
        let request = EditRequest { id: "D-132".to_owned(), from: staged, rename: None };

        let response = Handle_Spec_Preview(&request);

        let json = serde_json::to_string(&response).expect("a SpecPreviewResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecPreviewResponse always has this field");

        assert_eq!(outcome, "previewed", "{json}");
    }

    /// Builds from the embedded governing records alone, so a test naming it needs no corpus
    /// -- the same profile `nomos_spec_orchestration`'s own `tests.rs` and
    /// `crates/host/nomos-cli/tests/read_surface.rs` both use for the same reason.
    const EMBEDDED_PROFILE: &str = "domain-specification";

    #[test]
    fn Test_A_Real_Render_Should_Place_Both_Files_On_Disk()
    {
        let into = Unique_Scratch_Directory("render");
        let request = RenderRequest { profile: EMBEDDED_PROFILE.to_owned(), into: into.clone(), subject: None };

        let response = Handle_Spec_Render(&request);

        let SpecRenderResponse::Placed { id, body, sidecar, .. } = response
        else
        {
            panic!("{EMBEDDED_PROFILE} builds from the embedded governing records alone: {response:?}");
        };
        assert_eq!(id, EMBEDDED_PROFILE);
        assert!(body.starts_with(&into), "{}", body.display());
        let written = std::fs::read_to_string(&body).expect("Render actually wrote the body to disk");
        assert!(!written.is_empty());
        std::fs::read_to_string(&sidecar).expect("Render actually wrote the sidecar to disk");
    }

    #[test]
    fn Test_An_Unknown_Profile_Should_Report_No_Such_Profile()
    {
        let request = RenderRequest {
            profile: "definitely-not-a-real-profile".to_owned(),
            into: Unique_Scratch_Directory("render-unknown"),
            subject: None,
        };

        let response = Handle_Spec_Render(&request);

        assert!(matches!(response, SpecRenderResponse::NoSuchProfile { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Placed_Response_Should_Round_Trip_As_Json()
    {
        let request = RenderRequest {
            profile: EMBEDDED_PROFILE.to_owned(),
            into: Unique_Scratch_Directory("render-json"),
            subject: None,
        };

        let response = Handle_Spec_Render(&request);

        let json = serde_json::to_string(&response).expect("a SpecRenderResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecRenderResponse always has this field");

        assert_eq!(outcome, "placed", "{json}");
    }
}

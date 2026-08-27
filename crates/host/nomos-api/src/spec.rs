//! A third real caller of `nomos_spec_orchestration` verbs -- `Profiles`, the simplest of
//! its nine (no root, no corpus, no platform, not even an already-assembled store),
//! `Sources`, the next-simplest: a unit `SpecCommand` variant needing no field of its own,
//! but the first here to go through `nomos_spec_orchestration::corpus::Assemble` at all --
//! and `Record`, `Table`, `Markdown`, `Freshness`, `Preview`, `Render`, `Commit` and `Submit`
//! after it, closing `SpecCommand` and its one sibling verb entirely.
//!
//! Every handler and its own top-level response type share one file below, the same
//! locality `crate::response` keeps for Gate's three verbs -- [`record`] pairs
//! [`Handle_Spec_Record`] with [`SpecRecordResponse`], and so on for every other verb -- and
//! every twin type that exists only to serialize one field of a bigger response gets a file
//! of its own beside the response it belongs to. [`corpus_request`] is the one file with no
//! type at all: [`Build_Corpus_Request`] is the `CorpusRequest` composition every
//! store-touching handler here shares.

mod absence;
mod block_change;
mod commit;
mod commit_report;
mod committed_preview;
mod corpus_request;
mod decision_gap;
mod document_source;
mod failure;
mod field_value;
mod freshness;
mod identity_change;
mod markdown;
mod node_summary;
mod normative_movement;
mod normative_outcome;
mod origin;
mod path_match;
mod preview;
mod profile_outcome;
mod profiles;
mod record;
mod record_relation;
mod refusal;
mod render;
mod rendered_projection;
mod reproduction;
mod row_census;
mod severity;
mod sources;
mod submission;
mod submission_kind;
mod submission_state;
mod submit;
mod table;
mod table_line;
mod vacate_outcome;
mod vacated;
mod verdict;

pub use absence::AbsenceResponse;
pub use block_change::BlockChangeResponse;
pub use commit::{Handle_Spec_Commit, SpecCommitResponse};
pub use commit_report::CommitReportResponse;
pub use committed_preview::CommittedPreviewResponse;
pub(crate) use corpus_request::Build_Corpus_Request;
pub use decision_gap::DecisionGapResponse;
pub use document_source::DocumentSourceResponse;
pub use failure::FailureResponse;
pub use field_value::FieldValueResponse;
pub use freshness::{Handle_Spec_Freshness, SpecFreshnessResponse};
pub use identity_change::IdentityChangeResponse;
pub use markdown::{Handle_Spec_Markdown, SpecMarkdownResponse};
pub use node_summary::NodeSummaryResponse;
pub use normative_movement::NormativeMovementResponse;
pub use normative_outcome::NormativeOutcomeResponse;
pub use origin::OriginResponse;
pub use path_match::PathMatchResponse;
pub use preview::{Handle_Spec_Preview, SpecPreviewResponse};
pub use profile_outcome::ProfileOutcomeResponse;
pub use profiles::{Handle_Spec_Profiles, ProfilesResponse};
pub use record::{Handle_Spec_Record, SpecRecordResponse};
pub use record_relation::RecordRelationResponse;
pub use refusal::RefusalResponse;
pub use render::{Handle_Spec_Render, SpecRenderResponse};
pub use rendered_projection::RenderedProjectionResponse;
pub use reproduction::ReproductionResponse;
pub use row_census::RowCensusResponse;
pub use severity::SeverityResponse;
pub use sources::{Handle_Spec_Sources, SpecSourcesResponse};
pub use submission::SubmissionResponse;
pub use submission_kind::SubmissionKindResponse;
pub use submission_state::SubmissionStateResponse;
pub use submit::{Handle_Spec_Submit, SpecSubmitResponse};
pub use table::{Handle_Spec_Table, SpecTableResponse};
pub use table_line::TableLineResponse;
pub use vacate_outcome::VacateOutcomeResponse;
pub use vacated::VacatedResponse;
pub use verdict::VerdictResponse;

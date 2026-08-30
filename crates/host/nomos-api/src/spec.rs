//! A third real caller of `nomos_spec_orchestration` verbs -- `Profiles`, the simplest of
//! its nine (no root, no corpus, no platform, not even an already-assembled store),
//! `Sources`, the next-simplest: a unit `SpecCommand` variant needing no field of its own,
//! but the first here to go through `nomos_spec_orchestration::corpus::Assemble_Corpus` at all --
//! and `Record`, `Table`, `Markdown`, `Freshness`, `Preview`, `Render`, `Commit` and `Submit`
//! after it, closing `SpecCommand` and its one sibling verb entirely.
//!
//! Every handler and its own top-level response type share one file below, the same
//! locality `crate::response` keeps for Gate's three verbs -- [`record_response`] pairs
//! [`Handle_Spec_Record`] with [`RecordResponse`], and so on for every other verb -- and
//! every twin type that exists only to serialize one field of a bigger response gets a file
//! of its own beside the response it belongs to. [`corpus_request`] is the one file with no
//! type at all: [`Build_Corpus_Request`] is the `CorpusRequest` composition every
//! store-touching handler here shares.

mod absence_response;
mod block_change_response;
mod commit_report_response;
mod commit_response;
mod committed_preview_response;
mod corpus_request;
mod decision_gap_response;
mod document_source_response;
mod failure_response;
mod field_value_response;
mod freshness_response;
mod identity_change_response;
mod markdown_response;
mod node_summary_response;
mod normative_movement_response;
mod normative_outcome_response;
mod origin_response;
mod path_match_response;
mod preview_response;
mod profile_outcome_response;
mod profiles_response;
mod record_relation_response;
mod record_response;
mod refusal_response;
mod render_response;
mod rendered_projection_response;
mod reproduction_response;
mod row_census_response;
mod severity_response;
mod sources_response;
mod submission_kind_response;
mod submission_response;
mod submission_state_response;
mod submit_response;
mod table_line_response;
mod table_response;
mod vacate_outcome_response;
mod vacated_response;
mod verdict_response;

pub use absence_response::AbsenceResponse;
pub use block_change_response::BlockChangeResponse;
pub use commit_report_response::CommitReportResponse;
pub use commit_response::{CommitResponse, Handle_Spec_Commit};
pub use committed_preview_response::CommittedPreviewResponse;
pub(crate) use corpus_request::Build_Corpus_Request;
pub use decision_gap_response::DecisionGapResponse;
pub use document_source_response::DocumentSourceResponse;
pub use failure_response::FailureResponse;
pub use field_value_response::FieldValueResponse;
pub use freshness_response::{FreshnessResponse, Handle_Spec_Freshness};
pub use identity_change_response::IdentityChangeResponse;
pub use markdown_response::{Handle_Spec_Markdown, MarkdownResponse};
pub use node_summary_response::NodeSummaryResponse;
pub use normative_movement_response::NormativeMovementResponse;
pub use normative_outcome_response::NormativeOutcomeResponse;
pub use origin_response::OriginResponse;
pub use path_match_response::PathMatchResponse;
pub use preview_response::{Handle_Spec_Preview, PreviewResponse};
pub use profile_outcome_response::ProfileOutcomeResponse;
pub use profiles_response::{Handle_Spec_Profiles, ProfilesResponse};
pub use record_relation_response::RecordRelationResponse;
pub use record_response::{Handle_Spec_Record, RecordResponse};
pub use refusal_response::RefusalResponse;
pub use render_response::{Handle_Spec_Render, RenderResponse};
pub use rendered_projection_response::RenderedProjectionResponse;
pub use reproduction_response::ReproductionResponse;
pub use row_census_response::RowCensusResponse;
pub use severity_response::SeverityResponse;
pub use sources_response::{Handle_Spec_Sources, SourcesResponse};
pub use submission_kind_response::SubmissionKindResponse;
pub use submission_response::SubmissionResponse;
pub use submission_state_response::SubmissionStateResponse;
pub use submit_response::{Handle_Spec_Submit, SubmitResponse};
pub use table_line_response::TableLineResponse;
pub use table_response::{Handle_Spec_Table, TableResponse};
pub use vacate_outcome_response::VacateOutcomeResponse;
pub use vacated_response::VacatedResponse;
pub use verdict_response::VerdictResponse;

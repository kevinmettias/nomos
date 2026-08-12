//! Band 1 — ingest of the v14 corpus.
//!
//! I0 blobs, I1 source truth behind the manifest gate, I2 normative statements,
//! I3 the node graph.

#![forbid(unsafe_code)]

mod archaeology;
mod archive;
mod archive_error;
mod archive_error_kind;
mod artifact;
mod block_lineage;
mod block_field;
mod block_mismatch;
mod block_mismatch_kind;
mod catalog_entity;
mod catalog_report;
mod collision;
mod disposition;
mod document_fate;
mod family;
mod fate;
mod filler_block;
mod filler_census;
mod gate;
mod hollow;
mod identifier_outcome;
mod kind_census;
mod lineage;
mod listing;
mod member;
mod member_fate;
mod origin;
mod overlaid;
mod overlay;
mod pair_change;
mod phases;
mod recorded_block;
mod recorded_section;
mod recorded_statement;
mod reconciliation_report;
mod regression_report;
mod relocation;
mod restore;
mod restored;
mod revisions;
mod scope;
mod section_lineage;
mod siblings;
mod statement_divergence;
mod statement_file;
mod statement_report;
mod suite_report;
mod tally;
mod template;

pub use archaeology::{Regression, Revision, SHARED_BY};
pub use archive::{Archive, Archives_In};
pub use archive_error::ArchiveError;
pub use archive_error_kind::ArchiveErrorKind;
pub use artifact::Artifact;
pub use block_lineage::BlockLineage;
pub use block_field::BlockField;
pub use block_mismatch::BlockMismatch;
pub use block_mismatch_kind::BlockMismatchKind;
pub use catalog_entity::CatalogEntity;
pub use catalog_report::CatalogReport;
pub use disposition::Disposition;
pub use document_fate::DocumentFate;
pub use family::Family;
pub use fate::Fate;
pub use filler_block::FillerBlock;
pub use filler_census::FillerCensus;
pub use gate::{Check_Against_Manifest, GateReport, Parse_Block_Lineage};
pub use hollow::Hollow;
pub use identifier_outcome::IdentifierOutcome;
pub use kind_census::KindCensus;
pub use listing::Listing;
pub use lineage::{Ingest_Block_Dispositions, Ingest_Section_Lineage, Parse_Section_Lineage, SectionReport};
pub use member::Member;
pub use member_fate::MemberFate;
pub use origin::Origin;
pub use overlaid::Overlaid;
pub use overlay::{
    FILLER_PATTERNS, Ingest_Overlay_Document, Ingest_v15_Record, Is_Filler, OverlayReport, Parse_Artifact, Reconcile,
    Statements_In,
};
pub use pair_change::PairChange;
pub use phases::{
    Ingest_Blob, Ingest_Catalog, Ingest_Source_Document, Ingest_Statements, IngestError, Parse_Catalog,
    Parse_Statements,
};
pub use recorded_block::RecordedBlock;
pub use recorded_section::RecordedSection;
pub use recorded_statement::RecordedStatement;
pub use reconciliation_report::ReconciliationReport;
pub use regression_report::RegressionReport;
pub use relocation::Relocation;
pub use restore::{Models_In, Resolve, RestorationReport, Restore};
pub use restored::Restored;
pub use revisions::{Census, DOMAIN_VOLUMES, Fingerprint, Gaps, RevisionFingerprint, Revisions_In, Walk};
pub use scope::Scope;
pub use section_lineage::SectionLineage;
pub use siblings::{
    COMMENTARY, Ingest_Game_Plan, Ingest_Sibling_Suite, LINEAGE_NOTES, Prepare_Commentary_View, ROOT_SUITE, Sibling,
    Statements_Sourced_Only_From_Commentary,
};
pub use statement_divergence::StatementDivergence;
pub use statement_file::StatementFile;
pub use statement_report::StatementReport;
pub use suite_report::SuiteReport;
pub use tally::Tally;
pub use template::Template;

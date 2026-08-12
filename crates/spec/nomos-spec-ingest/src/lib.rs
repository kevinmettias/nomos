//! Band 1 — ingest of the v14 corpus.
//!
//! I0 blobs, I1 source truth behind the manifest gate, I2 normative statements,
//! I3 the node graph.

#![forbid(unsafe_code)]

mod archaeology;
mod reconciliation;
mod content;
mod archive;
mod family;
mod phases;
mod restore;
mod revisions;
mod siblings;

pub use archaeology::{Regression, Revision, SHARED_BY};
pub use archive::{Archive, Archives_In};
pub use archive::error::ArchiveError;
pub use archive::error_kind::ArchiveErrorKind;
pub use archive::artifact::Artifact;
pub use content::block::lineage::BlockLineage;
pub use content::block::field::BlockField;
pub use content::block::mismatch::BlockMismatch;
pub use content::block::mismatch_kind::BlockMismatchKind;
pub use archive::catalog_entity::CatalogEntity;
pub use archive::catalog_report::CatalogReport;
pub use reconciliation::disposition::Disposition;
pub use reconciliation::fate::document::DocumentFate;
pub use family::Family;
pub use reconciliation::fate::Fate;
pub use content::block::filler::FillerBlock;
pub use reconciliation::report::filler_census::FillerCensus;
pub use reconciliation::hollow::Hollow;
pub use reconciliation::identifier_outcome::IdentifierOutcome;
pub use reconciliation::report::kind_census::KindCensus;
pub use archive::listing::Listing;
pub use reconciliation::lineage::{Ingest_Block_Dispositions, Ingest_Section_Lineage, Parse_Section_Lineage, SectionReport};
pub use archive::member::Member;
pub use reconciliation::fate::member::MemberFate;
pub use reconciliation::lineage::origin::Origin;
pub use reconciliation::overlay::overlaid::Overlaid;
pub use reconciliation::overlay::{
    FILLER_PATTERNS, Ingest_Overlay_Document, Ingest_v15_Record, Is_Filler, OverlayReport, Parse_Artifact, Reconcile,
    Statements_In,
};
pub use reconciliation::overlay::pair_change::PairChange;
pub use phases::{Check_Against_Manifest, GateReport, IngestError, Ingest_Blob, Ingest_Catalog, Ingest_Source_Document, Ingest_Statements, Parse_Block_Lineage, Parse_Catalog, Parse_Statements};
pub use content::block::recorded::RecordedBlock;
pub use content::section::recorded::RecordedSection;
pub use content::statement::recorded::RecordedStatement;
pub use reconciliation::report::reconciliation::ReconciliationReport;
pub use reconciliation::report::regression::RegressionReport;
pub use reconciliation::lineage::relocation::Relocation;
pub use restore::{Models_In, Resolve, RestorationReport, Restore};
pub use reconciliation::restored::Restored;
pub use revisions::{Census, DOMAIN_VOLUMES, Fingerprint, Gaps, RevisionFingerprint, Revisions_In, Walk};
pub use reconciliation::scope::Scope;
pub use content::section::lineage::SectionLineage;
pub use siblings::{
    COMMENTARY, Ingest_Game_Plan, Ingest_Sibling_Suite, LINEAGE_NOTES, Prepare_Commentary_View, ROOT_SUITE, Sibling,
    Statements_Sourced_Only_From_Commentary,
};
pub use content::statement::divergence::StatementDivergence;
pub use content::statement::file::StatementFile;
pub use content::statement::report::StatementReport;
pub use reconciliation::report::suite::SuiteReport;
pub use reconciliation::report::tally::Tally;
pub use archive::template::Template;

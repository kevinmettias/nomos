//! Zone: Specification — ingest of the v14 corpus.
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

pub use archaeology::{Is_Template_Eligible, Regression_Between_Revisions, Revision, SHARED_BY, TEMPLATE_FLOOR};
pub use archive::{Archive, Archives_In};
pub use archive::error::Error;
pub use archive::error_kind::ErrorKind;
pub use archive::artifact::Artifact;
// `Lineage` would collide with `content::section::lineage::Lineage` if flattened bare, so
// each keeps its longer, table-naming public name here.
pub use content::block::lineage::Lineage as BlockLineage;
pub use content::block::field::Field;
pub use content::block::mismatch::Mismatch;
pub use content::block::mismatch_kind::MismatchKind;
pub use archive::catalog_entity::CatalogEntity;
pub use archive::catalog_report::CatalogReport;
pub use reconciliation::disposition::Disposition;
pub use reconciliation::fate::document_fate::DocumentFate;
pub use family::Family;
pub use reconciliation::fate::Fate;
pub use content::block::filler_block::FillerBlock;
pub use reconciliation::report::filler_census::FillerCensus;
pub use reconciliation::hollow::Hollow;
pub use reconciliation::identifier_outcome::IdentifierOutcome;
pub use reconciliation::report::kind_census::KindCensus;
pub use archive::listing::Listing;
pub use reconciliation::lineage::{Ingest_Block_Dispositions, Ingest_Section_Lineage, Parse_Section_Lineage, SectionReport};
pub use archive::member::Member;
pub use reconciliation::fate::member_fate::MemberFate;
pub use reconciliation::lineage::origin::Origin;
pub use reconciliation::overlay::artifact::Parse_Artifact;
pub use reconciliation::overlay::identifiers::Statements_In;
// `Report` would collide with `content::statement::report::Report` if flattened bare, so
// each keeps its longer, subsystem-naming public name here.
pub use reconciliation::overlay::report::{Ingest_Overlay_Document, Ingest_V15_Record, Report as OverlayReport};
pub use reconciliation::overlay::overlaid::Overlaid;
pub use reconciliation::overlay::pair_change::PairChange;
pub use reconciliation::overlay::reconcile::Reconcile_Artifacts;
pub use reconciliation::overlay::{FILLER_PATTERNS, Get_Filler_Pattern};
pub use phases::{Check_Against_Manifest, GateReport, IngestError, Ingest_Blob, Ingest_Catalog, Ingest_Source_Document, Ingest_Statements, Parse_Block_Lineage, Parse_Catalog, Parse_Statements};
pub use content::block::recorded_block::RecordedBlock;
pub use content::section::recorded_section::RecordedSection;
pub use content::statement::recorded_statement::RecordedStatement;
pub use reconciliation::report::reconciliation_report::ReconciliationReport;
pub use reconciliation::report::regression_report::RegressionReport;
pub use reconciliation::lineage::relocation::Relocation;
pub use restore::{Models_In, RestorationReport, Resolve_Model_Uid, Restore_Members};
pub use reconciliation::restored::Restored;
pub use revisions::{Census_Kinds, DOMAIN_VOLUMES, Fingerprint_Revision, Label_Gaps, RevisionFingerprint, Revisions_In, Walk_Revisions};
pub use reconciliation::scope::Scope;
pub use content::section::lineage::Lineage as SectionLineage;
pub use siblings::{
    COMMENTARY, Ingest_Game_Plan, Ingest_Sibling_Suite, LINEAGE_NOTES, Prepare_Commentary_View, ROOT_SUITE, Sibling,
    Statements_Sourced_Only_From_Commentary,
};
pub use content::statement::divergence::Divergence;
pub use content::statement::file::File;
pub use content::statement::report::Report as StatementReport;
pub use reconciliation::report::suite_report::SuiteReport;
pub use reconciliation::report::tally::Tally;
pub use archive::template::Template;

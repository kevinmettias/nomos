//! Band 1 — ingest of the v14 corpus.
//!
//! I0 blobs, I1 source truth behind the manifest gate, I2 normative statements,
//! I3 the node graph.

#![forbid(unsafe_code)]

mod archaeology;
mod archive;
mod gate;
mod lineage;
mod overlay;
mod phases;
mod restore;
mod revisions;
mod siblings;

pub use archaeology::{
    DocumentFate, Fate, FillerCensus, Hollow, MemberFate, Regression, RegressionReport,
    Relocation, Relocations, Revision, SHARED_BY, Tally, Template,
};
pub use archive::{Archive, ArchiveError, Archives_In};
pub use gate::{
    BlockLineage, BlockMismatch, Check_Against_Manifest, GateReport, Parse_Block_Lineage,
    RecordedBlock,
};
pub use lineage::{
    Ingest_Block_Dispositions, Ingest_Section_Lineage, Parse_Section_Lineage, RecordedSection,
    SectionLineage, SectionReport,
};
pub use overlay::{
    Artifact, Disposition, Family, FillerBlock, FILLER_PATTERNS, IdentifierOutcome,
    Ingest_Overlay_Document, Ingest_v15_Record, Is_Filler, Overlaid, OverlayReport, Parse_Artifact,
    ReconciliationReport, Reconcile, Statements_In,
};
pub use restore::{
    Collision, Extract, Member, Models_In, Origin, Resolve, RestorationReport, Restore, Restored,
};
pub use revisions::{
    Census, DOMAIN_VOLUMES, Fingerprint, Fingerprint_Of, Gaps, KindCensus, PairChange,
    RevisionFingerprint, Revisions_In, Scope, Walk,
};
pub use siblings::{
    COMMENTARY, Ingest_Game_Plan, Ingest_Sibling_Suite, LINEAGE_NOTES, Prepare_Commentary_View,
    ROOT_SUITE, Sibling, Statements_Sourced_Only_From_Commentary, SuiteReport,
};
pub use phases::{
    CatalogEntity, CatalogReport, IngestError, Ingest_Blob, Ingest_Catalog, Ingest_Source_Document,
    Ingest_Statements, Parse_Catalog, Parse_Statements, RecordedStatement, StatementDivergence,
    StatementFile, StatementReport,
};

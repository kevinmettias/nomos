//! Band 1 — ingest of the v14 corpus.
//!
//! I0 blobs, I1 source truth behind the manifest gate, I2 normative statements,
//! I3 the node graph.

#![forbid(unsafe_code)]

mod gate;
mod lineage;
mod phases;

pub use gate::{
    BlockLineage, BlockMismatch, Check_Against_Manifest, GateReport, Parse_Block_Lineage,
    RecordedBlock,
};
pub use lineage::{
    Ingest_Block_Dispositions, Ingest_Section_Lineage, Parse_Section_Lineage, RecordedSection,
    SectionLineage, SectionReport,
};
pub use phases::{
    CatalogEntity, CatalogReport, IngestError, Ingest_Blob, Ingest_Catalog, Ingest_Source_Document,
    Ingest_Statements, Parse_Catalog, Parse_Statements, RecordedStatement, StatementDivergence,
    StatementFile, StatementReport,
};

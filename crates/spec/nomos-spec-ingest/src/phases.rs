// The manifest gate I1's source truth sits behind.
mod gate_report;

pub use gate_report::{Check_Against_Manifest, GateReport, Parse_Block_Lineage};

mod blob;
mod ingest_error;
mod source_document;
mod statements;
mod catalog;
#[cfg(test)]
mod tests;

pub use blob::Ingest_Blob;
pub use ingest_error::IngestError;
pub use source_document::Ingest_Source_Document;
pub use statements::{Ingest_Statements, Parse_Statements};
pub use catalog::{Ingest_Catalog, Parse_Catalog};

use crate::CatalogReport;
use crate::CatalogEntity;
use crate::StatementDivergence;
use crate::RecordedStatement;
use crate::StatementReport;
use crate::StatementFile;
use nomos_spec_model::{ContentHash, Is_Normalized, Segment};
use nomos_spec_store::{NodeRow, SpecificationStore, StoreError};
use source_document::Store_Text;

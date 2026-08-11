mod blob;
mod source_document;
mod statements;
mod catalog;
#[cfg(test)]
mod tests;

pub use blob::Ingest_Blob;
pub use source_document::Ingest_Source_Document;
pub use statements::{Ingest_Statements, Parse_Statements};
pub use catalog::{Ingest_Catalog, Parse_Catalog};

use crate::catalog_report::CatalogReport;
use crate::catalog_entity::CatalogEntity;
use crate::statement_divergence::StatementDivergence;
use crate::recorded_statement::RecordedStatement;
use crate::statement_report::StatementReport;
use crate::statement_file::StatementFile;
use nomos_spec_model::{ContentHash, Is_Normalized, Segment};
use nomos_spec_store::{SpecificationStore, StoreError};
use source_document::Store_Text;

#[derive(Debug)]
pub enum IngestError
{
    Store(StoreError),
    Parse(String),
    /// The I1 gate found a disagreement. Ingest stops here.
    GateFailed
    {
        summary: String,
        first: Vec<String>,
    },
}

impl core::fmt::Display for IngestError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Parse(cause) => write!(formatter, "parse: {cause}"),
            Self::GateFailed { summary, first } => write!(
                formatter,
                "the source-truth gate failed ({summary}). Our segmentation disagrees with \
                 v14's manifest, so every hash computed downstream is untrustworthy:\n  {}",
                first.join("\n  ")
            ),
        };
    }
}

impl std::error::Error for IngestError
{}

impl From<StoreError> for IngestError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
}

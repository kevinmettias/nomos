//! Putting raw bytes in and getting their identity back.

use super::{IngestError, SpecificationStore};

/// I0 — store a byte-stream by content.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Blob(store: &mut SpecificationStore, content: &[u8]) -> Result<i64, IngestError>
{
    return Ok(store.Put_Blob(content)?);
}

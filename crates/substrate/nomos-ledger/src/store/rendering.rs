//! The document as it goes to disk, stamped with the schema this build writes.

use crate::LedgerDocument;
use crate::LedgerError;

use super::SCHEMA_VERSION;

/// The document as it goes to disk, stamped with the schema version this build writes.
///
/// Stamped here rather than taken from the document read in: a file that keeps whatever
/// version it arrived with is a file a build without a field can rewrite while still
/// claiming to speak the newer schema.
pub(super) fn Rendered(document: &LedgerDocument) -> Result<String, LedgerError>
{
    let stamped = LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items: document.items.clone(),
    };

    let mut rendered =
        serde_json::to_string_pretty(&stamped).map_err(|error| LedgerError::Unreadable {
            cause: error.to_string(),
        })?;
    rendered.push('\n');

    return Ok(rendered);
}

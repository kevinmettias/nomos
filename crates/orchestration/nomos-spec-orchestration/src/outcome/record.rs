//! What `nomos spec record` produced, or why it did not.

mod record_refusal;

pub use record_refusal::RecordRefusal;

use nomos_spec_store::DocumentSource;

/// The one document behind an identifier, resolved.
///
/// Content goes out verbatim: [`RecordAnswer::document`] is the source exactly as it was
/// ingested, byte for byte, which is `record`'s whole point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordAnswer
{
    /// The identifier that was asked about.
    pub id: String,
    /// The document it resolved to.
    pub document: DocumentSource,
}

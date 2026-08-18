//! Resolving `nomos spec markdown`'s request against an assembled store.

use nomos_spec_store::{EditError, RecordProjection};

use crate::corpus::Assembly;
use crate::request::RecordRequest;

/// `request.id` rendered from the store's own rows, or why it could not be.
///
/// A thin wrapper around [`nomos_spec_store::SpecificationStore::Record_Markdown`] rather
/// than new logic: that method already returns the typed answer this verb needs, and a
/// second type here would only restate it. Moved from
/// `nomos-cli::spec::verb::markdown::Markdown`, minus the writing and the
/// [`RecordProjection::Matches_Source`] check, which is a rendering decision (`Ok` versus
/// `Stale`) rather than a resolution one.
///
/// # Errors
///
/// Returns [`EditError`] naming why no single record answered -- see
/// [`nomos_spec_store::SpecificationStore::Record_Markdown`].
pub fn Markdown(assembly: &Assembly, request: &RecordRequest) -> Result<RecordProjection, EditError>
{
    let revision = request.revision.as_deref();

    return assembly.store.Record_Markdown(&request.id, revision);
}

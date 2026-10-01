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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Ingest_Blob_Should_Address_Identical_Content_By_The_Same_Identity()
    {
        let mut store = SpecificationStore::In_Memory()
            .expect("In_Memory migrates a fresh database, so no file or prior schema is involved");

        let first = Ingest_Blob(&mut store, b"hello")
            .expect("Put_Blob stores any byte slice, so this content is written and its identity returned");
        let second = Ingest_Blob(&mut store, b"hello")
            .expect("the first call stored these same bytes, so this one resolves to that identity");

        assert_eq!(first, second, "identical content must resolve to one blob identity");
        assert_eq!(
            store.Count(nomos_spec_store::Table::Blobs).expect("counts"),
            1,
            "content-addressing must not duplicate an unchanged blob"
        );
    }
}

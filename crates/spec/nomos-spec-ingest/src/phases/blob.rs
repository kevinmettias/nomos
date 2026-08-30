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
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let first = Ingest_Blob(&mut store, b"hello").expect("ingests");
        let second = Ingest_Blob(&mut store, b"hello").expect("ingests");

        assert_eq!(first, second, "identical content must resolve to one blob identity");
        assert_eq!(
            store.Count(nomos_spec_store::Table::Blobs).expect("counts"),
            1,
            "content-addressing must not duplicate an unchanged blob"
        );
    }
}

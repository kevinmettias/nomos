// What a fact is addressed, stored and read by, less the five names declared at the crate
// root: `lib.rs` says why there. What is left is the vocabulary whose published name does
// not repeat the folder it sits in -- GuaranteeDigest, MaterializedFact and MemoryFactStore
// -- together with the payload itself, which stays at this level because
// `fact/fact_payload.rs` would repeat its parent.
#[path = "fact/guarantee_digest.rs"] mod guarantee_digest;
#[path = "fact/materialized_fact.rs"] mod materialized;
#[path = "fact/memory_fact_store.rs"] mod memory_store;

pub use guarantee_digest::GuaranteeDigest;
pub use materialized::MaterializedFact;
pub use memory_store::MemoryFactStore;

use nomos_contracts::{Digest128, SchemaId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactPayload
{
    pub schema: SchemaId,
    pub bytes: Vec<u8>,
}

impl FactPayload
{
    #[must_use]
    pub fn New(schema: SchemaId, bytes: Vec<u8>) -> Self
    {
        return Self { schema, bytes };
    }

    #[must_use]
    pub fn Digest(&self) -> Digest128
    {
        use nomos_model::Content_Digest;

        return Content_Digest(&self.bytes);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_New_Should_Store_The_Schema_And_Bytes_Given()
    {
        let payload = FactPayload::New(SchemaId::New("nomos.test.payload.v1"), b"hello".to_vec());

        assert_eq!(payload.schema, SchemaId::New("nomos.test.payload.v1"));
        assert_eq!(payload.bytes, b"hello".to_vec());
    }

    #[test]
    fn Test_Digest_Should_Differ_For_Different_Bytes()
    {
        let one = FactPayload::New(SchemaId::New("nomos.test.payload.v1"), b"hello".to_vec());
        let other = FactPayload::New(SchemaId::New("nomos.test.payload.v1"), b"world".to_vec());

        assert_ne!(one.Digest(), other.Digest());
    }
}

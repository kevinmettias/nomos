// Everything a fact is addressed, stored and read by. Eight files whose only relationship
// was a shared name prefix now say it in the tree instead.
#[path = "fact/error.rs"] mod error;
#[path = "fact/guarantee_digest.rs"] mod guarantee_digest;
#[path = "fact/identity.rs"] mod identity;
#[path = "fact/key.rs"] mod key;
#[path = "fact/materialized_fact.rs"] mod materialized;
#[path = "fact/memory_fact_store.rs"] mod memory_store;
#[path = "fact/reader.rs"] mod reader;
#[path = "fact/store.rs"] mod store;

pub use error::Error as FactError;
pub use guarantee_digest::GuaranteeDigest;
pub use identity::Identity as FactIdentity;
pub use key::Key as FactKey;
pub use materialized::MaterializedFact;
pub use memory_store::MemoryFactStore;
pub use reader::Reader as FactReader;
pub use store::Store as FactStore;
pub(crate) use store::sealed;

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

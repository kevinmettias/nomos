// Everything a fact is addressed, stored and read by. Eight files whose only relationship
// was a shared name prefix now say it in the tree instead.
mod error;
mod guarantee_digest;
mod identity;
mod key;
mod materialized;
mod memory_store;
mod reader;
mod store;

pub use error::FactError;
pub use guarantee_digest::GuaranteeDigest;
pub use identity::FactIdentity;
pub use key::FactKey;
pub use materialized::MaterializedFact;
pub use memory_store::MemoryFactStore;
pub use reader::FactReader;
pub use store::FactStore;
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

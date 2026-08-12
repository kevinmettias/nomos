//! A fact key together with the inputs and guarantee that produced it.

use nomos_contracts::Digest128;
use nomos_contracts::GenerationId;
use crate::FactKey;
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FactIdentity
{
    pub key: FactKey,
    pub generation: GenerationId,
}

impl FactIdentity
{
    #[must_use]
    pub const fn Key(&self) -> &FactKey
    {
        return &self.key;
    }

    #[must_use]
    pub fn Digest(&self) -> Digest128
    {
        return self.key.Digest();
    }
}

impl core::fmt::Display for FactIdentity
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(
            formatter,
            "{}@{} of {} by {} at {}",
            self.key.contract, self.key.contract_version, self.key.subject, self.key.provider,
            self.generation
        );
    }
}

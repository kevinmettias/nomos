//! The identity of a document, which is the identity of its bytes.

use nomos_contracts::Digest128;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentId(Digest128);

impl DocumentId
{
    #[must_use]
    pub const fn From_Digest(digest: Digest128) -> Self
    {
        return Self(digest);
    }

    #[must_use]
    pub const fn Digest(self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for DocumentId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return self.0.fmt(formatter);
    }
}

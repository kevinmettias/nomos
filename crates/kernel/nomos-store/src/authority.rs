//! Who gets to say what a document holds.

use serde::{Deserialize, Serialize};

use crate::DocumentKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Authority
{
    Observed,
    Authored,
}

impl Authority
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Observed => "observed",
            Self::Authored => "authored",
        };
    }

    #[must_use]
    pub const fn Can_Admit(self, kind: DocumentKind) -> bool
    {
        return kind.Authority() as u8 == self as u8;
    }
}

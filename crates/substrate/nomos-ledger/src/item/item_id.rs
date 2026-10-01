//! What a work item is called.

use serde::Deserialize;
use serde::Serialize;
/// A ledger item's stable identifier.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemId(String);

impl ItemId
{
    /// Wraps an authored identifier.
    #[must_use]
    pub fn New(value: impl Into<String>) -> Self
    {
        return Self(value.into());
    }

    /// The identifier as authored.
    #[must_use]
    pub fn As_Text(&self) -> &str
    {
        return &self.0;
    }
}

impl core::fmt::Display for ItemId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        // `pad`, not `write_str`. Writing to the formatter directly discards the width
        // and alignment the caller asked for, so `{:<10}` silently does nothing and a
        // listing that was supposed to be columns comes out ragged.
        return formatter.pad(&self.0);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_New_Should_Wrap_Whatever_Identifier_Was_Authored()
    {
        assert_eq!(ItemId::New("P1-MODEL").As_Text(), "P1-MODEL");
        assert_eq!(ItemId::New("P1-MODEL".to_owned()).As_Text(), "P1-MODEL");
    }

    #[test]
    fn Test_As_Text_Should_Return_The_Identifier_As_Authored()
    {
        let id = ItemId::New("P1-MODEL");

        assert_eq!(id.As_Text(), "P1-MODEL");
    }
}

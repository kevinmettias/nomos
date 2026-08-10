//! The shared shape every syntax provider emits.

use crate::payload_item::PayloadItem;
/// A decoded `nomos.syntax.items.v2` payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxPayload
{
    /// The lower bound the provider offered on unexpanded regions.
    pub unexpanded: u32,
    /// The items the file declares, in source order.
    pub items: Vec<PayloadItem>,
}

impl SyntaxPayload
{
    /// The record that syntactically encloses the item at `ordinal`, if any.
    ///
    /// The most recent preceding record whose qualified name is this one's prefix. Source
    /// order is what makes this answerable: an `All` declared in `impl Display for Table`
    /// and one declared in `impl Table` carry the same qualified name, and only the record
    /// they follow tells them apart.
    #[must_use]
    pub fn Enclosing(&self, ordinal: usize) -> Option<&PayloadItem>
    {
        let item = self.items.get(ordinal)?;
        let (owner, _) = item.qualified_name.rsplit_once("::")?;

        return self
            .items
            .get(..ordinal)?
            .iter()
            .rev()
            .find(|candidate| return candidate.qualified_name == owner);
    }
}

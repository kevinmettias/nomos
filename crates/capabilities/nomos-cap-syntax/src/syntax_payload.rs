//! The shared shape every syntax provider emits.

use crate::PayloadItem;
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Observation;

    /// The position of the `Other` fixture item, the one whose name carries no `::`.
    const OTHER_ITEM_INDEX: usize = 2;

    fn Item(qualified_name: &str) -> PayloadItem
    {
        return PayloadItem {
            ordinal: 0,
            kind: "Function".to_owned(),
            visibility: "Public".to_owned(),
            qualified_name: qualified_name.to_owned(),
            documentation: Observation::Absent,
            shape: Observation::Absent,
        };
    }

    #[test]
    fn Test_Enclosing_Should_Find_The_Most_Recent_Record_Whose_Name_Prefixes_This_Ones()
    {
        let payload = SyntaxPayload {
            unexpanded: 0,
            items: vec![Item("Table"), Item("Table::All"), Item("Other")],
        };

        assert_eq!(
            payload.Enclosing(1).map(|item| return item.qualified_name.as_str()),
            Some("Table")
        );
        assert!(payload.Enclosing(0).is_none(), "a top-level item encloses nothing");
        assert!(payload.Enclosing(OTHER_ITEM_INDEX).is_none(), "`Other` has no `::` to look up an owner from");
    }
}

//! Everything one file was parsed into.

use crate::Item;
/// Everything one file says on its face.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Facts
{
    /// Items in source order.
    pub items: Vec<Item>,
    /// Places where the parse tree ends and an unexpanded token stream begins.
    ///
    /// A **lower bound**, and the reason completeness is
    /// [`nomos_contracts::Assurance::Unknown`]. Macro invocations and `derive`
    /// attributes are counted because they are syntactically identifiable; an attribute
    /// macro like `#[tokio::main]` is not, because telling it from `#[allow]` requires
    /// resolving the path — the thing this provider does not do.
    ///
    /// Reported rather than hidden so that a caller reading "3 items" can see whether
    /// the file also had 40 places those items could have been generated from.
    pub unexpanded: u32,
}

impl Facts
{
    /// Whether the file declared nothing at all.
    ///
    /// A real answer for a file that is empty or entirely comments, and never the answer
    /// for a file that failed to parse — that is [`Reading::Unparseable`], a different
    /// variant reached by a different path.
    #[must_use]
    pub fn Has_No_Declarations(&self) -> bool
    {
        return self.items.is_empty();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Item;
    use crate::ItemKind;
    use crate::Visibility;

    #[test]
    fn Test_Has_No_Declarations_Should_Be_True_For_A_Facts_With_No_Items()
    {
        assert!(Facts::default().Has_No_Declarations());
    }

    #[test]
    fn Test_Has_No_Declarations_Should_Be_False_Once_An_Item_Is_Recorded()
    {
        let facts = Facts {
            items: vec![Item {
                ordinal: 0,
                kind: ItemKind::Function,
                scope: Vec::new(),
                name: "One".to_owned(),
                visibility: Visibility::Public,
                documentation: None,
                shape: None,
            }],
            unexpanded: 0,
        };

        assert!(!facts.Has_No_Declarations());
    }
}

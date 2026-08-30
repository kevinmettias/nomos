//! Everything one file was parsed into.

use crate::Item;

/// Everything one file says on its face.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Facts
{
    /// Items in source order.
    pub items: Vec<Item>,
    /// Always `0`.
    ///
    /// `nomos-lang-rust` uses this field for a lower bound on macro invocations, and reports
    /// completeness as [`nomos_contracts::Assurance::Unknown`] because that bound cannot be
    /// tightened into a real count. Go has no macro system — no construct where the parse
    /// tree ends and an unexpanded token stream begins in its place — so there is nothing
    /// this field could honestly report a lower bound on. It stays in the type because the
    /// payload schema's header is `unexpanded` for every provider of this capability, not
    /// because this provider has anything to count into it. See [`crate::Declared_Guarantee`]
    /// for the completeness claim this field's permanent zero makes possible.
    pub unexpanded: u32,
}

impl Facts
{
    /// Whether the file declared nothing at all.
    ///
    /// A real answer for a file that is empty or entirely comments, and never the answer
    /// for a file that failed to parse — that is [`crate::Reading::Unparseable`], a
    /// different variant reached by a different path.
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

//! Everything one file was parsed into.

use crate::Item;

/// Everything one file says on its face.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Facts
{
    /// Items in source order.
    pub items: Vec<Item>,
    /// How many preprocessor conditional regions this reading declined to enter.
    ///
    /// The payload schema's header field, and this provider has something real to put in it.
    /// `nomos-lang-rust` reports a lower bound on macro invocations here and declares
    /// completeness [`nomos_contracts::Assurance::Unknown`] because that bound cannot be
    /// tightened into a count of what those macros generated; `nomos-lang-go` reports a
    /// permanent zero because Go has no such construct at all. C# has one — a `#if`/`#elif`/
    /// `#else` chain, whose branches this reading does not enter because at most one of them
    /// is in any compilation and nothing here knows which. So this is the same kind of number
    /// the Rust provider writes: how many places declarations were not read from, not how many
    /// declarations were missed. See [`crate::Declared_Guarantee`] for the completeness claim
    /// it forces.
    pub unexpanded: u32,
}

impl Facts
{
    /// Whether the file declared nothing at all.
    ///
    /// A real answer for a file that is empty or entirely comments, and never the answer for a
    /// file that failed to parse — that is [`crate::Reading::Unparseable`], a different variant
    /// reached by a different path.
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
                kind: ItemKind::Class,
                scope: Vec::new(),
                name: "Widget".to_owned(),
                visibility: Visibility::Public,
                documentation: None,
                shape: None,
            }],
            unexpanded: 0,
        };

        assert!(!facts.Has_No_Declarations());
    }
}

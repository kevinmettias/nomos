//! One consequence of applying a change.

// What kind of effect this is, beneath the effect it describes.
#[path = "effect/kind.rs"]
mod kind;

pub use kind::Kind as EffectKind;

/// What one change actually did, and to what.
///
/// Reported rather than assumed, because the submitter did not know. A checkout does not
/// diff before it lands and an editor's save hook does not consult the previous
/// generation, so [`Change::Present`] is a statement about the desired end state. This is
/// the answer to what it turned out to be.
///
/// The path is the type's and not the kind's. Every consequence is a consequence *to a
/// member*, so a caller reads `path` without matching on an outcome it does not otherwise
/// care about, and a new outcome cannot forget to say which member it happened to.
///
/// `path` is declared first so that ordering a set of effects orders it by member. The
/// order effects were applied in is not a fact about the workspace — it is a fact about
/// how the change set was iterated — and sorting by outcome would group by exactly the
/// thing the caller is usually reading past.
///
/// [`Change::Present`]: crate::Change::Present
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Effect
{
    pub path: String,
    pub kind: EffectKind,
}

impl Effect
{
    #[must_use]
    pub fn Path(&self) -> &str
    {
        return &self.path;
    }

    /// Whether this effect changed what the workspace is.
    #[must_use]
    pub const fn Is_Altered(&self) -> bool
    {
        return self.kind.Is_Altered();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Path_Should_Return_The_Effects_Own_Path()
    {
        let effect = Effect {
            path: "src/a.rs".to_owned(),
            kind: EffectKind::Added,
        };

        assert_eq!(effect.Path(), "src/a.rs");
    }

    #[test]
    fn Test_Is_Altered_Should_Reflect_Whether_The_Kind_Changed_The_Workspace()
    {
        let added = Effect {
            path: "a.rs".to_owned(),
            kind: EffectKind::Added,
        };
        let redundant = Effect {
            path: "a.rs".to_owned(),
            kind: EffectKind::Redundant,
        };

        assert!(added.Is_Altered());
        assert!(!redundant.Is_Altered());
    }
}

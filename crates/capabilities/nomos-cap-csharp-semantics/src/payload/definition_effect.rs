//! [`DefinitionEffect`], what a file-level `#define` or `#undef` does.

/// What one file-level definition directive does to a symbol from its line onward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionEffect
{
    /// `#define SYMBOL`.
    Define,
    /// `#undef SYMBOL`.
    Undefine,
}

impl DefinitionEffect
{
    /// This effect's canonical label, as the payload spells it: the directive's own name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Define => "define",
            Self::Undefine => "undef",
        };
    }

    /// The effect [`Self::Label`] spelled `label`, or `None` for any other string.
    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            "define" => Some(Self::Define),
            "undef" => Some(Self::Undefine),
            _ => None,
        };
    }
}

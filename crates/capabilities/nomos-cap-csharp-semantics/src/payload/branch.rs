//! [`Branch`], which directive opens a conditional region.

/// The directive that opens one branch of a conditional chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch
{
    /// `#if condition`, which opens a chain.
    If,
    /// `#elif condition`, taken only when no earlier branch of its chain was.
    Elif,
    /// `#else`, taken only when no earlier branch of its chain was.
    Else,
}

impl Branch
{
    /// This branch's canonical label, as the payload spells it.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::If => "if",
            Self::Elif => "elif",
            Self::Else => "else",
        };
    }

    /// The branch [`Self::Label`] spelled `label`, or `None` for any other string.
    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            "if" => Some(Self::If),
            "elif" => Some(Self::Elif),
            "else" => Some(Self::Else),
            _ => None,
        };
    }
}

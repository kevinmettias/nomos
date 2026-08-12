//! How badly an open decision gap bites.

/// How badly an open decision gap bites.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity
{
    /// Prevents acceptance.
    Blocking,
    /// Does not prevent acceptance, and survives into the design and the result.
    NonBlocking,
}

impl Severity
{
    /// The label this severity is stored under.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Blocking => "blocking",
            Self::NonBlocking => "non-blocking",
        };
    }

    /// The severity a label names, if it names one.
    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return match label
        {
            "blocking" => Some(Self::Blocking),
            "non-blocking" => Some(Self::NonBlocking),
            _ => None,
        };
    }
}

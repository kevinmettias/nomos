//! [`Directionality`], which way a metric gets worse.

/// MET-006's "directionality": which direction of change is a regression, so a reader
/// comparing two values cannot read an improvement as a worsening.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Directionality
{
    /// A larger value is worse.
    HigherIsWorse,
    /// A smaller value is worse.
    LowerIsWorse,
}

impl Directionality
{
    /// This direction's canonical label, as the payload spells it.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::HigherIsWorse => "higher-is-worse",
            Self::LowerIsWorse => "lower-is-worse",
        };
    }

    /// The direction [`Self::Label`] spelled `label`, or `None` for any other string.
    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            "higher-is-worse" => Some(Self::HigherIsWorse),
            "lower-is-worse" => Some(Self::LowerIsWorse),
            _ => None,
        };
    }
}

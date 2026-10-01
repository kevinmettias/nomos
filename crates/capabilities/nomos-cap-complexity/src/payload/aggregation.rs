//! [`Aggregation`], how a metric may be combined across subjects.

/// MET-006's "aggregation operator": whether a metric may be combined across its subjects, and
/// how. MET-007 binds every reader to it -- "A client may not aggregate or blend a metric beyond
/// the descriptor’s declared semantics".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aggregation
{
    /// Not combinable at all: the value stays at its native granularity. A sum of two
    /// functions' complexities is not the complexity of anything.
    NotAggregable,
    /// Combinable by adding.
    Sum,
    /// Combinable by taking the largest.
    Maximum,
}

impl Aggregation
{
    /// This operator's canonical label, as the payload spells it.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::NotAggregable => "not-aggregable",
            Self::Sum => "sum",
            Self::Maximum => "maximum",
        };
    }

    /// The operator [`Self::Label`] spelled `label`, or `None` for any other string.
    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            "not-aggregable" => Some(Self::NotAggregable),
            "sum" => Some(Self::Sum),
            "maximum" => Some(Self::Maximum),
            _ => None,
        };
    }
}

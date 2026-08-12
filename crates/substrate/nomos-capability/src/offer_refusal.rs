//! Why one provider's offer will not stand.

/// Why one provider's offer will not stand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferRefusal
{
    /// An offer against a capability no contract declares.
    ForUndeclared,
    /// A provider claimed more than its contract permits.
    ExceedsCeiling,
    /// A second offer from a provider that already offers this capability.
    Duplicate,
}

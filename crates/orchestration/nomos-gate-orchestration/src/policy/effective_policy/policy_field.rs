//! Which field of a gate policy a contribution, a refusal or a resolution is about.

/// One field of the gate policy, named the way `nomos-gate.json` names it.
///
/// A closed vocabulary rather than a string, because every refusal `OD-POLICY-001` decides
/// has to name the offending key and a key nobody can mistype is the only kind a reader can
/// match a file against. The spelling a [`Self::Label`] returns is the JSON key an author
/// actually wrote, so a refusal points at the line rather than at a Rust identifier.
///
/// Deliberately carries no `ALL`: nothing quantifies over the whole field set -- each shape
/// group is resolved by its own function, named for the shape -- and a list nothing reads
/// would be a declared universe owing a mirror for no reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyField
{
    Suppressions,
    Baseline,
    Adoption,
    Coverage,
    Phases,
    Approvals,
}

impl PolicyField
{
    /// The key an author writes in `nomos-gate.json` for this field.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Suppressions => "suppressions",
            Self::Baseline => "baseline",
            Self::Adoption => "adoption",
            Self::Coverage => "coverage",
            Self::Phases => "phases",
            Self::Approvals => "approvals",
        };
    }
}

impl std::fmt::Display for PolicyField
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

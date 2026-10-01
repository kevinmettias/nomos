//! Whether a rule's own judgment is ever partial by design.

/// The shape of `Applicability` states a rule's own judgment implementation may
/// legitimately raise for a genuine violation.
///
/// `ARCH-002`'s "applicability semantics" field. `OD-PACKAGE-008`'s four-rule
/// measurement found exactly two real shapes: three of the four shipped rules always
/// raise [`nomos_contracts::Applicability::Supported`] for a genuine violation, and one —
/// `Check_Unread_Reaches_A_Finding` — structurally never does, because its tier-1
/// provider evaluates only part of its question and reports that honestly rather than
/// rounding up. Two variants, not the full eleven-state `Applicability` enum: a manifest
/// declares which of these two shapes a rule's own implementation follows, not which
/// individual coverage-debt state a specific run happened to hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApplicabilitySemantics
{
    /// A genuine violation is always reported as
    /// [`nomos_contracts::Applicability::Supported`].
    AlwaysSupported,
    /// The rule's own provider evaluates only part of its subject by design, so every
    /// finding it raises carries
    /// [`nomos_contracts::Applicability::PartiallySupported`].
    StructurallyPartial,
}

impl ApplicabilitySemantics
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::AlwaysSupported => "always_supported",
            Self::StructurallyPartial => "structurally_partial",
        };
    }
}

impl core::fmt::Display for ApplicabilitySemantics
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        assert_ne!(
            ApplicabilitySemantics::AlwaysSupported.Label(),
            ApplicabilitySemantics::StructurallyPartial.Label()
        );
    }
}

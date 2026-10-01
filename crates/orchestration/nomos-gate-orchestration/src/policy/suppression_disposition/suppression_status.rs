//! Whether a disposition still applies at the moment a run is judged.

/// Whether a [`super::Suppression`] still applies at a given moment.
///
/// # Why an enum and not a boolean
///
/// Because "does not apply" is already more than one thing, and only one of them exists
/// today. [`Self::Expired`] is a *known policy condition*: somebody wrote an end date, the
/// run is at or past it, and the right report is that a waiver lapsed rather than that a
/// finding was never waived. A later state — a disposition stale because the rule it names
/// changed version, because the subject's identity moved, because the evidence it rested on
/// was invalidated, or because its scope no longer covers the finding — is a different claim
/// with a different remedy, and collapsing it into the same `false` would lose the
/// distinction at the only place a reader could see it.
///
/// None of those is produced yet, and none is added here. A variant with nothing to construct
/// it is the invented shape this workspace declines everywhere else; the enum is what leaves
/// room for it without pretending it exists. `OD-GATE-001`'s principle applies to the
/// vocabulary as much as to a result: a state that cannot occur must not be offered as one
/// that did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppressionStatus
{
    /// The disposition applies: it has no end date, or the run is before it.
    Active,
    /// The disposition named an end date and the run is at or past it.
    ///
    /// At, not only past. A waiver written as good until an instant is not good at that
    /// instant, the same way a deadline is not met by arriving on it.
    Expired,
}

impl SuppressionStatus
{
    /// Whether a run should honor the disposition this status describes.
    #[must_use]
    pub const fn Is_Suppressing(self) -> bool
    {
        return matches!(self, Self::Active);
    }
}

#[cfg(test)]
mod tests
{
    use super::SuppressionStatus;

    #[test]
    fn Test_Only_Active_Should_Suppress()
    {
        assert!(SuppressionStatus::Active.Is_Suppressing());
        assert!(!SuppressionStatus::Expired.Is_Suppressing());
    }
}

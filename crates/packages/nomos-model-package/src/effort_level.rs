//! The canonical reasoning-effort vocabulary every model backend maps onto.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-004`'s canonical effort values: "Canonical effort values shall be
/// BackendDefault, Minimal, Low, Medium, High, and Maximum."
///
/// Transcribed directly, the same `RustEdition`-shaped test `OD-PACKAGE-011` names for
/// when resolving a placeholder domain is licensed: the requirement itself closes the
/// enumeration, so nothing here is invented past what the corpus states. What each value
/// means for a *specific* backend is deliberately not this type's question --
/// `MODEL-ROUTE-004`'s own second and third sentences ("Backend packages shall map
/// supported values to native controls, reject unsupported values, or expose an
/// explicit approximation... Nomos shall never imply that effort levels are
/// quantitatively equivalent across providers or model families") are a mapping and a
/// non-equivalence rule for a real backend to carry out, not a fact this enum could
/// state about itself.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffortLevel
{
    /// Whatever the backend does when nothing is asked of it.
    BackendDefault,
    Minimal,
    Low,
    Medium,
    High,
    Maximum,
}

impl EffortLevel
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::BackendDefault => "BackendDefault",
            Self::Minimal => "Minimal",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Maximum => "Maximum",
        };
    }
}

impl core::fmt::Display for EffortLevel
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

    const ALL: [EffortLevel; 6] = [
        EffortLevel::BackendDefault,
        EffortLevel::Minimal,
        EffortLevel::Low,
        EffortLevel::Medium,
        EffortLevel::High,
        EffortLevel::Maximum,
    ];

    /// `MODEL-ROUTE-004` names exactly six values. A match with no wildcard is the guard
    /// that stops a seventh from being added silently.
    #[test]
    fn Test_Every_Value_Is_In_The_Tested_Universe()
    {
        for value in ALL
        {
            match value
            {
                EffortLevel::BackendDefault
                | EffortLevel::Minimal
                | EffortLevel::Low
                | EffortLevel::Medium
                | EffortLevel::High
                | EffortLevel::Maximum =>
                {}
            }
        }
    }

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|value| return value.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two effort levels share a label");
    }
}

//! How reproducible a model-assisted operation's output is expected to be.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-031`: "Every resolved and completed model-assisted operation shall
/// record an `OutputDeterminismExpectation` independently from
/// `RoutingReplayDisposition`. Canonical values shall be `Deterministic`,
/// `SeedControlled`, `ProviderBestEffort`, `ExecutorVariable`, `HumanVariable`, and
/// `Unknown`, with basis, controls, and known variability sources."
///
/// The same "canonical values shall be" closing pattern that licensed
/// [`crate::EffortLevel`] for `MODEL-ROUTE-004`. Deliberately a distinct type from
/// [`crate::RoutingReplayDisposition`] -- the corpus's own "independently from" says
/// so directly: replay disposition answers whether a *routing decision* can be
/// reproduced, this answers whether the *model's output* can be, and neither implies
/// the other.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputDeterminismExpectation
{
    pub value: OutputDeterminismValue,
    pub basis: String,
    pub controls: Vec<String>,
    pub known_variability_sources: Vec<String>,
}

/// `MODEL-ROUTE-031`'s six canonical values, in the corpus's own order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputDeterminismValue
{
    Deterministic,
    SeedControlled,
    ProviderBestEffort,
    ExecutorVariable,
    HumanVariable,
    Unknown,
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL: [OutputDeterminismValue; 6] = [
        OutputDeterminismValue::Deterministic,
        OutputDeterminismValue::SeedControlled,
        OutputDeterminismValue::ProviderBestEffort,
        OutputDeterminismValue::ExecutorVariable,
        OutputDeterminismValue::HumanVariable,
        OutputDeterminismValue::Unknown,
    ];

    /// `MODEL-ROUTE-031` names exactly six values. A match with no wildcard is the
    /// guard that stops a seventh from being added silently, the same idiom
    /// `EffortLevel`'s own exhaustive-universe test already uses.
    #[test]
    fn Test_Every_Value_Is_In_The_Tested_Universe()
    {
        for value in ALL
        {
            match value
            {
                OutputDeterminismValue::Deterministic
                | OutputDeterminismValue::SeedControlled
                | OutputDeterminismValue::ProviderBestEffort
                | OutputDeterminismValue::ExecutorVariable
                | OutputDeterminismValue::HumanVariable
                | OutputDeterminismValue::Unknown =>
                {}
            }
        }
    }

    #[test]
    fn Test_An_Expectation_Carries_Exactly_What_It_Was_Given()
    {
        let expectation = OutputDeterminismExpectation {
            value: OutputDeterminismValue::SeedControlled,
            basis: "provider documents a fixed seed parameter".to_owned(),
            controls: vec!["seed".to_owned()],
            known_variability_sources: vec!["floating-point accumulation order".to_owned()],
        };

        assert_eq!(expectation.value, OutputDeterminismValue::SeedControlled);
        assert_eq!(expectation.controls, ["seed"]);
        assert_eq!(expectation.known_variability_sources, ["floating-point accumulation order"]);
    }
}

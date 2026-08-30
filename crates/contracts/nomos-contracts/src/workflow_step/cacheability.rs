//! Whether a step's result may be reused for a later execution over the same inputs.

use serde::{Deserialize, Serialize};

/// Whether a step's result may be reused for a later execution over the same inputs.
///
/// `Cacheable::key_inputs` names, by raw field name, the subset of the step's own
/// `input_schema` the cache key is derived from -- raw strings rather than a typed
/// reference, the same "first maturity, unresolved sub-shape" choice `OD-PACKAGE-010`
/// made for `ModelSelection::Catalog`'s entries before a consumer existed to resolve
/// them further.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cacheability
{
    /// Every execution is real; a prior result is never substituted.
    NotCacheable,
    /// A prior result may be substituted when every field named here matches.
    Cacheable
    {
        /// Field names within `WorkflowStep::input_schema`'s shape the cache key is
        /// derived from.
        key_inputs: Vec<String>,
    },
}

impl Cacheability
{
    /// Whether a prior result may ever be substituted for a fresh execution.
    #[must_use]
    pub const fn Is_Cacheable(&self) -> bool
    {
        return matches!(self, Self::Cacheable { .. });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Not_Cacheable_Should_Report_So()
    {
        assert!(!Cacheability::NotCacheable.Is_Cacheable());
    }

    #[test]
    fn Test_Is_Cacheable_Should_Be_True_When_Key_Inputs_Are_Declared()
    {
        let cacheable = Cacheability::Cacheable { key_inputs: vec!["path".to_owned()] };

        assert!(cacheable.Is_Cacheable());
    }
}

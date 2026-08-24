//! What dimensions a fallback edge is declared allowed to change.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-014`: "Every fallback edge shall include a `FallbackAdmissibility`
/// record declaring whether it may change model family, backend package, executor
/// type, requested or native effort, data boundary or region, structured-output
/// guarantee, tool permissions, context policy, telemetry completeness, and cost or
/// latency class. A fallback may not make an undeclared change."
///
/// Ten fields, one per named dimension, in the corpus's own order. No fallback-edge
/// graph or routing infrastructure needs to exist first for this record's own shape to
/// be real -- it is a declaration a fallback edge would carry, the same standalone
/// vocabulary layer [`crate::EffortLevel`] and [`crate::ModelSelector`] already
/// established for `MODEL-ROUTE-001`/`003`/`004`.
#[allow(clippy::struct_excessive_bools)] // ten independent named permission dimensions, per MODEL-ROUTE-014's own list -- not a state machine in disguise
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackAdmissibility
{
    pub may_change_model_family: bool,
    pub may_change_backend_package: bool,
    pub may_change_executor_type: bool,
    pub may_change_effort: bool,
    pub may_change_data_boundary_or_region: bool,
    pub may_change_structured_output_guarantee: bool,
    pub may_change_tool_permissions: bool,
    pub may_change_context_policy: bool,
    pub may_change_telemetry_completeness: bool,
    pub may_change_cost_or_latency_class: bool,
}

impl FallbackAdmissibility
{
    /// `MODEL-ROUTE-014`: "A fallback may not make an undeclared change." `self`
    /// permits `changed` when every dimension `changed` marks `true` is also `true`
    /// here -- a dimension `changed` marks `false` says nothing about what actually
    /// happened, so it never blocks permission by itself.
    #[must_use]
    pub const fn Permits(&self, changed: &Self) -> bool
    {
        return (!changed.may_change_model_family || self.may_change_model_family)
            && (!changed.may_change_backend_package || self.may_change_backend_package)
            && (!changed.may_change_executor_type || self.may_change_executor_type)
            && (!changed.may_change_effort || self.may_change_effort)
            && (!changed.may_change_data_boundary_or_region || self.may_change_data_boundary_or_region)
            && (!changed.may_change_structured_output_guarantee || self.may_change_structured_output_guarantee)
            && (!changed.may_change_tool_permissions || self.may_change_tool_permissions)
            && (!changed.may_change_context_policy || self.may_change_context_policy)
            && (!changed.may_change_telemetry_completeness || self.may_change_telemetry_completeness)
            && (!changed.may_change_cost_or_latency_class || self.may_change_cost_or_latency_class);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_An_Undeclared_Change_Should_Not_Be_Permitted()
    {
        let declared = FallbackAdmissibility::default();
        let attempted = FallbackAdmissibility {
            may_change_effort: true,
            ..FallbackAdmissibility::default()
        };

        assert!(!declared.Permits(&attempted));
    }

    #[test]
    fn Test_A_Declared_Change_Should_Be_Permitted()
    {
        let declared = FallbackAdmissibility {
            may_change_effort: true,
            ..FallbackAdmissibility::default()
        };
        let attempted = FallbackAdmissibility {
            may_change_effort: true,
            ..FallbackAdmissibility::default()
        };

        assert!(declared.Permits(&attempted));
    }

    #[test]
    fn Test_No_Change_Should_Always_Be_Permitted()
    {
        let declared = FallbackAdmissibility::default();
        let no_change = FallbackAdmissibility::default();

        assert!(declared.Permits(&no_change));
    }
}

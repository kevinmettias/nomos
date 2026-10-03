//! The arity engine's configuration, carrying its own builders.
//!
//! Split from `function_shape` with every `impl` block that belongs to it, which is the
//! condition on a move like this one: a type whose functions end up in a module its
//! `pub use` route does not name loses them from `tests/contract/surface` silently, and the
//! diff that would show it is the blessed snapshot rather than a compiler error.

use super::ReceiverAllowance;
use crate::rule_descriptor::Population;
use nomos_contracts::GateCategory;

/// A configurable function-arity rule.
///
/// No `PartialEq`: [`Self::source`] is a [`Population`], which can hold a function pointer, and
/// comparing two of those compares addresses the compiler is free to merge or duplicate.
/// Nothing compares two policies.
#[derive(Clone, Copy, Debug)]
pub struct FunctionArityPolicy
{
    /// The rule id reported on findings.
    pub rule: &'static str,
    /// Which of the sources handed to the rule this policy judges: the population the rule's
    /// own descriptor declares, so the policy and the descriptor are one statement.
    pub source: Population,
    /// Maximum value parameters allowed before a finding is reported.
    pub max_value_parameters: u32,
    /// Whether qualified functions get one possible receiver input.
    pub receiver_allowance: ReceiverAllowance,
    /// The gate category reported for supported findings.
    pub gate: GateCategory,
}

impl FunctionArityPolicy
{
    /// Builds a policy for every source with no receiver allowance.
    #[must_use]
    pub const fn New(rule: &'static str, max_value_parameters: u32) -> Self
    {
        return Self {
            rule,
            source: Population::Every,
            max_value_parameters,
            receiver_allowance: ReceiverAllowance::None,
            gate: GateCategory::Blocking,
        };
    }

    /// Narrows this policy to `population` -- the constant its rule's descriptor declares.
    #[must_use]
    pub const fn Judging(mut self, population: Population) -> Self
    {
        self.source = population;
        return self;
    }

    /// Allows one possible receiver for qualified functions.
    #[must_use]
    pub const fn Allow_One_Receiver_For_Qualified_Functions(mut self) -> Self
    {
        self.receiver_allowance = ReceiverAllowance::OneForQualifiedFunctions;
        return self;
    }

    /// Changes the gate category reported by supported findings.
    #[must_use]
    pub const fn With_Gate(mut self, gate: GateCategory) -> Self
    {
        self.gate = gate;
        return self;
    }
}

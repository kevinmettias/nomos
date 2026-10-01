//! What an agent-assisted operation should use, before any resolution against a live
//! provider happens.

use crate::{EffortLevel, ModelSelector};

/// `MODEL-ROUTE-001`'s referenceable profile: "Gates, phases, workflows, rules/checks,
/// and agent task classes shall be able to reference a `ModelExecutionProfile` for every
/// agent-assisted operation. The smallest explicit assignment shall override inherited
/// defaults subject to organization and repository policy constraints."
///
/// This type is the profile itself -- what the requirement says an operation
/// *references* -- not the five reference sites or the override-precedence resolver.
/// Two of the five referencing surfaces have a real type that a reference field could be
/// added to today (`nomos_contracts::RuleId` for "rules/checks",
/// `nomos_gate_orchestration::GateCommand` for "gates" -- neither carries one yet);
/// "phases", "workflows" and "agent task classes" have no structured type anywhere in
/// this workspace to reference this profile from at all (`OD-WORKFLOW-002`,
/// `OD-PACKAGE-011`). Building the profile itself does not require those to exist
/// first: a value can be constructed and reasoned about before every place that would
/// eventually hold a reference to one does. Wiring an actual reference field onto
/// `GateCommand`, `RuleId`, or any future phase/workflow/task-class type is a later,
/// separate increment.
///
/// `MODEL-ROUTE-005`'s eleven-scope override-precedence resolver ("Invocation override;
/// exact logical-operation profile; ... backend default") is likewise not attempted
/// here -- most of those eleven scopes name the same missing phase/workflow/task-class
/// types this profile's own reference sites do, so a precedence chain over them would be
/// ordering scopes that do not exist yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelExecutionProfile
{
    /// How this profile chooses which model answers a request.
    pub selector: ModelSelector,
    /// The reasoning effort this profile asks for.
    pub effort: EffortLevel,
}

impl ModelExecutionProfile
{
    /// Constructs a profile.
    #[must_use]
    pub const fn New(selector: ModelSelector, effort: EffortLevel) -> Self
    {
        return Self { selector, effort };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Profiles_With_Equal_Content_Should_Be_Equal()
    {
        let one = ModelExecutionProfile::New(ModelSelector::BackendFamily("acme".to_owned()), EffortLevel::High);
        let other = ModelExecutionProfile::New(ModelSelector::BackendFamily("acme".to_owned()), EffortLevel::High);

        assert_eq!(one, other);
    }

    #[test]
    fn Test_A_Different_Effort_Should_Change_Equality()
    {
        let low = ModelExecutionProfile::New(ModelSelector::BackendFamily("acme".to_owned()), EffortLevel::Low);
        let high = ModelExecutionProfile::New(ModelSelector::BackendFamily("acme".to_owned()), EffortLevel::High);

        assert_ne!(low, high);
    }
}

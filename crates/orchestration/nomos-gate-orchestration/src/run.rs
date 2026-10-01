//! Planning one gate, apart from choosing a platform, selecting scope or rendering the
//! answer.

use crate::{GateCommand, GateOutcome};

/// Composes this gate's rule registry and reports what a real run would judge from it.
///
/// `command.rules` narrows the registry the same way [`crate::Run_Gate`] narrows a real
/// run's findings -- an empty [`crate::RuleSelector::include`] plans every registered rule,
/// the same "select everything" default every existing caller and CI's own `gate plan`
/// already have, so nothing about their behavior changes. `command.root` and `command.scope`
/// are still accepted in full but not yet read: `Plan` reports the registry, not a walk, so
/// neither has a file to narrow against -- see [`crate::gate_plan::GatePlan`]'s own doc for
/// why a rule-only registry has nothing for a path-shaped selector to filter.
#[must_use]
pub fn Run(command: &GateCommand) -> GateOutcome
{
    use crate::Registered;
    use crate::gate_plan::GatePlan;

    let registry = match Registered()
    {
        Ok(registry) => registry,
        Err(error) => return GateOutcome::Contradictory(error),
    };

    let rules = registry.Offers().filter(|offer| return command.rules.Is_Included(&offer.rule)).cloned().collect();

    return GateOutcome::Planned(GatePlan { rules });
}

#[cfg(test)]
mod tests
{
    use super::Run;
    use crate::{GateCommand, GateOutcome, RuleSelector};
    use nomos_contracts::RuleId;
    use std::path::PathBuf;

    /// Two different roots must plan identically: this increment does not select by scope, so
    /// a caller cannot mistake `Run`'s answer for one that reads `command.root`.
    #[test]
    fn Test_Run_Should_Plan_Identically_Regardless_Of_Root()
    {
        let GateOutcome::Planned(here) = Run(&GateCommand { root: PathBuf::from("."), ..Default::default() })
        else
        {
            // this crate's own rule registration is fixed at compile time; a mismatch here
            // is a bug in the registration, not a runtime condition a caller could hit.
            panic!("this crate's own registration must not be contradictory");
        };
        let GateOutcome::Planned(elsewhere) = Run(&GateCommand { root: PathBuf::from("elsewhere"), ..Default::default() })
        else
        {
            // this crate's own rule registration is fixed at compile time; a mismatch here
            // is a bug in the registration, not a runtime condition a caller could hit.
            panic!("this crate's own registration must not be contradictory");
        };

        assert_eq!(here, elsewhere, "root is not read yet, so the plan must not depend on it");
    }

    #[test]
    fn Test_Run_Should_Plan_Every_Rule_When_Rules_Is_Unset()
    {
        let GateOutcome::Planned(every_rule) = Run(&GateCommand::default())
        else
        {
            panic!("this crate's own registration must not be contradictory");
        };
        let GateOutcome::Planned(named_explicitly) = Run(&GateCommand { rules: RuleSelector { include: every_rule.rules.iter().map(|offer| return offer.rule.clone()).collect() }, ..Default::default() })
        else
        {
            panic!("this crate's own registration must not be contradictory");
        };

        assert_eq!(every_rule, named_explicitly, "an empty RuleSelector must plan every rule, the same as naming every rule explicitly");
    }

    #[test]
    fn Test_Run_Should_Plan_Fewer_Rules_When_Rules_Narrows_The_Selection()
    {
        let GateOutcome::Planned(every_rule) = Run(&GateCommand::default())
        else
        {
            panic!("this crate's own registration must not be contradictory");
        };
        let GateOutcome::Planned(narrowed) = Run(&GateCommand { rules: RuleSelector { include: vec![RuleId::New(nomos_rules::NAMING_CONVENTION)] }, ..Default::default() })
        else
        {
            panic!("this crate's own registration must not be contradictory");
        };

        assert_ne!(every_rule, narrowed, "narrowing command.rules must produce a different plan");
        let [only] = narrowed.rules.as_slice()
        else
        {
            panic!("expected exactly one planned rule, got {narrowed:?}");
        };
        assert_eq!(only.rule, RuleId::New(nomos_rules::NAMING_CONVENTION));
    }
}

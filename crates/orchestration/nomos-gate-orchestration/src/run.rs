//! Planning one gate, apart from choosing a platform, selecting scope or rendering the
//! answer.

use crate::{GateCommand, GateOutcome};

/// Composes this gate's rule registry and reports what it holds.
///
/// `command` is accepted in full -- including [`GateCommand::root`], which this increment
/// does not yet read -- so a caller writes against the shape `nomos gate plan` will keep once
/// a later increment gives `root` something to do.
#[must_use]
pub fn Run(_command: &GateCommand) -> GateOutcome
{
    use crate::Registered;
    use crate::gate_plan::GatePlan;

    let registry = match Registered()
    {
        Ok(registry) => registry,
        Err(error) => return GateOutcome::Contradictory(error),
    };

    let rules = registry.Offers().cloned().collect();

    return GateOutcome::Planned(GatePlan { rules });
}

#[cfg(test)]
mod tests
{
    use super::Run;
    use crate::{GateCommand, GateOutcome};
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
}

//! Planning one gate, apart from choosing a platform, selecting scope or rendering the
//! answer.

use crate::composition::Registered;
use crate::outcome::GatePlan;
use crate::{GateCommand, GateOutcome};

/// Composes this gate's rule registry and reports what it holds.
///
/// `command` is accepted in full -- including [`GateCommand::root`], which this increment
/// does not yet read -- so a caller writes against the shape `nomos gate plan` will keep once
/// a later increment gives `root` something to do.
#[must_use]
pub fn Run(_command: &GateCommand) -> GateOutcome
{
    let registry = match Registered()
    {
        Ok(registry) => registry,
        Err(error) => return GateOutcome::Contradictory(error),
    };

    let rules = registry.Offers().cloned().collect();

    return GateOutcome::Planned(GatePlan { rules });
}

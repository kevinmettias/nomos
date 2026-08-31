//! [`Handle_Gate_Run`] and its own [`GateRunResponse`], paired in one file: the response type
//! exists only for this one handler, the same "handler beside its own response" locality every
//! other endpoint file in this crate keeps.

use crate::{composition, sources};
use nomos_contracts::RunId;
use nomos_gate_orchestration::{GateCommand, GateRunResult};
use nomos_platform::Clock;
use nomos_platform_std::{StdProcessLauncher, SystemClock};
use serde::Serialize;
use std::path::{Path, PathBuf};

use super::{Disposition, GateFindings};

/// Walks `root` and judges it exactly as `nomos gate run` would, over the default
/// [`GateCommand`] -- every rule, every file, no baseline, no suppression, no adoption
/// calibration -- and hands back a JSON-serializable [`GateRunResponse`].
#[must_use]
pub fn Handle_Gate_Run(root: &Path) -> GateRunResponse
{
    let command = GateCommand { root: root.to_path_buf(), ..Default::default() };
    let walked = sources::Walked_Sources(root);
    let run = nomos_gate_orchestration::Fresh_Run_Id(SystemClock.Now());
    let result = nomos_gate_orchestration::Run_Gate(
        walked,
        nomos_gate_orchestration::GateEnvironment { variant: composition::Host_Variant(), launcher: &StdProcessLauncher },
        &command,
        run,
    );

    return GateRunResponse::From(result);
}

/// What a real gate run over `root` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
pub struct GateRunResponse
{
    /// The identity of this execution. `RunId` already derives `Serialize` -- unlike
    /// [`nomos_gate_orchestration::GateRunOutcome`], it needs no local twin.
    pub run: RunId,
    /// The tree this run judged.
    pub root: PathBuf,
    /// The reduced verdict.
    pub disposition: Disposition,
    /// Every finding this run reduced, grouped by why it does or does not block.
    pub findings: GateFindings,
}

impl GateRunResponse
{
    pub(crate) fn From(result: GateRunResult) -> Self
    {
        return Self {
            run: result.run,
            root: result.root,
            disposition: Disposition::From(result.disposition),
            findings: GateFindings::From(result.findings),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A real run over this crate's own tree reaches a real judgment -- not
    /// [`Disposition::Indeterminate`], the state a walk that never became a judged check
    /// outcome carries -- proving this crate, not `nomos-cli`, can produce one.
    #[test]
    fn Test_Handle_Gate_Run_And_Walked_Sources_Should_Reach_A_Judgment_Over_A_Real_Tree()
    {
        let response = Handle_Gate_Run(Path::new("."));

        assert_ne!(
            response.disposition,
            Disposition::Indeterminate,
            "this crate's own source tree has real .rs files to judge, so the check behind \
             this run must have reached Judged"
        );
    }

    /// An empty tree cannot be judged, the same distinction the check-orchestration layer's
    /// own `CheckOutcome::NoSource` already keeps apart from a clean judged run.
    #[test]
    fn Test_Host_Variant_Should_Compose_Into_A_Working_Environment_For_An_Empty_Tree()
    {
        let empty = std::env::temp_dir().join("nomos-api-gate-run-empty-tree");
        let _ignored = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).expect("creates an empty directory");

        let response = Handle_Gate_Run(&empty);

        let _ignored = std::fs::remove_dir_all(&empty);

        assert_eq!(response.disposition, Disposition::Indeterminate);
        assert!(response.findings.blocking_findings.is_empty());
    }

    /// The whole point of this crate: the response a real run produces is valid JSON, and
    /// its disposition round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_From_Should_Produce_A_Response_That_Round_Trips_As_Json()
    {
        let response = Handle_Gate_Run(Path::new("."));
        let expected = match response.disposition
        {
            Disposition::Passed => "passed",
            Disposition::Failed => "failed",
            Disposition::Indeterminate => "indeterminate",
        };

        let json = serde_json::to_string(&response).expect("a GateRunResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let disposition = parsed.get("disposition").expect("a serialized GateRunResponse always has this field");

        assert_eq!(disposition, expected, "{json}");
    }
}

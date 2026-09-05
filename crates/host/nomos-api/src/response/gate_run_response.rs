//! [`Handle_Gate_Run`] and its own [`GateRunResponse`], paired in one file: the response type
//! exists only for this one handler, the same "handler beside its own response" locality every
//! other endpoint file in this crate keeps.

use crate::{composition, sources};
use nomos_contracts::RunId;
use nomos_gate_orchestration::{GateCommand, GateRunResult};
use nomos_platform::Clock;
use nomos_platform_std::{StdFileSystem, StdProcessLauncher, SystemClock};
use serde::Serialize;
use std::path::PathBuf;

use super::{Disposition, GateFindings};

/// Walks `command.root` and judges it exactly as `nomos gate run` would, and hands back a
/// JSON-serializable [`GateRunResponse`].
///
/// `command` travels straight through to [`nomos_gate_orchestration::Run_Gate`], so its
/// `scope` and `rules` selectors reach the same materialization-skipping `OD-GATE-017` gives
/// `nomos-cli`'s own `run` verb -- a caller that narrows either pays for only what it asked
/// to judge, not for every registered rule on every call.
#[must_use]
pub fn Handle_Gate_Run(command: &GateCommand) -> GateRunResponse
{
    let walked = sources::Walked_Sources(&command.root);
    let run = nomos_gate_orchestration::Fresh_Run_Id(SystemClock.Now());
    let result = nomos_gate_orchestration::Run_Gate(
        walked,
        nomos_gate_orchestration::GateEnvironment {
            variant: composition::Host_Variant(),
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
        },
        command,
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
    use nomos_contracts::RuleId;
    use nomos_gate_orchestration::RuleSelector;
    use nomos_rules::NAMING_CONVENTION;

    /// [`GateCommand`] over `root`, every selector at its select-everything default -- the
    /// shape every test here builds before narrowing one field of its own.
    fn Command_At(root: PathBuf) -> GateCommand
    {
        return GateCommand { root, ..Default::default() };
    }

    /// A real run over this crate's own tree reaches a real judgment -- not
    /// [`Disposition::Indeterminate`], the state a walk that never became a judged check
    /// outcome carries -- proving this crate, not `nomos-cli`, can produce one.
    #[test]
    fn Test_Handle_Gate_Run_And_Walked_Sources_Should_Reach_A_Judgment_Over_A_Real_Tree()
    {
        let response = Handle_Gate_Run(&Command_At(PathBuf::from(".")));

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

        let response = Handle_Gate_Run(&Command_At(empty.clone()));

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
        let response = Handle_Gate_Run(&Command_At(PathBuf::from(".")));
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

    /// A [`GateCommand::rules`] selection reaches `Run_Gate` through this crate's real entry
    /// point, not only the response built from it: excluding the rule behind this fixture's
    /// one real finding turns a Failed run into a Passed one with no blocking findings at
    /// all, the same "the deselected rule was never asked to run" property
    /// `nomos_gate_orchestration`'s own `Test_A_Deselected_Rules_Finding_Should_Not_Exist`
    /// proves one layer down.
    #[test]
    fn Test_A_Rule_Selection_Should_Reach_Run_Gate_Through_This_Crates_Entry_Point()
    {
        let root = std::env::temp_dir().join("nomos-api-gate-run-rule-selection");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a fresh directory");
        std::fs::write(root.join("a.rs"), "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n")
            .expect("writes a fixture whose stale mirror is one real blocking finding");

        let every_rule = Handle_Gate_Run(&Command_At(root.clone()));
        let naming_only = Handle_Gate_Run(&GateCommand {
            rules: RuleSelector { include: vec![RuleId::New(NAMING_CONVENTION)] },
            ..Command_At(root.clone())
        });

        let _ignored = std::fs::remove_dir_all(&root);

        assert_eq!(every_rule.disposition, Disposition::Failed, "{every_rule:?}");
        assert_eq!(naming_only.disposition, Disposition::Passed, "{naming_only:?}");
        assert!(
            naming_only.findings.blocking_findings.is_empty(),
            "the deselected rule's finding must not exist at all: {naming_only:?}"
        );
    }
}

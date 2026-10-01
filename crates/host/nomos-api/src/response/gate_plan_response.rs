//! [`Handle_Gate_Plan`] and its own [`GatePlanResponse`], paired in one file the same way
//! [`super::gate_run_response`] pairs [`super::gate_run_response::Handle_Gate_Run`] with
//! [`super::gate_run_response::GateRunResponse`].

use nomos_gate_orchestration::{GateCommand, GateOutcome};
use serde::Serialize;

use super::RuleOfferResponse;

/// Composes this gate's rule registry and reports what it holds, exactly as `nomos gate
/// plan` would, and hands back a JSON-serializable [`GatePlanResponse`].
///
/// `nomos_gate_orchestration::Run` -- the `Plan` verb's own real body, not this crate's own
/// `Run_Gate` -- takes a full [`GateCommand`] but, by its own doc, reads none of it: `Plan`
/// reports what [`nomos_gate_orchestration::Registered`] holds, not a walk over `root`, so a
/// default command is built here rather than asking a caller to supply one nothing reads.
#[must_use]
pub fn Handle_Gate_Plan() -> GatePlanResponse
{
    let command = GateCommand::default();
    let outcome = nomos_gate_orchestration::Run(&command);

    return GatePlanResponse::From(outcome);
}

/// What a real `nomos gate plan` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum GatePlanResponse
{
    /// This gate's rule registry composed cleanly.
    Planned
    {
        /// Every rule this gate's registry holds.
        rules: Vec<RuleOfferResponse>,
    },
    /// This gate's own rule composition is self-contradictory -- a defect in the
    /// composition, not in anything a caller supplied. Not reachable today; see
    /// `nomos_gate_orchestration::Registered`'s own doc.
    Contradictory
    {
        /// What went wrong, as `RuleRegistryError`'s own `Debug` renders it -- it does not
        /// derive `Display`, the same reason `crates/host/nomos-cli/src/gate/report.rs`'s
        /// own `Render_Plan` prints `{error:?}` too.
        cause: String,
    },
}

impl GatePlanResponse
{
    pub(crate) fn From(outcome: GateOutcome) -> Self
    {
        return match outcome
        {
            GateOutcome::Planned(plan) => Self::Planned {
                rules: plan.rules.into_iter().map(RuleOfferResponse::From).collect(),
            },
            GateOutcome::Contradictory(error) => Self::Contradictory { cause: format!("{error:?}") },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A real plan composes this workspace's own real rule registry, not an empty one --
    /// proving this crate, not `nomos-cli`, can produce a real `GatePlanResponse::Planned`.
    #[test]
    fn Test_Handle_Gate_Plan_Should_Compose_This_Workspaces_Own_Registry()
    {
        let response = Handle_Gate_Plan();

        let GatePlanResponse::Planned { rules } = response
        else
        {
            // This workspace's own rule registry is composed from the same rules this crate
            // itself was built against, so reaching `Contradictory` here means the workspace's
            // own composition broke, not a runtime condition this test should tolerate.
            panic!("this crate's own rule registry composes cleanly");
        };
        assert!(!rules.is_empty(), "this workspace ships real rules");
    }

    /// The response a real plan produces is valid JSON, and its outcome round-trips through
    /// `serde_json` under the field name a wire caller would actually read.
    #[test]
    fn Test_From_Should_Produce_A_Planned_Response_That_Round_Trips_As_Json()
    {
        let response = Handle_Gate_Plan();

        let json = serde_json::to_string(&response).expect("a derived Serialize over owned data has nothing to refuse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("an internally tagged enum writes its tag under this field name");

        assert_eq!(outcome, "planned", "{json}");
    }
}

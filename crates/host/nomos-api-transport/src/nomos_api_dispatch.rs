//! The verbs this workspace serves, and the arguments each one reads.

use crate::{CheckParameters, CompareParameters, CorrectionParameters, FindingParameters, GateParameters, ServedMethod};
use serde::Serialize;
use xvpe_primitives::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use xvpe_remote_call::{RemoteCallOutcome, RemoteCallRefusal, RemoteCallStrategy};

/// This workspace's own served surface, as the engine's service.
///
/// Everything about *being* a transport -- the JSON-RPC envelope, the line
/// framing, the reserved codes, the socket -- is `xvpe-remote-call-backend-json`'s as of
/// 2026-09-10. What is left here is the half that was always this workspace's:
/// which verbs are served, and what each one's arguments mean.
///
/// A unit struct because this service holds nothing. Every operation below runs
/// against a tree the *request* names, so there is no configuration for an
/// instance to carry and nothing for two of them to disagree about.
#[derive(Clone, Copy, Debug, Default)]
pub struct NomosApiDispatch;

impl Strategy for NomosApiDispatch
{
    // `None`: every served verb walks and judges a real tree. What a run finds
    // depends on what is on disk when it looks, so nothing here can promise that
    // the same call twice produces the same answer.
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl RemoteCallStrategy for NomosApiDispatch
{
    /// The registry `OD-HOST-007` asks to be declared explicitly, projected from
    /// [`ServedMethod::REGISTRY`] rather than restated beside it.
    ///
    /// A further entry is a visible edit to that enum, in this crate, and
    /// `tests/contract`'s `Test_The_Transport_Should_Name_No_Repo_Tooling_Handler`
    /// is what makes reaching a further *handler* a deliberate act rather than a
    /// diff nobody is watching.
    fn Served_Methods(&self) -> Vec<&'static str>
    {
        return ServedMethod::REGISTRY.iter().map(|method| return method.Name()).collect();
    }

    fn Answer(&self, method: &str, parameters: &str) -> RemoteCallOutcome
    {
        let Some(method) = ServedMethod::Named(method)
        else
        {
            // Unreachable through the protocol layer, which refuses an unserved
            // name against `Served_Methods` before ever calling this. Answered
            // rather than asserted because the alternative is a panic inside a
            // request handler, which would take a connection down for a caller
            // who did nothing wrong.
            let unknown = RemoteCallRefusal::Unknown_Operation(method, &[]);
            return RemoteCallOutcome::Refused(unknown);
        };

        return Answered_Method(method, parameters);
    }
}

/// The operation `method` names, called with the arguments it carried.
fn Answered_Method(method: ServedMethod, parameters: &str) -> RemoteCallOutcome
{
    return match method
    {
        ServedMethod::GatePlan => Planned(),
        ServedMethod::GateRun => Ran_Gate(parameters),
        ServedMethod::GateExplain => Explained_Finding(parameters),
        ServedMethod::GateCompare => Compared_Trees(parameters),
        ServedMethod::Correction => Ran_Correction(parameters),
        ServedMethod::Check => Ran_Check(parameters),
    };
}

/// What the rule registry holds, which is the whole of `nomos.gate.plan`'s answer.
///
/// `Handle_Gate_Plan` takes no argument, by its own doc: `Plan` reports what the rule
/// registry holds and does not walk `root`, so there is nothing here for a caller's own
/// arguments to supply. Arguments are ignored rather than refused for that reason -- a
/// caller sending the same object it sends to `nomos.gate.run` is not making a mistake
/// this transport should fail.
fn Planned() -> RemoteCallOutcome
{
    return Serialized_Response(&nomos_api::Handle_Gate_Plan());
}

/// A judged run over the tree the caller named in `root`.
fn Ran_Gate(parameters: &str) -> RemoteCallOutcome
{
    return match Parsed_Parameters::<GateParameters>(parameters)
    {
        Ok(parameters) => Serialized_Response(&nomos_api::Handle_Gate_Run(&parameters.Command())),
        Err(refusal) => RemoteCallOutcome::Refused(refusal),
    };
}

/// Why one finding stands, as the caller's `root` and `query` name it.
fn Explained_Finding(parameters: &str) -> RemoteCallOutcome
{
    return match Parsed_Parameters::<FindingParameters>(parameters)
    {
        Ok(parameters) =>
        {
            let explanation =
                nomos_api::Handle_Gate_Explain(&parameters.Root(), &parameters.Query());
            Serialized_Response(&explanation)
        }
        Err(refusal) => RemoteCallOutcome::Refused(refusal),
    };
}

/// How two trees' judgements differ, named by the caller's `baseline` and `candidate`.
fn Compared_Trees(parameters: &str) -> RemoteCallOutcome
{
    return match Parsed_Parameters::<CompareParameters>(parameters)
    {
        Ok(parameters) =>
        {
            let comparison =
                nomos_api::Handle_Gate_Compare(&parameters.Baseline(), &parameters.Candidate());
            Serialized_Response(&comparison)
        }
        Err(refusal) => RemoteCallOutcome::Refused(refusal),
    };
}

/// A correction run over the tree the caller named in `root`.
fn Ran_Correction(parameters: &str) -> RemoteCallOutcome
{
    return match Parsed_Parameters::<CorrectionParameters>(parameters)
    {
        Ok(parameters) => Serialized_Response(&nomos_api::Handle_Correction_Run(&parameters.Command())),
        Err(refusal) => RemoteCallOutcome::Refused(refusal),
    };
}

/// A check run over the tree the caller named in `root`, with none of the gate's own policy
/// on top -- `OD-HOST-014`'s decision 2 admits it because it walks and judges that tree
/// exactly as [`Ran_Gate`] does.
fn Ran_Check(parameters: &str) -> RemoteCallOutcome
{
    return match Parsed_Parameters::<CheckParameters>(parameters)
    {
        Ok(parameters) => Serialized_Response(&nomos_api::Handle_Check_Run(&parameters.Command())),
        Err(refusal) => RemoteCallOutcome::Refused(refusal),
    };
}

/// A call's arguments as the parameter type an operation takes.
///
/// The engine hands an operation that carried no arguments an empty object, so
/// an operation whose arguments are all optional is callable with none and no
/// parameter type here has to spell that case itself.
fn Parsed_Parameters<Parameters: serde::de::DeserializeOwned>(parameters: &str) -> Result<Parameters, RemoteCallRefusal>
{
    return serde_json::from_str(parameters)
        .map_err(|error| return RemoteCallRefusal::Bad_Arguments(error.to_string()));
}

/// An operation's own response, as the document the answer carries.
fn Serialized_Response<Response: Serialize>(response: &Response) -> RemoteCallOutcome
{
    return match serde_json::to_string(response)
    {
        Ok(result) => RemoteCallOutcome::Answered(result),
        // Not reachable over the response types `nomos-api` exports, which are
        // plain derived `Serialize` implementations over owned data. Answered
        // rather than asserted for the same reason as above.
        Err(error) => RemoteCallOutcome::Refused(RemoteCallRefusal::Internal(error.to_string())),
    };
}

#[cfg(test)]
mod tests;

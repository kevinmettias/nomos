//! The four verbs this workspace serves, and the arguments each one reads.

use crate::{CompareParameters, CorrectionParameters, FindingParameters, GateParameters, ServedMethod};
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
pub struct NomosApiService;

impl Strategy for NomosApiService
{
    // `None`: every served verb walks and judges a real tree. What a run finds
    // depends on what is on disk when it looks, so nothing here can promise that
    // the same call twice produces the same answer.
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl RemoteCallStrategy for NomosApiService
{
    /// The registry `OD-HOST-007` asks to be declared explicitly, projected from
    /// [`ServedMethod::REGISTRY`] rather than restated beside it.
    ///
    /// A fifth entry is a visible edit to that enum, in this crate, and
    /// `tests/contract`'s `Test_The_Transport_Should_Name_No_Repo_Tooling_Handler`
    /// is what makes reaching a fifth *handler* a deliberate act rather than a
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
            return RemoteCallOutcome::Refused(RemoteCallRefusal::Unknown_Operation(method, &[]));
        };

        return Answered(method, parameters);
    }
}

/// The operation `method` names, called with the arguments it carried.
fn Answered(method: ServedMethod, parameters: &str) -> RemoteCallOutcome
{
    return match method
    {
        // `Handle_Gate_Plan` takes no argument, by its own doc: `Plan` reports
        // what the rule registry holds and does not walk `root`, so there is
        // nothing here for a caller's own arguments to supply. Arguments are
        // ignored rather than refused for that reason -- a caller sending the
        // same object it sends to `nomos.gate.run` is not making a mistake this
        // transport should fail.
        ServedMethod::GatePlan => Serialized(&nomos_api::Handle_Gate_Plan()),
        ServedMethod::GateRun => match Parsed::<GateParameters>(parameters)
        {
            Ok(parameters) => Serialized(&nomos_api::Handle_Gate_Run(&parameters.Command())),
            Err(refusal) => RemoteCallOutcome::Refused(refusal),
        },
        ServedMethod::GateExplain => match Parsed::<FindingParameters>(parameters)
        {
            Ok(parameters) => Serialized(&nomos_api::Handle_Gate_Explain(
                &parameters.Root(),
                &parameters.Query(),
            )),
            Err(refusal) => RemoteCallOutcome::Refused(refusal),
        },
        ServedMethod::GateCompare => match Parsed::<CompareParameters>(parameters)
        {
            Ok(parameters) => Serialized(&nomos_api::Handle_Gate_Compare(
                &parameters.Baseline(),
                &parameters.Candidate(),
            )),
            Err(refusal) => RemoteCallOutcome::Refused(refusal),
        },
        ServedMethod::Correction => match Parsed::<CorrectionParameters>(parameters)
        {
            Ok(parameters) => Serialized(&nomos_api::Handle_Correction_Run(&parameters.Command())),
            Err(refusal) => RemoteCallOutcome::Refused(refusal),
        },
    };
}

/// A call's arguments as the parameter type an operation takes.
///
/// The engine hands an operation that carried no arguments an empty object, so
/// an operation whose arguments are all optional is callable with none and no
/// parameter type here has to spell that case itself.
fn Parsed<T: serde::de::DeserializeOwned>(parameters: &str) -> Result<T, RemoteCallRefusal>
{
    return serde_json::from_str(parameters)
        .map_err(|error| return RemoteCallRefusal::Bad_Arguments(error.to_string()));
}

/// An operation's own response, as the document the answer carries.
fn Serialized<T: Serialize>(response: &T) -> RemoteCallOutcome
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

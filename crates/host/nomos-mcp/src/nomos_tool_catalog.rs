//! The tools this server offers, and the one call that reaches them.

use crate::ServedTool;
use nomos_api_transport::NomosApiDispatch;
use xvpe_primitives::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use xvpe_remote_call::{
    RemoteCallOutcome, RemoteCallStrategy as _, ServerIdentity, ToolAnswer, ToolCatalogStrategy,
    ToolDescriptor,
};

/// This server's own name, as a client shows it to a user and keys its
/// configuration on.
const SERVER_NAME: &str = "nomos-mcp";

/// This workspace's served surface, as the engine's tool catalogue.
///
/// Everything about *being* a tool server -- the handshake, the capability
/// declaration, the listing envelope, the call envelope, the `isError`
/// distinction, notification suppression, and the line framing underneath all of
/// it -- is `xvpe-remote-call-backend-json`'s as of 2026-09-10. What is left here is the
/// half that was always this workspace's: which tools exist, what each one
/// means, what arguments it takes, and where a call goes.
#[derive(Clone, Copy, Debug, Default)]
pub struct NomosToolCatalog;

impl Strategy for NomosToolCatalog
{
    // `None`: a call reaches a handler that walks and judges a real tree, so
    // what it answers depends on what is on disk when it looks.
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl ToolCatalogStrategy for NomosToolCatalog
{
    fn Identity(&self) -> ServerIdentity
    {
        // The version is this package's own rather than a literal: one typed by
        // hand stops being true at the next release and nothing notices.
        return ServerIdentity::Of(SERVER_NAME, env!("CARGO_PKG_VERSION"));
    }

    fn Tools(&self) -> Vec<ToolDescriptor>
    {
        return ServedTool::REGISTRY.iter().map(|tool| return tool.Descriptor()).collect();
    }

    /// A call, forwarded to the same service `nomos-api-transport` serves over a
    /// socket.
    ///
    /// # Why this crate still reaches no handler
    ///
    /// The boundary `OD-HOST-007` drew is unchanged, and is still structural: a
    /// tool name *is* a served method name, so a call goes to
    /// [`NomosApiDispatch`] under the name the client asked for, and this crate
    /// never names a `nomos_api::Handle_*` function or depends on `nomos-api` at
    /// all. Widening what a call can reach would first have to widen
    /// `ServedMethod`, in the crate where `tests/contract` already polices it.
    ///
    /// What changed is only how the call travels. It used to be re-serialized
    /// into a synthetic JSON-RPC line and handed back to that crate's own
    /// line parser -- a round trip through a wire format neither side was
    /// reading off a wire, which existed because the two crates had no shared
    /// contract to meet at. They have one now, so the call is a call.
    fn Call_With_Json_Arguments(&self, name: &str, arguments: &str) -> ToolAnswer
    {
        return match NomosApiDispatch.Answer(name, arguments)
        {
            RemoteCallOutcome::Answered(result) => ToolAnswer::Produced(result),
            RemoteCallOutcome::Refused(refusal) => ToolAnswer::Failed(refusal.message),
        };
    }
}

#[cfg(test)]
mod tests;

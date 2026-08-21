//! Band 90 — host. A second real caller of [`nomos_gate_orchestration::Run_Gate`].
//!
//! `nomos-gate-orchestration`'s own module doc, since `P13-GATE-RUN-SEAM-CRATE`, states the
//! point directly: `Run_Gate` is generic over `nomos-platform`'s traits "so a second adapter
//! can call it without depending on `nomos-cli`." Until this crate, nothing did --
//! `nomos-cli`'s own `gate.rs` was the only caller outside the orchestration crate itself,
//! apart from `nomos-ledger`'s internal gate step inside `work finish`, which is a
//! composition inside that flow rather than an external-facing adapter. `ARC-ROADMAP-001`
//! names `CLI / API / MCP projections` as a near-term-tier item next to the Gate object
//! itself; this is that item's first real increment.
//!
//! [`Handle_Gate_Run`] is a deliberate twin of `nomos-cli`'s `gate.rs`
//! `GateInvocation::Run` arm: it walks `root` for `.rs` sources
//! ([`sources::Walked`] -- a twin of `crates/host/nomos-cli/src/gate/sources.rs`, since a
//! walk is a composition-root concern `OD-HOST-002` does not seam), reads this crate's own
//! build variant ([`composition::Host_Variant`] -- `env!` resolves against the crate that
//! calls it, so this cannot be shared either), and calls `Run_Gate` with the default
//! `GateCommand` every verb had before `OD-GATE-014`'s scope/rule selectors gave one a real
//! caller -- narrowing `root`'s own tree is this crate's only input today. What differs from
//! `nomos-cli` is the return: [`response::GateRunResponse`] instead of rendered text, because
//! this crate's reason to exist is a JSON-serializable answer a wire transport can hand back,
//! not a terminal one.
//!
//! # What this crate deliberately does not do
//!
//! It does not read argv, listen on a socket, or speak MCP's JSON-RPC framing. Building a
//! transport ahead of a real second caller for it is exactly the premature-surface pattern
//! this workspace has repeatedly declined to build ahead of evidence (`OD-PACKAGE-006`,
//! `OD-PACKAGE-008`, and `nomos-cli`'s own `gate.rs` doc on why `compare` is not stubbed). A
//! follow-up increment wires a real transport over [`Handle_Gate_Run`] once one exists to
//! design it against; this increment's job is only to prove the seam is reachable and
//! projectable from a second composition root at all.
//!
//! It does not select scope or rules either -- `GateCommand::scope` and `GateCommand::rules`
//! stay at their select-everything default, the same starting point every construction site
//! that predates `OD-GATE-014`'s selectors has. A caller that needs to narrow either is a
//! later increment to this crate's request shape, not a gap this one leaves silently.

mod composition;
mod response;
mod sources;

pub use response::{Disposition, GateRunResponse};

use nomos_gate_orchestration::GateCommand;
use nomos_platform_std::StdProcessLauncher;
use std::path::Path;

/// Walks `root` and judges it exactly as `nomos gate run` would, over the default
/// [`GateCommand`] -- every rule, every file, no baseline, no suppression, no adoption
/// calibration -- and hands back a JSON-serializable [`GateRunResponse`].
#[must_use]
pub fn Handle_Gate_Run(root: &Path) -> GateRunResponse
{
    let command = GateCommand { root: root.to_path_buf(), ..Default::default() };
    let walked = sources::Walked(root);
    let result = nomos_gate_orchestration::Run_Gate(
        walked,
        composition::Host_Variant(),
        &command,
        &StdProcessLauncher,
    );

    return GateRunResponse::From(result);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A real run over this crate's own tree reaches a real judgment -- not
    /// [`Disposition::Indeterminate`], the state a walk that never became a judged check
    /// outcome carries -- proving this crate, not `nomos-cli`, can produce one.
    #[test]
    fn Test_A_Real_Run_Should_Reach_A_Judgment()
    {
        let response = Handle_Gate_Run(Path::new("."));

        assert_ne!(
            response.disposition,
            Disposition::Indeterminate,
            "this crate's own source tree has real .rs files to judge, so the check behind \
             this run must have reached Judged"
        );
    }

    /// An empty tree cannot be judged, the same distinction `nomos_check_orchestration::
    /// CheckOutcome::NoSource` already keeps apart from a clean judged run.
    #[test]
    fn Test_An_Empty_Tree_Should_Be_Indeterminate()
    {
        let empty = std::env::temp_dir().join("nomos-api-gate-run-empty-tree");
        let _ignored = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).expect("creates an empty directory");

        let response = Handle_Gate_Run(&empty);

        let _ignored = std::fs::remove_dir_all(&empty);

        assert_eq!(response.disposition, Disposition::Indeterminate);
        assert!(response.blocking_findings.is_empty());
    }

    /// The whole point of this crate: the response a real run produces is valid JSON, and
    /// its disposition round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_A_Real_Runs_Response_Should_Round_Trip_As_Json()
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

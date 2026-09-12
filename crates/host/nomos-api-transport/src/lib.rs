//! Band 92 — host. What this workspace serves over a wire, and nothing about the
//! wire.
//!
//! `nomos-api`'s own module doc names the hole this crate fills, under the
//! heading *What this crate deliberately does not do*: "It does not read argv,
//! listen on a socket, or speak MCP's JSON-RPC framing." This is still that
//! increment, and the boundary is still exactly where that sentence puts it:
//! `nomos-api` gains no socket, no protocol dependency and no new handler here.
//!
//! # What changed on 2026-09-10
//!
//! **Every byte of the wire is XVPE's now.** The JSON-RPC 2.0 envelope, the
//! one-object-per-line framing, the reserved error codes, the unrenderable
//! fallback, the socket and its one-connection-at-a-time serving all moved down
//! into `xvpe-remote-call-backend-json`, over the `xvpe-remote-call` service contract. Nomos is
//! an application over that engine, and a general capability sitting up here was
//! unreachable by everything down there — including the engine's own reference
//! miner, which has a served surface of its own and could not reach a line of
//! this.
//!
//! The two crates that had the same framing and the same fallback constant
//! written out twice — this one and `nomos-mcp` — now share one floor rather
//! than each carrying a copy.
//!
//! What stays is what was always this workspace's: **which verbs are served**
//! ([`ServedMethod`]), **what each one's arguments mean** ([`GateParameters`],
//! [`FindingParameters`], [`CorrectionParameters`]), and the dispatch that turns
//! one into a `nomos_api::Handle_*` call ([`NomosApiService`]).
//!
//! # What it serves, and why that is four verbs rather than twenty-six or more
//!
//! `OD-HOST-007` decided the shape of this before the crate existed, which is the
//! difference between a boundary and an apology for one. `nomos-api` now exports
//! twenty-six or more handlers and most of them belong to crates `README.md`
//! marks `[repo tooling]` — `nomos-work-orchestration` and
//! `nomos-spec-orchestration`, which "exist to develop or preserve this
//! repository, not to answer a question an end-user repository would ask Nomos".
//! A transport projecting `nomos-api` wholesale would publish many repo-tooling
//! verbs for every product verb, some of which carry authority rather than
//! information: `Handle_Spec_Commit` writes this repository's own governing
//! records, and `Handle_Work_Finish` writes `work/ledger.json`, which `AGENTS.md`
//! calls "global coordination state, shared with live sessions".
//!
//! So the registry is Gate's three verbs plus Correction's one, in
//! [`ServedMethod`], and it is an enum rather than a list precisely because
//! `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2` refused a registry "merely short
//! today, with nothing stopping a later increment from lengthening it".
//! `tests/contract`'s `Test_The_Transport_Should_Name_No_Repo_Tooling_Handler`
//! reads `nomos-api`'s own blessed surface snapshot and refuses this crate's
//! source for calling a handler its `ADMITTED` allow-list has not deliberately
//! named.
//!
//! # How a caller serves this
//!
//! [`NomosApiService`] is an `xvpe_remote_call::RemoteCallStrategy`. A caller hands it to
//! `xvpe_remote_call_backend_json::Serve_Listener` with a listener it bound, or to
//! `Serve_Methods` with any pair of streams.
//!
//! Nothing in this workspace binds one. That is the honest state of it: this
//! crate makes the seam reachable and the engine carries it over a wire, and a
//! host verb that runs a server is still a later increment rather than something
//! claimed here.
//!
//! There is no authentication, no authorization and no transport security, and a
//! caller must read that as a real boundary rather than an omission. Every
//! operation served here runs against a tree the *request* names, so anything
//! that can reach a listener can make that process walk and judge any directory
//! it can read. Bind it to loopback. `OD-HOST-007` is also explicit that this is
//! where the public-commitment question now lives: the trigger `OD-LEDGER-036`
//! and `OD-HOST-006` share "fires against the transport, not against
//! `nomos-api`", so what this registry admits is a commitment made by this crate.

mod correction_parameters;
mod finding_parameters;
mod gate_parameters;
mod nomos_api_service;
mod served_method;

pub use correction_parameters::CorrectionParameters;
pub use finding_parameters::FindingParameters;
pub use gate_parameters::GateParameters;
pub use nomos_api_service::NomosApiService;
pub use served_method::ServedMethod;

/// Test-only helpers shared by more than one module here.
///
/// Declared after every public item deliberately: the surface scanner reads
/// declarations in order and a `cfg(test)` module ahead of a public one takes it
/// out of the snapshot without changing anything that compiles.
#[cfg(test)]
mod test_support;

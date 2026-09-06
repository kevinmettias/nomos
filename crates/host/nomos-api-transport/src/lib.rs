//! Band 92 — host. The transport `nomos-api` deliberately does not wire.
//!
//! `nomos-api`'s own module doc names the hole this crate fills, under the heading *What this
//! crate deliberately does not do*: "It does not read argv, listen on a socket, or speak
//! MCP's JSON-RPC framing... A follow-up increment wires a real transport over
//! [`nomos_api::Handle_Gate_Run`] once one exists to design it against." `README.md`'s own
//! row for that crate ends the same way -- everything it does, "apart from choosing a
//! platform or wiring an actual transport over it." This is that increment, and the boundary
//! stays exactly where those two sentences put it: `nomos-api` gains no socket, no protocol
//! dependency and no new handler here, and this crate holds every byte of both.
//!
//! # What it serves, and why that is four verbs rather than twenty-six or more
//!
//! `OD-HOST-007` decided the shape of this before the crate existed, which is the
//! difference between a boundary and an apology for one. `nomos-api` now exports twenty-six
//! or more handlers (`Handle_Check_Run` and `Handle_Workflow_Run` landed after this crate
//! did) and most of them belong to crates `README.md` marks `[repo tooling]` --
//! `nomos-work-orchestration` and `nomos-spec-orchestration`, which "exist to develop or
//! preserve this repository, not to answer a question an end-user repository would ask
//! Nomos". A transport projecting `nomos-api` wholesale would publish many repo-tooling
//! verbs for every product verb, some of which carry authority rather than information:
//! `Handle_Spec_Commit` writes this repository's own governing records, and
//! `Handle_Work_Finish` writes `work/ledger.json`, which `AGENTS.md` calls "global
//! coordination state, shared with live sessions".
//!
//! So the registry is Gate's three verbs plus Correction's one,
//! `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2`'s own increment, in [`ServedMethod`], and it is
//! an enum rather than a list precisely because that record refused a registry "merely
//! short today, with nothing stopping a later increment from lengthening it". `tests/contract`'s
//! `Test_The_Transport_Should_Name_No_Repo_Tooling_Handler` is the artifact that record was
//! waiting for: it reads `nomos-api`'s own blessed surface snapshot and refuses this crate's
//! source for calling a handler its own `ADMITTED` allow-list has not deliberately named,
//! so widening the registry to reach a fifth handler is a conscious edit to that list, not a
//! diff nobody is watching. Check and Workflow, this crate's two newest handlers, are not
//! in it yet -- admitting either is a real increment of its own, not a consequence of this
//! one.
//!
//! # What it speaks
//!
//! JSON-RPC 2.0, one object per line, over TCP. `OD-HOST-007` explicitly left all three of
//! those open -- it "does not build the transport, choose a JSON-RPC implementation, decide
//! stdio versus socket, or name the crate" -- so each is this increment's own, and each is
//! argued where it is made: the framing in [`Serve_Connection`], the one-connection-at-a-time
//! serving in [`Serve`], the reserved error codes in [`WireError`], and the operation names
//! in [`ServedMethod::Name`], which projects `nomos_contracts::OperationName` rather than
//! inventing a second spelling of an operation's identity.
//!
//! A socket rather than stdio, because the gap `P40-API-TRANSPORT-2` names is that nothing
//! "carries them off the machine": a stdio transport is reachable only by a process this host
//! already launched, which is a smaller version of the Rust-caller-linking-the-crate
//! reachability that crate already had. An MCP surface over stdio is a second projection of
//! the same registry and is `P40-MCP-SURFACE-4`'s own work, not a reason for this one to
//! choose stdio first.
//!
//! No JSON-RPC library, and no async runtime. `serde_json` and `std::net` are already in this
//! workspace's dependency graph; the envelope is four fields and the framing is
//! `BufRead::lines`, so a dependency here would buy nothing this crate does not already have
//! and would put a third-party crate on the one path that reads bytes from a stranger.
//!
//! # What it still does not do
//!
//! Nothing in this workspace binds a listener: [`Serve`] takes a `TcpListener` a caller
//! already bound, and the caller that would bind one from a command line does not exist yet.
//! That is the honest state of it -- this crate makes the seam reachable over a wire and
//! proves it with a real socket, and a host verb that runs a server is the next increment
//! rather than something claimed here.
//!
//! There is no authentication, no authorization and no transport security, and a caller must
//! read that as a real boundary rather than an omission. Every operation served here runs
//! against a tree the *request* names, so anything that can reach the listener can make this
//! process walk and judge any directory it can read. Bind it to loopback. `OD-HOST-007` is
//! also explicit that this is where the public-commitment question now lives: the trigger
//! `OD-LEDGER-036` and `OD-HOST-006` share "fires against the transport, not against
//! `nomos-api`", so what this registry admits is a commitment made by this crate.

mod correction_parameters;
mod dispatch;
mod finding_parameters;
mod gate_parameters;
mod serve;
mod served_method;
mod wire_error;
mod wire_request;
mod wire_response;

pub use correction_parameters::CorrectionParameters;
pub use dispatch::Answer;
pub use finding_parameters::FindingParameters;
pub use gate_parameters::GateParameters;
pub use serve::{Serve, Serve_Connection};
pub use served_method::ServedMethod;
pub use wire_error::WireError;
pub use wire_request::WireRequest;
pub use wire_response::WireResponse;

/// Test-only helpers shared by more than one module here.
///
/// Declared after every public item deliberately: the surface scanner reads declarations in
/// order and a `cfg(test)` module ahead of a public one takes it out of the snapshot without
/// changing anything that compiles.
#[cfg(test)]
mod test_support;

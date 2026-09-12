//! Band 93 — host. What this workspace offers a model-facing client, and
//! nothing about the protocol that carries it.
//!
//! `AGT-006` names this workspace's own commitment: "integration shall occur
//! through neutral versioned contracts and MCP tools." `nomos-api-transport`
//! closed the socket half; this crate closes the tool half, over the identical
//! registry that crate serves.
//!
//! # What changed on 2026-09-10
//!
//! **The whole handshake is XVPE's now.** `initialize` and its one declared
//! capability, `tools/list`, `tools/call` and its `isError` distinction, `ping`,
//! notification suppression, and the line framing underneath all four moved down
//! into `xvpe-remote-call-backend-json`, over the `xvpe-remote-call` catalogue contract.
//!
//! That also closed a duplication this crate was carrying: its framing loop and
//! its unrenderable-response constant were a second copy of
//! `nomos-api-transport`'s, written out again because the two crates had no
//! shared floor to stand on. They have one now.
//!
//! What stays is what was always this workspace's: **which four tools exist**
//! ([`ServedTool`]), the sentence and the JSON Schema each one publishes, and
//! where a call to one lands ([`NomosToolCatalog`]).
//!
//! # What this crate still cannot reach
//!
//! The boundary `OD-HOST-007` drew is unchanged and is still structural. A tool
//! name *is* a served method name, so a call goes to
//! `nomos_api_transport::NomosApiService` under the name the client asked for,
//! and this crate never names a `nomos_api::Handle_*` function or depends on
//! `nomos-api` at all. Widening what a `tools/call` can reach would first have to
//! widen `ServedMethod`, in that crate, where the exclusion is already policed.
//! `tests/contract/tests/boundaries/mcp_registry.rs` asserts the dependency edge
//! that makes this structural rather than advisory.
//!
//! # How this is served
//!
//! An MCP server over stdio, unchanged in what it is: [`NomosToolCatalog`] is an
//! `xvpe_remote_call::ToolCatalogStrategy`, and `src/main.rs` hands it to
//! `xvpe_remote_call_backend_json::Serve_Tools` over this process's own stdin and
//! stdout, which is how a client launches it. `AGT-006` is assessed against that
//! sentence and against `tests/stdio.rs`, which drives a real subprocess.
//!
//! There is no authentication and no transport security, the identical boundary
//! `nomos-api-transport`'s own doc draws for its socket: a process that can write
//! to this server's stdin can run a gate over any tree the *request* names and
//! this process can read. That is ordinary for a server launched as a trusted
//! subprocess by its own client, which is the only way `src/main.rs` launches it
//! — there is no socket here for a stranger to reach.

mod nomos_tool_catalog;
mod served_tool;

pub use nomos_tool_catalog::NomosToolCatalog;
pub use served_tool::ServedTool;

/// Test-only helpers shared by more than one module here.
///
/// Declared after every public item deliberately: the surface scanner reads declarations in
/// order and a `cfg(test)` module ahead of a public one takes it out of the snapshot without
/// changing anything that compiles.
#[cfg(test)]
mod test_support;

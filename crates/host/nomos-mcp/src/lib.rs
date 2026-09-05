//! Band 93 — host. An MCP server over stdio, projecting `nomos-api-transport`'s own three
//! Gate verbs as MCP tools.
//!
//! `AGT-006` names this workspace's own commitment: "integration shall occur through
//! neutral versioned contracts and MCP tools." `nomos-api`'s own module doc had, until this
//! crate existed, stated the other half of that gap in as many words: it "does not read
//! argv, listen on a socket, or speak MCP's JSON-RPC framing." `nomos-api-transport` closed
//! the socket half; this crate closes the MCP half, over the identical registry that crate
//! already proves excludes every `[repo tooling]` handler `README.md` marks.
//!
//! # What this crate adds, and what it deliberately does not
//!
//! Everything product-shaped here is borrowed rather than built twice. [`serve::Serve`]
//! reads the same one-JSON-object-per-line framing `nomos_api_transport::Serve_Connection`
//! already uses -- that crate's own doc names it as "the framing MCP's own stdio transport
//! uses," which is why this crate owns no second framing decision. [`dispatch::Answer`]
//! answers exactly four MCP methods -- `initialize`, `tools/list`, `tools/call` and `ping`
//! -- and for `tools/call` it never touches `nomos-api` or any orchestration crate directly:
//! it re-serializes the call as the identical JSON-RPC line `nomos_api_transport::Answer`
//! already reads, and that function is what reaches `nomos_api::Handle_Gate_Plan`, `_Run`
//! and `_Explain`. So this crate's `Cargo.toml` names `nomos-api-transport` as its only
//! product dependency, and `tests/contract/tests/boundaries` asserts it never grows a second
//! one -- the structural exclusion `OD-HOST-007` asked for, inherited rather than repeated,
//! because a crate with no dependency on a handler cannot call it by construction.
//!
//! What is new here is only the MCP-shaped surface over that registry: [`served_tool::
//! ServedTool`] adds a human description and a JSON Schema per tool, and [`dispatch`]
//! translates MCP's own `tools/call` envelope into and out of `nomos-api-transport`'s.
//!
//! This server declares one capability (`tools`) and no others -- no resources, no prompts,
//! no sampling, no roots, no logging. A client asking for any of those sees them absent from
//! `initialize`'s own `capabilities` object, which is MCP's own way of saying a server does
//! not have them, rather than this crate refusing a request for one by hand.
//!
//! There is no authentication and no transport security, the identical boundary
//! `nomos-api-transport`'s own doc draws for its socket: a process that can write to this
//! server's stdin can run a gate over any tree the *request* names and this process can
//! read. That is ordinary for an MCP server launched as a trusted subprocess by its own
//! client, which is the only way `src/main.rs` launches it -- there is no socket here for a
//! stranger to reach.

mod dispatch;
mod served_tool;
mod serve;

pub use dispatch::{Answer, PROTOCOL_VERSION};
pub use serve::Serve;
pub use served_tool::ServedTool;

/// Test-only helpers shared by more than one module here.
///
/// Declared after every public item deliberately: the surface scanner reads declarations in
/// order and a `cfg(test)` module ahead of a public one takes it out of the snapshot without
/// changing anything that compiles.
#[cfg(test)]
mod test_support;

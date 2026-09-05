//! Why a request was refused, in the vocabulary JSON-RPC already has for it.

use serde::Serialize;

/// A JSON-RPC 2.0 error object.
///
/// The codes are the specification's own reserved range rather than codes invented here.
/// `nomos-cli`'s exit codes are deliberately not reused: `README.md`'s own table is a
/// contract with a shell, this is a contract with a JSON-RPC client, and a number that means
/// two things in two protocols is the second authority `AGENTS.md` refuses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WireError
{
    /// The reserved JSON-RPC code this refusal carries.
    pub code: i32,
    /// What went wrong, as the layer that refused it phrased the failure.
    pub message: String,
}

impl WireError
{
    /// The request was not valid JSON at all.
    pub const PARSE_ERROR: i32 = -32700;
    /// The request parsed, but is not a valid JSON-RPC 2.0 request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The request names an operation outside `crate::ServedMethod::REGISTRY`.
    ///
    /// This is the code a caller reaching for one of the twenty-one repo-tooling handlers
    /// gets, and it is deliberately the same code an outright typo gets: `OD-HOST-007`
    /// decided those handlers are not served, not that they are served conditionally, so
    /// there is nothing for a distinct code to tell a caller that it could act on.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// The operation exists, and its arguments are not the shape it takes.
    pub const INVALID_PARAMETERS: i32 = -32602;
    /// The operation ran and its own answer could not be rendered.
    pub const INTERNAL_ERROR: i32 = -32603;

    /// A refusal carrying `code` and `message`.
    #[must_use]
    pub fn New(code: i32, message: impl Into<String>) -> Self
    {
        return Self { code, message: message.into() };
    }
}

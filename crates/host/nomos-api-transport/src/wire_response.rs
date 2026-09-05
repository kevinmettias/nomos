//! One answer, in the shape a JSON-RPC client reads.

use crate::WireError;
use serde::Serialize;

/// A JSON-RPC 2.0 response object.
///
/// `result` and `error` are both optional and exactly one is ever set, which is the
/// specification's own rule. It is kept by the two constructors below rather than by a type
/// that makes it unrepresentable: an enum would serialize as a tagged or untagged union and
/// neither renders the flat `{jsonrpc, id, result}` object a client expects without a custom
/// `Serialize`, which would be more machinery than the invariant costs.
#[derive(Clone, Debug, Serialize)]
pub struct WireResponse
{
    /// Always [`Self::VERSION`].
    pub jsonrpc: &'static str,
    /// The correlation value the request carried, echoed back unchanged.
    pub id: serde_json::Value,
    /// The operation's own answer, serialized from the `nomos-api` response type it returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Why the request was refused.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WireError>,
}

impl WireResponse
{
    /// The protocol version every request must claim and every response carries.
    pub const VERSION: &'static str = "2.0";

    /// An answer to the request `id` identified.
    #[must_use]
    pub fn Answering(id: serde_json::Value, result: serde_json::Value) -> Self
    {
        return Self { jsonrpc: Self::VERSION, id, result: Some(result), error: None };
    }

    /// A refusal of the request `id` identified.
    #[must_use]
    pub fn Refusing(id: serde_json::Value, error: WireError) -> Self
    {
        return Self { jsonrpc: Self::VERSION, id, result: None, error: Some(error) };
    }
}

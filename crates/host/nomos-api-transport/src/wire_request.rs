//! One request as it arrives, before anything has decided whether it names a real operation.

use serde::Deserialize;

/// A JSON-RPC 2.0 request object.
///
/// Deserialized loosely on purpose: `method` and `params` are read as a string and an
/// uninterpreted value, so a request naming an operation this transport does not serve still
/// parses and is refused with JSON-RPC's own method-not-found code rather than as a parse
/// error. The two failures are different answers to a caller, and collapsing them would make
/// a typo in a method name look like malformed JSON.
#[derive(Clone, Debug, Deserialize)]
pub struct WireRequest
{
    /// The protocol version the caller claims. Refused unless it is exactly `2.0`.
    pub jsonrpc: String,
    /// The caller's own correlation value, echoed back on the answer whatever it is.
    ///
    /// JSON-RPC allows a string, a number or null, so this is an uninterpreted value rather
    /// than any narrower type. Absent, it is null -- which is also what a refusal carries
    /// when the request could not be parsed far enough to find one.
    #[serde(default)]
    pub id: serde_json::Value,
    /// The canonical operation name this request calls. See `crate::ServedMethod::Name`.
    pub method: String,
    /// The operation's own arguments, still uninterpreted.
    ///
    /// Each operation reads this into its own parameter type, so an argument shape belongs to
    /// the operation rather than to this envelope. Absent, it is null, which every parameter
    /// type here either accepts as its defaults or refuses on its own terms.
    ///
    /// Named in full and renamed on the wire, rather than spelled `params` here: JSON-RPC 2.0
    /// fixes the key and this workspace's `abbreviations` rule refuses the word, so the two
    /// are kept apart by `serde` instead of one of them giving way. The rule is right about
    /// the identifier and the specification is right about the key.
    #[serde(default, rename = "params")]
    pub parameters: serde_json::Value,
}

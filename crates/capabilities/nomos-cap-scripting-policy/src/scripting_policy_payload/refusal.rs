//! Why a `nomos.scripting.policy.v1` payload could not be decoded.

/// A payload's bytes did not decode: not UTF-8, a line with the wrong shape, or a `language`
/// line declaring an empty string (code-standards' own equivalence between an empty
/// declaration and no declaration at all means a well-formed encoder never emits one).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

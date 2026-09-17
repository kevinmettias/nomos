//! Why a `nomos.words.policy.v1` payload could not be decoded.

/// A payload's bytes did not decode: not UTF-8, or a line with no tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

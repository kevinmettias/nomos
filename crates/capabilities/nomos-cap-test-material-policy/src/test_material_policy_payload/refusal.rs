//! Why a `nomos.test.material.policy.v1` payload could not be decoded.

/// A payload's bytes did not decode: not UTF-8, a line with no tag, or a line with an
/// unrecognized tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

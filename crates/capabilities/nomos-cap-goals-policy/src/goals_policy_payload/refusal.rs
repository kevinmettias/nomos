//! Why a `nomos.goals.policy.v1` payload could not be decoded.

/// A payload's bytes did not decode: not UTF-8, a line with the wrong shape, a line with an
/// empty field, a second `ceiling` line, a `ceiling` that is not a number, or a `serves`
/// line naming a subsystem no `subsystem` line declared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

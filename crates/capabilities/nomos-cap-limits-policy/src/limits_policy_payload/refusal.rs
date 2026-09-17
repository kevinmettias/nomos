//! Why a payload could not be decoded.

/// A payload's bytes did not decode: not UTF-8, a line with the wrong field count, or a
/// value that is not a non-negative decimal integer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

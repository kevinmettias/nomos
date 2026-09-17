//! Why a declaration could not be decoded.

/// A payload's bytes did not decode: not UTF-8, a line whose tag this schema does not name, a
/// line with the wrong field count, or a statement naming a component the declaration never
/// declared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

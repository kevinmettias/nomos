//! Whether a qualified function's first input may be a receiver.
//!
//! The second of the arity engine's data dimensions, split out for the reason
//! `function_arity_source` is, and re-exported by the parent under the path it already had.

/// Whether the policy may treat one input on a qualified function as a receiver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiverAllowance
{
    /// Qualified and unqualified functions are judged against the same ceiling.
    None,
    /// A qualified function may have one extra input, because it may be a receiver.
    OneForQualifiedFunctions,
}

//! Which sources a function-arity policy judges.
//!
//! One of the three data dimensions `function_shape`'s module doc calls the arity engine's,
//! in its own file because `file-name-matches-declared-type` asks a public type to live in
//! the file its name spells and three public types cannot share one stem. The parent
//! re-exports it, so `nomos_rules::FunctionAritySource` is the same path it has always been.

/// Which source files a function-arity policy applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionAritySource
{
    /// Every source handed to the rule.
    All,
    /// Only files written in this language, as the composition root recognized it.
    Language(&'static str),
}

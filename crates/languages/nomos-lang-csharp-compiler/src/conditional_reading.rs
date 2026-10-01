//! [`ConditionalReading`], and [`Read_Conditionals`], which produces one.

use crate::conditional_walk::Walk;
use crate::ParseFailure;
use nomos_cap_csharp_semantics::{ConditionalRegion, FileDefinition};
use std::collections::BTreeSet;

/// The result of judging one file's conditional branches against a definition set.
///
/// Not a `Result`, for the reason every reading in this workspace gives: a file whose directives
/// do not nest and a file with no directive at all are different answers, and a `Result` invites
/// `.ok()` to make them the same.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionalReading
{
    /// Every branch of every chain, and every definition the file makes where compiled.
    Read
    {
        /// Each `#define` and `#undef` in a compiled part of the file, in line order.
        definitions: Vec<FileDefinition>,
        /// Each branch, in the order its directive appears.
        regions: Vec<ConditionalRegion>,
    },
    /// The directives do not form chains the compiler accepts -- an `#endif` with no `#if`, an
    /// `#elif` after `#else`, an `#if` never closed, a `#define` after the first token -- so the
    /// file does not compile and no branch of it has a state to report.
    Refused(ParseFailure),
}

/// Judges every conditional branch in `source` against `symbols`, the definition set the build
/// hands its compiler.
///
/// Takes text and a set, not a path: nothing here can reach a second file, which keeps the
/// guarantee's `IncrementalGranularity::File` true rather than asserted.
#[must_use]
pub fn Read_Conditionals(source: &str, symbols: &BTreeSet<String>) -> ConditionalReading
{
    let mut walk = Walk::New(symbols);
    for (index, line) in source.lines().enumerate()
    {
        if let Err(failure) = walk.Step(index.saturating_add(1), line)
        {
            return ConditionalReading::Refused(failure);
        }
    }
    if let Err(failure) = walk.Finish()
    {
        return ConditionalReading::Refused(failure);
    }

    return ConditionalReading::Read { definitions: walk.definitions, regions: walk.regions };
}

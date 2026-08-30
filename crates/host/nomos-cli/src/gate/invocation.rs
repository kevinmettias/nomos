//! What `nomos gate` was asked to do -- which verb, over which command.

use super::{FindingQuery, GateCommand};

/// What `nomos gate` was asked to do -- which verb, over which command.
///
/// A struct-per-verb enum rather than `GateCommand` itself growing a variant:
/// [`Invocation::Plan`]'s real computation lives entirely inside
/// `nomos-gate-orchestration`; [`Invocation::Run`]'s cannot, for the band reason this
/// module's own doc gives. The two verbs do not share a downstream function to route
/// through, so there is nothing for a single command enum inside
/// `nomos-gate-orchestration` to gain by carrying the verb itself.
#[derive(Debug)]
pub enum Invocation
{
    /// Compose this gate's rule registry and report what it holds.
    Plan(GateCommand),
    /// Walk the tree, judge it exactly as `nomos check` would, and report a real
    /// disposition.
    Run(GateCommand),
    /// Walk the tree, judge it, and answer what one named finding looks like and whether
    /// it would block.
    Explain
    {
        command: GateCommand,
        query: FindingQuery,
    },
}

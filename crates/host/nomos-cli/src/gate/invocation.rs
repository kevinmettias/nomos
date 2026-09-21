//! What `nomos gate` was asked to do -- which verb, over which command.

use std::path::PathBuf;

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
    /// Walk two trees, judge each, and report what moved between them.
    ///
    /// Two whole [`GateCommand`]s rather than one command and a second root: what a run
    /// finds depends on its scope and rule selectors as much as on its root, so a compare
    /// that shared everything but the path could only ever answer "did the tree change",
    /// never "did tightening this selector change what can fail the build". They carry the
    /// same selectors today because one set of flags authors both; the shape is what lets
    /// a later flag vary one side alone without changing this type.
    Compare
    {
        baseline: GateCommand,
        candidate: GateCommand,
    },
    /// Answer whether one crate may name another under the declared architecture, before any
    /// manifest carries the edge.
    ///
    /// The one verb carrying no [`GateCommand`], because it walks no tree: no root
    /// to walk, no scope to narrow, no rules to select. `OD-GATE-026` decided the subject is
    /// a crate pair rather than a proposed manifest edit or a change-set, since the answer is
    /// a pure function of two names and neither of those carries information the pair does
    /// not.
    Admits
    {
        depending: String,
        depended: String,
    },
    /// Execute the canonical gate's own step set on this host, and report every step.
    ///
    /// The one verb whose subject is the gate itself rather than a tree the gate judges, so
    /// it carries no [`GateCommand`]: no scope to narrow and no rules to select, because the
    /// step set is read out of `.github/workflows/gate.yml` and nothing selects from it.
    /// `OD-GATE-033` decided that file is the canonical model and that GitHub Actions and a
    /// local executor both project from it, which is what makes this a second executor
    /// rather than a second definition.
    ///
    /// `host` is the matrix label an execution runs as, not the operating system's own name.
    /// It is a value rather than a detection so that a person can ask what the other leg
    /// would do, and so that this crate's own tests can exercise both legs on either
    /// machine.
    Steps
    {
        root: PathBuf,
        host: String,
    },
}

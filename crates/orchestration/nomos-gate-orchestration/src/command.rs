//! What a `nomos gate` verb was asked for, independent of how it was spelled.
//!
//! `ARC-ROADMAP-001` names four eventual verbs -- `plan`, `run`, `explain`, `compare` -- but
//! only `Plan` has a real computation behind it today, so this is a struct rather than an
//! enum, the same shape `nomos_check_orchestration::CheckCommand` took while it, too, had
//! exactly one verb. Inventing argument shapes for `run`/`explain`/`compare` now, before any
//! of them has a real implementation to fit, would be exactly the kind of premature surface
//! this workspace has repeatedly declined to build ahead of a second real case
//! (`OD-PACKAGE-006`, `OD-RULES-005`, `OD-RULES-006`) -- so they are simply absent, not
//! stubbed, until an increment gives one of them a real body.

use std::path::PathBuf;

/// What to plan a gate over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateCommand
{
    /// The tree a gate would run against.
    ///
    /// Accepted and carried, not yet read: this increment's [`crate::GatePlan`] does not
    /// vary by root, because no scope selection exists yet to make it vary. It is here
    /// because every later verb needs it and `nomos_check_orchestration::CheckCommand`
    /// already establishes `root` as this workspace's one settled way to name a tree, not
    /// because this increment consults it.
    pub root: PathBuf,
}

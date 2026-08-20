//! What a `nomos gate` verb was asked for, independent of how it was spelled.
//!
//! `ARC-ROADMAP-001` names four eventual verbs -- `plan`, `run`, `explain`, `compare` -- and
//! `Plan` and `Run_Gate` both have real computations behind them now, but this stays a
//! struct rather than an enum: the two share every field, and inventing argument shapes for
//! `explain`/`compare` now, before either has a real implementation to fit, would be exactly
//! the kind of premature surface this workspace has repeatedly declined to build ahead of a
//! second real case (`OD-PACKAGE-006`, `OD-RULES-005`, `OD-RULES-006`) -- so they are simply
//! absent, not stubbed, until an increment gives one of them a real body.

use crate::{RuleSelector, ScopeSelector};
use std::path::PathBuf;

/// What to plan or run a gate over.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GateCommand
{
    /// The tree a gate would run against.
    ///
    /// Accepted and carried, not yet read by [`crate::GatePlan`]: it does not vary by root,
    /// because no scope selection composes it yet. `Run_Gate` does read it -- the tree it
    /// judges. It is here because every verb needs it and
    /// `nomos_check_orchestration::CheckCommand` already establishes `root` as this
    /// workspace's one settled way to name a tree.
    pub root: PathBuf,
    /// Which files under `root` a real run judges. Read by [`crate::Run_Gate`], not by
    /// [`crate::GatePlan`] -- the same asymmetry `root` already has, for the same reason:
    /// `Plan` reports the registry, not a walk, so nothing about a file selection applies
    /// to it yet.
    pub scope: ScopeSelector,
    /// Which rules' findings count toward a real run's disposition. Read by
    /// [`crate::Run_Gate`] only, the same asymmetry as `scope`.
    pub rules: RuleSelector,
}

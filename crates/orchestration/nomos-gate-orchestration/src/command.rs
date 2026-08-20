//! What a `nomos gate` verb was asked for, independent of how it was spelled.
//!
//! `ARC-ROADMAP-001` names four eventual verbs -- `plan`, `run`, `explain`, `compare` -- and
//! `Plan`, `Run_Gate` and `Explain_Gate` all have real computations behind them now.
//! `explain` needs one more input `plan`/`run` do not, `crate::FindingQuery`, which travels
//! as `Explain_Gate`'s own separate parameter rather than a field here: every verb shares
//! `GateCommand`, `explain` alone also needs to name which finding, and folding that into
//! this struct would make every other verb carry a field it never reads, the same
//! `root`/`scope`/`rules`/`suppressions`/`baseline` asymmetry this struct already has for
//! `Plan`. This
//! stays a struct rather than an enum for the same reason it always has: the verbs share
//! every field of it, so there is nothing for an enum to gain. `compare` is the one verb
//! left with no real implementation -- inventing an argument shape for it now would be
//! exactly the kind of premature surface this workspace has repeatedly declined to build
//! ahead of a second real case (`OD-PACKAGE-006`, `OD-RULES-005`, `OD-RULES-006`) -- so it
//! is simply absent, not stubbed, until an increment gives it a real body.

use crate::{BaselinePolicy, RuleSelector, ScopeSelector, SuppressionPolicy};
use std::path::PathBuf;

/// What to plan, run or explain a gate over.
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
    /// Which findings a real run must not let fail the build, despite `Finding::
    /// Can_Fail_A_Build`. Read by [`crate::Run_Gate`] only, the same asymmetry as `scope`
    /// and `rules`. Nothing constructs a non-empty one yet -- see
    /// [`crate::SuppressionPolicy`]'s own doc for what authors one, and what does not yet.
    pub suppressions: SuppressionPolicy,
    /// Existing debt a real run must not let fail the build either, checked after
    /// `suppressions` so a finding matched by both reports as suppressed. Read by
    /// [`crate::Run_Gate`] only, the same asymmetry as `scope`, `rules` and `suppressions`.
    /// Nothing constructs a non-empty one yet -- see [`crate::BaselinePolicy`]'s own doc for
    /// what authors one, and what does not yet.
    pub baseline: BaselinePolicy,
}

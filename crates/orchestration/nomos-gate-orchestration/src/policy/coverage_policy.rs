//! Whether unsupported or unanalyzed scope should keep a real run from reporting a
//! disposition it did not actually reach a judgment about.

/// Whether `Run_Gate` should let coverage debt over the rule-and-scope-selected findings
/// affect a run's disposition, beyond riding through [`crate::GateRunResult::check_outcome`]
/// for information only.
///
/// `OD-GATE-016`'s own first increment: a repository that never sets this is in exactly the
/// state `OD-COMPLETENESS-004` already settled for `nomos check`'s own exit code -- coverage
/// debt reported, not gated on -- so `Default` gives [`Self::Unset`] and every construction
/// site that predates this type is unchanged in behavior, the same guarantee
/// [`crate::SuppressionPolicy`], [`crate::BaselinePolicy`] and [`crate::AdoptionPolicy`]
/// each make for their own default. [`Self::Unset`] is also what [`crate::Run_Gate`] reads
/// as "take this from the `nomos-gate.json` under the run's root, if there is one"; CI's own
/// `gate run --root .` is unchanged because this repository declares no such file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CoveragePolicy
{
    /// Coverage debt does not affect disposition. `OD-COMPLETENESS-004`'s own default,
    /// carried forward.
    #[default]
    Unset,
    /// A disposition this run would otherwise report [`crate::GateRunOutcome::Passed`] is
    /// reported [`crate::GateRunOutcome::Indeterminate`] instead whenever recomputing
    /// `Claim` over the rule-and-scope-selected findings finds it incomplete -- `OD-GATE-016`'s
    /// own decision. Deliberately does not touch a run that would otherwise report `Failed`:
    /// a blocking finding this run did reach a judgment about is not made any less true by a
    /// different subject the run could not judge, so there is nothing for this variant to
    /// downgrade in that case, and `Failed` already is not a silent `Passed`.
    RequireCompleteness,
}

#[cfg(test)]
mod tests
{
    use super::CoveragePolicy;

    #[test]
    fn Test_Default_Should_Be_Unset()
    {
        assert_eq!(CoveragePolicy::default(), CoveragePolicy::Unset);
    }
}

//! Which declared target a declared [`ModelExecutionProfile`] resolves to, against a package
//! set the caller declares rather than one this resolver discovers.
//!
//! `OD-PACKAGE-016` decides this and `P40-MODEL-ROUTING-RESOLVER` built it. Every piece but
//! the map already existed: `ModelExecutionProfile` names a selector and an effort, and
//! [`DispatchConfig`] names an effort and what answers. This module is the map between them
//! and nothing else.
//!
//! # What a target is, since the port
//!
//! `OD-ROADMAP-005` decision 2 moved `DeclaredTarget` into `nomos-agent-contracts` and gave
//! it a port. Before that it named a two-variant `Backend` enum declared in this crate, so
//! resolving meant choosing between two vendors this crate had listed; `OD-PACKAGE-016`
//! decision 1 put the resolver here for exactly that reason -- this was the one crate
//! reaching both backend crates. It reaches neither now. A target arrives carrying its own
//! family label and the port that answers it, and resolving means matching a label and handing
//! back what was already there.
//!
//! # What resolves, and what does not
//!
//! [`ModelSelector::BackendFamily`] is the one variant that resolves, because it is the one
//! whose answer this workspace already holds: a declared target states its own family label,
//! so a family selector is answered by a declaration rather than by a catalog nothing has
//! written.
//!
//! The other four are unresolved by named reason rather than stubbed. `ModelSelector`'s own
//! doc states why: every one of them carries a raw, unresolved string, and there is no live
//! provider, catalog or entitlement system anywhere in this workspace to resolve an identity,
//! a set, a predicate or a ranking against (`OD-PACKAGE-011`). Naming that absence as a value
//! rather than as prose is what makes a later change to one of them a decision -- the match in
//! [`Resolve_Profile`] has no wildcard, so a sixth selector variant does not compile until
//! somebody says what it resolves to.
//!
//! Maps no effort onto a backend's native level, which is the backend's own job
//! (`MODEL-ROUTE-004`). Ranks nothing: where more than one declared target satisfies a
//! selector, the first declared answers and every other one comes back beside it, because a
//! ranking invented here would be a judgement no record makes. Builds no
//! `ResolvedModelExecution`, which `MODEL-ROUTE-029` conditions on an accounting that is not
//! met.

use nomos_agent_contracts::DeclaredTarget;
use nomos_model_package::{ModelExecutionProfile, ModelSelector};

use crate::DispatchConfig;

/// Why a profile did not resolve against the declared set.
///
/// Every variant is a measured absence rather than free text, so a caller can branch on it and
/// a test can assert it. Only the first is measured from the declared set itself; the other
/// four are measurements about this workspace's declared vocabulary, which `OD-PACKAGE-011`
/// states: a raw identity, set, predicate or ranking is the *shape* of a choice a profile
/// author writes, not an answer anything in this workspace resolves. A caller supplying a
/// catalog does not change that -- a catalog of raw identifiers is not a resolved identity,
/// which is the same reading `OD-EXECUTOR-008` refused in the other direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileAbsence
{
    /// No declared target answers to the selector's family name.
    NoDeclaredTargetOfThatFamily,
    /// [`ModelSelector::ExactIdentity`] names a raw identity, and nothing here resolves one.
    NoModelIdentityVocabulary,
    /// [`ModelSelector::AllowedSet`] names raw identities, and nothing here resolves a set.
    NoModelSetVocabulary,
    /// [`ModelSelector::CapabilityPredicate`] names a raw expression, and nothing evaluates
    /// one.
    NoPredicateVocabulary,
    /// [`ModelSelector::PolicyRankedCandidates`] needs a policy that ranks candidates, and
    /// this workspace has no such policy to consult.
    NoRankingVocabulary,
}

/// What [`Resolve_Profile`] answers: the resolved config, or the selector that did not resolve
/// together with the absence measured for it. Two variants, and no third shape, no `Option`
/// and no `bool`.
///
/// The resolved arm carries the config a dispatch takes as it stands, and the family label of
/// every other declared target that satisfied the same selector in the order the caller
/// declared them -- never a ranking, which is what lets a caller see what its own choice was
/// chosen over.
///
/// Carries `Clone`, and neither `Debug` nor `PartialEq`, because the resolved arm holds a
/// [`DispatchConfig`] and that type carries neither: a port is a trait object, and two trait
/// objects have no meaningful equality. A caller or a test reads a result by matching rather
/// than by asserting equality.
#[derive(Clone)]
pub enum ProfileResolution<'port>
{
    /// The profile resolved against the declared set.
    Resolved
    {
        /// The effort the profile asked for, carried unchanged, and what answers it.
        config: DispatchConfig<'port>,
        /// The family label of every other declared target that satisfied the same selector,
        /// in the order the caller declared them.
        also_satisfied: Vec<&'port str>,
    },
    /// The profile did not resolve.
    Unresolved
    {
        /// The selector that did not resolve, as the profile stated it.
        selector: ModelSelector,
        /// Why, as a value rather than a sentence.
        absence: ProfileAbsence,
    },
}

/// Resolves `profile` against `declared`.
///
/// The only input beyond the profile is `declared`: this reads no file, discovers no package
/// and consults no environment variable.
///
/// The match below has no wildcard on purpose. A sixth [`ModelSelector`] variant is a compile
/// error here until somebody decides what it resolves to, which is the whole point of
/// answering the other four with named absences rather than with a catch-all.
#[must_use]
pub fn Resolve_Profile<'port>(
    profile: &ModelExecutionProfile, declared: &'port [DeclaredTarget<'port>],
) -> ProfileResolution<'port>
{
    return match &profile.selector
    {
        ModelSelector::BackendFamily(family) => Resolve_Family(profile, family, declared),
        ModelSelector::ExactIdentity { .. } => Unresolved(profile, ProfileAbsence::NoModelIdentityVocabulary),
        ModelSelector::AllowedSet(_) => Unresolved(profile, ProfileAbsence::NoModelSetVocabulary),
        ModelSelector::CapabilityPredicate(_) => Unresolved(profile, ProfileAbsence::NoPredicateVocabulary),
        ModelSelector::PolicyRankedCandidates(_) => Unresolved(profile, ProfileAbsence::NoRankingVocabulary),
    };
}

/// [`Resolve_Profile`]'s one resolving arm: the declared targets answering to `family`.
///
/// A family selector can be satisfied by more than one declared target, because the same
/// family may be declared by more than one package. The others come back anyway rather than
/// being deduplicated, because deciding that a repeated declaration is not worth reporting is
/// a judgement this module does not make.
fn Resolve_Family<'port>(
    profile: &ModelExecutionProfile, family: &str, declared: &'port [DeclaredTarget<'port>],
) -> ProfileResolution<'port>
{
    let mut answers = Targets_Of_Family(family, declared);

    if answers.is_empty()
    {
        return Unresolved(profile, ProfileAbsence::NoDeclaredTargetOfThatFamily);
    }

    let answering = answers.remove(0);
    let config =
        DispatchConfig { effort: profile.effort, family: answering.family.as_str(), port: answering.port };

    return Resolved(config, answers.into_iter().map(|target| return target.family.as_str()).collect());
}

/// Every declared target answering to `family`, in the order the caller declared them.
fn Targets_Of_Family<'port>(family: &str, declared: &'port [DeclaredTarget<'port>]) -> Vec<&'port DeclaredTarget<'port>>
{
    let mut answers: Vec<&DeclaredTarget<'_>> = Vec::new();

    for target in declared
    {
        if target.family == family
        {
            answers.push(target);
        }
    }

    return answers;
}

/// A resolved result.
fn Resolved<'port>(config: DispatchConfig<'port>, also_satisfied: Vec<&'port str>) -> ProfileResolution<'port>
{
    return ProfileResolution::Resolved { config, also_satisfied };
}

/// An unresolved result, carrying `profile`'s own selector rather than a paraphrase of it.
fn Unresolved<'port>(profile: &ModelExecutionProfile, absence: ProfileAbsence) -> ProfileResolution<'port>
{
    return ProfileResolution::Unresolved { selector: profile.selector.clone(), absence };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{Declared_Executor, Declared_Model, RefusingExecutor, RefusingModel};
    use nomos_model_package::EffortLevel;

    fn Profile(selector: ModelSelector, effort: EffortLevel) -> ModelExecutionProfile
    {
        return ModelExecutionProfile::New(selector, effort);
    }

    /// The absence the resolver reported, or a panic naming what it resolved to instead.
    fn Absence_Of(profile: &ModelExecutionProfile, declared: &[DeclaredTarget<'_>]) -> ProfileAbsence
    {
        return match Resolve_Profile(profile, declared)
        {
            ProfileResolution::Unresolved { absence, .. } => absence,
            ProfileResolution::Resolved { .. } => panic!("expected an absence, and it resolved"),
        };
    }

    #[test]
    fn Test_A_Declared_Family_Should_Resolve_With_The_Profiles_Effort_Unchanged()
    {
        let executor = RefusingExecutor;
        let declared = [Declared_Executor("acme-agent", &executor)];
        let profile = Profile(ModelSelector::BackendFamily("acme-agent".to_owned()), EffortLevel::High);

        match Resolve_Profile(&profile, &declared)
        {
            ProfileResolution::Resolved { config, also_satisfied } =>
            {
                assert_eq!(config.family, "acme-agent");
                assert_eq!(config.effort, EffortLevel::High, "the effort is carried, never mapped");
                assert!(also_satisfied.is_empty(), "one declared target satisfied the selector");
            }
            ProfileResolution::Unresolved { absence, .. } => panic!("expected a resolution, got {absence:?}"),
        }
    }

    /// The other family answers the other target, so a resolution reads the label rather than
    /// the position the target arrived in.
    #[test]
    fn Test_The_Other_Declared_Family_Should_Resolve_To_The_Other_Target()
    {
        let executor = RefusingExecutor;
        let model = RefusingModel;
        let declared = [Declared_Executor("acme-agent", &executor), Declared_Model("acme-model", &model)];
        let profile = Profile(ModelSelector::BackendFamily("acme-model".to_owned()), EffortLevel::Low);

        match Resolve_Profile(&profile, &declared)
        {
            ProfileResolution::Resolved { config, .. } => assert_eq!(config.family, "acme-model"),
            ProfileResolution::Unresolved { absence, .. } => panic!("expected a resolution, got {absence:?}"),
        }
    }

    /// Nothing is ranked. The first declared target satisfying the selector answers, and the
    /// rest come back beside it in the order the caller declared them.
    #[test]
    fn Test_Every_Other_Satisfying_Target_Should_Come_Back_In_Declared_Order()
    {
        let executor = RefusingExecutor;
        let model = RefusingModel;
        let declared = [
            Declared_Executor("acme-agent", &executor),
            Declared_Model("acme-model", &model),
            Declared_Executor("acme-agent", &executor),
        ];
        let profile = Profile(ModelSelector::BackendFamily("acme-agent".to_owned()), EffortLevel::Low);

        match Resolve_Profile(&profile, &declared)
        {
            ProfileResolution::Resolved { config, also_satisfied } =>
            {
                assert_eq!(config.family, "acme-agent", "the first declared one answers");
                assert_eq!(also_satisfied, vec!["acme-agent"], "and the later one is not dropped");
            }
            ProfileResolution::Unresolved { absence, .. } => panic!("expected a resolution, got {absence:?}"),
        }
    }

    #[test]
    fn Test_A_Family_No_Target_Declares_Should_Not_Resolve()
    {
        let model = RefusingModel;
        let declared = [Declared_Model("acme-model", &model)];
        let profile = Profile(ModelSelector::BackendFamily("acme-agent".to_owned()), EffortLevel::Low);

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoDeclaredTargetOfThatFamily);
    }

    /// The four selectors that do not resolve, one test each, so that a later change to any of
    /// them is a decision rather than a drift.
    #[test]
    fn Test_An_Exact_Identity_Should_Not_Resolve_And_Should_Name_Its_Absence()
    {
        let executor = RefusingExecutor;
        let declared = [Declared_Executor("acme-agent", &executor)];
        let profile = Profile(
            ModelSelector::ExactIdentity { identity: "acme-large".to_owned(), pinned: true },
            EffortLevel::Low,
        );

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoModelIdentityVocabulary);
    }

    #[test]
    fn Test_An_Allowed_Set_Should_Not_Resolve_And_Should_Name_Its_Absence()
    {
        let executor = RefusingExecutor;
        let declared = [Declared_Executor("acme-agent", &executor)];
        let profile = Profile(ModelSelector::AllowedSet(vec!["acme-large".to_owned()]), EffortLevel::Low);

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoModelSetVocabulary);
    }

    #[test]
    fn Test_A_Capability_Predicate_Should_Not_Resolve_And_Should_Name_Its_Absence()
    {
        let executor = RefusingExecutor;
        let declared = [Declared_Executor("acme-agent", &executor)];
        let profile = Profile(ModelSelector::CapabilityPredicate("context >= 200k".to_owned()), EffortLevel::Low);

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoPredicateVocabulary);
    }

    #[test]
    fn Test_Policy_Ranked_Candidates_Should_Not_Resolve_And_Should_Name_Its_Absence()
    {
        let executor = RefusingExecutor;
        let declared = [Declared_Executor("acme-agent", &executor)];
        let profile =
            Profile(ModelSelector::PolicyRankedCandidates(vec!["acme-large".to_owned()]), EffortLevel::Low);

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoRankingVocabulary);
    }

    /// An unresolved result names the selector it was handed rather than a paraphrase of it.
    #[test]
    fn Test_An_Unresolved_Result_Should_Carry_The_Selector_That_Did_Not_Resolve()
    {
        let executor = RefusingExecutor;
        let declared = [Declared_Executor("acme-agent", &executor)];
        let profile = Profile(ModelSelector::AllowedSet(vec!["acme-large".to_owned()]), EffortLevel::Low);

        match Resolve_Profile(&profile, &declared)
        {
            ProfileResolution::Unresolved { selector, .. } => assert_eq!(selector, profile.selector),
            ProfileResolution::Resolved { .. } => panic!("expected an absence, and it resolved"),
        }
    }
}

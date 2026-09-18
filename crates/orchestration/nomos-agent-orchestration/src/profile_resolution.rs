//! Which backend a declared [`ModelExecutionProfile`] resolves to, against a package set the
//! caller declares rather than one this resolver discovers.
//!
//! `OD-PACKAGE-016` decides this and `P40-MODEL-ROUTING-RESOLVER` builds it. Every piece but
//! the map already existed: `ModelExecutionProfile` names a selector and an effort,
//! `DispatchConfig` names an effort and a backend, and [`Backend`]'s own doc already calls
//! itself the resolved choice. This module is the map between them and nothing else.
//!
//! # What resolves, and what does not
//!
//! [`ModelSelector::BackendFamily`] is the one variant that resolves, because it is the one
//! whose answer this workspace already holds: [`Backend::Label`] returns the family name a
//! person types, so a declared target answers a family selector by its own label rather than
//! by a catalog nothing has written.
//!
//! The other four are unresolved by named reason rather than stubbed. `ModelSelector`'s own
//! doc states why: every one of them carries a raw, unresolved string, and there is no live
//! provider, catalog or entitlement system anywhere in this workspace to resolve an identity,
//! a set, a predicate or a ranking against (`OD-PACKAGE-011`). Naming that absence as a value
//! rather than as prose is what makes a later change to one of them a decision -- the match in
//! [`Resolve_Profile`] has no wildcard, so a sixth selector variant does not compile until
//! somebody says what it resolves to.
//!
//! # What this module deliberately does not do
//!
//! Reads no file, discovers no package and consults no environment variable. A resolver that
//! walked the tree for packages would be a second place that decides which packages exist,
//! the division `nomos_capability::Registry` already draws and `OD-HOST-002` states. Adds no
//! variant to `ModelSelector`, to `ModelSelection` or to the manifest reader --
//! [`Backend::Label`] is the only addition to existing vocabulary `OD-PACKAGE-016` authorizes.
//! Maps no effort onto a backend's native level, which is the backend's own job
//! (`MODEL-ROUTE-004`). Ranks nothing: where more than one declared target satisfies a
//! selector, the first declared answers and every other one comes back beside it, because a
//! ranking invented here would be a judgement no record makes. Builds no
//! `ResolvedModelExecution`, which `MODEL-ROUTE-029` conditions on an accounting that is not
//! met.
//!
//! Wiring a dispatch to call this is a separate item with territory this one does not reserve
//! -- `OD-PACKAGE-016` decision 9 makes it a second item rather than a widening of this one.

use crate::{Backend, DispatchConfig};
use nomos_model_package::{ModelExecutionProfile, ModelRoutePackage, ModelSelector};

/// One dispatch target and the package that declares it a routing target.
///
/// The pairing is the whole of the input beyond the profile: a target is available to this
/// resolver because a caller declared it, never because a package was found. `package` is
/// that declaration -- `ModelRoutePackage`'s own reader already restricts `package_kind` to
/// the two kinds that declare a model selection, so a value here is one of those by
/// construction and needs no second check here. Its `model_selection` is not read by the one
/// selector that resolves today; it is what the four selectors that do not resolve would
/// eventually be answered against, which is why it is carried rather than dropped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredTarget
{
    /// The backend a dispatch would reach.
    pub backend: Backend,
    /// The package that declares this target.
    pub package: ModelRoutePackage,
}

/// Why a profile did not resolve against the declared set.
///
/// Every variant is a measured absence rather than free text, so a caller can branch on it and
/// a test can assert it. Only the first is measured from the declared set itself; the other
/// four are measurements about this workspace's declared vocabulary, which `OD-PACKAGE-011`
/// states: a raw identity, set, predicate or ranking is the *shape* of a choice a profile
/// author writes, not an answer anything in this workspace resolves. A caller-supplying a
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
/// The resolved arm carries the config a dispatch takes as it stands, and every other declared
/// target that satisfied the same selector in the order the caller declared them -- never a
/// ranking, which is what lets a caller see what its own choice was chosen over.
///
/// Carries `Clone`, and neither `Debug` nor `PartialEq`, because the resolved arm holds a
/// [`DispatchConfig`] and that type carries neither. Widening `DispatchConfig` is a change to
/// a file this item's territory does not reach, so a caller or a test reads a result by
/// matching rather than by asserting equality.
#[derive(Clone)]
pub enum ProfileResolution
{
    /// The profile resolved against the declared set.
    Resolved
    {
        /// The effort the profile asked for, carried unchanged, and the backend that answers.
        config: DispatchConfig,
        /// Every other declared target that satisfied the same selector, in the order the
        /// caller declared them.
        also_satisfied: Vec<Backend>,
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
pub fn Resolve_Profile(profile: &ModelExecutionProfile, declared: &[DeclaredTarget]) -> ProfileResolution
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
/// A family selector can be satisfied by more than one declared target only because the same
/// backend may be declared by more than one package -- this workspace's two backends have two
/// distinct labels, so one family names one backend. The others come back anyway rather than
/// being deduplicated, because deciding that a repeated declaration is not worth reporting is
/// a judgement this module does not make.
fn Resolve_Family(profile: &ModelExecutionProfile, family: &str, declared: &[DeclaredTarget]) -> ProfileResolution
{
    let mut answers = Targets_Of_Family(family, declared);

    if answers.is_empty()
    {
        return Unresolved(profile, ProfileAbsence::NoDeclaredTargetOfThatFamily);
    }

    let backend = answers.remove(0);

    return Resolved(DispatchConfig { effort: profile.effort, backend }, answers);
}

/// Every declared target answering to `family`, in the order the caller declared them.
fn Targets_Of_Family(family: &str, declared: &[DeclaredTarget]) -> Vec<Backend>
{
    let mut answers: Vec<Backend> = Vec::new();

    for target in declared
    {
        if target.backend.Label() == family
        {
            answers.push(target.backend);
        }
    }

    return answers;
}

/// A resolved result.
fn Resolved(config: DispatchConfig, also_satisfied: Vec<Backend>) -> ProfileResolution
{
    return ProfileResolution::Resolved { config, also_satisfied };
}

/// An unresolved result, carrying `profile`'s own selector rather than a paraphrase of it.
fn Unresolved(profile: &ModelExecutionProfile, absence: ProfileAbsence) -> ProfileResolution
{
    return ProfileResolution::Unresolved { selector: profile.selector.clone(), absence };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{ContractVersion, PackageId, PackageKind};
    use nomos_model_package::{EffortLevel, ModelSelection, PackageVersion, ProtocolRange};

    /// A declared target for `backend`, paired with a package of one of the two routing kinds
    /// -- `ModelRoutePackage`'s own reader restricts `package_kind` to these, so a fixture of
    /// any other kind is a value the real path cannot produce.
    fn Target(backend: Backend) -> DeclaredTarget
    {
        return DeclaredTarget { backend, package: Package(backend.Label()) };
    }

    fn Package(id: &str) -> ModelRoutePackage
    {
        return ModelRoutePackage {
            package_id: PackageId::New(id),
            package_kind: PackageKind::ModelBackendPackage,
            package_version: PackageVersion::New(1, 0, 0),
            protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
            model_selection: ModelSelection::Catalog(vec!["acme-large".to_owned()]),
        };
    }

    fn Profile(selector: ModelSelector, effort: EffortLevel) -> ModelExecutionProfile
    {
        return ModelExecutionProfile::New(selector, effort);
    }

    /// The absence the resolver reported, or a panic naming what it resolved to instead.
    fn Absence_Of(profile: &ModelExecutionProfile, declared: &[DeclaredTarget]) -> ProfileAbsence
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
        let declared = [Target(Backend::ClaudeCode)];
        let profile = Profile(ModelSelector::BackendFamily("claude-code".to_owned()), EffortLevel::High);

        match Resolve_Profile(&profile, &declared)
        {
            ProfileResolution::Resolved { config, also_satisfied } =>
            {
                assert_eq!(config.backend, Backend::ClaudeCode);
                assert_eq!(config.effort, EffortLevel::High, "the effort is carried, never mapped");
                assert!(also_satisfied.is_empty(), "one declared target satisfied the selector");
            }
            ProfileResolution::Unresolved { absence, .. } => panic!("expected a resolution, got {absence:?}"),
        }
    }

    /// The other backend's family answers the other backend, so a resolution reads the label
    /// rather than the position the target arrived in.
    #[test]
    fn Test_The_Other_Declared_Family_Should_Resolve_To_The_Other_Backend()
    {
        let declared = [Target(Backend::ClaudeCode), Target(Backend::Ollama)];
        let profile = Profile(ModelSelector::BackendFamily("ollama".to_owned()), EffortLevel::Low);

        match Resolve_Profile(&profile, &declared)
        {
            ProfileResolution::Resolved { config, .. } => assert_eq!(config.backend, Backend::Ollama),
            ProfileResolution::Unresolved { absence, .. } => panic!("expected a resolution, got {absence:?}"),
        }
    }

    /// Nothing is ranked. The first declared target satisfying the selector answers, and the
    /// rest come back beside it in the order the caller declared them.
    #[test]
    fn Test_Every_Other_Satisfying_Target_Should_Come_Back_In_Declared_Order()
    {
        let declared = [
            DeclaredTarget { backend: Backend::ClaudeCode, package: Package("first") },
            DeclaredTarget { backend: Backend::Ollama, package: Package("other") },
            DeclaredTarget { backend: Backend::ClaudeCode, package: Package("second") },
        ];
        let profile = Profile(ModelSelector::BackendFamily("claude-code".to_owned()), EffortLevel::Low);

        match Resolve_Profile(&profile, &declared)
        {
            ProfileResolution::Resolved { config, also_satisfied } =>
            {
                assert_eq!(config.backend, Backend::ClaudeCode, "the first declared one answers");
                assert_eq!(also_satisfied, vec![Backend::ClaudeCode], "and the later one is not dropped");
            }
            ProfileResolution::Unresolved { absence, .. } => panic!("expected a resolution, got {absence:?}"),
        }
    }

    #[test]
    fn Test_A_Family_No_Target_Declares_Should_Not_Resolve()
    {
        let declared = [Target(Backend::Ollama)];
        let profile = Profile(ModelSelector::BackendFamily("claude-code".to_owned()), EffortLevel::Low);

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoDeclaredTargetOfThatFamily);
    }

    /// The four selectors that do not resolve, one test each, so that a later change to any of
    /// them is a decision rather than a drift.
    #[test]
    fn Test_An_Exact_Identity_Should_Not_Resolve_And_Should_Name_Its_Absence()
    {
        let declared = [Target(Backend::ClaudeCode)];
        let profile = Profile(
            ModelSelector::ExactIdentity { identity: "acme-large".to_owned(), pinned: true },
            EffortLevel::Low,
        );

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoModelIdentityVocabulary);
    }

    #[test]
    fn Test_An_Allowed_Set_Should_Not_Resolve_And_Should_Name_Its_Absence()
    {
        let declared = [Target(Backend::ClaudeCode)];
        let profile = Profile(ModelSelector::AllowedSet(vec!["acme-large".to_owned()]), EffortLevel::Low);

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoModelSetVocabulary);
    }

    #[test]
    fn Test_A_Capability_Predicate_Should_Not_Resolve_And_Should_Name_Its_Absence()
    {
        let declared = [Target(Backend::ClaudeCode)];
        let profile = Profile(ModelSelector::CapabilityPredicate("context >= 200k".to_owned()), EffortLevel::Low);

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoPredicateVocabulary);
    }

    #[test]
    fn Test_Policy_Ranked_Candidates_Should_Not_Resolve_And_Should_Name_Its_Absence()
    {
        let declared = [Target(Backend::ClaudeCode)];
        let profile =
            Profile(ModelSelector::PolicyRankedCandidates(vec!["acme-large".to_owned()]), EffortLevel::Low);

        assert_eq!(Absence_Of(&profile, &declared), ProfileAbsence::NoRankingVocabulary);
    }

    /// An unresolved result names the selector it was handed rather than a paraphrase of it.
    #[test]
    fn Test_An_Unresolved_Result_Should_Carry_The_Selector_That_Did_Not_Resolve()
    {
        let declared = [Target(Backend::ClaudeCode)];
        let profile = Profile(ModelSelector::AllowedSet(vec!["acme-large".to_owned()]), EffortLevel::Low);

        match Resolve_Profile(&profile, &declared)
        {
            ProfileResolution::Unresolved { selector, .. } => assert_eq!(selector, profile.selector),
            ProfileResolution::Resolved { .. } => panic!("expected an absence, and it resolved"),
        }
    }
}

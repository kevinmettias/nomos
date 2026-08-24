use serde::{Deserialize, Serialize};

use super::{AuthorityClass, MutationClass};
use crate::BuildVariantId;

const LOCAL_IMPLEMENTATION_LABEL: &str = "LocalImplementation";
const CONTRACT_COMPATIBLE_EXTENSION_LABEL: &str = "ContractCompatibleExtension";
const PUBLIC_INTERFACE_CHANGE_LABEL: &str = "PublicInterfaceChange";
const DEPENDENCY_BOUNDARY_CHANGE_LABEL: &str = "DependencyBoundaryChange";
const ARCHITECTURE_TOPOLOGY_CHANGE_LABEL: &str = "ArchitectureTopologyChange";
const BEHAVIORAL_DEFAULT_CHANGE_LABEL: &str = "BehavioralDefaultChange";
const CORRECTNESS_OR_SAFETY_OBLIGATION_CHANGE_LABEL: &str = "CorrectnessOrSafetyObligationChange";
const POLICY_OR_CONFIGURATION_CHANGE_LABEL: &str = "PolicyOrConfigurationChange";
const RULE_CHANGE_LABEL: &str = "RuleChange";
const GENERATED_ARTIFACT_CHANGE_LABEL: &str = "GeneratedArtifactChange";
const DESTRUCTIVE_REMOVAL_LABEL: &str = "DestructiveRemoval";

/// `AGT-018`: "Nomos shall classify \[an agent-originated mutation's\] predicted effects
/// into `SemanticChangeClass` values including at minimum local implementation,
/// contract-compatible extension, public-interface change, dependency-boundary change,
/// architecture-topology change, behavioral-default change, correctness or safety
/// obligation change, policy or configuration change, rule change, generated-artifact
/// change, and destructive removal."
///
/// Eleven variants, in the corpus's own order. Distinct from [`AuthorityClass`] (who may
/// invoke an operation) and [`MutationClass`] (what an operation does to workspace
/// state, `Preview -> Validate -> Apply -> Rollback`): this classifies what an
/// agent-originated change *means*, the axis `SemanticChangeAuthorityResolution`
/// resolves an authority and mode against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticChangeClass
{
    LocalImplementation,
    ContractCompatibleExtension,
    PublicInterfaceChange,
    DependencyBoundaryChange,
    ArchitectureTopologyChange,
    BehavioralDefaultChange,
    CorrectnessOrSafetyObligationChange,
    PolicyOrConfigurationChange,
    RuleChange,
    GeneratedArtifactChange,
    DestructiveRemoval,
}

impl SemanticChangeClass
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::LocalImplementation => LOCAL_IMPLEMENTATION_LABEL,
            Self::ContractCompatibleExtension => CONTRACT_COMPATIBLE_EXTENSION_LABEL,
            Self::PublicInterfaceChange => PUBLIC_INTERFACE_CHANGE_LABEL,
            Self::DependencyBoundaryChange => DEPENDENCY_BOUNDARY_CHANGE_LABEL,
            Self::ArchitectureTopologyChange => ARCHITECTURE_TOPOLOGY_CHANGE_LABEL,
            Self::BehavioralDefaultChange => BEHAVIORAL_DEFAULT_CHANGE_LABEL,
            Self::CorrectnessOrSafetyObligationChange => CORRECTNESS_OR_SAFETY_OBLIGATION_CHANGE_LABEL,
            Self::PolicyOrConfigurationChange => POLICY_OR_CONFIGURATION_CHANGE_LABEL,
            Self::RuleChange => RULE_CHANGE_LABEL,
            Self::GeneratedArtifactChange => GENERATED_ARTIFACT_CHANGE_LABEL,
            Self::DestructiveRemoval => DESTRUCTIVE_REMOVAL_LABEL,
        };
    }
}

impl core::fmt::Display for SemanticChangeClass
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

/// `AGT-018`: "Organization and repository policy shall resolve every applicable class
/// to an allowed authority level, required approver role, evidence obligations,
/// permitted mutation mode, rollback boundary, validation scope, and
/// autonomous-execution disposition. ... Nomos shall expose a user-visible projection of
/// the resolved decision that states the classified effects, governing policy, trust
/// evidence where used, required validation, approver roles, and the reason the
/// operation is autonomous, preview-only, proposal-only, review-required, or
/// prohibited."
///
/// Same glossary entry (5.6): "An immutable, policy-resolved authorization record for an
/// agent-originated or agent-assisted mutation." `governing ChangeIntent` is
/// deliberately not a field: `ChangeIntent` itself is not built anywhere in this
/// workspace, so naming it here would type a field against an identity that does not
/// exist yet rather than reuse one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticChangeAuthorityResolution
{
    pub predicted_class: SemanticChangeClass,
    pub observed_class: Option<SemanticChangeClass>,
    pub required_authority: AuthorityClass,
    pub approver_role: Option<String>,
    pub permitted_mutation_mode: MutationClass,
    pub autonomous_execution: bool,
    pub evidence_obligations: Vec<String>,
    pub rollback_boundary: Option<String>,
    pub unresolved_effects: Vec<String>,
    pub scope_expansion: bool,
    pub decision_rationale: String,
    pub affected_build_variants: Vec<BuildVariantId>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL: [SemanticChangeClass; 11] = [
        SemanticChangeClass::LocalImplementation,
        SemanticChangeClass::ContractCompatibleExtension,
        SemanticChangeClass::PublicInterfaceChange,
        SemanticChangeClass::DependencyBoundaryChange,
        SemanticChangeClass::ArchitectureTopologyChange,
        SemanticChangeClass::BehavioralDefaultChange,
        SemanticChangeClass::CorrectnessOrSafetyObligationChange,
        SemanticChangeClass::PolicyOrConfigurationChange,
        SemanticChangeClass::RuleChange,
        SemanticChangeClass::GeneratedArtifactChange,
        SemanticChangeClass::DestructiveRemoval,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|class| return class.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two classes share a wire spelling");
    }

    #[test]
    fn Test_A_Resolution_Carries_Exactly_What_It_Was_Given()
    {
        let resolution = SemanticChangeAuthorityResolution {
            predicted_class: SemanticChangeClass::LocalImplementation,
            observed_class: None,
            required_authority: AuthorityClass::Mutate,
            approver_role: None,
            permitted_mutation_mode: MutationClass::Apply,
            autonomous_execution: true,
            evidence_obligations: vec![],
            rollback_boundary: None,
            unresolved_effects: vec![],
            scope_expansion: false,
            decision_rationale: "no cross-boundary effect predicted".to_owned(),
            affected_build_variants: vec![],
        };

        assert_eq!(resolution.predicted_class, SemanticChangeClass::LocalImplementation);
        assert_eq!(resolution.required_authority, AuthorityClass::Mutate);
        assert!(resolution.autonomous_execution);
    }
}

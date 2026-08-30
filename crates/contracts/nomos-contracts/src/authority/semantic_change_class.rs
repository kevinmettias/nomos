// The policy-resolved decision this classification feeds, kept in its own file.
mod semantic_change_authority_resolution;

pub use semantic_change_authority_resolution::SemanticChangeAuthorityResolution;

use serde::{Deserialize, Serialize};

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
/// Eleven variants, in the corpus's own order. Distinct from [`crate::AuthorityClass`]
/// (who may invoke an operation) and [`crate::MutationClass`] (what an operation does to
/// workspace state, `Preview -> Validate -> Apply -> Rollback`): this classifies what an
/// agent-originated change *means*, the axis [`SemanticChangeAuthorityResolution`]
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
    fn Test_Label_Should_Be_Distinct_Per_Variant()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|class| return class.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two classes share a wire spelling");
    }
}

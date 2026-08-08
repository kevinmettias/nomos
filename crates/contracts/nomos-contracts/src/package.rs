//! The kinds of thing that can be installed.

use serde::{Deserialize, Serialize};

const KERNEL_MODULE_LABEL: &str = "KernelModule";
const SERVICE_MODULE_LABEL: &str = "ServiceModule";
const FEATURE_MODULE_LABEL: &str = "FeatureModule";
const LANGUAGE_PACKAGE_LABEL: &str = "LanguagePackage";
const RULE_PACKAGE_LABEL: &str = "RulePackage";
const TOOL_PROVIDER_PACKAGE_LABEL: &str = "ToolProviderPackage";
const METRIC_PROVIDER_PACKAGE_LABEL: &str = "MetricProviderPackage";
const REPOSITORY_PROVIDER_PACKAGE_LABEL: &str = "RepositoryProviderPackage";
const RUNTIME_PROVIDER_PACKAGE_LABEL: &str = "RuntimeProviderPackage";
const MODEL_BACKEND_PACKAGE_LABEL: &str = "ModelBackendPackage";
const AGENT_EXECUTOR_PACKAGE_LABEL: &str = "AgentExecutorPackage";
const CLIENT_PACKAGE_LABEL: &str = "ClientPackage";
const INTEGRATION_PACKAGE_LABEL: &str = "IntegrationPackage";
const SDK_PACKAGE_LABEL: &str = "SdkPackage";
const PROJECTION_PACKAGE_LABEL: &str = "ProjectionPackage";
const FEATURE_PACK_LABEL: &str = "FeaturePack";

/// What kind of thing an installable unit is.
///
/// Sixteen kinds exist so that "plugin" does not become an undifferentiated bucket.
/// Each kind carries different activation, isolation, permission and conformance
/// semantics, and a single `Plugin` variant would force all of that into per-package
/// configuration where nothing could validate it.
///
/// This is the only such enum in the system. Nomos owns these semantics; a platform may
/// materialize a package on disk without knowing any of them, and physical installation
/// alone never establishes an effective capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PackageKind
{
    /// Core machinery that is not optional.
    KernelModule,
    /// An application service.
    ServiceModule,
    /// An optional product capability.
    FeatureModule,
    /// Recognition and canonical mapping for one language.
    LanguagePackage,
    /// Normative rules, their judgments, corrections and fixtures.
    RulePackage,
    /// An external analysis or transformation tool, wrapped behind a capability.
    ToolProviderPackage,
    /// A source of metric observations.
    MetricProviderPackage,
    /// An adapter for a repository host.
    RepositoryProviderPackage,
    /// An adapter for a runtime evidence source.
    RuntimeProviderPackage,
    /// A model inference backend.
    ModelBackendPackage,
    /// An agent execution backend.
    AgentExecutorPackage,
    /// A user-facing client.
    ClientPackage,
    /// A connection to a peer system.
    IntegrationPackage,
    /// A development kit for authoring packages.
    SdkPackage,
    /// A renderer producing artifacts from canonical records.
    ProjectionPackage,
    /// A selection manifest naming other packages. Not a deployment unit in itself.
    FeaturePack,
}

impl PackageKind
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::KernelModule => KERNEL_MODULE_LABEL,
            Self::ServiceModule => SERVICE_MODULE_LABEL,
            Self::FeatureModule => FEATURE_MODULE_LABEL,
            Self::LanguagePackage => LANGUAGE_PACKAGE_LABEL,
            Self::RulePackage => RULE_PACKAGE_LABEL,
            Self::ToolProviderPackage => TOOL_PROVIDER_PACKAGE_LABEL,
            Self::MetricProviderPackage => METRIC_PROVIDER_PACKAGE_LABEL,
            Self::RepositoryProviderPackage => REPOSITORY_PROVIDER_PACKAGE_LABEL,
            Self::RuntimeProviderPackage => RUNTIME_PROVIDER_PACKAGE_LABEL,
            Self::ModelBackendPackage => MODEL_BACKEND_PACKAGE_LABEL,
            Self::AgentExecutorPackage => AGENT_EXECUTOR_PACKAGE_LABEL,
            Self::ClientPackage => CLIENT_PACKAGE_LABEL,
            Self::IntegrationPackage => INTEGRATION_PACKAGE_LABEL,
            Self::SdkPackage => SDK_PACKAGE_LABEL,
            Self::ProjectionPackage => PROJECTION_PACKAGE_LABEL,
            Self::FeaturePack => FEATURE_PACK_LABEL,
        };
    }

    /// Whether a package of this kind may be uninstalled without disabling the product.
    ///
    /// Identity, protocols, package loading, capability resolution, configuration,
    /// evidence envelopes and security belong in core. Making every subsystem optional
    /// is not modularity, it is an unbounded configuration space in which most points
    /// are untested.
    #[must_use]
    pub const fn Is_Optional(self) -> bool
    {
        return !matches!(self, Self::KernelModule);
    }

    /// Whether a package of this kind runs code Nomos did not write.
    ///
    /// Determines whether the deployment profile's isolation requirements apply. This
    /// is a property of the kind rather than of the individual package, so a package
    /// cannot opt itself out of sandboxing by declaring itself trustworthy.
    #[must_use]
    pub const fn Hosts_Foreign_Code(self) -> bool
    {
        return matches!(
            self,
            Self::ToolProviderPackage
                | Self::MetricProviderPackage
                | Self::RepositoryProviderPackage
                | Self::RuntimeProviderPackage
                | Self::ModelBackendPackage
                | Self::AgentExecutorPackage
                | Self::IntegrationPackage
        );
    }
}

impl core::fmt::Display for PackageKind
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

    const ALL: [PackageKind; 16] = [
        PackageKind::KernelModule,
        PackageKind::ServiceModule,
        PackageKind::FeatureModule,
        PackageKind::LanguagePackage,
        PackageKind::RulePackage,
        PackageKind::ToolProviderPackage,
        PackageKind::MetricProviderPackage,
        PackageKind::RepositoryProviderPackage,
        PackageKind::RuntimeProviderPackage,
        PackageKind::ModelBackendPackage,
        PackageKind::AgentExecutorPackage,
        PackageKind::ClientPackage,
        PackageKind::IntegrationPackage,
        PackageKind::SdkPackage,
        PackageKind::ProjectionPackage,
        PackageKind::FeaturePack,
    ];

    #[test]
    fn Test_Kernel_Modules_Should_Not_Be_Optional()
    {
        assert!(!PackageKind::KernelModule.Is_Optional());

        for kind in ALL
        {
            if kind != PackageKind::KernelModule
            {
                assert!(kind.Is_Optional(), "{kind} should be optional");
            }
        }
    }

    /// A kind that hosts foreign code and is not recognized as doing so would be loaded
    /// in-process without isolation, which is the failure this predicate prevents.
    #[test]
    fn Test_Provider_Kinds_Should_Be_Recognized_As_Hosting_Foreign_Code()
    {
        assert!(PackageKind::ToolProviderPackage.Hosts_Foreign_Code());
        assert!(PackageKind::ModelBackendPackage.Hosts_Foreign_Code());
        assert!(PackageKind::AgentExecutorPackage.Hosts_Foreign_Code());
        assert!(!PackageKind::KernelModule.Hosts_Foreign_Code());
        assert!(!PackageKind::FeaturePack.Hosts_Foreign_Code());
    }

    /// A `RulePackage` is content, not a host: its judgments are data Nomos evaluates,
    /// so it does not by itself bring foreign code into the process.
    #[test]
    fn Test_Rule_And_Language_Packages_Should_Not_Host_Foreign_Code()
    {
        assert!(!PackageKind::RulePackage.Hosts_Foreign_Code());
        assert!(!PackageKind::LanguagePackage.Hosts_Foreign_Code());
    }

    #[test]
    fn Test_Labels_Should_Be_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|kind| kind.Label()).collect();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), ALL.len(), "two package kinds share a label");
    }
}

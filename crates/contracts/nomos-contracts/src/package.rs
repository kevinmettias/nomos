//! The kinds of thing that can be installed.

use serde::{Deserialize, Serialize};

const KERNEL_MODULE_LABEL: &str = "KernelModule";
const SERVICE_MODULE_LABEL: &str = "ServiceModule";
const FEATURE_MODULE_LABEL: &str = "FeatureModule";
const LANGUAGE_PACKAGE_LABEL: &str = "LanguagePackage";
const RULE_PACKAGE_LABEL: &str = "RulePackage";
const TOOL_PROVIDER_LABEL: &str = "ToolProvider";
const METRIC_PROVIDER_LABEL: &str = "MetricProvider";
const REPOSITORY_PROVIDER_LABEL: &str = "RepositoryProvider";
const RUNTIME_PROVIDER_LABEL: &str = "RuntimeProvider";
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
///
/// # Every name here is transcribed, including the four that look irregular
///
/// [`PackageKind::Label`] is the serialized form. A peer reimplementing this enum in
/// another language reads the label and never sees this Rust, so a label is a protocol
/// commitment rather than a local identifier, and no name below was chosen here. All
/// sixteen are transcribed from the package taxonomy sentence of volume 03 of the
/// corpus, in the order that sentence names them. `OD-PACKAGE-002` quotes it, resolves
/// its path, and records why the corpus rather than the game plan fixes these spellings.
///
/// That taxonomy uses four suffixes and each says what the installable unit *is*:
/// `Module` for machinery this product itself ships, `Package` for a versioned bundle of
/// declarations and data, `Provider` for an implementation standing behind a capability
/// contract, and `Pack` for a manifest that only names other units. So `ToolProvider`,
/// `MetricProvider`, `RepositoryProvider` and `RuntimeProvider` carry no `Package`
/// suffix, and appending one would not regularize the taxonomy — it would move four
/// kinds into a family whose semantics they do not have, which is a provider's whole
/// distinction from the thing that ships it. `ARCH-003` and `ARCH-004` are normative and
/// name `ToolProvider` and `MetricProvider` in exactly this spelling.
///
/// Those four did carry a `Package` suffix here until `OD-PACKAGE-002`, which is the
/// mistake this note exists to stop a later reader from making again in the name of
/// consistency. `Test_Every_Kind_Should_Carry_The_Label_The_Corpus_Names` pins the
/// result, and its table is where a seventeenth kind has to be argued for.
///
/// # Nothing consumes this, and what it is waiting for
///
/// No code outside this crate names `PackageKind`, and none is written to make it look
/// used. The reason is not a forgotten call site: this workspace contains no installable
/// unit for a consumer to be about. No package manifest exists on disk, nothing installs
/// or resolves one, and `PackageId` is likewise declared and never constructed.
///
/// `ARCH-001` and `ARCH-002` require a `LanguagePackage` and a `RulePackage` to be
/// independently versioned, and `PKG-007` requires a package's own version, the Nomos
/// protocol range, the language versions and the provider versions to stay four distinct
/// domains. A Cargo `version` field is none of those and a `Cargo.toml` is not the
/// manifest `PKG-022` asks for, so nothing here is answered by changing what this
/// workspace publishes. `OD-PACKAGE-001` accepts both requirements and records that the
/// missing thing is the package rather than the attribute.
///
/// This enum gains its first consumer when something reads a declared package manifest
/// and refuses one it cannot resolve — a reader that maps a manifest to a `PackageId`, a
/// `PackageKind` and `PKG-007`'s version domains. That is the change to watch for; until
/// it lands the declaration is a protocol commitment held deliberately, which is also why
/// `OD-PACKAGE-001` recorded that four of the labels below then followed the game plan's
/// spelling rather than volume 03's. That divergence cost nothing while nobody read them
/// and had to be settled before anybody did; `OD-PACKAGE-002` settled it, and the labels
/// are volume 03's. Having no consumer is still this enum's condition, and it is
/// `OD-PACKAGE-001`'s open subject rather than something the spelling repair touched.
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
    ToolProvider,
    /// A source of metric observations.
    MetricProvider,
    /// An adapter for a repository host.
    RepositoryProvider,
    /// An adapter for a runtime evidence source.
    RuntimeProvider,
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
            Self::ToolProvider => TOOL_PROVIDER_LABEL,
            Self::MetricProvider => METRIC_PROVIDER_LABEL,
            Self::RepositoryProvider => REPOSITORY_PROVIDER_LABEL,
            Self::RuntimeProvider => RUNTIME_PROVIDER_LABEL,
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
            Self::ToolProvider
                | Self::MetricProvider
                | Self::RepositoryProvider
                | Self::RuntimeProvider
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

    /// The sixteen kinds the corpus names, in the order it names them, each paired with
    /// the label a peer actually reads.
    ///
    /// Transcribed from the package taxonomy sentence of volume 03,
    /// `03_Packages_Providers_Rules_and_Applicability.md`, which `OD-PACKAGE-002` quotes
    /// in full with its resolved path. This table is the enumeration that record fixes,
    /// so a row is a claim about the corpus and not a local preference: editing one
    /// changes what Nomos asserts the corpus says, which is an argument to have in a
    /// record rather than in a diff.
    const CORPUS_TAXONOMY: [(PackageKind, &str); 16] = [
        (PackageKind::KernelModule, "KernelModule"),
        (PackageKind::ServiceModule, "ServiceModule"),
        (PackageKind::FeatureModule, "FeatureModule"),
        (PackageKind::LanguagePackage, "LanguagePackage"),
        (PackageKind::RulePackage, "RulePackage"),
        (PackageKind::ToolProvider, "ToolProvider"),
        (PackageKind::MetricProvider, "MetricProvider"),
        (PackageKind::RepositoryProvider, "RepositoryProvider"),
        (PackageKind::RuntimeProvider, "RuntimeProvider"),
        (PackageKind::ModelBackendPackage, "ModelBackendPackage"),
        (PackageKind::AgentExecutorPackage, "AgentExecutorPackage"),
        (PackageKind::ClientPackage, "ClientPackage"),
        (PackageKind::IntegrationPackage, "IntegrationPackage"),
        (PackageKind::SdkPackage, "SdkPackage"),
        (PackageKind::ProjectionPackage, "ProjectionPackage"),
        (PackageKind::FeaturePack, "FeaturePack"),
    ];

    /// Where a kind sits in `CORPUS_TAXONOMY`.
    ///
    /// The match is exhaustive on purpose, and that is the whole mechanism for
    /// membership: a seventeenth variant makes it non-exhaustive, so this module stops
    /// compiling and whoever added the kind has to arrive here, beside the table and the
    /// record it cites, rather than adding a kind these assertions would never visit.
    const fn Corpus_Position(kind: PackageKind) -> usize
    {
        return match kind
        {
            PackageKind::KernelModule => 0,
            PackageKind::ServiceModule => 1,
            PackageKind::FeatureModule => 2,
            PackageKind::LanguagePackage => 3,
            PackageKind::RulePackage => 4,
            PackageKind::ToolProvider => 5,
            PackageKind::MetricProvider => 6,
            PackageKind::RepositoryProvider => 7,
            PackageKind::RuntimeProvider => 8,
            PackageKind::ModelBackendPackage => 9,
            PackageKind::AgentExecutorPackage => 10,
            PackageKind::ClientPackage => 11,
            PackageKind::IntegrationPackage => 12,
            PackageKind::SdkPackage => 13,
            PackageKind::ProjectionPackage => 14,
            PackageKind::FeaturePack => 15,
        };
    }

    /// `Label` is the serialized form and a non-Rust peer reimplements it from the
    /// corpus, so a label that drifts is a divergent wire format rather than a rename.
    /// This checks every declared kind against the enumeration `OD-PACKAGE-002` fixes:
    /// the label it serializes as, and the position it holds, so neither a relabelling
    /// nor a reordering can pass without moving the table.
    #[test]
    fn Test_Every_Kind_Should_Carry_The_Label_The_Corpus_Names()
    {
        for (position, (kind, label)) in CORPUS_TAXONOMY.into_iter().enumerate()
        {
            assert_eq!(
                kind.Label(),
                label,
                "PackageKind::{kind:?} serializes as a label the corpus taxonomy does not \
                 name; OD-PACKAGE-002 fixes the sixteen"
            );
            assert_eq!(
                Corpus_Position(kind),
                position,
                "PackageKind::{kind:?} is declared out of the corpus taxonomy's order"
            );
        }
    }

    /// `Display` is how a kind reaches a log line or a message, and a `Display` that
    /// disagreed with `Label` would put two spellings of one protocol commitment into
    /// circulation.
    #[test]
    fn Test_Display_Should_Render_The_Serialized_Label()
    {
        for (kind, label) in CORPUS_TAXONOMY
        {
            assert_eq!(
                kind.to_string(),
                label,
                "PackageKind::{kind:?} displays as something other than its label"
            );
        }
    }

    #[test]
    fn Test_Kernel_Modules_Should_Not_Be_Optional()
    {
        assert!(!PackageKind::KernelModule.Is_Optional());

        for (kind, _) in CORPUS_TAXONOMY
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
        assert!(PackageKind::ToolProvider.Hosts_Foreign_Code());
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
        let mut labels: Vec<&str> = CORPUS_TAXONOMY.iter().map(|(kind, _)| kind.Label()).collect();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(
            labels.len(),
            CORPUS_TAXONOMY.len(),
            "two package kinds share a label"
        );
    }
}

//! The kinds of thing that can be installed.

use serde::{Deserialize, Serialize};

const KERNEL_MODULE_LABEL: &str = "KernelModule";
// "ServiceModule" is the corpus-quoted PackageKind variant name transcribed from
// OD-PACKAGE-002's source volume, not a vague filler word -- renaming it would break this
// constant's 1:1 mirror with its sibling *_MODULE_LABEL constants and the enum variant it
// labels, and diverge from the taxonomy's own transcribed spelling. A code-standards gate
// still flags the word "service" here mechanically; this needs a suppressions.json waiver
// rather than a rename (see the naming-clarity finding for this line).
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
/// # Four kinds have a consumer now; the other twelve still wait
///
/// `nomos-package`'s reader (`crates/packages/nomos-package/src/reader.rs`) is
/// `LanguagePackage`'s real consumer: `Package_Kind_Field` deserializes a manifest's
/// `package_kind` against this enum and the reader returns
/// `ManifestError::WrongPackageKind` when the resolved kind is anything other than
/// `PackageKind::LanguagePackage`. That was this enum's first consumer, landed by
/// `P13-PACKAGE-GENERIC-CORE` (`OD-PACKAGE-007`). `nomos-model-package`'s reader is
/// `ModelBackendPackage`'s and `AgentExecutorPackage`'s real consumer (`OD-PACKAGE-010`),
/// and `nomos-rule-package`'s reader is `RulePackage`'s (`OD-PACKAGE-008`,
/// `OD-ROADMAP-001`) — each the same shape: a manifest's `package_kind` resolved and
/// refused if it names anything the reader does not accept.
///
/// The other twelve kinds remain genuinely unconsumed. No code outside this crate names
/// them, and none is written to make them look used: this workspace contains no
/// installable unit of any of those kinds for a consumer to be about, no manifest
/// declaring one exists on disk, and `PackageId` is likewise declared and never
/// constructed for them.
///
/// `ARCH-001` and `ARCH-002` require a `LanguagePackage` and a `RulePackage` to be
/// independently versioned, and `PKG-007` requires a package's own version, the Nomos
/// protocol range, the language versions and the provider versions to stay four distinct
/// domains. A Cargo `version` field is none of those and a `Cargo.toml` is not the
/// manifest `PKG-022` asks for, so nothing here is answered by changing what this
/// workspace publishes. `OD-PACKAGE-001` accepts both requirements and records that the
/// missing thing is the package rather than the attribute.
///
/// Each remaining kind gains its consumer when something reads a declared package
/// manifest of that kind and refuses one it cannot resolve, the way the reader above now
/// does for `LanguagePackage`. That is the change to watch for per kind; until one lands
/// for a given kind, that kind's declaration is a protocol commitment held deliberately,
/// which is also why `OD-PACKAGE-001` recorded that four of the labels below then
/// followed the game plan's spelling rather than volume 03's. That divergence cost
/// nothing while nobody read them and had to be settled before anybody did;
/// `OD-PACKAGE-002` settled it, and the labels are volume 03's. Having no consumer is
/// still fifteen of this enum's sixteen conditions, and it is `OD-PACKAGE-001`'s open
/// subject rather than something the spelling repair touched.
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
    ///
    /// `IntegrationPackage` is deliberately absent: `OD-PACKAGE-003` finds it carrying no
    /// executable code in nearly every surface it places, and warns that one which started
    /// carrying its own logic would be exactly the undifferentiated `Plugin` bucket this
    /// enum's own doc comment refuses to become.
    #[must_use]
    pub const fn Can_Host_Foreign_Code(self) -> bool
    {
        return matches!(
            self,
            Self::ToolProvider
                | Self::MetricProvider
                | Self::RepositoryProvider
                | Self::RuntimeProvider
                | Self::ModelBackendPackage
                | Self::AgentExecutorPackage
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
    use alloc::string::ToString;
    use alloc::vec::Vec;
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

    /// `Label` is the serialized form and a non-Rust peer reimplements it from the
    /// corpus, so a label that drifts is a divergent wire format rather than a rename.
    /// This checks every declared kind against the label `OD-PACKAGE-002` fixes.
    ///
    /// Ordinal position is deliberately not asserted. `Label` is documented as the wire
    /// form a peer reimplements; nothing establishes the enum's declaration order as
    /// protocol-relevant, and asserting it anyway would treat the taxonomy sentence's
    /// listing order as a wire commitment it was never claimed to be. `CORPUS_TAXONOMY`
    /// still transcribes the corpus's own order for a reader's benefit. Membership stays
    /// checkable without an ordinal: `Label`'s own match in `impl PackageKind` is
    /// exhaustive, so a seventeenth variant makes it non-exhaustive and this module stops
    /// compiling until whoever added the kind arrives there, beside the labels this test
    /// checks.
    #[test]
    fn Test_Every_Kind_Should_Carry_The_Label_The_Corpus_Names()
    {
        for (kind, label) in CORPUS_TAXONOMY
        {
            assert_eq!(
                kind.Label(),
                label,
                "PackageKind::{kind:?} serializes as a label the corpus taxonomy does not \
                 name; OD-PACKAGE-002 fixes the sixteen"
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
    fn Test_Is_Optional_Should_Be_False_For_Kernel_Modules_Only()
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
    fn Test_Can_Host_Foreign_Code_Should_Be_True_For_Provider_And_Model_Backend_Kinds()
    {
        assert!(PackageKind::ToolProvider.Can_Host_Foreign_Code());
        assert!(PackageKind::ModelBackendPackage.Can_Host_Foreign_Code());
        assert!(PackageKind::AgentExecutorPackage.Can_Host_Foreign_Code());
        assert!(!PackageKind::KernelModule.Can_Host_Foreign_Code());
        assert!(!PackageKind::FeaturePack.Can_Host_Foreign_Code());
    }

    /// A `RulePackage` is content, not a host: its judgments are data Nomos evaluates,
    /// so it does not by itself bring foreign code into the process.
    #[test]
    fn Test_Rule_And_Language_Packages_Should_Not_Host_Foreign_Code()
    {
        assert!(!PackageKind::RulePackage.Can_Host_Foreign_Code());
        assert!(!PackageKind::LanguagePackage.Can_Host_Foreign_Code());
    }

    /// An `IntegrationPackage` places or wires a peer's connection; `OD-PACKAGE-003`
    /// finds it carrying no executable code in nearly every surface it places, and one
    /// that started carrying its own logic would be the undifferentiated `Plugin` bucket
    /// this enum exists to refuse.
    #[test]
    fn Test_Integration_Packages_Should_Not_Host_Foreign_Code()
    {
        assert!(!PackageKind::IntegrationPackage.Can_Host_Foreign_Code());
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

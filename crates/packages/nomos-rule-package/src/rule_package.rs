//! The first `RulePackage` manifest shape.

use crate::{
    ApplicabilitySemantics, CapabilityRequirement, CorrectionAndSuppressionContract,
    DiagnosticMapping, Judgment, RuleContract,
};
use nomos_contracts::{EvidenceClass, PackageId, PackageKind, RuleId};
use nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration};

/// A `RulePackage`, at its first manifest maturity.
///
/// Every field is transcribed from `ARCH-002`'s own contents list — "rule identity/
/// version, normative specification, applicability semantics, required canonical
/// capabilities, deterministic judgment implementations, optional enhanced
/// implementations, external diagnostic mappings, correction and suppression contracts,
/// evidence schema, examples/counterexamples, conformance fixtures, evaluation corpus,
/// agent-guidance fragments, and presentation/protocol metadata" — checked field by
/// field against `OD-PACKAGE-008`'s real four-rule measurement (`Check_Completeness_
/// Mirrors`, `Check_Naming_Convention`, `Check_Dependency_Direction`,
/// `Check_Unread_Reaches_A_Finding`) rather than invented from the corpus text alone.
/// Five fields have no real instance across those four rules yet
/// (`enhanced_implementation`, `external_diagnostics`, `correction_and_suppression`,
/// `evaluation_corpus`, `agent_guidance`) and are typed as empty/absent-by-default
/// rather than left out, per `OD-ROADMAP-001`: a field with no real instance today is
/// still real content this manifest format should be able to carry once one exists.
///
/// Reuses `nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration}`
/// unchanged, the same way `nomos-model-package` does — `PKG-007`'s first two version
/// domains are genuinely kind-agnostic, and `ProviderRegistration` is exactly the shape
/// `OD-PACKAGE-008` names for `enhanced_implementation`'s tool dependencies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RulePackage
{
    /// This package's identity.
    pub package_id: PackageId,
    /// Always [`PackageKind::RulePackage`] for a value [`crate::Parse_Manifest`]
    /// produced — the reader refuses every other kind before a value is constructed.
    pub package_kind: PackageKind,
    /// This package's own release. `PKG-007`'s first version domain.
    pub package_version: PackageVersion,
    /// The Nomos protocol versions this package speaks. `PKG-007`'s second version
    /// domain.
    pub protocol_range: ProtocolRange,
    /// The rule this package declares. `ARCH-002`'s "rule identity" half.
    pub rule_id: RuleId,
    /// The governing record this rule cites, if it has one. `ARCH-002`'s "rule ...
    /// version, normative specification" — `Option` because `Check_Naming_Convention`
    /// genuinely has none, by its own module doc's design.
    pub contract: Option<RuleContract>,
    /// Whether a linked implementation decides this rule at all, or a model does.
    /// `OD-RULES-022`'s own clause, and what makes composition's resolution total.
    ///
    /// Read before `applicability` and `required_capabilities`, because both of those
    /// describe an implementation and [`Judgment::ModelJudged`] says there is none. A
    /// model-judged declaration still carries them, at whatever the manifest states, and
    /// nothing resolves against them.
    pub judgment: Judgment,
    /// Which shape of `Applicability` this rule's own judgment implementation raises
    /// for a genuine violation. `ARCH-002`'s "applicability semantics".
    pub applicability: ApplicabilitySemantics,
    /// The canonical capabilities this rule's judgment implementation needs.
    /// `ARCH-002`'s "required canonical capabilities" — a list, since the real
    /// population already shows three distinct shapes across four rules.
    pub required_capabilities: Vec<CapabilityRequirement>,
    /// The evidence class this rule's findings carry. `ARCH-002`'s "evidence schema" —
    /// convergent across all four shipped rules as
    /// [`EvidenceClass::Derived`](nomos_contracts::EvidenceClass::Derived).
    pub evidence_schema: EvidenceClass,
    /// External tool providers an enhanced (non-deterministic-judgment) implementation
    /// of this rule depends on, if one exists. `ARCH-002`'s "optional enhanced
    /// implementations" — empty for every shipped rule today.
    pub enhanced_implementation: Vec<ProviderRegistration>,
    /// External tools' own diagnostics this rule's findings correspond to.
    /// `ARCH-002`'s "external diagnostic mappings" — empty for every shipped rule
    /// today.
    pub external_diagnostics: Vec<DiagnosticMapping>,
    /// Whether this rule's violations may be mechanically corrected or explicitly
    /// suppressed. `ARCH-002`'s "correction and suppression contracts" — absent for
    /// every shipped rule today, since none has a safe mechanical fix yet.
    pub correction_and_suppression: Option<CorrectionAndSuppressionContract>,
    /// Free-text descriptions of a genuine violation this rule should catch.
    /// `ARCH-002`'s "examples" half.
    pub examples: Vec<String>,
    /// Free-text descriptions of code this rule must not flag.
    /// `ARCH-002`'s "counterexamples" half.
    pub counterexamples: Vec<String>,
    /// Identifiers or paths of fixtures this rule's conformance is checked against.
    /// `ARCH-002`'s "conformance fixtures".
    pub conformance_fixtures: Vec<String>,
    /// Where this rule's broader evaluation corpus lives, if a dedicated one exists
    /// beyond `conformance_fixtures`. `ARCH-002`'s "evaluation corpus" — absent for
    /// every shipped rule today.
    pub evaluation_corpus: Option<String>,
    /// Free-text fragments guiding an agent proposing a fix for this rule's
    /// violations. `ARCH-002`'s "agent-guidance fragments" — empty for every shipped
    /// rule today.
    pub agent_guidance: Vec<String>,
    /// The rule's human-readable title, for presentation. Part of `ARCH-002`'s
    /// "presentation/protocol metadata".
    pub title: String,
}

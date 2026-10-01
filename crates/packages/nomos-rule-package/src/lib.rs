//! Zone: Provider — the first `RulePackage` manifest maturity, and the first consumer of
//! `PackageKind::RulePackage` anywhere in this workspace.
//!
//! `OD-PACKAGE-008` measured every field of `ARCH-002`'s contents list against this
//! workspace's four real shipped rules (`Check_Completeness_Mirrors`,
//! `Check_Naming_Convention`, `Check_Dependency_Direction`, `Check_Unread_Reaches_A_
//! Finding`) before this crate typed anything, and `OD-ROADMAP-001` retires the wait for
//! a fifth rule or further convergence that measurement's own conclusion once required.
//! Every field below is transcribed from that measurement or from `ARCH-002`'s own text
//! where no real instance exists yet — never invented past what either source states.
//!
//! A deliberate peer of `nomos-lang-package` and `nomos-model-package`, not a dependent
//! of either: all three wrap `nomos-package`'s generic core for one `PackageKind`
//! family. This crate re-exports `nomos_package::{PackageVersion, ProtocolRange,
//! ProviderRegistration}` unchanged, because `PKG-007`'s first two version domains are
//! genuinely kind-agnostic and `ProviderRegistration` is exactly the shape
//! `OD-PACKAGE-008` names for a rule's enhanced-implementation tool dependencies.

#![forbid(unsafe_code)]

mod applicability_semantics;
mod capability_requirement;
mod correction_and_suppression_contract;
mod diagnostic_mapping;
mod judgment;
mod rule_package;
mod reader;
mod rule_contract;

pub use applicability_semantics::ApplicabilitySemantics;
pub use capability_requirement::CapabilityRequirement;
pub use correction_and_suppression_contract::CorrectionAndSuppressionContract;
pub use diagnostic_mapping::DiagnosticMapping;
pub use judgment::Judgment;
pub use rule_package::RulePackage;
pub use nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration};
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest, SCHEMA_VERSION};
pub use rule_contract::RuleContract;

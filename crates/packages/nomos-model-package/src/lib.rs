//! Band 26 -- the first `ModelBackendPackage`/`AgentExecutorPackage` manifest maturity,
//! and the first consumer of those two `PackageKind` variants anywhere in this workspace.
//!
//! `OD-PACKAGE-010` draws the boundary this crate builds inside: identity, `PackageKind`
//! (restricted to `ModelBackendPackage` and `AgentExecutorPackage`), `PKG-007`'s first two
//! version domains reused unchanged from `nomos-package`, and a new [`ModelSelection`]
//! domain answering `MODEL-ROUTE-037`'s opening clause -- a versioned discovered model
//! catalog, or an explicit declaration that selection is opaque or executor-controlled.
//! `MODEL-ROUTE-037`'s own catalog-entry detail and `MODEL-ROUTE-038`..`049`'s
//! routing/conformance system are a later manifest maturity, named in that record and not
//! attempted here -- the same split `OD-PACKAGE-001` already drew between `LanguagePackage`'s
//! first step and `PKG-022`'s larger field list.
//!
//! A deliberate peer of `nomos-lang-package`, not a dependent of it: both wrap
//! `nomos-package`'s generic core for one `PackageKind` family, and neither depends on the
//! other. This crate re-exports [`nomos_package::PackageVersion`] and
//! [`nomos_package::ProtocolRange`] unchanged rather than redefining them, because
//! `PKG-007`'s first two version domains are genuinely kind-agnostic; it does not reuse
//! `nomos_package::PackageManifest`, `Parse_Manifest` or `ProviderRegistration`, because
//! their fourth domain (`language_versions`/`providers`) is shaped for a language's
//! capability providers, which a model backend does not register.
//!
//! # A second maturity: the execution-profile vocabulary
//!
//! [`EffortLevel`], [`ModelSelector`] and [`ModelExecutionProfile`] answer
//! `MODEL-ROUTE-001`, `003` and `004` -- a distinct question from the manifest above.
//! Where `ModelRoutePackage` is what a *package* declares about itself, these three
//! types are what a per-operation *profile* asks for at the point an agent-assisted
//! operation runs. `OD-PACKAGE-011` found none of this vocabulary had any real type to
//! check a shape against; `OD-ROADMAP-001` retires the wait for one, so these are built
//! directly from the corpus text rather than left unbuilt. Nothing outside this crate
//! references a profile yet -- see [`ModelExecutionProfile`]'s own doc for exactly which
//! of `MODEL-ROUTE-001`'s five referencing surfaces are real types today and which are
//! not, and why that gap does not block building the profile type itself.
//!
//! # A third maturity: the strongest-grounded validation vocabulary
//!
//! [`FallbackAdmissibility`], [`ModelInputAssemblyIdentity`] and
//! [`RuntimeCandidateDisqualification`]/[`DisqualificationReason`] answer
//! `MODEL-ROUTE-014`, `029` and `034` -- reclassified from "no real case" to licensed
//! by `OD-PACKAGE-011` v3, on the same closed-enumeration test that already licensed
//! the execution-profile vocabulary above. Each has a dedicated corpus sentence that
//! closes its own field or value list, so nothing here is invented past what the
//! corpus states.
//!
//! # A fourth maturity: replay and determinism vocabulary
//!
//! [`RoutingReplayDisposition`]/[`ReplayFacts`], [`AssemblyComponentAvailability`] and
//! [`OutputDeterminismExpectation`]/[`OutputDeterminismValue`] answer `MODEL-ROUTE-017`,
//! `030`, `031` and `032`, on the same `OD-PACKAGE-011` v3 licensing. `030` attaches to
//! [`ModelInputAssemblyIdentity`]'s per-field pinning; `017`/`032` together define one
//! `RoutingReplayDisposition` type rather than two competing ones.

#![forbid(unsafe_code)]

mod assembly_component_availability;
mod effort_level;
mod fallback_admissibility;
mod manifest;
mod model_execution_profile;
mod model_input_assembly_identity;
mod model_selection;
mod model_selector;
mod output_determinism_expectation;
mod reader;
mod routing_replay_disposition;
mod runtime_candidate_disqualification;

pub use assembly_component_availability::AssemblyComponentAvailability;
pub use effort_level::EffortLevel;
pub use fallback_admissibility::FallbackAdmissibility;
pub use manifest::ModelRoutePackage;
pub use model_execution_profile::ModelExecutionProfile;
pub use model_input_assembly_identity::ModelInputAssemblyIdentity;
pub use model_selection::ModelSelection;
pub use model_selector::ModelSelector;
pub use nomos_package::{PackageVersion, ProtocolRange};
pub use output_determinism_expectation::{OutputDeterminismExpectation, OutputDeterminismValue};
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest, SCHEMA_VERSION};
pub use routing_replay_disposition::{ReplayFacts, RoutingReplayDisposition};
pub use runtime_candidate_disqualification::{DisqualificationReason, RuntimeCandidateDisqualification};

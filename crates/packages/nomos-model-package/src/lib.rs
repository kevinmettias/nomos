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
//!
//! # A fifth maturity: fallback-transition vocabulary
//!
//! [`FallbackTransitionTrace`]/[`DisqualificationEligibility`] and
//! [`CandidateFitAdjustment`]/[`ChangeAuthorization`]/[`AuthorizedCandidateFitAdjustment`]
//! answer `MODEL-ROUTE-035` and `036`, extending [`RuntimeCandidateDisqualification`]
//! (`034`) and [`ModelInputAssemblyIdentity`] (`029`) directly rather than
//! re-deriving either.
//!
//! # A sixth maturity: scope-precedence and mapping vocabulary
//!
//! [`ExecutionScope`], [`MappingQuality`]/[`EffortMappingRecord`] and
//! [`SelectorSpecificity`] answer `MODEL-ROUTE-005`, `015` and `018`, on the same
//! `OD-PACKAGE-011` v3 licensing. `005`'s own second sentence names further fields of
//! `ResolvedModelExecution`; that record stays unbuilt for the same reason
//! [`ModelInputAssemblyIdentity`]'s own doc comment already gives -- `ExecutionScope`
//! is one more field contributed to its eventual assembly, not a license to type a
//! partial wrapper now.
//!
//! # An eighth maturity: validation-failure and telemetry vocabulary
//!
//! [`WorkflowValidationFailure`], [`ModelExecutionTelemetry`] and
//! [`PipelineStage`]/[`TelemetryJunction`] answer `MODEL-ROUTE-009`, `010` and `016`,
//! on the same `OD-PACKAGE-011` v3 licensing.
//!
//! # A ninth maturity: named validation diagnostics
//!
//! [`ShadowedProfile`], [`EqualSpecificityConflict`],
//! [`TelemetryGuaranteeKind`]/[`ImpossibleTelemetryGuarantee`]/
//! [`ImpossibleReplayRequirement`] and [`ValidationDiagnostic`]/[`CandidateOutcome`]
//! answer `MODEL-ROUTE-021`, `023`, `027` and `028` -- the four strongest of the
//! `021`-`028` validation-diagnostics family, each with a dedicated corpus sentence
//! that closes its own field or value list.
//!
//! # A tenth maturity: classification diagnostics, closing the licensed-25
//!
//! [`SelectorValidationFailure`]/[`SelectorValidationSeverity`],
//! [`CorrectionExecutorAuthorityMismatch`]/[`CorrectionRouteClassification`] and
//! [`ModelSelectorViolation`] answer `MODEL-ROUTE-022`, `025` and `026` -- the
//! remaining three of the `021`-`028` family, closing every `MODEL-ROUTE` id
//! `OD-PACKAGE-011` v3 licensed. None of the three corpus sentences names its own
//! diagnostic, the same gap `MODEL-ROUTE-004`'s own text has for `EffortLevel`; each
//! still closes a value or field list, which is the actual test. `026`'s
//! `AgentExecutorRouteKind` half (subscription/chat/CLI/IDE) is deliberately not
//! built -- see [`ModelSelectorViolation`]'s own doc comment for why.

#![forbid(unsafe_code)]

mod assembly_component_availability;
mod budget_estimate;
mod candidate_fit_adjustment;
mod correction_executor_authority_mismatch;
mod effort_level;
mod effort_mapping;
mod equal_specificity_conflict;
mod execution_scope;
mod executor_exposure;
mod fallback_admissibility;
mod fallback_transition_trace;
mod manifest;
mod model_execution_profile;
mod model_execution_telemetry;
mod model_input_assembly_identity;
mod model_selection;
mod model_selector;
mod model_selector_violation;
mod output_determinism_expectation;
mod reader;
mod routing_replay_disposition;
mod rule_model_configuration;
mod runtime_candidate_disqualification;
mod selector_specificity;
mod selector_validation_failure;
mod shadowed_profile;
mod telemetry_guarantee;
mod telemetry_junction;
mod validation_diagnostic;
mod workflow_validation_failure;

pub use assembly_component_availability::AssemblyComponentAvailability;
pub use budget_estimate::{BudgetEstimate, CheckOrFixStage};
pub use candidate_fit_adjustment::{AuthorizedCandidateFitAdjustment, CandidateFitAdjustment, ChangeAuthorization};
pub use correction_executor_authority_mismatch::{CorrectionExecutorAuthorityMismatch, CorrectionRouteClassification};
pub use effort_level::EffortLevel;
pub use effort_mapping::{EffortMappingRecord, MappingQuality};
pub use equal_specificity_conflict::EqualSpecificityConflict;
pub use execution_scope::ExecutionScope;
pub use executor_exposure::ExecutorExposure;
pub use fallback_admissibility::FallbackAdmissibility;
pub use fallback_transition_trace::{DisqualificationEligibility, FallbackTransitionTrace};
pub use manifest::ModelRoutePackage;
pub use model_execution_profile::ModelExecutionProfile;
pub use model_execution_telemetry::ModelExecutionTelemetry;
pub use model_input_assembly_identity::ModelInputAssemblyIdentity;
pub use model_selection::ModelSelection;
pub use model_selector::ModelSelector;
pub use model_selector_violation::ModelSelectorViolation;
pub use nomos_package::{PackageVersion, ProtocolRange};
pub use output_determinism_expectation::{OutputDeterminismExpectation, OutputDeterminismValue};
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest, SCHEMA_VERSION};
pub use routing_replay_disposition::{ReplayFacts, RoutingReplayDisposition};
pub use rule_model_configuration::RuleModelConfiguration;
pub use runtime_candidate_disqualification::{DisqualificationReason, RuntimeCandidateDisqualification};
pub use selector_specificity::SelectorSpecificity;
pub use selector_validation_failure::{SelectorValidationFailure, SelectorValidationSeverity};
pub use shadowed_profile::{ShadowClassification, ShadowedProfile};
pub use telemetry_guarantee::{ImpossibleReplayRequirement, ImpossibleTelemetryGuarantee, TelemetryGuaranteeKind};
pub use telemetry_junction::{PipelineStage, TelemetryJunction};
pub use validation_diagnostic::{CandidateOutcome, ValidationDiagnostic};
pub use workflow_validation_failure::WorkflowValidationFailure;

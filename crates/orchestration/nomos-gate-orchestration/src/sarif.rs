//! [`SarifLog`] and its own projection functions: the SARIF 2.1.0 interchange projection of a
//! gate run and of a check run.
//!
//! `P123-SARIF-PROJECTION-OF-A-GATE-RUN` measured the hole this fills. Grepping this
//! workspace's crates for `sarif`, `graphml`, `digraph`, `csv` and `jsonl` finds an
//! abbreviation list, the spec bundle's own JSONL, and one v14 corpus row
//! (`nomos-spec-model/tests/corpus/discriminating-blocks.json`) requiring a SARIF run-evidence
//! export that nothing implemented -- no host emitted findings in any interchange format at
//! all, so the gate could not be read by the CI systems it exists to run inside.
//!
//! # Why it lives here
//!
//! `OD-HOST-017` decides it: an interchange projection belongs to the service that owns the
//! judgment, and every host emits it from there. A SARIF log's `level`, `suppressions` and
//! `invocation` are all derived from what a run found and what a policy did about it, in
//! vocabularies that are this crate's and the check service's own -- [`crate::GateRunResult`],
//! [`crate::FindingDisposition`], [`crate::NoVerdict`] and
//! `nomos_check_orchestration::CheckOutcome` -- so this module reads those and nothing a host
//! rendered from them. It lived in `nomos-api` first, beside that crate's serializable twins of
//! these types, which made it reachable from one host and its client and from neither the
//! command line nor the language server. Application Service is a zone every Host may name,
//! so here it is reachable by all of them with no exception.
//!
//! It lives in this crate rather than the check service because a gate log needs both
//! judgments and the bucket vocabulary is this crate's; a check run's log is the degenerate
//! case with no policy applied and no bucket on a result.
//!
//! # What the type layer is shaped by
//!
//! One file per SARIF object, named for the object, each carrying in its own doc which
//! properties of the specification it emits and which it deliberately does not -- because a
//! SARIF object has many optional properties and "absent" is a claim about what this
//! workspace measured. Nothing here is a general SARIF library: every type is `pub(crate)`
//! and exists to be written, never read back, so no `Deserialize` is derived and no property
//! is carried that a judgment cannot answer for.
//!
//! [`SarifLog::Serialized`] hands a host the document as text. That reverses what this module
//! said in `nomos-api`, where a caller chose the serializer because that crate's `serde_json`
//! was a dev-dependency; this crate carries `serde_json` in production, so the host that most
//! needs the export -- the command line a CI job runs -- writes it without declaring a
//! serializer of its own (`OD-HOST-017` decision 4).

mod artifact_location;
mod descriptor_properties;
#[cfg(test)]
pub(crate) mod fixtures;
mod physical_location;
mod reporting_descriptor;
mod result_properties;
mod run_properties;
mod sarif_invocation;
mod sarif_level;
mod sarif_location;
mod sarif_log;
mod sarif_message;
mod sarif_notification;
mod sarif_region;
mod sarif_result;
mod sarif_run;
mod sarif_suppression;
mod sarif_tool;
mod tool_driver;

pub use sarif_log::SarifLog;

// What the other exports share with this log rather than derive again (`crate::export`): the
// six finding groups, the word each bucket is spelled in, the suppression a policy-answered
// bucket carries, and whether the run reached a complete judgment and why not.
pub(crate) use result_properties::Bucket_Word;
pub(crate) use sarif_invocation::SarifInvocation;
pub(crate) use sarif_log::Bucketed_Findings;
pub(crate) use sarif_suppression::SarifSuppression;

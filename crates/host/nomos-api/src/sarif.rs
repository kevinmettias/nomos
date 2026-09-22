//! [`SarifLog`] and its own projection functions, paired in one file below: the SARIF 2.1.0
//! interchange projection of a gate run and of a check run.
//!
//! `P123-SARIF-PROJECTION-OF-A-GATE-RUN` measured the hole this fills. Grepping this
//! workspace's crates for `sarif`, `graphml`, `digraph`, `csv` and `jsonl` finds an
//! abbreviation list, the spec bundle's own JSONL, and one v14 corpus row
//! (`nomos-spec-model/tests/corpus/discriminating-blocks.json`) requiring a SARIF run-evidence
//! export that nothing implemented -- no host emitted findings in any interchange format at
//! all, so the gate could not be read by the CI systems it exists to run inside.
//!
//! # Why it lives here and not in a host
//!
//! `OD-HOST-002` divides a surface from a composition root: a surface holds no state its
//! canonical service cannot reconstruct, and a walk or a build variant is the composition
//! root's own business. A SARIF log is entirely a function of a response this crate already
//! owns -- [`crate::response::GateRunResponse`] and [`crate::check::CheckResponse`], the two
//! values CLI, transport and MCP all project -- so it reconstructs from the canonical answer
//! and belongs beside it. Putting it in `nomos-cli` would make the one export a CI consumer
//! ingests reachable only from a terminal, and putting it in each host would be three
//! projections of one thing.
//!
//! `OD-HOST-013` is the other half: a projection adds no verb. Nothing here reads argv,
//! listens on a socket or registers a tool, and this item deliberately adds no
//! `--format sarif` flag, no transport method and no MCP tool -- each of those is a host's own
//! decision about its own surface, and one made with a real consumer in hand rather than
//! ahead of one. What this module owes those hosts is a function they can call, which is
//! [`SarifLog::Of_Gate_Run`] and [`SarifLog::Of_Check_Run`].
//!
//! # What the type layer is shaped by
//!
//! One file per SARIF object, named for the object, each carrying in its own doc which
//! properties of the specification it emits and which it deliberately does not -- because a
//! SARIF object has many optional properties and "absent" is a claim about what this
//! workspace measured. Nothing here is a general SARIF library: every type is `pub(crate)`
//! and exists to be written, never read back, so no `Deserialize` is derived and no property
//! is carried that a response cannot answer for.
//!
//! `serde_json` is a dev-dependency of this crate and stays one. Every type here derives
//! `Serialize` and nothing in production serializes: a caller chooses the serializer, so the
//! projection costs a host no dependency it did not already have. The tests do use
//! `serde_json`, which is what makes an assertion about the emitted document's shape possible
//! at all.

mod artifact_location;
mod descriptor_properties;
mod physical_location;
mod reporting_descriptor;
mod result_properties;
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

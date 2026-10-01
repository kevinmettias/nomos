//! The judgment in every interchange form beside SARIF: [`FindingLines`] (newline-delimited
//! JSON), [`FindingTable`] (a delimited table with a named header) and [`SupportingFactGraph`]
//! (`GraphML`).
//!
//! `P128-A-JUDGMENT-LEAVES-IN-ONE-FORMAT-AND-EVERY-OTHER-CONSUMER-IS-UNSERVED` measured the gap:
//! the one run-evidence export the v14 corpus names, `nomos runs export <run> --format
//! capsule|json|sarif`, had SARIF behind it and nothing else, so a consumer reading
//! newline-delimited JSON, a spreadsheet or a graph format read nothing this workspace produced.
//!
//! # One reading of a finding, shared with the SARIF log
//!
//! Every form here lives beside [`crate::SarifLog`] because `OD-HOST-017` decides an interchange
//! projection belongs to the service that owns the judgment. Every one reads the same input --
//! a [`crate::GateRunResult`], or a check's `CheckOutcome` -- and the parts of a finding that a
//! policy decided come from the SARIF module's own functions rather than a second derivation:
//! the six finding groups, the bucket word each is spelled in, the suppression a bucket a policy
//! answered carries, and whether the run reached a complete judgment and why not. So an export
//! and the SARIF log of one run cannot disagree about what a policy did to a finding.
//!
//! Every form states the run's own status, not only its findings. A run that judged nothing,
//! judged incompletely, or reached no verdict is written as such, never as a clean empty file.
//!
//! # Why `GraphML`
//!
//! The graph a run already holds is not the crate dependency graph: a run materializes those
//! edges into a fact store it never returns. What a [`crate::GateRunResult`] carries is the
//! check's supporting-fact trail -- which capability families, from which providers, each
//! rule's judgment read, and how each read came back (`OD-HOST-016`). That is a dependency
//! graph in the sense incremental analysis means the word: what a judgment depended on.
//!
//! `GraphML` is the form because its readers are analysis tools that keep an edge's attributes as
//! data: `NetworkX`, igraph, Gephi, yEd and Cytoscape all read it, and it declares each attribute's
//! type, so a read count arrives as a number and an absent guarantee as an absence. DOT was the
//! easier form to emit and was not chosen for that reason: it is a drawing language whose
//! attributes are untyped strings, read chiefly to lay a picture out. JSON Graph Format has no
//! comparable body of readers. The corpus row names no graph format at all, so nothing there
//! decides it.
//!
//! # What this module adds no route to
//!
//! No verb and no flag. Reaching these from a command line is the question the SARIF move
//! answered with one flag, `--sarif`, and a second family of flags beside it would be two
//! answers to one question; a caller that wants these today calls the constructors below.

mod exported_finding;
mod finding_lines;
mod finding_table;
mod run_status;
mod supporting_fact_graph;
#[cfg(test)]
mod xml_reader;

pub use finding_lines::FindingLines;
pub use finding_table::FindingTable;
pub use supporting_fact_graph::SupportingFactGraph;

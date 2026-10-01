//! Zone: Host. What this workspace tells an editor, and nothing about the protocol
//! that carries it.
//!
//! # What changed on 2026-09-10
//!
//! **The whole server is XVPE's now.** The handshake, the workspace root, the capability
//! declaration, the `file://` URI conversion, the whole-line spans, and the stale-marker
//! clearing an editor needs all moved down into `xvpe-language-server-backend-lsp`, over
//! the `xvpe-diagnostics` provider contract. Nomos is an application over that engine, and
//! a general capability sitting up here was unreachable by everything down there.
//!
//! What stays is what was always this workspace's: the walk, the judgement, which severity
//! each of its own rule categories deserves ([`severity`]), and what a finding carries with
//! it ([`WalkOutward`]). [`NomosDiagnosticProvider`] is the one seam, and `src/main.rs`
//! hands it to the engine.
//!
//! `P42-LSP-PROJECTION` named the shape this crate must not become: not another analyzer.
//! Every finding published here was judged by `nomos_check_orchestration::Run`, the
//! identical seam `nomos-cli::check` and `nomos-correction-orchestration::run` already call
//! -- this crate adds a walk (`sources.rs`, a thin wrapper over the one walk
//! `nomos-workspace-discovery` holds beneath every composition root since `OD-HOST-008`,
//! since `nomos-check-orchestration` composes no platform `FileSystem` port and that port's
//! own `Read_Directory` is one level) and a
//! translation ([`file_diagnostic::Diagnostics_For`]) from [`nomos_contracts::Finding`] to
//! `xvpe_diagnostics::SourceDiagnostic`, never a second judgment.
//!
//! # What "walk outward from a diagnostic" answers
//!
//! [`walk_outward::WalkOutward`] is attached to every diagnostic's own detail -- the
//! standard per-diagnostic extension point. All five targets the ledger item's own
//! `done_when` named have a real, mechanical answer, carried there: which rule
//! governs a finding ([`walk_outward::GoverningRule`], read straight off
//! `nomos_rules::DESCRIPTORS`), which architectural component it belongs to
//! ([`walk_outward::ArchitecturalComponent`], read off the architecture the repository under
//! check declares, when the location is a `crates/...` path), which correction is available
//! ([`walk_outward::AvailableCorrection`], the two rule ids `nomos-correction-orchestration`
//! composes today), which facts the finding's rule read
//! ([`walk_outward::SupportingFacts`], the trail `nomos_check_orchestration::CheckOutcome`
//! now carries back on the run itself), and which corpus requirements it bears on
//! ([`walk_outward::RequirementLink`], the `rule` lines an assessment declares, read through
//! `nomos_cap_requirement_trace::Assessments_In`). Evidence strength and applicability were
//! already directly on [`nomos_contracts::Finding`] and are carried through unchanged.
//!
//! The last two were open until recently, and how they were closed matters more than that
//! they were. `OD-HOST-010` named both as undecided rather than declined, with the concrete
//! evidence behind each; `OD-HOST-016` decided the supporting fact and `OD-HOST-015` the
//! requirement link, and each is read here from the mechanism its own building item shipped
//! rather than reconstructed. Neither refused answer was quietly adopted: a per-finding fact
//! answer, a `FactKey` rebuilt from outside a rule, and a requirement derived from a finding's
//! location are all still refused, by those records and by name. Cited here rather than
//! restated, the same rule `AGENTS.md` states for this crate's own doc and every governing
//! record in this workspace.
//!
//! Each of the five says "I do not know" in its own vocabulary, and the vocabularies are
//! deliberately not the same. `walk_outward`'s own module doc is where that is stated.
//!
//! # The `LintDiagnostic` contract is unchanged
//!
//! Nothing here reaches into `nomos-cap-lint`, and its payload schema is untouched.
//! The engine is where a tool-reported line becomes an editor-grade span, at projection
//! time, exactly where `nomos_cap_lint::LintDiagnostic`'s own doc says that
//! combination belongs (`OD-HOST-003`).

mod build_variant;
mod file_diagnostic;
mod location;
mod nomos_diagnostic_provider;
mod severity;
mod sources;
mod walk_outward;

pub use file_diagnostic::Diagnostics_For;
pub use nomos_diagnostic_provider::NomosDiagnosticProvider;
pub use walk_outward::{
    ArchitecturalComponent, AvailableCorrection, FactRead, GoverningRule, RequirementLink, SupportingFacts, WalkContext,
    WalkOutward,
};

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
//! -- this crate adds a walk (`sources.rs`, a third copy of the same shape those two
//! crates' own composition roots each carry, since `nomos-check-orchestration` composes no
//! [`nomos_platform::FileSystem`] and the port's own `Read_Directory` is one level) and a
//! translation ([`file_diagnostic::Diagnostics_For`]) from [`nomos_contracts::Finding`] to
//! `xvpe_diagnostics::SourceDiagnostic`, never a second judgment.
//!
//! # What "walk outward from a diagnostic" answers today, and what it does not
//!
//! [`walk_outward::WalkOutward`] is attached to every diagnostic's own detail -- the
//! standard per-diagnostic extension point. Three of the five targets the ledger
//! item's own `done_when` named have a real, mechanical answer, carried there: which rule
//! governs a finding ([`walk_outward::GoverningRule`], read straight off
//! `nomos_rules::DESCRIPTORS`), which architectural component it belongs to
//! ([`walk_outward::ArchitecturalComponent`], read off `nomos_rules::ZONES` when the
//! location is a `crates/...` path), and which correction is available
//! ([`walk_outward::AvailableCorrection`], the two rule ids `nomos-correction-orchestration`
//! composes today). Evidence strength and applicability were already directly on
//! [`nomos_contracts::Finding`] and are carried through unchanged.
//!
//! Two targets are not answered: which fact backed one specific judgment (`CheckOutcome`
//! exposes findings, not the fact store a judgment read from, and reconstructing the exact
//! key a rule's own internal `Reader::Require` call used is not something a caller outside
//! that rule can do without duplicating its own private `Requirement`), and which corpus
//! requirement (`AGT-007`, `CHK-003`, ...) a finding bears on (`tests/contract/requirements/
//! *.assessment` is hand-authored prose keyed on no `RuleId`, and `requirement_trace`'s own
//! module doc states it is never derived from the corpus at check time). Both are named in
//! full, with the concrete evidence behind each, in `docs/records/OD-HOST-010-*.md` --
//! cited here rather than restated, the same rule `AGENTS.md` states for this crate's own
//! doc and every governing record in this workspace.
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
pub use walk_outward::{ArchitecturalComponent, AvailableCorrection, GoverningRule, WalkOutward};

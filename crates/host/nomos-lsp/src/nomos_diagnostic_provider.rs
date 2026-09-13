//! What this workspace judges when an editor asks -- and the two things it keeps between
//! being asked twice.

use crate::build_variant::Host_Variant;
use crate::file_diagnostic::Diagnostics_For;
use crate::sources::Walked_Sources;
use nomos_analysis::MemoryFactStore;
use nomos_composer_std::{ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_workspace::Workspace;
use std::path::Path;
use xvpe_diagnostics::{DiagnosticProviderStrategy, SourceDiagnostic};
use xvpe_primitives::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// This workspace's own judgement, as the engine's diagnostic provider.
///
/// Everything about *being* a language server -- the handshake, the workspace root, the
/// capability declaration, the URI conversion, the whole-line spans, and the stale-marker
/// clearing an editor needs -- is `xvpe-language-server-backend-lsp`'s as of 2026-09-10.
/// What is left here is the half that was always this workspace's: walking a tree, running
/// every composed rule over it, and turning a `Finding` into something an editor can show.
///
/// It owns no judgment of its own either way: every finding was produced by
/// `nomos_check_orchestration::Run`, the identical seam `nomos-cli::check` and
/// `nomos-correction-orchestration::run` already call.
///
/// # What it keeps between calls
///
/// `workspace` and `store`, reused across every judgement rather than rebuilt per call.
/// `P14-ANALYSIS-009-STORE-WORKSPACE-REUSE-FIRST-INCREMENT` gave `Run` a caller-supplied
/// pair for exactly this, and an editor session is this workspace's first caller with a
/// process lifetime long enough to hold either across two calls: `OD-ANALYSIS-009`'s own
/// second amendment names this crate as the concrete case its first trigger was written
/// for.
///
/// Reused, not persisted -- both still end when this process does. Nothing here asks either
/// to survive past that.
///
/// # Why nothing is derived
///
/// Neither `Debug` nor `Default`: a `Workspace` implements neither, and a fact
/// store has no meaningful empty value distinct from [`Self::New`]. Deriving what cannot be
/// derived honestly is how a type ends up with a `Default` nobody meant.
pub struct NomosDiagnosticProvider
{
    workspace: Option<Workspace>,
    store: MemoryFactStore,
}

impl NomosDiagnosticProvider
{
    /// A provider that has judged nothing yet.
    #[must_use]
    pub fn New() -> Self
    {
        return Self { workspace: None, store: MemoryFactStore::New() };
    }
}

impl Strategy for NomosDiagnosticProvider
{
    // `None`: judging walks a real tree and launches real tool subprocesses. What it finds
    // depends on what is on disk and what those tools say when asked.
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl DiagnosticProviderStrategy for NomosDiagnosticProvider
{
    /// Walks `root`, runs every composed rule over it, and reports what
    /// [`Diagnostics_For`] makes of the result.
    ///
    /// A root that cannot be walked, and a run that reached no judgement, both report
    /// nothing. That is a real limitation rather than a silent one: this provider has no
    /// finding of its own to raise about a directory it could not read, because a `Finding`
    /// is something a rule produces about a subject, and there is no subject here.
    fn Diagnose(&mut self, root: &Path) -> Vec<SourceDiagnostic>
    {
        let Some(sources) = Walked_Sources(root)
        else
        {
            return Vec::new();
        };

        let outcome = nomos_check_orchestration::Run(
            &sources,
            nomos_check_orchestration::RunContext {
                variant: Host_Variant(),
                root,
                launcher: &LAUNCHER,
                filesystem: &FILE_SYSTEM,
                environment: &ENVIRONMENT,
                workspace: &mut self.workspace,
                store: &mut self.store,
            },
            &[],
        );

        let nomos_check_orchestration::CheckOutcome::Judged { findings, .. } = outcome
        else
        {
            return Vec::new();
        };

        return findings.iter().flat_map(Diagnostics_For).collect();
    }
}

#[cfg(test)]
mod tests;

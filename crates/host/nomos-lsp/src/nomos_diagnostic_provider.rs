//! What this workspace judges when an editor asks -- and the three things it keeps between
//! being asked twice.

use crate::build_variant::Host_Variant;
use crate::walk_outward::WalkContext;
use crate::Diagnostics_For;
use crate::sources::Walked_Sources;
use nomos_analysis::MemoryFactStore;
use nomos_cap_requirement_trace::Assessment;
use nomos_composer_std::{ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_rules::SourceFile;
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
/// `nomos_check_orchestration::Run_Reassessing`, which is `Run`'s own pipeline given a
/// cache to keep -- `Run` delegates to it with a cache built fresh per call. What this
/// crate supplies is a longer-lived cache, never a different judgement, so the seam
/// `nomos-cli::check` and `nomos-correction-orchestration::run` call is still the seam
/// answering here.
///
/// # What it keeps between calls
///
/// `workspace`, `store` and `reassessment`, reused across every judgement rather than
/// rebuilt per call. `P14-ANALYSIS-009-STORE-WORKSPACE-REUSE-FIRST-INCREMENT` gave `Run` a
/// caller-supplied workspace and store for exactly this, and an editor session is this
/// workspace's first caller with a process lifetime long enough to hold any of them across
/// two calls: `OD-ANALYSIS-009`'s own second amendment names this crate as the concrete
/// case its first trigger was written for.
///
/// The first two buy a fact that is not re-derived for a subject whose bytes did not move.
/// The third buys the rule that reads that fact not running again either, which `Run`
/// cannot do for any caller: its own doc says it builds a cache fresh for the call and
/// drops it at the end, so every composed rule re-runs on every call by construction. A
/// caller wanting the skip has to keep the cache itself, and an editor is the caller that
/// can -- one process across many `didOpen` and `didSave` notifications, most of which move
/// one file or none.
///
/// Reused, not persisted -- all three still end when this process does. Nothing here asks
/// any of them to survive past that.
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
    reassessment: nomos_check_orchestration::RuleReassessmentCache,
}

impl NomosDiagnosticProvider
{
    /// A provider that has judged nothing yet.
    #[must_use]
    pub fn New() -> Self
    {
        return Self {
            workspace: None,
            store: MemoryFactStore::New(),
            reassessment: nomos_check_orchestration::RuleReassessmentCache::New(),
        };
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
        let Some(sources) = Walked_Sources(root) else { return Vec::new(); };

        let outcome = Run_Judgment(self, root, &sources);

        let nomos_check_orchestration::CheckOutcome::Judged { findings, supporting_facts, .. } = outcome else { return Vec::new(); };

        // Read once for the whole batch rather than per finding: each is one file or one
        // directory, and every diagnostic in the batch is walked against the same answer. A
        // declaration this root does not have is an empty one, and every finding then simply
        // carries no component and no requirement link -- the same answers
        // `ArchitecturalComponent::Of` and `RequirementLink::Declared_For` give when nothing
        // was declared, rather than a failure that would cost the reader its diagnostics.
        let architecture = nomos_repo_policy::architecture::Discover_Workspace(root, &FILE_SYSTEM).unwrap_or_default();
        let assessments = Committed_Assessments(root);
        let context = WalkContext { architecture: &architecture, trail: &supporting_facts, assessments: &assessments };

        return findings.iter().flat_map(|finding| return Diagnostics_For(&context, finding)).collect();
    }
}

/// The assessments the repository under `root` commits, or none when it commits no registry.
///
/// `OD-HOST-015`'s sixth decision: read once per batch through the reader that capability
/// already has, over `nomos_cap_requirement_trace::REGISTRY`, the same declaration-reading
/// shape the architectural component already takes. Every repository but this one keeps no
/// `tests/contract/requirements/` at all, and one that does may still hold an entry this build
/// cannot parse -- either way the honest answer here is the empty set, because a link nobody
/// declared and a registry nobody wrote are the same absence from a diagnostic's side, and
/// neither is worth costing a reader the diagnostics themselves.
///
/// What is *not* silently absorbed is a malformed entry going uncounted: the reader refuses
/// rather than skips, so a typo cannot quietly shrink the set behind this call, and
/// `tests/contract`'s own `requirement_trace` suite is what reports it for this repository.
fn Committed_Assessments(root: &Path) -> Vec<Assessment>
{
    let registry = root.join(nomos_cap_requirement_trace::REGISTRY);

    return nomos_cap_requirement_trace::Assessments_In(&registry, &FILE_SYSTEM).unwrap_or_default();
}

/// The outcome `nomos_check_orchestration::Run_Reassessing` reaches over `sources` under
/// `root`, through the workspace, fact store and reassessment cache `provider` keeps between
/// calls -- the three things whose reuse is the whole reason this crate has a provider of its
/// own rather than calling the orchestration fresh each time.
fn Run_Judgment(provider: &mut NomosDiagnosticProvider, root: &Path, sources: &[SourceFile]) -> nomos_check_orchestration::CheckOutcome
{
    return nomos_check_orchestration::Run_Reassessing(
        sources,
        nomos_check_orchestration::RunContext {
            variant: Host_Variant(),
            root,
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            workspace: &mut provider.workspace,
            store: &mut provider.store,
        },
        &[],
        &mut provider.reassessment,
    );
}

#[cfg(test)]
mod tests;

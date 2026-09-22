//! Materializing `lint.diagnostics` facts through `cargo clippy`, and what that produced.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_platform::{Environment, ProgramLauncher};
use nomos_rules::SourceFile;

use std::path::Path;

use super::subprocess::Subprocess;

/// What materializing `lint.diagnostics` facts produced: the sources a rule can judge them
/// under, and any finding the materialization itself already raised -- the identical shape
/// [`super::DependencyMaterialization`] already has for the identical reason.
pub struct LintMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}

/// Runs `cargo clippy --workspace --message-format=json` over `root` through `launcher`,
/// materializes one `nomos.cap.lint.diagnostics` fact per workspace member, and returns
/// the subjects a rule can judge them under -- the identical shape
/// [`super::Materialize_Dependencies`] already has for the identical reason: this
/// workspace's second provider with I/O of its own.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero lint findings" either, the same
/// `NoFacts`-shaped vacuity [`super::Materialize_Syntax`]'s own case exists to catch one
/// layer over. So a failed materialization returns no lint sources and one synthetic
/// finding reporting why, rather than nothing at all.
pub fn Materialize_Lint<Launcher: ProgramLauncher, Env: Environment>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    subprocess: Subprocess<'_, Launcher, Env>,
) -> LintMaterialization
{
    let materialized = super::Materialize_Through(
        || nomos_lang_rust_clippy::Materialize_Workspace(root, Clippy_Production(context), subprocess.launcher, subprocess.environment),
        store,
        Materialized_Lint_Sources,
        Lint_Capability_Unavailable,
    );

    return LintMaterialization { sources: materialized.sources, findings: materialized.findings };
}

/// The reading context as `nomos_lang_rust_clippy`'s provider takes it.
fn Clippy_Production(context: &Context) -> nomos_lang_rust_clippy::FactContext
{
    return nomos_lang_rust_clippy::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Every `lint.diagnostics` fact the store accepted, as the source list
/// [`nomos_rules::Check_Lint_Diagnostics`] can judge -- one per workspace member
/// `store.Materialize` did not refuse.
fn Materialized_Lint_Sources(
    facts: Vec<nomos_lang_rust_clippy::DiagnosticsFact>,
    store: &mut MemoryFactStore,
) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for member in facts
    {
        if store.Materialize(member.fact, &[]).is_ok()
        {
            let source = SourceFile::New(member.path, member.subject, String::new());
            sources.push(source);
        }
    }

    return sources;
}

/// The one finding a failed [`nomos_lang_rust_clippy::Materialize_Workspace`] call
/// produces -- the identical shape
/// [`super::dependencies::Dependency_Capability_Unavailable`] already has for the
/// identical reason: attributed to the whole tree, since a `cargo clippy` failure is
/// not about any subject this run walked.
fn Lint_Capability_Unavailable(error: &nomos_lang_rust_clippy::ClippyError) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(nomos_rules::LINT_DIAGNOSTICS),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the lint-diagnostics capability could not be materialized, so lint \
             diagnostics were not judged for anything in this run: {error}"
        ),
        locations: Vec::new(),
    };
}

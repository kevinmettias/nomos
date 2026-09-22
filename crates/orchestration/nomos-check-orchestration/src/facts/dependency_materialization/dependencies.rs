//! `dependency.edges` facts, materialized by running `cargo metadata` over the workspace.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_platform::{Environment, ProgramLauncher};
use nomos_rules::SourceFile;

use std::path::Path;

use super::subprocess::Subprocess;
use super::DependencyMaterialization;

/// Runs `cargo metadata` over `root` through `launcher`, materializes one
/// `nomos.cap.dependency.edges` fact per workspace member, and returns the subjects a rule
/// can judge them under.
///
/// A second, independent materialization step beside [`super::Materialize_Syntax`] rather
/// than a generalization of it: `dependency.edges` has exactly one consumer today and
/// nothing to share or schedule against `syntax.items`'s own materialization, so composing
/// this as one more hardcoded step is `OD-HOST-004`'s "composition, not choice" again, not
/// a case for the shared demand planner `ARC-ROADMAP-001` still leaves for later.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero dependency findings" either, which is
/// exactly the vacuity [`super::Materialize_Syntax`]'s own `NoFacts` case exists to catch
/// one layer over. So a failed materialization returns no dependency sources and one
/// synthetic finding reporting why, rather than nothing at all.
pub fn Materialize_Dependencies<Launcher: ProgramLauncher, Env: Environment>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    subprocess: Subprocess<'_, Launcher, Env>,
) -> DependencyMaterialization
{
    let materialized = super::Materialize_Through(
        || nomos_lang_rust_cargo::Materialize_Workspace(root, Cargo_Production(context), subprocess.launcher, subprocess.environment),
        store,
        Materialized_Dependency_Sources,
        Dependency_Capability_Unavailable,
    );

    return DependencyMaterialization { sources: materialized.sources, findings: materialized.findings };
}

/// The reading context as `nomos_lang_rust_cargo`'s provider takes it.
fn Cargo_Production(context: &Context) -> nomos_lang_rust_cargo::FactContext
{
    return nomos_lang_rust_cargo::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Every `dependency.edges` fact the store accepted, as the source list
/// [`nomos_rules::Check_Dependency_Direction`] can judge -- one per workspace member
/// `store.Materialize` did not refuse.
fn Materialized_Dependency_Sources(
    facts: Vec<nomos_lang_rust_cargo::PackageFact>,
    store: &mut MemoryFactStore,
) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for package in facts
    {
        if store.Materialize(package.fact, &[]).is_ok()
        {
            let source = SourceFile::New(package.path, package.subject, String::new());
            sources.push(source);
        }
    }

    return sources;
}

/// The one finding a failed [`nomos_lang_rust_cargo::Materialize_Workspace`] call produces.
///
/// Attributed to the whole tree (`Subject_Of_Path("")`, the root's own subject per
/// `nomos_model::path`'s convention) rather than to any one file, because a `cargo metadata`
/// failure is not about any subject this run walked -- it is about whether the dependency
/// capability could answer at all. [`Applicability::ProviderUnavailable`] because the
/// provider is registered and offered; it ran and did not answer, which is exactly that
/// variant's own distinction from `MissingCapability`.
fn Dependency_Capability_Unavailable(error: &nomos_lang_rust_cargo::MetadataError) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(nomos_rules::DEPENDENCY_DIRECTION),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the dependency-edges capability could not be materialized, so dependency \
             direction was not judged for anything in this run: {error}"
        ),
        locations: Vec::new(),
    };
}

//! Materializing facts from already-walked source: syntax, reachability and dependency
//! edges.
//!
//! Grouped by what they share -- pure, per-source materialization over a [`Context`] this
//! module does not build -- not by the capability each one answers; see each function's own
//! doc for why the three stayed independent steps rather than one generalization.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_lang_rust::{FactContext, Materialization};
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;

use std::path::Path;

/// Produces one syntax fact per source and returns how many were written.
///
/// A file the provider refuses materializes nothing and is not dropped silently: the count
/// returned is the denominator the report prints beside the file count, and the rule's own
/// unread-subject finding names each one individually. Two independent readings of one
/// file -- this provider's and `nomos-rules`' own universe parser -- can refuse
/// independently, and the run is entitled to see which.
pub fn Materialize_Syntax(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> usize
{
    let production = Production(context);
    let mut written = 0_usize;

    for source in sources
    {
        let Materialization::Materialized(fact) =
            nomos_lang_rust::Materialize(source.subject, &source.text, production)
        else
        {
            continue;
        };

        // No dependency edges: a syntax fact is a leaf, read from one file's bytes and
        // from nothing this store holds.
        if store.Materialize(*fact, &[]).is_ok()
        {
            written = written.saturating_add(1);
        }
    }

    return written;
}

/// The reading context as the provider takes it.
fn Production(context: &Context) -> FactContext
{
    return FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Produces one `nomos.cap.controlflow.reachability` fact per source and returns how many
/// were written.
///
/// The identical shape [`Materialize_Syntax`] has: pure, per-source, no I/O --
/// `nomos_lang_rust::reachability::Materialize` takes only a subject, source text and this
/// same [`FactContext`], the tier-1 heuristic provider `P13-CONTROLFLOW-REACHABILITY-
/// CAPABILITY` built. Not folded into `Materialize_Syntax` itself: the two are independent
/// capabilities read from the same bytes, and a shared materialization step would hide
/// which one a caller is actually asking about, the same reason [`Materialize_Dependencies`]
/// stayed a second, separate step rather than a generalization of the first.
pub fn Materialize_Reachability(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> usize
{
    let production = Production(context);
    let mut written = 0_usize;

    for source in sources
    {
        let Materialization::Materialized(fact) =
            nomos_lang_rust::reachability::Materialize(source.subject, &source.text, production)
        else
        {
            continue;
        };

        // No dependency edges: a reachability fact is a leaf, the identical reasoning
        // Materialize_Syntax gives for its own fact.
        if store.Materialize(*fact, &[]).is_ok()
        {
            written = written.saturating_add(1);
        }
    }

    return written;
}

/// Runs `cargo metadata` over `root` through `launcher`, materializes one
/// `nomos.cap.dependency.edges` fact per workspace member, and returns the subjects a rule
/// can judge them under.
///
/// A second, independent materialization step beside [`Materialize_Syntax`] rather than a
/// generalization of it: `dependency.edges` has exactly one consumer today and nothing to
/// share or schedule against `syntax.items`'s own materialization, so composing this as one
/// more hardcoded step is `OD-HOST-004`'s "composition, not choice" again, not a case for
/// the shared demand planner `ARC-ROADMAP-001` still leaves for later.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero dependency findings" either, which is
/// exactly the vacuity [`Materialize_Syntax`]'s own `NoFacts` case exists to catch one layer
/// over. So a failed materialization returns no dependency sources and one synthetic
/// finding reporting why, rather than nothing at all.
pub fn Materialize_Dependencies<P: ProcessLauncher>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &P,
) -> DependencyMaterialization
{
    let production = Cargo_Production(context);

    let facts = match nomos_lang_rust_cargo::Materialize_Workspace(root, production, launcher)
    {
        Ok(facts) => facts,
        Err(error) => return DependencyMaterialization {
            sources: Vec::new(),
            findings: vec![Dependency_Capability_Unavailable(&error)],
        },
    };

    let sources = Materialized_Dependency_Sources(facts, store);

    return DependencyMaterialization { sources, findings: Vec::new() };
}

/// What materializing `dependency.edges` facts produced: the sources a rule can judge them
/// under, and any finding the materialization itself already raised (a failed `cargo
/// metadata` call, reported rather than judged) -- named rather than left as a positional
/// pair, so a caller reads which is which without re-deriving it from
/// [`Materialize_Dependencies`]'s own body.
pub struct DependencyMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
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

/// Every `dependency.edges` fact the store accepted, as the source list `Check_Dependency_
/// Direction` can judge -- one per workspace member `store.Materialize` did not refuse.
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

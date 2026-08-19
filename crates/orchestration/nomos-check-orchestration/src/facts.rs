//! Getting from already-walked source to the facts a rule can read.
//!
//! Moved from `nomos-cli::check::facts`, minus the walk itself (`Walked` stays in
//! `nomos-cli::check::sources` -- no [`nomos_platform::FileSystem`] directory-listing
//! operation exists, the same reason `nomos-cli::work::Published_Records` stayed put) and
//! minus the registry composition (`Composed` -- `crate::Run` calls
//! [`crate::composition::Registered`] directly and turns its own error into
//! [`crate::CheckOutcome::Contradictory`], so there is no longer an intermediate `ExitCode`
//! for this module to produce).

use nomos_analysis::Context;
use nomos_capability::Registry;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_lang_rust::{FactContext, Materialization};
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet, WorkspaceError};

use nomos_analysis::MemoryFactStore;

use crate::composition::Resolved_Configuration;
use std::path::Path;

/// The walk applied to an empty workspace, and the context every fact is filed under.
///
/// `variant` is the composition root's own -- what this binary was compiled as is read
/// through `env!`, which resolves against the crate that calls it, so it cannot be read
/// correctly from inside this one. `nomos-cli::check::composition::Host_Variant` still
/// computes it and hands it in, the way `nomos_work_orchestration::Run`'s `published`
/// argument crosses the same kind of boundary.
///
/// # Errors
///
/// [`WorkspaceError`] if the walk cannot be ingested -- in practice only reachable if
/// `sources` is empty, which the composition root has already refused before calling this
/// (`CheckOutcome::NoSource`), or if two entries name the same workspace-relative path.
pub fn Ingested(
    sources: &[SourceFile],
    registry: &Registry,
    variant: BuildVariant,
) -> Result<Context, WorkspaceError>
{
    let configuration = Resolved_Configuration(registry);
    let variant_id = variant.Id();
    let mut workspace = Workspace::Empty(variant, configuration);
    let checkout = As_One_Checkout(sources);

    let applied = workspace.Apply(&checkout)?;

    return Ok(Context {
        snapshot: applied.Snapshot(),
        variant: variant_id,
        configuration,
        generation: applied.Generation(),
    });
}

/// The whole walk as one change set, because a walk is one event.
///
/// Applying a file at a time would produce one generation per file, and every intermediate
/// one would describe a tree that never existed.
fn As_One_Checkout(sources: &[SourceFile]) -> WorkspaceChangeSet
{
    let mut checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout);

    for source in sources
    {
        checkout = checkout.Present(source.path.clone(), source.text.clone());
    }

    return checkout;
}

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

/// Runs `cargo metadata` over `root`, materializes one `nomos.cap.dependency.edges` fact
/// per workspace member, and returns the subjects a rule can judge them under.
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
pub fn Materialize_Dependencies(root: &Path, context: &Context, store: &mut MemoryFactStore) -> (Vec<SourceFile>, Vec<Finding>)
{
    let production = nomos_lang_rust_cargo::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };

    let facts = match nomos_lang_rust_cargo::Materialize_Workspace(root, production)
    {
        Ok(facts) => facts,
        Err(error) => return (Vec::new(), vec![Dependency_Capability_Unavailable(&error)]),
    };

    let mut sources = Vec::new();
    for package in facts
    {
        if store.Materialize(package.fact, &[]).is_ok()
        {
            sources.push(SourceFile::New(package.path, package.subject, String::new()));
        }
    }

    return (sources, Vec::new());
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

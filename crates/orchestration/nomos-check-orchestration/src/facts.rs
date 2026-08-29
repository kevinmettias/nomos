//! Getting from already-walked source to the facts a rule can read.
//!
//! Moved from `nomos-cli::check::facts`, minus the walk itself (`Walked` stays in
//! `nomos-cli::check::sources` -- no [`nomos_platform::FileSystem`] directory-listing
//! operation exists, the same reason `nomos-cli::work::Published_Records` stayed put) and
//! minus the registry composition (`Composed` -- `crate::Run` calls
//! [`crate::composition::Registered`] directly and turns its own error into
//! [`crate::CheckOutcome::Contradictory`], so there is no longer an intermediate `ExitCode`
//! for this module to produce).

mod dependency_materialization;

pub use dependency_materialization::{
    DependencyMaterialization, LintMaterialization, Materialize_Dependencies, Materialize_Lint,
    Materialize_Policy, Materialize_Reachability, Materialize_Syntax, PolicyMaterialization,
};

use nomos_analysis::Context;
use nomos_capability::Registry;
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet, WorkspaceError};

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
pub fn Ingested_Workspace(
    sources: &[SourceFile],
    registry: &Registry,
    variant: BuildVariant,
) -> Result<Context, WorkspaceError>
{
    use crate::composition::Resolved_Configuration;

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

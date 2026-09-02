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
mod policy_materialization;

pub use dependency_materialization::{
    DependencyMaterialization, LintMaterialization, Materialize_Dependencies, Materialize_Lint,
    Materialize_Policy, Materialize_Reachability, Materialize_Syntax, PolicyMaterialization,
};
pub use policy_materialization::{
    Materialize_Goals_Policy, Materialize_Limits_Policy, Materialize_Naming_Policy, Materialize_Scripting_Policy,
    Materialize_Words_Policy,
};

use nomos_analysis::Context;
use nomos_capability::Registry;
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet, WorkspaceError};

/// The walk applied to `workspace`, and the context every fact is filed under.
///
/// `variant` is the composition root's own -- what this binary was compiled as is read
/// through `env!`, which resolves against the crate that calls it, so it cannot be read
/// correctly from inside this one. `nomos-cli::check::composition::Host_Variant` still
/// computes it and hands it in, the way `nomos_work_orchestration::Run`'s `published`
/// argument crosses the same kind of boundary.
///
/// `workspace` is the caller's, not this function's own -- `OD-ANALYSIS-009`'s first real
/// increment. `None` builds a fresh [`Workspace::Empty`] and stores it back, reproducing
/// exactly what this function always did before this parameter existed. `Some` reuses the
/// workspace a previous call already advanced: [`Workspace::Apply`] diffs `sources` against
/// what it already holds, so a call that resubmits an unchanged file leaves it `Redundant`
/// and only a real edit advances the generation -- the caller does not compute that diff
/// itself, it falls out of applying the same, full, current source list to the same
/// [`Workspace`] a second time.
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
    workspace: &mut Option<Workspace>,
) -> Result<Context, WorkspaceError>
{
    use crate::composition::Resolved_Configuration;

    let configuration = Resolved_Configuration(registry);
    let variant_id = variant.Id();
    let checkout = As_One_Checkout(sources);

    let ws = workspace.get_or_insert_with(move || return Workspace::Empty(variant, configuration));
    let applied = ws.Apply(&checkout)?;

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

#[cfg(test)]
mod tests
{
    use super::*;

    /// A single real file ingests, and the [`Context`] it produces carries the registry's
    /// own configuration -- not a second, independently-computed one -- because a
    /// [`ConfigurationId`] is documented as a digest of the resolved policy this run
    /// composed, and there is exactly one of those per run.
    #[test]
    fn Test_Ingested_Workspace_Should_Carry_The_Registrys_Own_Configuration()
    {
        let registry = crate::composition::Registered().expect("fixture composition");
        let sources = [SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n")];

        let context = Ingested_Workspace(&sources, &registry, Test_Variant(), &mut None).expect("a single valid file must ingest");

        assert_eq!(
            context.configuration,
            crate::composition::Resolved_Configuration(&registry),
            "the ingested context's configuration must be the registry's own, not a second rendering of it"
        );
    }

    /// Two entries naming the same workspace-relative path cannot both be present in one
    /// checkout, so ingesting them must refuse rather than silently keep one and drop the
    /// other.
    #[test]
    fn Test_Ingested_Workspace_Should_Refuse_Two_Sources_At_One_Path()
    {
        let registry = crate::composition::Registered().expect("fixture composition");
        let sources = [
            SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn one() {}\n"),
            SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn two() {}\n"),
        ];

        let ingested = Ingested_Workspace(&sources, &registry, Test_Variant(), &mut None);

        assert!(ingested.is_err(), "duplicate paths must not be ingested as one checkout");
    }

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
    }
}

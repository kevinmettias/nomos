//! Getting from a checkout to the facts a rule can read.

use super::{SourceFile, Registry, Context, Path, Write, ExitCode, Walked, Registered, Resolved_Configuration, Host_Variant, Workspace, WorkspaceChangeSet, ChangeSource, MemoryFactStore, Materialization, FactContext};

/// Everything the run needs before it can judge anything.
pub(super) struct Prepared
{
    pub(super) sources: Vec<SourceFile>,
    pub(super) registry: Registry,
    pub(super) context: Context,
}

/// Walks the tree and ingests it as one workspace state.
pub(super) fn Prepare(root: &Path, stderr: &mut impl Write) -> Result<Prepared, ExitCode>
{
    let sources = Walked(root, stderr)?;
    let registry = Registered();
    let context = Ingested(&sources, root, &registry, stderr)?;

    return Ok(Prepared {
        sources,
        registry,
        context,
    });
}

/// The walk applied to an empty workspace, and the context every fact is filed under.
pub(super) fn Ingested(
    sources: &[SourceFile],
    root: &Path,
    registry: &Registry,
    stderr: &mut impl Write,
) -> Result<Context, ExitCode>
{
    let configuration = Resolved_Configuration(registry);
    let variant = Host_Variant();
    let mut workspace = Workspace::Empty(variant.clone(), configuration);
    let checkout = As_One_Checkout(sources);

    let applied = workspace.Apply(&checkout).map_err(|error| {
        let _ignored = writeln!(
            stderr,
            "the walk of `{}` could not be ingested as a workspace state: {error:?}",
            root.display()
        );

        return ExitCode::Unreadable;
    })?;

    return Ok(Context {
        snapshot: applied.Snapshot(),
        variant: variant.Id(),
        configuration,
        generation: applied.Generation(),
    });
}

/// The whole walk as one change set, because a walk is one event.
///
/// Applying a file at a time would produce one generation per file, and every intermediate
/// one would describe a tree that never existed.
pub(super) fn As_One_Checkout(sources: &[SourceFile]) -> WorkspaceChangeSet
{
    let mut checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout);

    for source in sources
    {
        checkout = checkout.Present(source.path.clone(), source.text.clone());
    }

    return checkout;
}

/// A walk that read source and produced no fact from any of it.
///
/// The same lie as an empty walk, one layer in: the rule would resolve every mirror claim
/// against an empty index and, because the index is empty, refuse to block on any of them.
pub(super) fn Nothing_Materialized(read: usize, root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stderr,
        "{read} file(s) were read under `{}` and no syntax fact was materialized for any \
         of them, so no mirror claim could be resolved.\n\
         A clean result here would mean only that the analysis never ran.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Produces one syntax fact per source and returns how many were written.
///
/// A file the provider refuses materializes nothing and is not dropped silently: the count
/// returned is the denominator the report prints beside the file count, and the rule's own
/// unread-subject finding names each one individually. Two independent readings of one
/// file — this provider's and `nomos-rules`' own universe parser — can refuse
/// independently, and the run is entitled to see which.
pub(super) fn Materialize_Syntax(
    sources: &[SourceFile],
    context: &Context,
    store: &mut MemoryFactStore,
) -> usize
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
pub(super) fn Production(context: &Context) -> FactContext
{
    return FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

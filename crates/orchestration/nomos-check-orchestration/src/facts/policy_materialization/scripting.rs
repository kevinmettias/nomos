//! Reading `standards.json`'s scripting policy into one fact the store can answer with.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// scripting.policy` fact it declares into `store`.
///
/// Same shape as its two siblings above, but the consequence of *not* calling it differs and
/// that is the reason this one mattered most. `Check_Declared_Tooling_Language_For_Scripts`
/// has no prior default to fall back to -- the rule never existed before the capability did
/// -- so an absent fact makes it report nothing rather than report against an assumption.
/// Without this materialization the rule ran in every real check and could never fire.
/// Returns `1` if the fact landed in `store`, `0` if it did not.
pub fn Materialize_Scripting_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Scripting_Production(context);

    let Ok(fact) = nomos_repo_policy::scripting::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(store.Materialize(fact.fact, &[]).is_ok());
}

/// The reading context as `nomos_repo_policy::scripting`'s provider takes it, at the same
/// four-field carry every other provider here states.
fn Scripting_Production(context: &Context) -> nomos_repo_policy::scripting::FactContext
{
    return nomos_repo_policy::scripting::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

//! Reading `standards.json`'s naming policy into one fact the store can answer with.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// naming.policy` fact it declares into `store`.
///
/// A missing or unreadable `standards.json` is not reported as a finding: `OD-CAPABILITY-004`
/// already decided an absent optional capability is silently "no override" from whichever
/// caller reads it (`nomos_rules::Resolve_Case`), so a materialization that cannot produce
/// the fact simply leaves the store without one, the same way an unmaterialized syntax fact
/// for one file does not abort judging the rest. Returns `1` if the fact landed in `store`,
/// `0` if it did not.
pub fn Materialize_Naming_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Naming_Production(context);

    let Ok(fact) = nomos_repo_policy::naming::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(store.Materialize(fact.fact, &[]).is_ok());
}

/// The reading context as `nomos_repo_policy::naming`'s provider takes it -- the same
/// four-field carry [`super::super::dependency_materialization::Deny_Production`] already
/// does for `nomos_lang_rust_deny`, since every provider's own `FactContext` shape agrees on
/// snapshot/variant/configuration/generation.
fn Naming_Production(context: &Context) -> nomos_repo_policy::naming::FactContext
{
    return nomos_repo_policy::naming::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

//! Reading `standards.json`'s goals policy into one fact the store can answer with.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

use crate::facts::currency::Materialized_Or_Already_Current;

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// goals.policy` fact it declares into `store`.
///
/// The same shape as its three siblings above. The difference is what an absent fact means
/// to the one rule that reads it: `Check_Goals_And_Parts_Line_Up` treats no policy and an
/// empty policy as the same answer, because a repository that declared no purposes has not
/// taken goal traceability on and there is no prior default to fall back to either. So this
/// materialization changes nothing for a repository like this one -- which declares no goals
/// -- and everything for one that does. Returns `1` once `store` holds a current fact for this
/// family -- whether this call wrote it, or [`crate::facts::currency`]'s own check found the
/// store already serving one byte-for-byte identical to it -- and `0` for a reason it does
/// not.
pub fn Materialize_Goals_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Goals_Production(context);

    let Ok(fact) = nomos_repo_policy::goals::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(Materialized_Or_Already_Current(fact.fact, context, store));
}

/// The reading context as `nomos_repo_policy::goals`'s provider takes it, at the same
/// four-field carry every other provider here states.
fn Goals_Production(context: &Context) -> nomos_repo_policy::goals::FactContext
{
    return nomos_repo_policy::goals::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

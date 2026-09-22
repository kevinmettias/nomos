//! Reading `standards.json`'s limits policy into one fact the store can answer with.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

use crate::facts::currency::Materialized_Or_Already_Current;

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// limits.policy` fact it declares into `store`.
///
/// The identical shape [`super::naming::Materialize_Naming_Policy`] has, and absent for the
/// identical `OD-CAPABILITY-004` reason: the five rules that read this capability through
/// `nomos_rules`' own `Resolve_Limit` each treat a failed read as "no override" and keep
/// their prior hardcoded threshold.
///
/// Worth knowing what this does and does not buy on *this* repository: `standards.json`
/// declares exactly the numbers those fallbacks already carry, so no finding here moves.
/// What moves is that the thresholds are read rather than assumed. Returns `1` once `store` holds a current fact for this
/// family -- whether this call wrote it, or [`crate::facts::currency`]'s own check found the
/// store already serving one byte-for-byte identical to it -- and `0` for a reason it does
/// not.
pub fn Materialize_Limits_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Limits_Production(context);

    let Ok(fact) = nomos_repo_policy::limits::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(Materialized_Or_Already_Current(fact.fact, context, store));
}

/// The reading context as `nomos_repo_policy::limits`'s provider takes it, at the same
/// four-field carry every other provider here states.
fn Limits_Production(context: &Context) -> nomos_repo_policy::limits::FactContext
{
    return nomos_repo_policy::limits::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

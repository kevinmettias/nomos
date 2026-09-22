//! Reading `nomos-architecture.json` into one fact the store can answer with.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

use crate::facts::currency::Materialized_Or_Already_Current;

/// Reads `root`'s own `nomos-architecture.json` through `filesystem` and writes the one
/// `nomos.cap.architecture.declaration` fact it declares into `store`.
///
/// The same shape its five siblings here have, over a different file, and for a reason
/// `nomos_repo_policy::architecture::reading` states in full: `standards.json` is shared with a
/// tool that decodes it with unknown fields disallowed, so the one key this repository would
/// own outright is not available in it.
///
/// Unlike every policy section beside it, an absent fact here is not a fallback. The three
/// dependency rules report a declaration they cannot read rather than judging against a
/// remembered default, because there is no default an architecture could have. Returns `1` once `store` holds a current fact for this
/// family -- whether this call wrote it, or [`crate::facts::currency`]'s own check found the
/// store already serving one byte-for-byte identical to it -- and `0` for a reason it does
/// not.
pub fn Materialize_Architecture<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Architecture_Production(context);

    let Ok(fact) = nomos_repo_policy::architecture::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(Materialized_Or_Already_Current(fact.fact, context, store));
}

/// The reading context as `nomos_repo_policy::architecture`'s provider takes it, at the same
/// four-field carry every other provider here states.
fn Architecture_Production(context: &Context) -> nomos_repo_policy::architecture::FactContext
{
    return nomos_repo_policy::architecture::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

//! Reading `standards.json`'s words policy into one fact the store can answer with.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

use crate::facts::currency::Materialized_Or_Already_Current;

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// words.policy` fact it declares into `store`.
///
/// The last of the four, and the only one whose payload *extends* a default rather than
/// replacing or overriding it: `Check_Abbreviations` ships code-standards' own approved and
/// banned vocabularies and reads this capability for a repository's own additions to the
/// approved half. So an absent fact is not a missing threshold or a silent rule -- it is the
/// shipped vocabulary with nothing added, which is a perfectly good answer and the one every
/// check gave before this call existed. Returns `1` once `store` holds a current fact for this
/// family -- whether this call wrote it, or [`crate::facts::currency`]'s own check found the
/// store already serving one byte-for-byte identical to it -- and `0` for a reason it does
/// not.
pub fn Materialize_Words_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Words_Production(context);

    let Ok(fact) = nomos_repo_policy::words::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(Materialized_Or_Already_Current(fact.fact, context, store));
}

/// The reading context as `nomos_repo_policy::words`'s provider takes it, at the same
/// four-field carry every other provider here states.
fn Words_Production(context: &Context) -> nomos_repo_policy::words::FactContext
{
    return nomos_repo_policy::words::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

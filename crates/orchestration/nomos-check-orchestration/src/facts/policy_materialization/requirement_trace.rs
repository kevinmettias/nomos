//! Reading `tests/contract/requirements/` into one fact the store can answer with.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

use crate::facts::currency::Materialized_Or_Already_Current;

/// Reads `root`'s own `tests/contract/requirements/` through `filesystem` and writes the
/// one `nomos.cap.requirement.trace` fact it declares into `store`.
///
/// Unlike its four `standards.json`-reading siblings above, this one can never fail to
/// produce a fact: [`nomos_cap_requirement_trace::Materialize_Workspace`] is infallible by
/// its own design, because a missing `tests/contract/requirements/` directory is this
/// capability's ordinary case (every repository this rule judges except this one, today)
/// rather than a read failure — see that function's own module doc. Returns `1` once `store` holds a current fact for this
/// family -- whether this call wrote it, or [`crate::facts::currency`]'s own check found the
/// store already serving one byte-for-byte identical to it -- and `0` for a reason it does
/// not.
pub fn Materialize_Requirement_Trace<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Requirement_Trace_Production(context);
    let fact = nomos_cap_requirement_trace::Materialize_Workspace(root, production, filesystem);

    return usize::from(Materialized_Or_Already_Current(fact.fact, context, store));
}

/// The reading context as `nomos_cap_requirement_trace`'s provider takes it, at the same
/// four-field carry every other provider here states.
fn Requirement_Trace_Production(context: &Context) -> nomos_cap_requirement_trace::FactContext
{
    return nomos_cap_requirement_trace::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

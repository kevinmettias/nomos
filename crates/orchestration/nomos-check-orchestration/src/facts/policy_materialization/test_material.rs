//! Reading `nomos-test-material.json`'s test-material policy into one fact the store can
//! answer with.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

use crate::facts::currency::Materialized_Or_Already_Current;

/// Reads `root`'s own `nomos-test-material.json` through `filesystem` and writes the one
/// `nomos.cap.test.material.policy` fact it declares into `store`.
///
/// The sixth of the six, and the one whose payload *extends* a default rather than
/// replacing it: every rule that reads this capability ships its own toolchain-fixed clauses
/// (`tests/`, `examples/`, `_test.go`, …) and reads this capability for a repository's own
/// additions to them. So an absent fact is not a missing threshold or a silent rule — it is
/// the fixed clauses with nothing added, which is a perfectly good answer and the one every
/// check gave before this call existed. Returns `1` once `store` holds a current fact for this
/// family -- whether this call wrote it, or [`crate::facts::currency`]'s own check found the
/// store already serving one byte-for-byte identical to it -- and `0` for a reason it does
/// not.
pub fn Materialize_Test_Material_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Test_Material_Production(context);

    let Ok(fact) = nomos_repo_policy::test_material::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(Materialized_Or_Already_Current(fact.fact, context, store));
}

/// The reading context as `nomos_repo_policy::test_material`'s provider takes it, at the same
/// four-field carry every other provider here states.
fn Test_Material_Production(context: &Context) -> nomos_repo_policy::test_material::FactContext
{
    return nomos_repo_policy::test_material::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

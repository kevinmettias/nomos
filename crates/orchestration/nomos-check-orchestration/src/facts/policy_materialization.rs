//! Turning a repository-declared policy capability's own provider into a fact the store can
//! answer `facts.Require` with.
//!
//! Unlike [`super::dependency_materialization`]'s subprocess-backed providers, a policy
//! capability like `nomos.cap.naming.policy` has no subprocess and no per-file source list a
//! rule needs handed back: `nomos_rules::Resolve_Case` already reads it directly, by its own
//! fixed empty-path subject, from whichever real sources `Run` already walked. So this
//! module's own materialization step writes at most one fact into the store and reports how
//! many the store now holds current -- `1` once it does, whether this call wrote the fact or
//! [`super::currency`]'s own check found the store already serving one byte-for-byte
//! identical to it, and `0` for any reason it holds none. That is the identical coverage
//! meaning [`super::dependency_materialization::Materialize_Syntax`] already returns a count
//! by for the identical reason: a caller that does not need the answer is free to ignore it,
//! and one that does (a future caller, or a test) is not left re-deriving it from the store's
//! own side effects -- and `nomos_analysis::MemoryFactStore::Materializations`' own delta,
//! not this count, is what answers how much work a call did.
//!
//! # Currency
//!
//! The write goes through [`super::currency`], so a second call over a reused store does not
//! file a policy fact whose bytes have not moved a second time.
//! `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY` is that change, and that module's doc
//! carries the whole argument -- including why it is not the demand planner `OD-RULES-009`
//! declines and `OD-ROADMAP-003` conditions its own lapse on. What is worth knowing here is
//! the limit: the provider still reads its policy file on every call its family is demanded
//! on. Only the store write, and with it the family's place in
//! `crate::run_context::capabilities`' own `changed` list, is skipped.
//!
//! # One function, where there were eight files
//!
//! This module held eight submodules, one per provider, and its own doc said why: reading
//! `standards.json` five times was five responsibilities that happened to share a spelling,
//! and the shared `Materialize_` prefix named a boundary the module had not drawn. The
//! provider's *name* was that boundary -- it was the only thing the eight bodies did not
//! share.
//!
//! `OD-ROADMAP-005` decision item 1 moved the name out, and the boundary went with it.
//! Against a [`WorkspacePolicyProvider`] the eight bodies are not merely similar, they are
//! the same four statements with nothing left to differ in, so eight files would be eight
//! copies of one function distinguished by a comment. The boundary did not disappear: it is
//! declared where the names now live, one row per family in
//! `crate::composition::Composed_Providers`, and one arm per family in
//! `crate::run_context::capabilities::Materialize_Policy_Family`, whose match is total over
//! `nomos_rules::RequiredFact` so that a family added to the set is a compile error until it
//! is placed. That arm, not a file here, is what `OD-ROADMAP-003` requires a materialization
//! section to be: a declared row, never a condition.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::FileSystem;
use std::path::Path;

use crate::composed_providers::WorkspacePolicyProvider;
use crate::facts::currency::Materialized_Or_Already_Current;

/// The root a policy is declared in, the context its fact is filed under, the store it is
/// filed into and the filesystem it is read through -- grouped into one value so
/// [`Materialize_Policy_Fact`] names the provider as its own second parameter rather than as
/// its fifth.
pub struct PolicyReading<'a, Fs: FileSystem>
{
    pub root: &'a Path,
    pub context: &'a Context,
    pub store: &'a mut MemoryFactStore,
    pub filesystem: &'a Fs,
}

/// Reads `reading.root`'s own declaration through `provider` and writes the one fact it
/// declares into `reading.store`.
///
/// A missing or unreadable declaration is not reported as a finding, for seven of the eight
/// families this is called for: `OD-CAPABILITY-004` already decided an absent optional
/// capability is silently "no override" from whichever caller reads it
/// (`nomos_rules::Resolve_Case`), so a materialization that cannot produce the fact simply
/// leaves the store without one, the same way an unmaterialized syntax fact for one file
/// does not abort judging the rest.
///
/// The eighth is `nomos.cap.architecture.declaration`, and its absence is not a fallback: the
/// three dependency rules report a declaration they could not read rather than judging
/// against a remembered default, because there is no default an architecture could have.
/// That difference is in the rules that read the fact, not here -- this function's answer for
/// an absent fact is the same `0` either way, which is why the eight families share it.
///
/// Returns `1` once `reading.store` holds a current fact for this family -- whether this call
/// wrote it, or [`crate::facts::currency`]'s own check found the store already serving one
/// byte-for-byte identical to it -- and `0` for any reason it does not.
pub fn Materialize_Policy_Fact<Fs: FileSystem>(reading: PolicyReading<'_, Fs>, provider: WorkspacePolicyProvider<Fs>) -> usize
{
    let Some(fact) = provider(reading.root, reading.context, reading.filesystem)
    else
    {
        return 0;
    };

    return usize::from(Materialized_Or_Already_Current(fact, reading.context, reading.store));
}

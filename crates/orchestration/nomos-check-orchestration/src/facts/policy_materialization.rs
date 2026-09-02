//! Turning a repository-declared policy capability's own `Materialize_Workspace` into a
//! fact the store can answer `facts.Require` with.
//!
//! Unlike [`super::dependency_materialization`]'s three providers, a policy capability like
//! `nomos.cap.naming.policy` has no subprocess and no per-file source list a rule needs
//! handed back: `nomos_rules::Resolve_Case` already reads it directly, by its own fixed
//! empty-path subject, from whichever real sources `Run` already walked. So this module's
//! own materialization step writes one fact into the store and returns nothing, the
//! identical shape [`super::dependency_materialization::Materialize_Reachability`] already
//! has for the identical reason.

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
/// for one file does not abort judging the rest.
pub fn Materialize_Naming_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs)
{
    let production = Naming_Production(context);

    if let Ok(fact) = nomos_repo_standards::Materialize_Workspace(root, production, filesystem)
    {
        let _ = store.Materialize(fact.fact, &[]);
    }
}

/// The reading context as `nomos_repo_standards`'s provider takes it -- the same
/// four-field carry [`super::dependency_materialization::Deny_Production`] already does for
/// `nomos_lang_rust_deny`, since every provider's own `FactContext` shape agrees on
/// snapshot/variant/configuration/generation.
fn Naming_Production(context: &Context) -> nomos_repo_standards::FactContext
{
    return nomos_repo_standards::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

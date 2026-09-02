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

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// limits.policy` fact it declares into `store`.
///
/// The identical shape [`Materialize_Naming_Policy`] has, and absent for the identical
/// `OD-CAPABILITY-004` reason: the five rules that read this capability through
/// `nomos_rules`' own `Resolve_Limit` each treat a failed read as "no override" and keep
/// their prior hardcoded threshold.
///
/// Worth knowing what this does and does not buy on *this* repository: `standards.json`
/// declares exactly the numbers those fallbacks already carry, so no finding here moves.
/// What moves is that the thresholds are read rather than assumed.
pub fn Materialize_Limits_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs)
{
    let production = Limits_Production(context);

    if let Ok(fact) = nomos_repo_limits::Materialize_Workspace(root, production, filesystem)
    {
        let _ = store.Materialize(fact.fact, &[]);
    }
}

fn Limits_Production(context: &Context) -> nomos_repo_limits::FactContext
{
    return nomos_repo_limits::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// scripting.policy` fact it declares into `store`.
///
/// Same shape as its two siblings above, but the consequence of *not* calling it differs and
/// that is the reason this one mattered most. `Check_Declared_Tooling_Language_For_Scripts`
/// has no prior default to fall back to -- the rule never existed before the capability did
/// -- so an absent fact makes it report nothing rather than report against an assumption.
/// Without this materialization the rule ran in every real check and could never fire.
pub fn Materialize_Scripting_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs)
{
    let production = Scripting_Production(context);

    if let Ok(fact) = nomos_repo_scripting::Materialize_Workspace(root, production, filesystem)
    {
        let _ = store.Materialize(fact.fact, &[]);
    }
}

fn Scripting_Production(context: &Context) -> nomos_repo_scripting::FactContext
{
    return nomos_repo_scripting::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

//! Turning a repository-declared policy capability's own `Materialize_Workspace` into a
//! fact the store can answer `facts.Require` with.
//!
//! Unlike [`super::dependency_materialization`]'s three providers, a policy capability like
//! `nomos.cap.naming.policy` has no subprocess and no per-file source list a rule needs
//! handed back: `nomos_rules::Resolve_Case` already reads it directly, by its own fixed
//! empty-path subject, from whichever real sources `Run` already walked. So this module's
//! own materialization step writes at most one fact into the store and reports how many
//! landed -- `1` once `store.Materialize` accepted it, `0` for any reason it did not, the
//! identical shape [`super::dependency_materialization::Materialize_Syntax`] already
//! returns a count by for the identical reason: a caller that does not need the answer is
//! free to ignore it, and one that does (a future caller, or a test) is not left
//! re-deriving it from the store's own side effects.

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
/// four-field carry [`super::dependency_materialization::Deny_Production`] already does for
/// `nomos_lang_rust_deny`, since every provider's own `FactContext` shape agrees on
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
/// What moves is that the thresholds are read rather than assumed. Returns `1` if the fact
/// landed in `store`, `0` if it did not.
pub fn Materialize_Limits_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Limits_Production(context);

    let Ok(fact) = nomos_repo_policy::limits::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(store.Materialize(fact.fact, &[]).is_ok());
}

fn Limits_Production(context: &Context) -> nomos_repo_policy::limits::FactContext
{
    return nomos_repo_policy::limits::FactContext {
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
/// Returns `1` if the fact landed in `store`, `0` if it did not.
pub fn Materialize_Scripting_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Scripting_Production(context);

    let Ok(fact) = nomos_repo_policy::scripting::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(store.Materialize(fact.fact, &[]).is_ok());
}

fn Scripting_Production(context: &Context) -> nomos_repo_policy::scripting::FactContext
{
    return nomos_repo_policy::scripting::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// goals.policy` fact it declares into `store`.
///
/// The same shape as its three siblings above. The difference is what an absent fact means
/// to the one rule that reads it: `Check_Goals_And_Parts_Line_Up` treats no policy and an
/// empty policy as the same answer, because a repository that declared no purposes has not
/// taken goal traceability on and there is no prior default to fall back to either. So this
/// materialization changes nothing for a repository like this one -- which declares no goals
/// -- and everything for one that does. Returns `1` if the fact landed in `store`, `0` if it
/// did not.
pub fn Materialize_Goals_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Goals_Production(context);

    let Ok(fact) = nomos_repo_policy::goals::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(store.Materialize(fact.fact, &[]).is_ok());
}

fn Goals_Production(context: &Context) -> nomos_repo_policy::goals::FactContext
{
    return nomos_repo_policy::goals::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Reads `root`'s own `standards.json` through `filesystem` and writes the one `nomos.cap.
/// words.policy` fact it declares into `store`.
///
/// The last of the four, and the only one whose payload *extends* a default rather than
/// replacing or overriding it: `Check_Abbreviations` ships code-standards' own approved and
/// banned vocabularies and reads this capability for a repository's own additions to the
/// approved half. So an absent fact is not a missing threshold or a silent rule -- it is the
/// shipped vocabulary with nothing added, which is a perfectly good answer and the one every
/// check gave before this call existed. Returns `1` if the fact landed in `store`, `0` if it
/// did not.
pub fn Materialize_Words_Policy<Fs: FileSystem>(root: &Path, context: &Context, store: &mut MemoryFactStore, filesystem: &Fs) -> usize
{
    let production = Words_Production(context);

    let Ok(fact) = nomos_repo_policy::words::Materialize_Workspace(root, production, filesystem)
    else
    {
        return 0;
    };

    return usize::from(store.Materialize(fact.fact, &[]).is_ok());
}

fn Words_Production(context: &Context) -> nomos_repo_policy::words::FactContext
{
    return nomos_repo_policy::words::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

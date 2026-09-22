//! Syntax and reachability facts, materialized per source from already-walked text.

use nomos_analysis::{Context, FactKey, FactStore, GuaranteeDigest, InputDigest, MemoryFactStore};
use nomos_rules::SourceFile;

use crate::composed_providers::{Recognized_Syntax_Provider, SubjectFactProvider, SyntaxProvider};
use crate::facts::currency::Materialized_Or_Already_Current;

/// Produces one syntax fact per source and returns how many sources now have a current
/// one -- whether this call wrote it or [`Is_Already_Current`] found the store already held
/// it. `crate::run_context::Materialized_Syntax_Facts`'s own vacuity gate and
/// `Examined::facts`'s own coverage statistic both read this as "how many of `sources`
/// this run can judge," and neither means "how much work this call actually did" --
/// [`nomos_analysis::MemoryFactStore::Materializations`]'s own delta across two calls is
/// the honest answer to that question, and what `P40-INCREMENTAL-SKIP-UNCHANGED-
/// SUBJECTS`'s own test now reads instead (`P40-INCREMENTAL-SKIP-COVERAGE-REGRESSION`
/// fixed this return value back to the coverage meaning every real caller already
/// depended on, after briefly repurposing it to mean the newly-written count with nothing
/// downstream updated to match).
///
/// A file no composed provider recognizes, or one its own recognized provider refuses to
/// parse, materializes nothing and is not dropped silently: the count returned is the
/// denominator the report prints beside the file count, and the rule's own unread-subject
/// finding names each one individually. Two independent readings of one file -- this
/// provider's and `nomos-rules`' own universe parser -- can refuse independently, and the
/// run is entitled to see which.
///
/// Dispatches per [`Recognized_Syntax_Provider`], `OD-CAPABILITY-009`'s write-side half:
/// `crate::run_context`'s own enrichment step narrows the read side to the same provider identity
/// through the identical function over the identical `providers` list, and the two must agree
/// for `Key_From` to ever find what this function wrote. Neither side names a provider crate:
/// which offers are composed is `crate::composition`'s answer, and this function works over
/// however many rows it is handed.
///
/// # Skipping a subject the store already has current
///
/// `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS`: before parsing, [`Is_Already_Current`] builds
/// the exact [`FactKey`] parsing `source` would produce -- from `source.text`'s own raw
/// digest, never from a parse -- and asks `store` whether it already holds a live fact
/// under it. A caller that reuses the same `store` (and therefore the same, monotonically
/// advancing generation `crate::facts::Ingested_Workspace` reads off a reused `Workspace`)
/// across two calls pays for a real parse only for a subject whose own bytes moved since
/// the store last saw it -- but still counts that subject as covered either way.
pub fn Materialize_Syntax(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore, providers: &[SyntaxProvider]) -> usize
{
    let mut current = 0_usize;

    for source in sources
    {
        if Try_Materialize_Source_Syntax(source, context, store, providers)
        {
            current = current.saturating_add(1);
        }
    }

    return current;
}

/// One source's contribution to [`Materialize_Syntax`]'s own count: `true` when `source`
/// has a current `nomos.cap.syntax.items` fact once this returns -- either because
/// [`Is_Already_Current`] found the store already held one, or because this call wrote one --
/// and `false` for a path no composed provider recognizes, or for a recognized path its own
/// provider could not parse.
///
/// Extracted from [`Materialize_Syntax`]'s own loop rather than inlined there, and not a
/// promise about coverage: a `false` is still counted by the caller exactly the way
/// [`Materialize_Syntax`]'s own doc counts every uncovered source -- never dropped
/// silently, named by the rule's own unread-subject finding instead.
fn Try_Materialize_Source_Syntax(source: &SourceFile, context: &Context, store: &mut MemoryFactStore, providers: &[SyntaxProvider]) -> bool
{
    let Some(provider) = Recognized_Syntax_Provider(providers, &source.path)
    else
    {
        return false;
    };

    if Is_Already_Current(source, context, store, provider)
    {
        return true;
    }

    let Some(fact) = (provider.materialize)(source.subject, &source.text, context)
    else
    {
        return false;
    };

    // No dependency edges: a syntax fact is a leaf, read from one file's bytes and from
    // nothing this store holds.
    return store.Materialize(*fact, &[]).is_ok();
}

/// Whether `store` already holds a live `nomos.cap.syntax.items` fact for `source` at
/// `context.generation`, under `provider`'s own identity.
///
/// Built from the same public pieces a syntax provider combines into a [`FactKey`]
/// internally -- the capability contract, the provider identity and the declared guarantee --
/// restated here rather than exposed as a shared constructor: this is the one caller outside
/// a provider that ever needs to know what a fact it did not produce would be keyed under,
/// and [`InputDigest::Of`] over `source.text`'s own bytes is a raw hash, not a parse -- the
/// whole reason this check can run before one.
///
/// The identity and the guarantee are read off the composed row rather than from a named
/// crate, which is the one thing this function stopped doing: the two it used to name were
/// the whole of what made the pre-parse check impossible to extend to a third provider.
fn Is_Already_Current(source: &SourceFile, context: &Context, store: &MemoryFactStore, provider: &SyntaxProvider) -> bool
{
    let key = Syntax_Fact_Key(source, provider, context);

    return store.Current(&key.At(context.generation), context.generation).is_some();
}

/// The [`FactKey`] a real materialization of `source` through `provider` would produce.
fn Syntax_Fact_Key(source: &SourceFile, provider: &SyntaxProvider, context: &Context) -> FactKey
{
    return FactKey {
        contract: nomos_cap_syntax::Capability(),
        contract_version: nomos_cap_syntax::CONTRACT_VERSION,
        subject: source.subject,
        semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
        provider: provider.provider.clone(),
        provider_version: nomos_cap_syntax::CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&provider.guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

/// Produces one `nomos.cap.controlflow.reachability` fact per source through `provider` and
/// returns how many sources now have a current one.
///
/// The identical shape [`Materialize_Syntax`] has: pure, per-source, no I/O -- the composed
/// reachability provider takes only a subject, source text and the same [`Context`], which is
/// why [`SubjectFactProvider`] is its port rather than the whole-workspace ones the three
/// subprocess-backed families take. Not folded into `Materialize_Syntax` itself: the two are
/// independent capabilities read from the same bytes, and a shared materialization step
/// would hide which one a caller is actually asking about, the same reason
/// [`super::Materialize_Dependencies`] stayed a second, separate step rather than a
/// generalization of the first.
///
/// # Skipping a subject the store already has current
///
/// `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY`: the write goes through
/// [`crate::facts::currency`], so a source whose reachability answer has not moved since the
/// reused store last saw it is not filed a second time -- and the count returned is
/// [`Materialize_Syntax`]'s own coverage meaning, not a newly-written count.
///
/// This check runs *after* the provider has read the source, unlike [`Is_Already_Current`]
/// one function up, which runs before the parse. That is not an oversight and it is the one
/// difference between the two: the digest this fact's `semantic_inputs` is filed under -- the
/// only thing a caller could rebuild the key from without reading the source -- is private to
/// the provider that produces it. Publishing it would let this step skip the read as well,
/// and needs territory in a provider crate that this crate's own item does not hold.
pub fn Materialize_Reachability(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore, provider: SubjectFactProvider) -> usize
{
    let mut current = 0_usize;

    for source in sources
    {
        let Some(fact) = provider(source.subject, &source.text, context)
        else
        {
            continue;
        };

        if Materialized_Or_Already_Current(*fact, context, store)
        {
            current = current.saturating_add(1);
        }
    }

    return current;
}

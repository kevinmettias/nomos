//! Whether a materializer's write would tell the store anything it does not already hold.
//!
//! `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS` gave exactly one family a currency check:
//! [`super::dependency_materialization::Materialize_Syntax`]'s own `Is_Already_Current`,
//! which rebuilds the [`nomos_analysis::FactKey`] a parse would produce -- from the source's
//! raw bytes, before any parse -- and skips the subject when `store` already holds a live
//! fact under it. Every other materializer this crate composes wrote unconditionally, so a
//! family other than `nomos.cap.syntax.items` read as *changed* on every call it was
//! selected for, and
//! `crate::run_context::rule_reassessment_cache::RuleReassessmentCache` re-judged every rule
//! declaring it. `P40-INCREMENTAL-DEMAND-DRIVEN-RECOMPUTE-2` was declined on exactly that
//! measurement. This module is the check the other eleven families now share.
//!
//! # Why this is not the demand planner `OD-RULES-009` has declined
//!
//! `OD-RULES-009` guards one sentence shape, and `OD-ROADMAP-003` restates it as the
//! constraint its own lapse is conditioned on: a materialization section is a declared
//! constant -- a `RequiredFact` a rule's descriptor names and
//! `crate::run_context::capabilities`' own `Demanded_Families` reads -- and **never a
//! condition that consults store state, cost or prior materialization**. The forbidden
//! sentence is named there literally: *materialize this family unless the store already
//! holds it*.
//!
//! Nothing here is that sentence, and the difference is which question the store is allowed
//! to answer. **Demand decides whether a family is produced; currency decides only whether
//! the write is redundant.** `Demanded_Families` is untouched: it is still the union of
//! `nomos_rules::RuleDescriptor::requires` over the selection, it still reads no store
//! state, and every section a selection demands still runs its provider in full. What this
//! module changes is one step later and one subject at a time -- the provider has already
//! answered, and the answer is either new to the store or byte-for-byte the fact the store
//! is already serving under that identity. Declining to file a second copy of a fact the
//! store already holds is not a decision about *what to produce*; the production has
//! happened.
//!
//! That is the side of the line `Materialize_Syntax` has been on since
//! `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS`, and this check is strictly further from the
//! planner than that one: syntax's own `Is_Already_Current` runs *before* the parse and so
//! does skip a provider's work, keyed on a digest a caller can compute without the provider.
//! [`Is_Already_Current`] here runs *after* the provider has answered and skips no provider
//! work at all. Reading this module as the planner would therefore convict the check the
//! records already bless, on weaker facts.
//!
//! # Why the comparison is against the store's own fact and not a digest kept beside it
//!
//! The obvious alternative is a side table of input digests, one per family and subject,
//! carried across calls the way `crate::RunContext`'s own workspace and store are. It was
//! not built, for two reasons.
//!
//! The first is that none of these providers publishes an input digest to record. Every
//! non-syntax provider in this workspace files its fact with an empty `semantic_inputs`
//! (`nomos_lang_rust_cargo`, `nomos_lang_rust_clippy`, `nomos_lang_rust_deny`,
//! `nomos_repo_policy`'s own `Compute_Fact_Key` for all seven repository-declared policies,
//! and `nomos_cap_requirement_trace`), each stating the same reason: the provider's real
//! input is a manifest tree, a subprocess's own analysis or a policy file's current text,
//! which no caller holds independently, so a caller building a lookup key has nothing to
//! reconstruct it from. Reconstructing one here anyway -- reading `standards.json` a second
//! time, or enumerating what `cargo metadata` reads -- would put a second copy of each
//! provider's input set in this crate, which is the shape `P102` removed from
//! `Demanded_Families` after the two copies had already drifted. A copy that missed one
//! input (`Cargo.lock`, a member manifest, a transitively read file) would skip a write whose
//! inputs had genuinely moved, silently, and serve a stale fact.
//!
//! The second is that a side table is a second record of what the store already knows, and
//! it can go stale against the store while looking authoritative. Comparing the candidate
//! against the fact `store` is actually serving cannot: if the inputs moved, the provider's
//! answer differs, the digest differs, and the write happens. `OD-ANALYSIS-005`'s
//! recomputation-equivalence property is what this has to preserve, and it is preserved by
//! construction rather than by a bookkeeping invariant -- there is no second record to keep
//! true.
//!
//! What this therefore does *not* buy, and no doc here should be read as claiming: the
//! provider still runs. A `cargo clippy` launch, a `standards.json` read and a reachability
//! parse all still happen on every call their family is demanded on. Skipping those needs
//! each provider to publish, from its own inputs and before it works, the digest it files
//! its fact under -- a change in the provider crates, not in this one.

use nomos_analysis::{Context, FactStore, InputDigest, MaterializedFact, MemoryFactStore};

/// Files `fact` into `store` unless the store is already serving an identical one, and
/// reports whether `store` holds a current fact under `fact`'s own identity once this
/// returns.
///
/// `true` either way -- because this call wrote the fact, or because
/// [`Is_Already_Current`] found the store already holding it -- and `false` only when the
/// write was attempted and refused. That is deliberately the coverage meaning
/// `super::dependency_materialization::Materialize_Syntax`'s own per-source step already
/// answers with, and not "this call did work": `P40-INCREMENTAL-SKIP-COVERAGE-REGRESSION`
/// is the regression that came of briefly making a materializer's return value mean the
/// newly-written count with nothing downstream updated to match.
/// [`nomos_analysis::MemoryFactStore::Materializations`]'s own delta across two calls is the
/// honest answer to how much work a call did.
pub(super) fn Materialized_Or_Already_Current(fact: MaterializedFact, context: &Context, store: &mut MemoryFactStore) -> bool
{
    if Is_Already_Current(&fact, context, store)
    {
        return true;
    }

    // No dependency edges: every fact this crate materializes is a leaf, read from one
    // provider's own inputs and from nothing this store holds -- the identical reasoning
    // `Materialize_Syntax`'s own write states for its own fact.
    return store.Materialize(fact, &[]).is_ok();
}

/// Whether `store` is already serving, at `context.generation`, a fact indistinguishable
/// from `fact` to every reader of it.
///
/// Two facts are indistinguishable when [`Input_Digest_Of`] agrees on them, which is the
/// identity `store` files by together with everything a `nomos_analysis::Reader` can read
/// back out. A rewrite of one would leave every later read answering exactly what it
/// answers now, so the only thing it could change is
/// [`nomos_analysis::MemoryFactStore::Materializations`] -- and the run's own
/// `crate::run_context::capabilities`' `Materialization_Tracking` reads that counter to
/// decide whether the family moved, so a redundant write is not merely wasted but actively
/// misreported.
///
/// `MaterializedFact::snapshot` is deliberately not part of the comparison, and that field's
/// own documentation is why: it records which tree the observation was made against, a fact
/// that outliving several workspace states keeps naming the first one, and "re-stamping it
/// on reuse would be a claim that the measurement was repeated."
pub(super) fn Is_Already_Current(fact: &MaterializedFact, context: &Context, store: &MemoryFactStore) -> bool
{
    let identity = fact.Key().clone().At(context.generation);
    let Some(current) = store.Current(&identity, context.generation)
    else
    {
        return false;
    };

    return Input_Digest_Of(&current) == Input_Digest_Of(fact);
}

/// Everything about `fact` a later read of it can observe, as one digest: the identity it
/// files under, the schema and bytes of what it says, and the evidence class it says it
/// under.
///
/// A digest of the provider's own answer rather than of a second enumeration of that
/// provider's inputs, for the reason this module's own doc gives: the answer is a total
/// function of exactly the inputs the provider read, and this crate cannot enumerate those
/// inputs without keeping a second, unverifiable copy of each provider's reading.
///
/// The key's own digest already carries the guarantee (as a `nomos_analysis::
/// GuaranteeDigest`), the subject, the contract and provider identities and versions, the
/// build variant and the configuration, so none of those is restated here.
fn Input_Digest_Of(fact: &MaterializedFact) -> InputDigest
{
    return InputDigest::Of(&[
        fact.Key().Digest().Bytes(),
        fact.payload.schema.As_Str().as_bytes(),
        &fact.payload.bytes,
        fact.evidence.Label().as_bytes(),
    ]);
}

#[cfg(test)]
#[path = "currency/tests.rs"] mod tests;

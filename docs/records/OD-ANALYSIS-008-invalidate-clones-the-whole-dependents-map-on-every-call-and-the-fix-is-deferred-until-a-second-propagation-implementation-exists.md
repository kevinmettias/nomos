---
id: OD-ANALYSIS-008
type: decision
title: Invalidate clones the whole dependents map on every call, and the fix is deferred until a second propagation implementation exists
status: closed
version: 3
authority: canonical-normative-record
tags:
  - analysis
  - invalidation
  - performance
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: D-130
    type: relates-to
  - target: D-135
    type: relates-to
  - target: D-138
    type: relates-to
  - target: OD-ANALYSIS-002
    type: relates-to
---

# Invalidate clones the whole dependents map on every call, and the fix is deferred until a second propagation implementation exists

**Closed at version 3. The title and every section from "Question" to "What Would Decide
It" are the measurement as it was taken, in the present tense it was taken in, and no
longer describe this workspace: the clone they are about does not exist. They are left
standing rather than rewritten so the closure can be checked against what it closed —
`ARC-ROADMAP-001`'s own amendment discipline. "Disposition" is what is true now, and the
two citations a reader would follow out of those sections are corrected in place, because a
pointer at a file that is not there is not a dated measurement of anything.**

## Question

`ARC-ROADMAP-001` records that two independent architecture reviews measured eight
performance claims against `crates/substrate/nomos-analysis`, and that six were confirmed as
real hazards deliberately left for later work. None of the six is named in a record of its
own; the count exists only in that aggregate. This record names one of them, traced to
ground during ordinary work on this crate, so it is an accepted cost with a stated trigger
rather than a number nobody can check.

## What Was Found

`MemoryFactStore::Invalidate` clones `self.dependents` — the full reverse-dependency index,
one entry per fact this store has ever recorded a dependency edge against — on every call,
before the walk that actually uses it:

```rust
let dependents = self.dependents.clone();
let propagation = self
    .propagation
    .take()
    .expect("propagation implementation is always present between calls");
propagation.Spread(&dependents, roots, &mut |consumer| {
    seen.insert(consumer);
    if !self.Invalidate_One(consumer, from, &described)
    {
        return false;
    }
    ...
});
```

That code was in `crates/substrate/nomos-analysis/src/fact/memory_store.rs`, which this
record cited and which no longer exists. `MemoryFactStore` itself is now
`crates/substrate/nomos-analysis/src/fact/memory_fact_store.rs`, and the walk quoted above
is `crates/substrate/nomos-analysis/src/fact/memory_store/invalidation.rs`. The clone is in
neither; see "Disposition".

The clone's cost is `O(the whole store's dependents map)` — every fact with a recorded
dependent, not just the ones a given invalidation's frontier actually reaches. A store
carrying a large corpus pays this on every edit, independent of how small that edit's own
reach is, which is the opposite of what an incremental store exists to guarantee.
`OD-ANALYSIS-001` measured a comparable shape (a fact-key component recomputing 9,440 facts
per keystroke over a six-file corpus) before it was fixed; this clone has not been measured
at that scale, which is exactly why this record states a trigger rather than a number.

## Why The Clone Is There

Not an oversight. The comment immediately above it explains the constraint:
`DependencyPropagation::Spread` takes its adjacency map by immutable borrow, while its own
`on_reach` callback needs `&mut self` for `Invalidate_One` and `self.keys`. Rust's borrow
checker refuses a live immutable borrow of `self.dependents` coexisting with a closure that
mutates `self`, so the two ways to satisfy both sides are: clone `dependents` out of `self`
before the walk (what this store does), or restructure `Invalidate` into two passes —
collect the reachable digest set under an immutable borrow, release it, then apply every
invalidation afterward without a live borrow across the callback.

The borrow this section is about is still the borrow that is there. The spelling it quoted
for it is not. Where this section read `dependents: &BTreeMap<Digest128,
BTreeSet<Digest128>>`, `crates/substrate/nomos-analysis/src/propagation.rs` now declares
`trait DependencyPropagation<Node> where Node: Copy + Ord` with
`dependents: &BTreeMap<Node, BTreeSet<Node>>` — a signature naming no identity type at all,
per `D-135` and `D-138`, with the store addressing a node by a `FactSlot` of its own. What
forced the clone was the shape of the borrow against a callback needing `&mut self`, and
that is unchanged by the type parameter; the type inside it was never the reason.

## Why It Is Deferred Rather Than Fixed Here

`DependencyPropagation` (`crates/substrate/nomos-analysis/src/propagation.rs`) is not
ordinary internal plumbing. Its module doc names it as the mechanism `D-130` marks an XVPE
candidate, deliberately kept free of fact-identity vocabulary per `D-135` and `D-138` so a
later move is a rename rather than a rewrite. `LocalGraphPropagation` is its only
implementation today — a population of one, the same shape `D-122` and `D-130` already name
as the reason no second implementation exists yet, and the same shape this workspace has
repeatedly declined to design ahead of elsewhere (`OD-PACKAGE-006`, `OD-PACKAGE-008`,
`OD-RULES-007`). Two fixes are available and neither is free of that risk:

- **Change `Spread`'s own signature** so an implementation is not handed a borrow that
  conflicts with `Invalidate`'s need for `&mut self` — for example, taking ownership of
  `dependents`, or returning a reachable set rather than calling back into the store at all.
  This is exactly the seam `D-130` cares about matching against a second real party before
  it is redesigned; changing it now would be shaping the interface from the one
  implementation that exists, not from evidence a second one disagrees with it.
- **Restructure `Invalidate` into two passes without touching `DependencyPropagation` at
  all** — collect the reachable digests under the existing borrow, drop it, then mutate.
  This does not touch the seam and is a real candidate for a future item, but it is its own
  scoped correction with its own risk (the callback's early-decline semantics, tested in
  `propagation.rs`'s own suite, would need to survive being split across two passes) and is
  not something this record's job is to design inline.

## What Would Decide It

Either of:

- **A second real `DependencyPropagation` implementation.** Its own shape would show
  whether `Spread`'s signature is what forces the clone, or whether the clone is an
  accident of how `LocalGraphPropagation` alone was integrated — the same evidence bar
  `D-130`'s own adoption trigger already names.
- **Measured evidence the cost is live in a real run** — a corpus and an edit-invalidate
  workload large enough that this clone shows up the way `OD-ANALYSIS-001`'s snapshot
  component did, rather than argued from complexity alone.

Until either arrives, the clone stands as a named, accepted cost rather than a silent one.

## The Seam-Free Fix Is Built; The Seam-Facing Question Stays Open

**Version 2's account, kept as written. Its second half — that the seam-facing question
stays open *here* — is superseded by "Disposition" below, which sends that question back to
`D-130` rather than closing it.**

`P13-ANALYSIS-008-TWO-PASS-INVALIDATE` built the second of this record's two "Why It Is
Deferred" options — restructure `Invalidate` into two passes without touching
`DependencyPropagation` at all — under the user's roadmap override for incremental-analysis
performance hardening, though this fix does not actually need that override: it changes no
public seam, only `MemoryFactStore`'s own internal walk, so it was never blocked by the
`DependencyPropagation`-seam risk this record's "Why It Is Deferred Rather Than Fixed Here"
section named for the *other* option.

Verified directly against the real code, not assumed:
`crates/substrate/nomos-analysis/src/fact/memory_store/invalidation.rs`'s
`Propagate_To_Dependents` no longer clones `self.dependents`. It reads `store.dependents`
by real immutable borrow in a first pass that decides, through the new
`MemoryFactStore::Is_Already_Invalidated` — the read-only half of what
`Try_Invalidate_One` already checked before mutating — which nodes to
keep spreading past and to collect; a second pass, after that borrow ends, calls
`Try_Invalidate_One` and writes the report for each collected node. Every existing
invalidation test passed unmodified, including a 100,000-deep chain stress test and the
test that substitutes an alternate `DependencyPropagation` implementation and checks the
report agrees — the observable behavior this record's own "must survive being split across
two passes" risk for the early-decline semantics named as the cost of this exact option.
`propagation.rs`'s `DependencyPropagation` trait and `LocalGraphPropagation` are byte-for-
byte unchanged.

The other option — changing `Spread`'s own signature — is untouched by this, and this
record's primary question (a second real `DependencyPropagation` implementation, or measured
evidence from a real workload) is unaffected: nothing about the clone's *cost model* changed
who may design that seam next, only that the clone this record measured no longer exists to
be a cost at all.

## Disposition

**Closed. The deferral ended because its subject was removed by other work, not because
either of the conditions "What Would Decide It" names ever arrived.** Those are different
reasons, and the difference is the whole of what a later reader cannot re-derive from the
code: a trigger that fired would mean this record's question had been answered, and it has
not been. It was made moot.

Each half of that was read off the tree rather than inherited from the item that asked for
this closure.

**The subject is gone.** `Invalidate` does not clone `self.dependents`, and the reason is
the second of the two fixes "Why It Is Deferred Rather Than Fixed Here" lists — the one
that does not touch the seam. `memory_store/invalidation.rs` walks in two passes:
`Walked_Dependents` takes `store: &MemoryFactStore`, hands `&store.dependents` straight to
`Spread`, and returns an owned `Walk`; that shared borrow ends with the value it returns,
and the second pass's mutable use of `store` begins only afterwards. Nothing is copied out
of the store to make the two borrows fit. `P13-ANALYSIS-008-TWO-PASS-INVALIDATE`
built it, and the section above is that item's own account of it, corrected here for three
renames it could not have known about: the walk is `Propagate_To_Dependents` rather than
`Propagate`, and its two store calls are `Is_Already_Invalidated` and `Try_Invalidate_One`.

**Neither trigger fired.** There is still exactly one real `DependencyPropagation`
implementation, `LocalGraphPropagation` in `propagation.rs`. The only other implementation
in the workspace is `QueueOrderPropagation` in
`crates/substrate/nomos-analysis/src/fact/memory_store/tests.rs`, which is inside that
crate's `#[cfg(test)]` module and exists to prove the seam is substitutable — it is the
evidence `D-130` asks a population of one to produce, not a second real party with a shape
of its own to disagree with. And the second condition, measured evidence of the cost in a
real run, was never taken and now cannot be: the code it would have measured is not there.

**So closing this decides nothing about `Spread`'s signature.** The seam-facing question —
whether an implementation should be handed the map by borrow at all — was always separable
from the clone, and the clone was the only reason it was asked *here*. It goes back to
`D-130`, which holds the adoption trigger for this mechanism and has held it throughout.
This record adds nothing to that trigger and no longer holds a second copy of it open. A
record that stays open on a cost nobody pays teaches its next reader that the cost is
there, which is the opposite of what it was written for.

## What This Closure Does To `ARC-ROADMAP-001`'s Count, Measured

`ARC-ROADMAP-001` is where this record's "Question" got its six from, and it carries two
sentences containing that number. They are about different sets, and only one of them is
about hazards at all.

The one this record is carved out of is in "What Was Measured": eight performance claims
against `crates/substrate/nomos-analysis`, "six confirmed as real hazards deliberately left
for later work", one — the recursive `Tarjan::Visit` with no depth guard — fixed by
`P13-TARJAN-ITERATIVE` and named there as fixed. The other, in the same section's corpus
paragraph, says "the assessed set stays at the six entries those records already describe",
and those entries are the per-requirement `Met`/`Diverges`/`NotBinding` assessments
`OD-TRACE-001` and `OD-TRACE-002` govern — the same six that record's own constraint 1
spells out as "Six requirements are formally assessed today (`OD-TRACE-002`); 357 remain
`Unassessed`". Nothing here writes an `.assessment` file, so that sentence is untouched by
this closure. It shares a number with the hazards and nothing else.

**The hazard sentence is not falsified either, and that was measured rather than assumed.**
It is a dated reading in a section titled "What Was Measured", reporting what two reviews
claimed and what was checked against the tree at the time, down to naming the one claim
already fixed when it was written. `ARC-ROADMAP-001` version 4 states the discipline that
settles it, in its own words: a row is taken at a named revision, and "a measurement that
quietly absorbs later work is not a measurement". A measurement does not go false when a
later commit pays one of the costs it found. So that record needs no correction for this
closure and this one edits none of it.

**What is real, and is not being dropped by saying so: the other five cannot be checked
against HEAD by anybody, because nothing in this repository names them.** That was
searched, not assumed — every file under `docs/records/`, the rendered
`spec/domain-specification.md`, and `work/ledger.json`. The count occurs twice: in
`ARC-ROADMAP-001`'s own sentence, and in `OD-ROADMAP-004` quoting it. Neither enumerates a
member. This record's "Question" said so when it was written — "None of the six is named in
a record of its own; the count exists only in that aggregate" — and it is still true, so
closing this one leaves five hazards that are counted, asserted to be real, and unauditable
by construction. That is a worse state than the one this record was written to escape, and
it is boarded rather than resolved here, as
`P131-ARC-ROADMAP-001-COUNTS-SIX-ANALYSIS-HAZARDS-AND-NAMES-NONE-OF-THE-FIVE-THAT-ARE-LEFT`,
because recovering the five needs the review text rather than a read of this repository.

Four recent fixes look like they close some of the five, and none of them can be shown to.
`27050a92` and `bb79f7a5` reply
to a *later* external review — four bootstrap choices in the analysis substrate, not the
eight claims counted above — and nothing ties the two sets. Three of that review's four are
answered at HEAD: a key is interned once and facts are addressed by a `FactSlot` rather
than rehashing `FactKey::Digest` per read (`fact/memory_store/identities.rs`);
`FactStore::Current_Borrowed` is a lending read beside the owning `Current`; and the
per-key history has a stated retention rule of one entry, written at `memory_fact_store.rs`
where the history grows. The fourth, the quadratic ranking, is `Ranked_Offers` in
`crates/substrate/nomos-capability/src/resolution/selection.rs` — a different crate, so
whatever else it was, it was not one of eight claims counted against `nomos-analysis`.
Whether any of the other three is also a member of the unnamed five cannot be determined
from anything in this repository, and is not claimed here.

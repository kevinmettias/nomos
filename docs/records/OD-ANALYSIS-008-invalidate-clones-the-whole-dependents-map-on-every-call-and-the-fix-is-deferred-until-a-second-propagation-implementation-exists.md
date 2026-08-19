---
id: OD-ANALYSIS-008
type: decision
title: Invalidate clones the whole dependents map on every call, and the fix is deferred until a second propagation implementation exists
status: open
version: 1
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

## Question

`ARC-ROADMAP-001` records that two independent architecture reviews measured eight
performance claims against `crates/substrate/nomos-analysis`, and that six were confirmed as
real hazards deliberately left for later work. None of the six is named in a record of its
own; the count exists only in that aggregate. This record names one of them, traced to
ground during ordinary work on this crate, so it is an accepted cost with a stated trigger
rather than a number nobody can check.

## What Was Found

`MemoryFactStore::Invalidate`
(`crates/substrate/nomos-analysis/src/fact/memory_store.rs`) clones `self.dependents` — the
full reverse-dependency index, one entry per fact this store has ever recorded a dependency
edge against — on every call, before the walk that actually uses it:

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

The clone's cost is `O(the whole store's dependents map)` — every fact with a recorded
dependent, not just the ones a given invalidation's frontier actually reaches. A store
carrying a large corpus pays this on every edit, independent of how small that edit's own
reach is, which is the opposite of what an incremental store exists to guarantee.
`OD-ANALYSIS-001` measured a comparable shape (a fact-key component recomputing 9,440 facts
per keystroke over a six-file corpus) before it was fixed; this clone has not been measured
at that scale, which is exactly why this record states a trigger rather than a number.

## Why The Clone Is There

Not an oversight. The comment immediately above it explains the constraint:
`DependencyPropagation::Spread` takes `dependents: &BTreeMap<Digest128,
BTreeSet<Digest128>>` — an immutable borrow — while its own `on_reach` callback needs `&mut
self` for `Invalidate_One` and `self.keys`. Rust's borrow checker refuses a live immutable
borrow of `self.dependents` coexisting with a closure that mutates `self`, so the two ways
to satisfy both sides are: clone `dependents` out of `self` before the walk (what this store
does), or restructure `Invalidate` into two passes — collect the reachable digest set under
an immutable borrow, release it, then apply every invalidation afterward without a live
borrow across the callback.

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

## Status

Open. Revisit when a second `DependencyPropagation` implementation exists to check the
seam's shape against, or when a real workload measures this clone's cost rather than its
complexity.

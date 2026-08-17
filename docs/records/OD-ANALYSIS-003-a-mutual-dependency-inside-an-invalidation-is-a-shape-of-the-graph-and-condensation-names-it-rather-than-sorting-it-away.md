---
id: OD-ANALYSIS-003
type: decision
title: A mutual dependency inside an invalidation is a shape of the graph, and condensation names it rather than sorting it away
status: closed
version: 1
authority: canonical-normative-record
tags:
  - analysis
  - invalidation
  - determinism
relations:
  - target: OD-ANALYSIS-002
    type: relates-to
  - target: OD-GATE-002
    type: relates-to
---

# A mutual dependency inside an invalidation is a shape of the graph, and condensation names it rather than sorting it away

## Question

`MemoryFactStore::Invalidate` walks dependency edges transitively and gets the hard parts
right: a seen set guards re-entry, `Invalidate_One` is idempotent, and the traversal is
deterministic because `dependents` is a `BTreeMap` of `BTreeSet`. `Settle` even sorts
`InvalidationReport::dependent` before returning it, so the field a caller reads back is
reproducible.

Reproducible is not the same claim as ordered. `dependent` is sorted by `FactKey`'s own
`Ord` — lexicographically over contract, subject and the rest — which has nothing to do with
which fact reads which. A caller holding that vector cannot rematerialize from it: for A
depending on B depending on C, alphabetical order says nothing about which of the three has
to be recomputed first, and a fact reached through a cycle is written down exactly like one
reached through an acyclic edge — both are a `FactKey` pushed into one flat list. So a caller
cannot tell "some valid order exists" from "no order can", and the second case is not an
error. Mutually recursive subjects are the ordinary shape once facts are computed from other
facts, and `nomos-analysis` ships a store built for exactly that future.

What does a caller need in hand to rematerialize what one invalidation reached, and to know
when no total order over it exists?

## What Was Actually Wrong

Nothing in the walk. `direct` and `dependent` answer *what became stale*, correctly and
deterministically, and `OD-ANALYSIS-002` is why a real edge exists at all to walk. The gap is
that dependency *structure* — how the stale facts depend on each other — was never computed
and never exposed. `MemoryFactStore` already keeps it: `Dependencies_Of` reads back exactly
the edges `Materialize` recorded, one call per key, and always has. The store was withholding
an answer it already had the data to give.

Nothing has noticed because the workspace has one derived fact
(`nomos.cap.module.index`, `OD-ANALYSIS-002`), and its dependency graph is one edge deep. One
edge cannot form a cycle and has no order to get wrong.

## The Answer

`Condensation_Of(report, store)`, a free function taking the report `Invalidate` returned
and the store it came from, returning `Vec<RematerializationGroup>`: an ordered list of
groups, each either one fact or a set of facts that depend on each other and admit no order
among themselves. `RematerializationGroup::Is_Cycle` says which case a group is without a
caller having to count.

**It is a free function, not a method on `InvalidationReport`, and that is a decision about
the report's job, not a convenience.** `direct` and `dependent` are the report's own record
of what became stale, filled in while the walk runs. The edges between those facts are not
the walk's to own — they are `store`'s, already public through `Dependencies_Of`, and a
second copy of them living on the report would be a graph the store already keeps,
duplicated the moment anyone asks for structure. `Condensation_Of` reads `report.direct` and
`report.dependent` to know which keys matter and asks `store` what each one depends on among
them; nothing is computed that either value did not already expose on its own.

That the crate's own public-surface accounting agrees is confirmation, not the reason: this
workspace's `tests/contract` surface scanner resolves a `pub use`'d type's associated items
from the module the `pub use` names, and `InvalidationReport` is re-exported from
`invalidation`, not from `store`. A method written under `impl InvalidationReport` anywhere
outside `invalidation/report.rs` compiles, passes every test, and is invisible to
`tests/contract/surface/nomos-analysis.txt` — the exact failure mode this item exists to
close, reproduced reflexively by the tool meant to catch it. `Condensation_Of` is declared
and used as a plain function, so it is a name the scanner resolves the same way it already
resolves `GenerationCause` and `RematerializationGroup`: directly, by its own name, off the
`pub use store::{…}` list in `crates/substrate/nomos-analysis/src/lib.rs`.

**The algorithm is Tarjan's strongly-connected-components search, run over the subgraph
induced by the invalidated keys.** An edge `u → v` means `u` depends on `v`; a dependency
that leads outside the invalidated set is not an edge here, because that fact was not
invalidated and is read as-is rather than rematerialized. Tarjan's completion order is a
property of the algorithm, not of which node is visited first: a component only finishes
once every component reachable from it has finished, so appending components to the result
in finishing order is always a valid order over the condensation — the caller-visible
guarantee does not depend on `report.dependent`'s traversal order, and the implementation
never consults it.

**Broadening and the condensation are computed from disjoint inputs.** `Note_Broadening`
answers how far a cause was widened, keyed on `IncrementalGranularity`; `Condensation_Of`
answers how the invalidated facts relate to each other, keyed on dependency edges. Neither
function's body names the other's field.

## Why The Store Reports Structure Rather Than Ordering Rematerialization

`FactStore`'s trait is sealed to three methods: `Current`, `Historical`, `Invalidate`.
Rematerialization — actually recomputing a stale fact — is deliberately outside it, and this
item does not bring it in. `Condensation_Of` answers a structural question the store is
positioned to answer honestly (it holds the edges), and stops there. It does not decide *how*
a mutually recursive group gets recomputed, because the store has no opinion to have: nothing
in this crate rematerializes anything yet.

What the future rematerialization orchestrator owes, once it exists: rematerializing
according to `Condensation_Of`'s groups — recomputing each group in the order returned, and
resolving whatever a cycle's own group needs by whatever means it uses — must produce the
same current facts as a clean recomputation from nothing. That predicate belongs to the
orchestrator and is named here rather than asserted here, because there is no orchestrator to
assert it against yet. `Condensation_Of` only has to be honest about the graph; it is not the
place that predicate gets proven.

## What Was Considered And Rejected

**Sorting `dependent` into topological order in place.** It would have silently changed a
field a caller already reads today, and it still could not represent a cycle — a topological
sort does not exist over a graph that has one, so the function would have had to choose an
arbitrary order among mutually dependent facts and hand it out looking exactly as valid as an
acyclic one.

**Returning an error when a cycle is found.** A cycle is a shape of the dependency graph, not
a fault the store can refuse. Mutually recursive facts are the expected result of computing
facts from facts, and an invalidation that reaches one has not done anything wrong.

**Computing groups eagerly inside `Invalidate` and adding a field to `InvalidationReport`
for them.** This is the design the report's shipped shape suggests, and it is not available
inside `MemoryFactStore::Invalidate` and `InvalidationReport` without editing
`crates/substrate/nomos-analysis/src/fact/memory_store.rs` and
`crates/substrate/nomos-analysis/src/invalidation/report.rs` directly, neither of which this
item's territory reaches. Independent of territory, it also pays the condensation's cost on
every invalidation whether or not a caller ever asks for it, where `Condensation_Of` is paid
for only on the call.

## What Holds It

`crates/substrate/nomos-analysis/tests/invalidation_order.rs`, three graphs driven against a
real `MemoryFactStore`:

- **a four-fact chain** condenses into four singleton groups in dependency order — the
  fact with no dependencies of its own first, the fact that reads it last — asserted without
  reading `report.dependent`'s incidental traversal order;
- **a four-fact cycle** condenses into exactly one group naming all four, with
  `Is_Cycle` true and no order asserted among its members;
- **a chain that enters and leaves a two-fact cycle** condenses into three groups in one
  call — the leaf before the cycle, the cycle as one group, the fact that reads the cycle
  after it — so an acyclic ordering and a mutual-dependency group are shown to coexist in one
  result rather than being two designs that were only ever tested apart.

## Status

Closed by P11-INVALIDATION-ORDER.

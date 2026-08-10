---
id: OD-ANALYSIS-002
type: decision
title: The dependent half of invalidation had no producer because every shipping capability is a leaf, and a second capability is what gives it one
status: closed
version: 1
authority: canonical-normative-record
tags:
  - analysis
  - capability
  - invalidation
relations:
  - target: OD-ANALYSIS-001
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-CAPABILITY-003
    type: relates-to
---

# The dependent half of invalidation had no producer because every shipping capability is a leaf, and a second capability is what gives it one

## Question

`nomos-analysis` ships a complete transitive invalidation mechanism. `MemoryFactStore` keeps
a reverse index of dependents, `Materialize` takes a `&[Dependency]`, `Dependencies_Of` reads
them back, `Reader` records every read and hands them over as `Into_Dependencies`, and
`InvalidationReport` separates `direct` from `dependent` and prints "N through dependency
edges".

Nothing that ships created an edge. Every producer in `crates/` passed an empty slice, and
the only non-empty dependency arrays in the tree were in `tests/` — one in
`nomos-analysis`'s own fixture, which invents a fact to exercise its own index, and one in
`tests/integration`'s slice, which is the instrument rather than the product. So
`InvalidationReport::dependent` was a field no shipping run could make non-empty.

Where does the first shipping edge come from?

## What Was Actually Wrong

**It was structural, not an omission.** Every fact is filed under a capability, and the one
capability this build shipped — `nomos-cap-syntax`'s `nomos.cap.syntax.items` — is a
per-file leaf *by construction*. `nomos-lang-rust`'s provider sets `semantic_inputs` to the
file text alone, declares `IncrementalGranularity::File`, and stamps
`EvidenceClass::Verified` with a comment refusing `Derived` because "the source is not a
fact, it is the territory". All three are correct. A capability whose semantic input is one
file's bytes can never depend on another answer, so no amount of care inside that capability
would have produced an edge.

**The capability could not be reused for a rollup either.** Its schema is fixed at
`nomos.syntax.items.v2`, and `PayloadItem` carries a name qualified by syntactic nesting
*within one file* with no field for which file that was. A union of syntax payloads across
subjects therefore loses the only thing that would make a module-level answer worth having:
two files each declaring `Read` produce two indistinguishable records.

**The missing consumer was already named.** `nomos-analysis`'s reader has carried a comment
since it was written — a read that found nothing is still a real dependency, because "a
rollup that later has to be invalidated when the parser does have something needs the edge".
The rollup it describes had never been written.

## The Answer

A second capability, `nomos.cap.module.index`, answering *which items a module declares and
which of its files declared each one*, produced by `nomos-lang-rust`'s `rollup` module from
the syntax facts of its members.

**The subject is the module, and it must not be any member's.** This is the load-bearing
part and it is easy to get wrong. A rollup keyed on one of its own inputs is named by the
same `GenerationCause` that names that input, so a change to a member would report the
rollup under `direct` — and a test asserting "the change reached the rollup" would pass
while measuring the path that was never in doubt. The module gets a subject of its own, and
the assertion is that the rollup arrives under `dependent` and *not* under `direct`.

**The edges are observed, never listed.** `Materialize_Index` reads through
`nomos_analysis::Reader` and hands the store `Into_Dependencies()`. It never assembles a
dependency array. A hand-written edge list is a claim about what was read; this is a record
of it, and the two diverge the first time a read is added and the list is not. Reads that
found nothing are edges too, which is the case the reader's comment named.

**The guarantee is no stronger than its inputs, on every axis.** `FactVariant::Syntactic`
and `Assurance::Sound` carry over; completeness stays `Assurance::Unknown` because the
syntax facts may have missed macro-generated items, so the rollup has missed them too; and
`EvidenceClass::Derived` says the same thing about provenance. `IncrementalGranularity` is
`Project`, because a rollup over a module cannot refresh half a module — declaring `File`
would let the engine refresh one member's contribution and treat the rest as current. The
engine broadens a file-granular cause and records having done so on
`InvalidationReport::broadened`, which is the cost of the rollup stated rather than
absorbed.

**The semantic inputs are every member's subject and every member's input digest.** Both
halves are load-bearing. Without the inputs, editing a member leaves the rollup addressable
at its old key and a stale answer reads as current. Without the subjects, a module that
swapped one file for another holding identical bytes would key the same — and a module is
which files it has, not only what they contain. Members are canonically ordered and
deduplicated by subject before anything is read, so the same module described two ways is
one fact.

## Why The Contract Lives Beside Its Provider

`OD-CAPABILITY-002` sets the criterion and it is **contention, not principle**: a capability
with a single provider is not wrongly filed for living beside that provider, and moving it
out buys a crate and no property. `nomos-cap-syntax` exists because two crates offer against
it and, sitting at one band, neither may name the other. Nothing offers against
`nomos.cap.module.index` but the function that answers it.

So this record deliberately does *not* create a crate under `crates/capabilities`. The day a
second provider exists — one reading a module from a compiler's own item table rather than
from syntax facts — the contract moves, and
`Test_A_Capability_Id_Should_Be_Written_In_One_Crate` will say so the moment the id is
spelled in two crates' `src`. That guard is what makes this a deferral with an alarm on it
rather than a decision to revisit by memory.

## What Was Considered And Rejected

**Widening `nomos.cap.syntax.items` to accept several files.** It is the change that looks
smallest and it is the one that destroys the capability. `IncrementalGranularity::File` and
a `semantic_inputs` of one file's text are what make that contract honest; a multi-file
answer under the same contract would have to claim one of them falsely. The schema could not
carry the answer either, per the payload objection above.

**Promoting the slice's `nomos.cap.module.surface`.** `tests/integration` already computes a
directory rollup with real edges, and it is where the mechanism was proven. It is not the
product: that crate is the instrument that measures the workspace, its facts are five counts
rather than an index, and `Fact_Domains` excludes it from the determinism universe for
exactly the reason that it is not an execution domain. Moving it would also have made the
slice assert against itself.

**A derived fact over a single file, keyed on that file.** Cheaper, and it proves nothing:
the derived fact and its input share a subject, so `SubjectChanged` names both and the
derived one is reported `direct`. The test would have been green over the direct path.

**A second `impl Strategy` for the rollup's determinism.** `tests/contract`'s determinism
guard requires every declaration to be held to it by the integration harness, and
registering one there is outside this item's territory. A declaration with nothing behind it
is worse than none — that is what the guard exists to catch — so the rollup runs under
`nomos-lang-rust`'s existing `SyntaxFactProduction` declaration, whose triple it genuinely
shares, and a separate item carries the split.

## What Holds It

`crates/languages/nomos-lang-rust/tests/rollup.rs`, ten assertions, of which four are the
ones that could have been faked:

- **the rollup arrives under `dependent` and not under `direct`** — the property the whole
  item is about, stated in both directions so a rollup keyed on its own input fails it;
- **a change to a file that is neither a member nor read invalidates nothing** — the
  negative control, without which a store that invalidated everything would satisfy the
  first;
- **a member with no fact is still an edge**, with an outcome that is not `Materialized` —
  the reader's own comment, exercised;
- **`Dependencies_Of` returns what the reader recorded**, one edge per member.

Plus the payload's negative controls in `rollup.rs`: the empty byte string, a missing header,
a repeated header, an unknown record tag, an unknown outcome and a short item record are all
refused. A decoder that fell back to an empty index would report every unreadable rollup as
a module that declares nothing, which is indistinguishable from a module that genuinely
does.

## Status

Closed by P10-DEPENDENT-EDGE. `P10-DERIVED-FACT` states the same defect and was released
because its territory could not reach the fix; the measurement it recorded is unchanged.

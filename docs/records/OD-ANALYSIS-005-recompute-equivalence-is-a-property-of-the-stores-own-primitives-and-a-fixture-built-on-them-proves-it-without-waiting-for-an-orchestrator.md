---
id: OD-ANALYSIS-005
type: decision
title: Recompute equivalence is a property of the store's own primitives, and a fixture built directly on them proves it without waiting for an orchestrator
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
  - target: OD-ANALYSIS-003
    type: relates-to
---

# Recompute equivalence is a property of the store's own primitives, and a fixture built directly on them proves it without waiting for an orchestrator

## Question

`OD-ANALYSIS-003` names the predicate that matters once anything is actually rematerialized:
recomputing according to `Condensation_Of`'s groups must produce the same current facts as a
clean recomputation from nothing. It names the predicate and stops, deliberately —
"there is no orchestrator to assert it against yet." `Condensation_Of` answers a structural
question and does not decide how a stale fact gets recomputed; nothing in `nomos-analysis`
rematerializes anything.

An under-invalidating dependency graph is the failure that predicate exists to catch, and it
is silent under exactly the conditions a test suite runs: a stale fact reads as a correct
answer computed earlier, not as an error. It surfaces only once a dependency edge exists and
the recomputation order lets a stale read win — which is why writing the equivalence now,
while `nomos.cap.module.index` (`OD-ANALYSIS-002`) is still the workspace's only derived
fact, is cheap, and why leaving it unwritten until a second hop arrives is not.

Nothing in the shipped tree can call a rematerialization orchestrator, because none exists.
Can the property still be asserted, or does it have to wait for one?

## What Was Actually Wrong

Nothing was wrong; nothing existed. `crates/substrate/nomos-analysis/tests/
invalidation_order.rs` (`OD-ANALYSIS-003`) proves a caller can recover a valid
rematerialization order from an `InvalidationReport`. No test anywhere follows that order and
checks the answer it produces against a clean build. The gap `OD-ANALYSIS-003` named by
deferring to a future orchestrator was still open.

The orchestrator `OD-ANALYSIS-003` deferred to is a shipped-product concern — something that
would sit above `nomos-analysis` and decide how a real capability recomputes a real fact. It
does not exist, and this record does not create it. What it establishes is narrower and does
not need the orchestrator to exist first: `MemoryFactStore`'s own primitives —
`Materialize`, `Invalidate`, `Current` — are sufficient by themselves to drive both an
incremental recomputation and a clean one, and to compare their results. The property under
test belongs to those primitives' interaction, not to any orchestrator built on top of them;
an orchestrator that gets this wrong would be composing correct primitives incorrectly, and
that is a different defect from the primitives disagreeing with themselves.

## The Answer

`crates/substrate/nomos-analysis/tests/recomputation_equivalence.rs`, a two-fact graph with
one real derived hop, driven against a real `MemoryFactStore`:

- an **upstream leaf fact**, addressed at its own subject, carrying arbitrary bytes;
- a **downstream fact at a subject of its own** — not the upstream's, for the same reason
  `OD-ANALYSIS-002`'s rollup is keyed on its module rather than a member: a downstream fact
  keyed on its own input would be named `direct` by `SubjectChanged` rather than `dependent`,
  and a test built on it would measure a path that was never in doubt — whose payload is a
  real function of the upstream's payload (the bytes reversed), materialized with a
  dependency edge recorded through `Materialize`'s own `&[Dependency]` parameter, the same
  edge `Dependencies_Of` and `Follow_Edges` already use in shipped code.

One test builds this graph twice: once incrementally — materialize at `GenerationId::
INITIAL`, invalidate the upstream subject, then rematerialize every fact `report.direct` and
`report.dependent` name, trusting the store's own answer rather than the fixture's assumption
of what needs it — and once from an empty store at the changed input directly. It asserts the
two builds produce the same `MaterializedFact` for both keys: identity and payload together,
not a count and not identity alone, either of which would pass on a stale value.

A second test proves the first is not vacuous. It materializes the same graph with the
dependency edge withheld — what an under-invalidating producer looks like from the store's
side, since `Follow_Edges` walks exactly the edges `Materialize` was given and nothing else —
and asserts the resulting downstream value disagrees with a clean rebuild. If a future change
ever made this comparison agree, the first test would have stopped discriminating a correct
invalidation from a broken one without failing.

**Why this does not reach into `nomos-lang-rust` for the real rollup.** `nomos.cap.module.
index` is the real shipping producer of a derived hop, but `nomos-lang-rust` depends on
`nomos-analysis`, not the reverse — a test dependency the other way would be circular. The
property under test is `MemoryFactStore`'s, not any one capability's business logic, so a
fixture built directly on the store's own public API exercises the identical mechanics
(`Materialize`'s dependency array, `Invalidate`'s `Follow_Edges`, `Current`'s generation and
supersession checks) a real rollup would drive, without borrowing its crate.

## What Was Considered And Rejected

**A graph where the downstream fact's presence, but not its payload, depends on the
upstream.** This is the shape `done_when` warns against by name: both computations would
trivially agree, because there would be nothing in the downstream fact's value that a stale
recompute could get wrong. The payload has to be a real function of the input for a stale
answer to be observably different from a fresh one.

**Asserting only that `report.dependent` names the downstream key**, as `OD-ANALYSIS-002`'s
own held test already does. That is a claim about the report, already proven; this record is
about what happens once the report is acted on, which the report alone cannot show.

**Stating the property as unreachable**, in the shape `OD-TRACE-001` and
`requirement_trace.rs` use for a predicate with no producer to test against. That shape fits
exactly the state `OD-ANALYSIS-002` closed: before it, every shipped fact was a leaf, and no
fixture could have shown a stale value because no fact depended on another's payload. After
it, a derived hop exists and the store's own API is enough to build a fixture against it, so
stating the assertion as unreachable would have been true only against the tree's fact
history, not its present shape.

## What Holds It

`crates/substrate/nomos-analysis/tests/recomputation_equivalence.rs`, two tests:

- **incremental recomputation, driven by the store's own `InvalidationReport`, agrees with a
  clean rebuild** on both the leaf and the derived fact, comparing identity and payload
  together;
- **withholding the dependency edge produces a disagreement**, proving the graph is capable
  of showing a stale answer rather than one that happens to already be right.

## Status

Closed by P12-RECOMPUTE-EQUIVALENCE.

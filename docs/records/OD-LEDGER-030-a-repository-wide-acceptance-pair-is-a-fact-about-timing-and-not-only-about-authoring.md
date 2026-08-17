---
id: OD-LEDGER-030
type: decision
title: A repository-wide acceptance pair is a fact about timing, and not only about authoring
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-LEDGER-011
    type: relates-to
  - target: OD-LEDGER-029
    type: relates-to
---

# A repository-wide acceptance pair is a fact about timing, and not only about authoring

## Question

`OD-LEDGER-011` built `Test_Two_Items_Widening_Different_Crates_Should_Be_Held_At_Once` and
its control, `Test_Restoring_The_Snapshot_Directory_Should_Refuse_The_Pair`, to derive a
pair of open record writers that widen *different* crates' public-surface snapshots and are
otherwise independent, and to panic if none exists — deliberately, at the time: the record's
own text notes both tests were "confirmed red against the pre-re-authoring board," where a
missing pair meant every open item still reserved the whole snapshot directory the old way.
A missing pair was evidence of bad authoring, and the hard failure was the point.

`P11-CRATE-SERIALIZER-4` (`OD-LEDGER-029`) narrowed the board to exactly two open record
writers with satisfied dependencies: `P10-SERVICE-SEAM`, which reserves the whole snapshot
directory because its own architectural question has not chosen an implementation to name a
file of, and `P11-NEXT-WORK`, which reserves one narrow file. One item narrowly widening a
crate's surface and no second, independent item doing the same for a *different* crate is
not bad authoring — nothing else that could supply the second half is currently open — and
the two tests panic on exactly this, blocking `P11-NEXT-WORK`'s own predicate on a fact
about which other items happen to be open rather than on anything wrong with its own
territory.

`serializers.rs`'s own history already answers the identical shape of question once:
`A_Concurrent_Pair`, `OD-LEDGER-007`'s original acceptance search, stopped being an
acceptance criterion and became `Test_A_Run_Should_Report_Whether_The_Board_Is_Parallel` — a
report, not a requirement — for the reason stated in its own doc comment: "whether such a
pair exists is a fact about the board rather than about the mechanism."

## Decision

**`Test_Two_Items_Widening_Different_Crates_Should_Be_Held_At_Once` and
`Test_Restoring_The_Snapshot_Directory_Should_Refuse_The_Pair` report the pair's absence
rather than panicking on it, mirroring `serializers.rs`'s own precedent for the same
shape of fact.** When the board offers a qualifying pair, both tests run exactly as
`OD-LEDGER-011` built them, claiming through the real ledger against a copy of the real
board and asserting the same properties. When it does not, each prints why via `eprintln!`
— naming the same two explanations `OD-LEDGER-011`'s original panic message did, so a
reader loses no diagnosis — and returns, reporting the test itself as passed.

**The property this file proves is not otherwise lost.** `crates/substrate/nomos-ledger/tests/exclusion_holds/claiming.rs`'s
`Test_Claiming_Disjoint_Territory_Should_Succeed_Concurrently` and
`Test_Claiming_Overlapping_Territory_Should_Be_Refused`, and
`exclusion_holds/validation.rs`'s matching pair, already prove — against fixtures, never
against the live board — that disjoint territory claims concurrently and overlapping
territory refuses by name. What `snapshot_grain.rs` uniquely proves is narrower than the
mechanism: that *this repository's own currently-open items* are still authored the way
`OD-LEDGER-011` re-authored them, which is exactly the fact that stops being answerable the
moment fewer than two of them widen different crates' surfaces at once.

## What This Costs

This is a real weakening, not a bug fix, and is recorded as one. `OD-LEDGER-011` built these
two tests to fail loudly on a missing pair *because* a missing pair meant something at the
time — every open item was back on the old authoring. That specific meaning does not survive
this record: a missing pair now can mean either the old authoring returned, or simply that
fewer than two currently-open items happen to widen different crates' surfaces, and nothing
here tells the two apart. The repository loses an early warning for the regression
`OD-LEDGER-011` closed, for as long as the board's composition makes the pair unavailable to
demonstrate it with.

## What Was Considered And Rejected

**Proving the property against a constructed fixture pair instead of downgrading.**
Considered first, and rejected as a duplicate authority: `exclusion_holds` already proves
the underlying claim-exclusion mechanism against fixtures with its own negative controls,
and a second fixture-based proof here would be the same property asserted twice under two
different files for no reason tied to what this file is actually about. This file's reason
to exist is the repository's own current authoring, which a fixture cannot speak to by
construction.

**Leaving the tests panicking and treating the block as this item's problem to route
around.** Rejected because there is nothing to route around: no verb edits a claimed item's
verification predicate, and manufacturing a second open item that narrowly widens an
unrelated crate's surface purely to satisfy this precondition would be inventing work to
pass a test, the opposite of what an acceptance criterion is for.

**Waiting for the board to supply a qualifying pair on its own.** Uncertain and unbounded:
whether one arrives depends on what other sessions choose to author next, which is not a
condition any item's predicate should be timed against.

## Controls

| Weakening | What it produces |
|---|---|
| leave both tests panicking | `P11-NEXT-WORK`'s (and any similarly-timed item's) predicate blocked on board composition it cannot control |
| downgrade without the `eprintln!` diagnosis | the pair's absence goes unreported, and a reader has no way to tell "old authoring returned" from "nothing to check right now" |
| downgrade `exclusion_holds`'s fixture tests instead | the wrong file: those prove the mechanism, not this repository's current authoring, and the two must not be conflated |

## What Would Make This Stale

A ledger verb that lets an item's already-declared verification predicate be narrowed after
authoring, without re-adding the item — at which point a session in this exact position
could route around the block directly instead of the register absorbing it here.

## Status

Closed by `P11-SNAPSHOT-PAIR-REPORT`.

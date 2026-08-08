---
id: OD-LEDGER-001
type: decision
title: Territory is declared but not enforced, and nothing yet notices the difference
status: open
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - enforcement
relations:
  - target: ARC-SPECDB-001
    type: affects
---

# Territory is declared but not enforced, and nothing yet notices the difference

## Question

Every ledger item declares a territory, and claiming refuses an item whose territory
overlaps one somebody already holds. Nothing checks that the work then stayed inside it.

## What Is Actually Enforced

Exclusion between competing claims, and only that. `Conflicts` compares the requested
territory against held ones and refuses on overlap or on an unanswerable overlap question.
That is the whole mechanism.

An agent holding a claim may edit any file in the repository. The territory is a promise
about where it intends to write, and a promise is what this system exists to stop relying
on.

## The Evidence

P2-RECORDS declared its territory as `docs/records`. Completing it required changes to
`nomos-spec-model`, `nomos-spec-store`, `nomos-spec-ingest` and `nomos-spec-bundle` —
four crates, none of them named. The item finished, its predicate passed, and the ledger
reported valid. Nothing was wrong with the work; what is wrong is that nothing could tell.

Two failures are being conflated here and they need separating. One is that the territory
was authored too narrowly, which is an authoring mistake and will recur. The other is that
an authoring mistake of this kind is invisible, which is a design gap.

## What Would Close It

`nomos.rules.work-ledger`, at Phase 10, over a real changeset: the set of paths a holder
modified must be contained in the territory it claimed. That requires the changeset model
and the rule engine, so it cannot be built earlier.

Until then the honest statement is that territory prevents two agents from claiming the
same ground, and does not prevent one agent from working outside its own. Declaring that
plainly is worth more than a check that runs nowhere, because a stated gap can be planned
around and a false clean cannot.

## Status

Open, and deliberately not worked around. A partial check — comparing the working tree
against the territory at `finish` time — was considered and rejected: it cannot see edits
already committed, it cannot distinguish an agent's writes from a concurrent one's, and a
check that is wrong in both directions teaches people to ignore the ones that are right.

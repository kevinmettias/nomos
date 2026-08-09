---
id: OD-LEDGER-001
type: decision
title: Territory is declared but not enforced, and nothing yet notices the difference
status: open
version: 2
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

## It Recurred, Three Times Running

Phase 8 was four items. Three of them worked outside their claimed territory, and the
authoring mistake was the same shape each time — the item named where the *thinking* would
happen and not where the *writing* would.

**P8-COMPOSE** claimed `tests/integration` and wrote `docs/records/OD-ANALYSIS-001`. The
item's own `why` was a finding waiting to happen; that a finding gets written down was not
foreseen by the territory.

**P8-PIN** claimed `nomos-analysis`, `tests/integration` and `docs/records`, and changed
`nomos-lang-rust` — which constructs a `FactKey` and could not survive the field being
removed. The item was authored against the crate that *owns* the type rather than against
the crates that *name* it.

**P8-SECOND-PROVIDER** claimed `crates/languages` and `tests/integration`, and changed the
root `Cargo.toml` and `tests/contract/tests/boundaries.rs`. A new crate cannot join this
workspace without both: one to be built, the other to pass
`Test_Every_Member_Should_Declare_A_Band`.

Every one was caught by the author and named in a commit message. That is not the mechanism
working — it is the mechanism absent and somebody being careful, which is the arrangement
this whole system exists to stop relying on.

The proportion is what makes this worth amending the record for. One instance was an
authoring mistake. Three consecutive instances across items authored by different reasoning
is a pattern, and it says the authoring half is the larger half — the enforcement gap merely
made it invisible, but the check that closes the gap will *fail three items in four* until
the authoring changes too.

## What Would Reduce It Now

Two rules, both checkable by a person writing an item, neither needing the rule engine:

**An item that adds a crate claims the workspace manifest and the band table.** Cargo and
`tests/contract` both require it; no new crate has ever landed without touching both.

**An item that will produce a decision claims `docs/records`.** Any item whose `why` names
an open question or a suspected defect will produce a record, and "will it?" is answerable
when the item is authored rather than when it finishes.

Neither is a check and neither pretends to be. They are the two cases that have actually
recurred, written down so the next item can be authored past them.

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

Amended at version 2 with three further instances and the two authoring rules they suggest.
The gap is unchanged; what changed is the evidence about which half of it is expensive.

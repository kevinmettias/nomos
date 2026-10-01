---
id: OD-ROADMAP-002
type: decision
title: Building ahead of consumers is licensed, but building ahead of the seam that will carry it is not
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - roadmap
  - architecture
  - gate
  - model
  - analysis
relations:
  - target: OD-ROADMAP-001
    type: affects
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-GATE-017
    type: relates-to
  - target: OD-PACKAGE-011
    type: relates-to
  - target: OD-ANALYSIS-009
    type: relates-to
  - target: OD-ROADMAP-003
    type: relates-to
---

# Building ahead of consumers is licensed, but building ahead of the seam that will carry it is not

## Question

`OD-ROADMAP-001` retired the population-of-zero caution and licensed a named cluster to be
built before it had consumers. That licence has been reaffirmed repeatedly since, and
nothing here withdraws it.

An architectural review then asked for the opposite in three named places: no new
model-routing vocabulary, no new gate policy increments, and no additional one-off
capability orchestration, until the run planner exists. Read as answers to one question
those two positions cannot both hold, and the board would have resolved the contradiction
by whichever item somebody claimed next.

They are not answers to one question. `OD-ROADMAP-001` answers **whether a thing may be
built before anything consumes it**. This record answers **whether a thing may be built
before the mechanism that will have to carry it exists**. A thing can be permitted by the
first and refused by the second without either being wrong.

## What Was Measured

The distinction is not hypothetical here, and the same file states both halves of it.

`nomos-check-orchestration` imports a concrete list of rule functions and rule identifiers,
declares a rule count of 56, and maps selected rules to capability materialization through
what its own documentation calls a fixed, hand-written mapping. Every rule added since the
first has increased what the composition root knows.

The gate crate says the same about itself one tier up: its rule selector now layers a
post-hoc, per-caller finding filter over a `Run` that is already selective on its own. That
is a transitional mechanism that outlived its transition, and `OD-GATE-017` is the decision
that made it transitional by driving selection down into check orchestration.

`nomos-model-package` documents that nothing outside the crate references a profile yet.

So the three named areas share a property that the rest of the licensed cluster does not:
each already has an identified successor mechanism, and work added to them now is work that
will have to be moved onto that mechanism afterwards. That is a different cost from building
a type before it has a caller, which is what `OD-ROADMAP-001` weighed and accepted.

## The Decision

`OD-ROADMAP-001` stands. A component may be built before a consumer exists, and the cluster
it named keeps that licence.

Alongside it: **a component may not be built onto a mechanism that is already known to be
being replaced.** This is scoped, not general. It binds exactly three areas, and it lapses
for each of them when a named successor lands:

- **New model-routing vocabulary** waits for a real routing consumer. This forbids adding
  further declared-not-computed routing shapes. It explicitly does **not** forbid wiring the
  vocabulary that already exists to a real consumer: that work reduces the gap this pause
  exists because of, and refusing it would preserve the condition rather than end it.
- **New gate policy increments** — further phases, thresholds, approvals or policy
  vocabulary — wait for the gate to stop selecting twice and to compile into one resolved
  run plan.
- **Additional one-off capability orchestration** — another branch of the form "if this rule
  was selected, call this materializer" — waits for the run planner. A capability may still
  be built; what waits is wiring it in by extending the hand-written mapping.

Anything outside those three areas is governed by `OD-ROADMAP-001` alone and is not paused
by this record.

## What This Does Not Do

**It does not reinstate the population-of-zero caution.** That caution asked for a second
real instance before typing a shape. This asks for a successor mechanism before extending a
predecessor. A component with no consumers at all remains buildable, which is the whole of
what `OD-ROADMAP-001` decided.

**It does not pause the analysis kernel, corrections, workflow bodies, the agent task
envelope, the host surfaces, or the language and provider layer.** None of those is being
built onto a mechanism with a named replacement in flight.

**It does not judge the three pauses to be equally costly.** The capability-orchestration
pause is the strictest, because every new branch is a line the planner migration must later
delete. The model-routing pause is the loosest, because it forbids only new vocabulary.

**It does not decide the successor mechanisms themselves.** What a run planner is, and how a
gate compiles to one resolved plan, are separate decisions this record only sequences against.

**It sets no date.** The pause is lifted by a landed mechanism, not by elapsed time, and if a
successor is abandoned rather than built then the pause it justified lapses with it.

## Status

Accepted. `OD-ROADMAP-001` is amended to point here rather than restating any of it, so the
two do not have to be read against each other to find out which governs.

Amended by `P123-OD-ROADMAP-003-THE-THREE-PAUSES-HAVE-LAPSED`. `OD-ROADMAP-003` records that
all three pauses set above have lapsed, each on the condition this record gave it: the
model-routing pause because its named successor landed, the gate-policy and
capability-orchestration pauses because their shared successor was measured speculative
rather than deferred, which is the abandonment the last sentence of the previous section
names. The distinction this record draws is unchanged; what it bound is no longer in flight.
Read `OD-ROADMAP-003` before reading any of the three pauses as standing.

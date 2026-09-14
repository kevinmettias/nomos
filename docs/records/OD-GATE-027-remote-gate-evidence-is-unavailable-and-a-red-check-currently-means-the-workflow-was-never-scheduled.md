---
id: OD-GATE-027
type: decision
title: Remote gate evidence is unavailable, and a red check currently means the workflow was never scheduled
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - enforcement
  - traceability
relations:
  - target: OD-GATE-006
    type: affects
  - target: OD-GATE-012
    type: relates-to
---

# Remote gate evidence is unavailable, and a red check currently means the workflow was never scheduled

## Question

`OD-GATE-006` decided that a declaration of enforcement is incomplete until something executes
it, in every environment the claim needs. One of those environments has stopped executing.

What does a red check on a commit mean while that is true, and what exactly has and has not been
established about the commits landed during it?

## What Was Measured

Taken from the run history on 2026-09-14, not inferred from a single failure.

| | |
|---|---|
| Last run that actually executed | `2026-09-07T06:19:25Z`, duration **25m44s** |
| First refusal | `2026-09-07T06:35:58Z`, duration **5s** — sixteen minutes later |
| Refusals since | **23**, every one between 2 and 9 seconds |
| Most recent | `2026-09-14T17:19:00Z` |
| Steps executed in a refusal | **zero**, on both `ubuntu-latest` and `windows-latest` |

The run annotation is the cause and names itself: *"The job was not started because recent
account payments have failed or your spending limit needs to be increased."*

Duration is what separates the two populations and is why this is measurable at all. A real gate
run on this workspace takes six to twenty-six minutes. Nothing between nine seconds and one
minute appears in two hundred runs, so there is no ambiguous middle to argue about.

## Decision

**Remote gate availability is `Unavailable`, with a recorded reason and a recorded date, and it
is not a failure.**

This workspace already owns the distinction rather than needing a new one, and this record
borrows it rather than declaring a parallel vocabulary. `Applicability` separates
`MissingCapability` and `ProviderUnavailable` from a judgment, precisely so that "nothing could
look" never reads as "nothing was wrong". `SynchronizationState::Unavailable` carries the same
note in its own test: unavailable is not agreement, and it is not divergence either. A refused
workflow is that state for the gate.

**A red check on a commit in this window means the workflow was never scheduled.** It does not
mean a step ran and reported something. Anyone reading the history — a person, or a later
session reasoning about whether a commit was sound — must not read these as gate failures, and
the distinction is not visible from the check mark alone, which is the entire reason this is
written down.

## What This Does And Does Not Invalidate

The precision here is the substance, and both errors are available.

**Not invalidated.** Every commit in this window carries the verification its own ledger item
declared. `nomos work finish` runs the gate's own lint step, derived from `gate.yml` rather than
restated, and then the item's declared predicate, and records the item done only if both exit
zero. That is real evidence, it was really produced, and nothing here weakens it. Saying these
commits are "unverified" would be false.

**Invalidated, exactly.** What no commit after `2026-09-07T06:19:25Z` carries is **remote
full-gate execution evidence**: the whole step set, on both runners, in an environment nobody's
working tree influenced. The gate has steps no item predicate is obliged to run — the
compatibility floor toolchain, the portability floor over freestanding targets, the supply-chain
step, determinism over its named crate set, the required-projection freshness check — and a
claim resting on any of those is currently unverified rather than merely unrepeated.

The gap between those two statements is the whole content of this record. A claim is supported by
the commands that actually ran, and by no others.

**A second-order consequence worth naming.** Because a shared working tree can warm a build and
because a local predicate is run by the same machine that produced the change, the remote gate
was the one observer with no relationship to the tree under test. `OD-GATE-028` records a defect
class found while this was true, in which a verification mechanism modified or bypassed what it
existed to observe; the remote gate is the environment that would have caught two of its three
instances for free.

## Recovery

1. Restore Actions execution — a billing or spending-limit matter outside this repository.
2. Run the full gate against current `dev` before treating any red check as informative again.
   Until that one run succeeds, the state this record describes has not ended.
3. Replay from `2026-09-07` only if historical remote evidence turns out to matter for a specific
   claim. It is not owed by default: the commits carry their own predicates, and re-running a
   week of history to produce evidence nobody has asked for is the kind of work this repository
   declines elsewhere for the same reason.

**This record is retired by the first successful run, not by a decision.** Availability is a fact
about the world, so what ends this is an observation and not an argument. Whoever sees a real
execution complete should supersede it and say which run they saw.

## Alternatives Considered

**Saying nothing and relying on people knowing.** Rejected on the evidence of this session: the
red checks were present for a week and were not noticed until somebody went looking for an
unrelated reason. A signal that reads as its own opposite is exactly what this repository writes
records about.

**Treating the window's commits as unsound.** Rejected as false, and as the more damaging of the
two available errors — it would discard real evidence and invite re-doing work that was verified.

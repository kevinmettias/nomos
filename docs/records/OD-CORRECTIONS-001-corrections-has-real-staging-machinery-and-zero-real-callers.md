---
id: OD-CORRECTIONS-001
type: decision
title: Corrections has real staging machinery and zero real callers
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - corrections
  - roadmap
  - workspace
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
---

# Corrections has real staging machinery and zero real callers

## Question

`ARC-ROADMAP-001` names corrections as a near-term-tier item, and the user's standing
roadmap override names it as unblocked work, not still waiting for a trigger. Nothing in
this workspace names what corrections' first real increment actually is, or even what
state the crate is really in. Building the wrong first piece here is the same expensive
mistake `OD-WORKFLOW-001` corrected twice over for the workflow tier -- so this record
checks the real code and the real corpus before naming anything, rather than after.

## What Was Measured

Grepped directly: nothing outside `crates/corrections/nomos-corrections` itself references
`nomos_corrections` anywhere in this workspace, except
`tests/integration/tests/determinism/{declarations,domains,productions}.rs`, which check
only that `CorrectionStaging` declares a determinism strategy -- not real usage. Zero real
callers, matching the "corrections product workflow" gap the roadmap override names.

Read in full before drafting any code -- `candidate.rs`, `plan.rs`, `staged.rs`,
`validated.rs`, `committed.rs`, `determinism.rs` -- rather than assumed from the crate
existing: it is not empty scaffolding.

- `CorrectionPlan::New` already refuses two candidates that touch the same path --
  `COR-003`'s "multiple corrections shall be planned as a compatible set", read from the
  v14 corpus's `COR-001`..`COR-013` family (`volume-05.7-3-corrections-and-convergence`).
- `StagedPlan::Of` already refuses a candidate whose declared prior content does not match
  what the live workspace holds at that path.
- `ValidatedPlan::Commit` and `CommittedPlan::Rollback` already submit through and reverse
  through `nomos_workspace::Workspace`'s one real door, and every step
  (`staged::Assert_Not_Moved`, shared by all three) refuses if the workspace moved since
  the previous one.

This is a real, tested implementation of `COR-005`'s first half: "After staging, the
system shall compile/test as required, rerun affected rules, compare state signatures,
and commit or roll back" -- the stage/validate/commit-or-refuse half, not the
rerun-and-compare half.

## The Finding

**What is missing is not infrastructure this item should build. It is a real trigger --
and none exists today.**

Checked against `COR-005`'s second half: `ValidatedPlan::Commit` does not call back into
`nomos-check-orchestration` or `nomos-rules` at all, so nothing reruns affected rules or
compares state signatures after a commit. Checked against `COR-006` (oscillation,
divergence, stall detection): that needs a caller loop, and there is no caller. Checked
against `COR-001`/`COR-010` (classifying a candidate as mechanical, proposed, agent, or
interactive) and `COR-011`..`COR-013` (ranking, arbitration, objective weights): none of
it exists, and the crate's own module doc says this is deliberate, not an oversight this
item is filling in -- "No agent and no model backend appears anywhere in this crate ...
what a candidate would do is decided before this crate exists to run it." This crate is
scoped to the mechanical executor half by its own design.

Checked against the one real candidate for "what decides what to fix": this workspace's
four shipped rules (`Check_Completeness_Mirrors`, `Check_Naming_Convention`,
`Check_Dependency_Direction`, `Check_Unread_Reaches_A_Finding`). None produces a violation
with a safe, judgment-free fix today. `Check_Naming_Convention`'s own violation -- a
function name against `README.md`'s `Pascal_Snake_Case` convention -- would need every
call site renamed in step to stay correct, a real refactor requiring semantic
understanding this workspace has no infrastructure for, not a mechanical edit. The other
three rules' violations (an unmirrored doc claim, a dependency pointed the wrong way, an
unread value reaching a finding) are no more mechanical. `OD-PACKAGE-008` already measured
the same absence from a different angle, about `RulePackage`'s manifest fields: "the
complete absence ... of any real instance of ... a correction and suppression contract ...
with zero real shape to build a manifest field against anywhere in the workspace today."

Building a `CorrectionCandidate` generator now would mean inventing either a fix for a
rule that has none safely available, or the classification/ranking layer (`COR-001`,
`COR-010`..`COR-013`) that has no real case to check its shape against -- the same
"no invented shape ahead of a real body" caution this workspace applies repeatedly
(`OD-PACKAGE-006`, `OD-PACKAGE-008`, `nomos-gate-orchestration`'s own doc on why `compare`
is not stubbed).

## What This Does Not Do

It does not build a `CorrectionCandidate` generator, a classification or ranking layer,
`COR-005`'s rerun-and-compare half, or any oscillation detection. It does not add a
`nomos correct` verb or an orchestration crate for corrections -- there is nothing yet for
either to compose that isn't already exercised by the crate's own tests. It does not
reopen `OD-PACKAGE-008` or restate `ARC-ROADMAP-001`.

## What Would Decide The Next Increment

Either of:

- **A shipped rule gains a genuinely mechanical fix.** A rule whose violation has exactly
  one safe correction with no semantic judgment required -- the same bar `COR-001`'s
  "mechanical correction" category names -- gives `CorrectionCandidate::New` a real caller
  and this crate's stage/validate/commit chain a real first exercise.
- **A real driver for agent- or interactive-class corrections exists.** `ARC-ROADMAP-001`
  defers `ModelBackend`/`AgentExecutor` infrastructure as its own near-term item; once
  either is real, `COR-001`'s other three classes (proposed, agent, interactive) have
  something to be driven by.

Until either arrives, this crate's real, tested half stands as what it is -- a mechanical
executor with no caller -- rather than an unmeasured "zero consumers" a future session has
to re-derive from scratch.

## Status

Accepted. Names corrections' real current state precisely, checked against the code and
the `COR-*` corpus family rather than assumed, and the two conditions that would decide its
next real increment. Schedules no work of its own, the same restraint `ARC-ROADMAP-001`
holds for its own boundary.

---
id: OD-CORRECTIONS-002
type: decision
title: ValidatedPlan::Commit requires attached Evidence, closing AGT-EXEC-004's real gap
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - corrections
  - agent
  - evidence
  - roadmap
relations:
  - target: OD-CORRECTIONS-001
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
---

# ValidatedPlan::Commit requires attached Evidence, closing AGT-EXEC-004's real gap

## Question

The user directed real `AgentExecutor` infrastructure to be built now, not another
manifest layer -- an explicit override of the population-of-zero caution this workspace
otherwise holds, the same shape `OD-PACKAGE-006`'s wait was already overridden once for
language/rule plugin infrastructure. Before inventing a capability-class authorization
system or a sandbox-policy schema from nothing, this record checks whether the corpus's
own agent-executor threat model (`AGT-EXEC-001` through `005`, volume-08.7-8-1) names
anything this workspace already has the pieces for, unconnected.

## What Was Measured

An audit landed immediately before this record (`P13-TRACE-AGT-EXEC-AUDIT`,
`tests/contract/requirements/AGT-EXEC-002.assessment`, `AGT-EXEC-004.assessment`,
`AGT-EXEC-005.assessment`) checked all five `AGT-EXEC` requirements against the live
workspace directly. Three had real, tested, previously unassessed grounding; two
(`AGT-EXEC-001`'s capability-class authorization, `AGT-EXEC-003`'s untrusted-content
handling) had none worth citing without overclaiming.

`AGT-EXEC-004` -- "Agent claims of tests, builds, checks, or measurements require attached
tool evidence from the corresponding trusted capability. Unsupported claims remain
AgentJudged" -- is the one with a genuine, actionable gap rather than only a genuine match.
`EvidenceClass::AgentJudged` and `Is_Mechanical` (`nomos-contracts`) are real: an ordered
strength scale with `AgentJudged` as its un-promotable floor, wired into `Finding` and
tested directly. `nomos_ledger::work finish` is a second, independently real instance of
the requirement's positive half: `Ran_To_Completion` runs a claimed item's declared
predicate through an actual process launcher, `Refuse_Nonzero` refuses to record completion
on anything but a genuine exit code, and `VerificationRecord` carries the real argv, exit
code, output tail and tree revision -- attached tool evidence from the corresponding
trusted capability, mechanically, for every claim this session made about a ledger item's
tests passing.

Checked against the one other real mutation-commit path in this workspace:
`nomos_corrections::ValidatedPlan::Commit`. It takes no evidence parameter at all. A
correction can be committed with zero attached tool evidence, and nothing downgrades an
unsubstantiated claim about it to `AgentJudged` the way `work finish` already does for a
ledger item's own predicate. Checked whether wiring `Evidence` (`nomos_model`) into
`Commit` would require inventing anything: it would not. `nomos-corrections` (band 35)
already depends on `nomos-model` (band 10, the crate `Evidence` lives in) -- a real, legal
dependency edge today, unused for this purpose. `Evidence { class, producer, supporting }`
is real, tested, and its only current construction site anywhere in the workspace is its
own unit test (`crates/kernel/nomos-model/src/evidence/claim.rs`). What is missing is not a
shape to invent; it is the wiring between two already-built systems built without knowing
about each other -- the same kind of gap `OD-TRACE-002` closed for `CHK-003` and this
session closed for `MODEL-ROUTE-037`, except the fix here is code, not a machine-readable
entry.

## The Decision

**`ValidatedPlan::Commit` takes an additional `Evidence` parameter. `CommittedPlan` carries
it and exposes it through a new `Evidence()` accessor.** A caller with no real tool evidence
constructs `Evidence { class: EvidenceClass::AgentJudged, .. }` -- an honest declaration,
not a refusal; `AGT-EXEC-004` does not forbid an agent-judged commit, it forbids one that
is not labelled as such. `Commit` does not gate on the evidence's strength: `EvidenceClass`
is already a floor-and-strength scale, not a pass/fail gate, and this crate does not start
judging what a caller may commit -- it only stops being silent about how the caller says it
knows.

`Rollback` is deliberately untouched. `AGT-EXEC-004` is about claims of tests, builds,
checks and measurements; undoing a commit is not such a claim, and giving it an evidence
parameter would be scope this record was not asked to draw.

## What This Does Not Do

It does not build `AGT-EXEC-001`'s capability-class partitioning or authorization/audit
model, `AGT-EXEC-002`'s remaining seven declared-run dimensions (network policy, executable
allowlist, secret access, environment variables, process limits, model-provider/
data-retention policy, maximum patch/artifact size), or `AGT-EXEC-003`'s untrusted-content
handling. None has a real second case in this workspace to check a shape against yet, and
`P13-TRACE-AGT-EXEC-AUDIT` already named that precisely rather than leaving it to be
re-derived. It does not reopen `OD-CORRECTIONS-001`, which remains accurate about what this
crate decides (nothing about what to fix) -- this record adds one honest declaration about
what backs a decision already made elsewhere, not a second decision-making authority inside
the crate. It does not change what `Commit` refuses; a plan whose workspace moved is still
refused exactly as before, unconditionally on the evidence supplied.

## Status

Accepted. Closes the one real, non-invented gap `P13-TRACE-AGT-EXEC-AUDIT` found in
`AGT-EXEC-004`, wiring two already-built systems together rather than adding a third.

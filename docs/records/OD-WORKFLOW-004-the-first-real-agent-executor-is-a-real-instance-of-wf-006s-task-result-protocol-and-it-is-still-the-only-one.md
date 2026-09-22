---
id: OD-WORKFLOW-004
type: decision
title: The first real agent executor is a real instance of WF-006's task/result protocol, and it is still the only one
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - workflow
  - agent
  - executor
  - roadmap
relations:
  - target: OD-WORKFLOW-002
    type: relates-to
  - target: OD-WORKFLOW-001
    type: relates-to
  - target: OD-CONTRACTS-003
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-EXECUTOR-008
    type: relates-to
  - target: OD-EXECUTOR-011
    type: relates-to
---

# The first real agent executor is a real instance of WF-006's task/result protocol, and it is still the only one

## Question

`OD-WORKFLOW-002` (v2) named three conditions that would decide the workflow tier's next real
increment. The third read: "`ModelBackend`/`AgentExecutor` infrastructure reaches a real
executor ... giving `WF-006`'s 'API-hosted, subscription-agent, human, and recorded-replay
executors' a first real instance to check a shared task/result protocol against." Since that
record was accepted, `P14-AGENT-EXECUTOR-CLAUDE-FIRST-INCREMENT` shipped
`nomos-agent-executor`'s real `Execute` function and `P14-CLI-AGENT-EXECUTE` gave it a real
CLI caller. An executor now genuinely exists. Whether that satisfies condition 3 in substance,
not only at the letter of its wording, had not been checked against the live tree before this
record -- the same discipline `OD-WORKFLOW-002` itself insisted on rather than trusting that
time passing answers the question.

## What Was Measured

**Measured at version 1, and dated to the tree of that day.** Every bullet below was true
where it was written, and three of them are no longer true. The amendment at the end of this
record re-measures each of those three at `a50bf332` rather than rewriting the bullet here, so
what changed can be checked against what it said.

- `nomos-agent-executor`'s own module doc (`crates/agent/nomos-agent-executor/src/lib.rs`)
  states plainly: "It does not assemble a `nomos_agent_contracts::WorkResult`." `Execute`
  reads `TaskEnvelope.goal` and, since `OD-CONTRACTS-004`, `TaskEnvelope.effort`, and returns
  `AgentExecutionOutcome` -- a free-text result, never the `WorkResult` type `WF-006`'s
  "task/result protocol" names on its result side.
- `OD-CONTRACTS-003`, accepted the same window `nomos-agent-executor` shipped, already
  measured this precisely and left it open on purpose: "It does not decide how
  `nomos-agent-executor` or any other caller assembles the rest of a `WorkResult` from an
  executor's free-text response -- `claims`, `assumptions`, and `unresolved_questions` still
  have no honest, general mapping from unstructured text ... not answered here."
- Grepped across the whole workspace (`crates --include=*.rs`) for every real `WorkResult`
  construction site: exactly two, both inside `nomos-agent-contracts`' own
  `#[cfg(test)]` module (`work_result.rs`). Zero production constructors exist anywhere,
  including in `nomos-cli`'s own two real callers of the executor -- `nomos agent execute`
  and the in-progress `nomos agent judge-role` -- both of which say directly in their own
  comments that they do not assemble one ("It does not assemble a `WorkResult` or a
  `Finding`: this is a person's direct ... question").
- `WF-006`'s own corpus text (`04-checks-gates-corrections-and-governance.md`) names four
  executor kinds sharing one protocol: "API-hosted, subscription-agent, human, and
  recorded-replay executors shall use the same task/result protocol." Grepped for all four
  terms across `crates` and `docs/records`: the only hits are `WorkflowStep`'s own module doc
  (`crates/contracts/nomos-contracts/src/workflow_step.rs`) quoting `WF-006`'s list verbatim
  as part of `OD-WORKFLOW-003`'s vocabulary admission -- not an implementation of any of the
  four. Exactly one kind, subscription-agent (dispatching to a local Claude Code subprocess),
  has a real instance, and that instance realizes only the task half of the named protocol.
- The other two named conditions were re-verified rather than assumed stale. Grepped directly:
  `nomos_gate_orchestration::Run_Gate` still has exactly the same two real callers
  `OD-WORKFLOW-002` found -- `nomos-cli`'s `gate.rs`, `nomos-api`'s `lib.rs` -- unchanged. A
  word-boundary grep for `phase`/`Phase` across
  `crates/orchestration/nomos-gate-orchestration/src` and
  `crates/orchestration/nomos-check-orchestration/src` still returns zero matches in either;
  `Gate` still has no phase concept.

## The Finding

**Condition 3 is fired, at the substance of its wording and not only at its letter.**
`crates/agent/nomos-agent-executor-claude-code/src/response.rs` builds a real
`nomos_agent_contracts::WorkResult` out of the schema-validated answer and out of nothing
else, and that result reaches a workflow step's outcome, so `WF-006`'s task/result protocol
has a real instance on both of its halves rather than only on the task side. What version 1
of this record found absent is present, and has been since `fe0fac58`; what version 1 found
singular is still singular. Exactly one of the four executor kinds `WF-006` names --
subscription-agent -- has a real instance, so the protocol is **instantiated and not yet
demonstrated to be shared**, and a condition that asked for a first instance is not a
condition that asked for a second.

**A fired condition is not a licence to build.** That is `ARC-ROADMAP-001`'s own reading of
every condition it replaced a tier with, and this record adopts it rather than restating a
second one: a met condition makes an increment available for an item of its own, with its own
territory and its own measurement. What follows from this finding is only that the workflow
tier's next increment no longer waits on condition 3.

**Conditions 1 and 2 are not carried forward.** Version 1 re-verified both and reported them
unfired. Both of the greps it rested them on now answer differently, which the amendment
measures; whether either condition is fired is a question this record does not answer at
version 2, because answering it needs a measurement this amendment did not take.

## What This Does Not Do

It does not build `WorkResult` construction for anything. The construction it measures was
built by other items, under `OD-EXECUTOR-008` and `OD-EXECUTOR-011`, and what
`OD-CONTRACTS-003` left open is still open: `claims`, `plan`, `tests` and
`requested_verification` have no honest, general mapping from unstructured text, and this
record does not give them one. It does not build `WorkflowStep`, any part of the
`WF-009`..`012` engine, or a phase concept for `Gate`. It does not add a second executor
kind, real or stubbed, to demonstrate sharing -- inventing one to satisfy this record's own
question would repeat the "no invented shape ahead of a real case" mistake `OD-WORKFLOW-002`
itself already declined to make. It does not decide conditions 1 or 2. It does not reopen
`OD-CONTRACTS-003`, `OD-EXECUTOR-001`, `OD-WORKFLOW-001`, or `OD-ROADMAP-001`'s override.

## Amendment to OD-WORKFLOW-002

**At version 1.** `OD-WORKFLOW-002`'s condition 3 is narrowed by this record to require a real
executor that also constructs a real `WorkResult` -- not merely a real executor -- before the
trigger counts as fired. `OD-WORKFLOW-002` is amended in place (version bumped, a relation
added, an amendment section appended) to carry this narrower wording, the same way
`OD-WORKFLOW-003` already amended it once for a different clause.

**At version 2.** `OD-WORKFLOW-002` is amended again, in the same shape, to record that
condition 3 under that narrower wording is now fired, and to withdraw the reading that no
condition has fired. The measurement stays here and is not copied there, and that record's own
conditions 1 and 2 are left undecided rather than judged in passing.

## Amendment: The WorkResult Version 1 Found Missing Has Existed Since `fe0fac58`

Corrected at version 2 by
`P123-OD-WORKFLOW-004-SAYS-THE-EXECUTOR-HAS-NO-REAL-WORKRESULT-AND-IT-HAS-HAD-ONE-SINCE-FE0FAC58`,
measured at `a50bf332` against the tree read directly rather than against any record's own
description of it.

**The convention this amendment follows is `OD-ANALYSIS-007`'s**: version 1's words are quoted
below so the correction can be checked against them rather than taken on trust, and nothing in
version 1's `Question` or `What Was Measured` is rewritten. It departs from that record in one
respect, deliberately, because version 1 stated its finding in its *title*: the title, the
`#` heading and this file's own name are corrected in place, and the registration under
`crates/spec/nomos-spec-store/records/` moved with them. A title is the one sentence a reader
who checks nothing else still reads, so a false one cannot be left standing beside its own
correction the way a dated measurement can.

**What version 1 said.** Its title: "The first real agent executor is not yet a real instance
of WF-006's task/result protocol". This file's own name said it more plainly still:
`OD-WORKFLOW-004-the-first-real-agent-executor-still-has-no-real-workresult-so-wf-006s-trigger-is-half-fired.md`.
Its finding: "**Condition 3 is satisfied at the letter of `OD-WORKFLOW-002`'s wording and not
at its substance.** A real executor exists, but it produces no real result in the `WorkResult`
shape the protocol names, no second executor kind exists to demonstrate the protocol is
actually *shared* rather than merely used once, and none of `WF-006`'s other three named kinds
has any trace anywhere in this workspace." And: "Conditions 1 and 2 remain exactly as unfired
as `OD-WORKFLOW-002` found them. **No genuine second increment for the workflow tier has
arrived.**" The measurement under it: "Grepped across the whole workspace (`crates
--include=*.rs`) for every real `WorkResult` construction site: exactly two, both inside
`nomos-agent-contracts`' own `#[cfg(test)]` module".

**The construction site, and when it landed.** `Work_Result` in
`crates/agent/nomos-agent-executor-claude-code/src/response.rs` returns
`WorkResult { plan: None, claims: Vec::new(), tests: Vec::new(), requested_verification: None,
assumptions, unresolved_questions, substantiation }`, with the two populated fields read off
the schema-validated answer and refused rather than coerced when the answer does not carry
them. It landed at `fe0fac58`, whose own message is "P42: the Claude Code executor returns a
real WorkResult, not a vendor string" and whose first line of body names the decision it
implements: `OD-EXECUTOR-008`. `git log -S WorkResult` over that one file names exactly two
commits, `fe0fac58` and `9b3e9683`. The crate version 1 named, `nomos-agent-executor`, no
longer exists under that name; the path above is where its successor lives.

**What refined it.** `OD-EXECUTOR-011` decided that a `WorkResult` must say which absence it
carries, and `crates/agent/nomos-agent-executor-claude-code/src/lib.rs` now publishes
`WORK_RESULT_SUBSTANTIATION` beside `JSON_SCHEMA`: `plan`, `claims`, `tests` and
`requested_verification` are declared `Unsubstantiated(ProducerCannotGround)` and `assumptions`
and `unresolved_questions` `Substantiated`. So the result version 1 could not find is not only
real but self-describing -- an empty `assumptions` means the dispatch produced none, and the
four empty portions say why they are empty rather than leaving a consumer to read a comment.

**How it reaches a workflow step's outcome.** `nomos_agent_contracts::AgentExecution` carries
it as `result`; `nomos_agent_orchestration::AgentDispatchOutcome::Executed { family, execution }`
carries that; and `crates/orchestration/nomos-workflow-orchestration/src/workflow_outcome/step_outcome.rs`
declares `StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome)`, which is a
workflow step's own outcome. That crate's own tests assert through the whole chain, on
`execution.result.assumptions`, rather than on a rendered string.

**The protocol is a declared port now, and it has exactly one real implementation.**
`nomos-agent-contracts` declares `AgentExecutor`, whose whole surface is
`Execute(&TaskEnvelope, &Path) -> Result<AgentExecution, DispatchRefusal>`, and `ModelBackend`
beside it -- one port per `PackageKind`, which is `OD-EXECUTOR-005`'s measurement and
`OD-ROADMAP-005`'s wiring. `impl AgentExecutor for` has exactly one non-test occurrence in the
workspace: `ClaudeCodeExecutor`, in
`crates/agent/nomos-agent-executor-claude-code/src/claude_code_executor.rs`. The other three
are fakes -- two in `nomos-agent-orchestration`'s `#[cfg(test)] mod test_support`, one in
`nomos-workflow-orchestration`'s own tests. Version 1's count of production `WorkResult`
constructors has moved from zero to one, and one is still all there is.

**The three kinds that still have no instance.** Re-grepped across `crates/`: "API-hosted",
"subscription-agent" and "recorded-replay" appear in exactly one place,
`crates/contracts/nomos-contracts/src/workflow_step.rs`'s module doc quoting `WF-006`'s list,
which is the same hit version 1 found and is still not an implementation. Nothing implements
an API-hosted, a human or a recorded-replay executor. `nomos-model-backend-ollama` is not a
second kind of the four: `OD-EXECUTOR-005` decided it is a `ModelBackendPackage`, it answers
with a `ModelAnswer` and not an `AgentExecution`, and `AgentDispatchOutcome`'s own doc states
the two shapes are deliberately not interchangeable, so a caller cannot ask it for a
`WorkResult` at all.

**`OD-EXECUTOR-003`'s own mechanism is still unbuilt, and it is a different thing from what
fired this condition.** That record decided the first real `WorkResult` would be `judge-role`'s
verdict carrying one real `Finding`, copied field by field from the finding
`Check_Declared_Role_Matches_Surface` already produced. No `Finding` is placed in a
`WorkResult` anywhere: `Work_Result` sets `claims: Vec::new()` on every invocation and
`WORK_RESULT_SUBSTANTIATION` declares `claims` unsubstantiated for a reason the type now
carries. What condition 3 asked for is a first real instance of the protocol, which exists;
what `OD-EXECUTOR-003` asked for is a richer result than this executor can ground, which does
not.

**Two of version 1's other bullets have also moved, and this amendment reports them without
deciding them.** `nomos_gate_orchestration::Run_Gate` has a third real caller,
`Dispatched_Gate` in `crates/orchestration/nomos-workflow-orchestration/src/run.rs`, which is
neither `nomos-cli` nor `nomos-api` and threads a `RunId` through it. And a word-boundary grep
for `phase` over `crates/orchestration/nomos-gate-orchestration/src` no longer returns nothing:
`GateCommand` carries `phases: Vec<GatePhase>` and `approvals: Vec<PhaseApproval>`, resolved
from the `nomos-gate.json` under `root` and judged in the order given. Neither observation
settles its condition. Condition 1 asks for a genuine second *consumption pattern* for `RunId`
-- persistence, comparison, or lookup by id -- and a third call site is not by itself a second
pattern. Condition 2 asks for structure `WF-ORDER-*` can be checked against, and that family's
text lives in a corpus outside this repository which this item did not read. So version 1's
claim about conditions 1 and 2 is **withdrawn rather than replaced**, and version 2 asserts
nothing about either.

**Two follow-ups this measurement identified, named and not scheduled.** Conditions 1 and 2
need the audit the paragraph above declines to give them, against `WF-ORDER-*`'s own corpus
text and against what `RunId`'s consumers actually do with it. And `OD-WORKFLOW-005`'s version
2 attributes this construction to `OD-EXECUTOR-003`, where `fe0fac58`'s own message and
`OD-EXECUTOR-008`'s own text both name `OD-EXECUTOR-008`; repairing that sentence belongs to an
item reserving that record and not to this one. `work/ledger.json` is where work is scheduled,
and nothing here claims either.

**What this amendment does not do.** It does not reopen `OD-ROADMAP-001`'s override, widen it,
or read the workflow tier into it. It does not re-measure version 1's `Question`, which is
dated history about why this record was written. It does not schedule the workflow tier's next
increment or name what it should be -- a fired condition makes one available for an item of its
own, and that item is not authored here.

## Status

Accepted at version 2. Version 1 re-checked `OD-WORKFLOW-002`'s third named condition against
the live tree after the first real `AgentExecutor` arrived, found it fired only at the letter
of its wording, and sharpened the condition's text rather than declaring victory. That finding
was true at the revision it measured and stopped being true at `fe0fac58`, where the Claude
Code executor began returning a real `WorkResult` under `OD-EXECUTOR-008`; `OD-EXECUTOR-011`
later made that result say which of its absences it carries, and the result reaches a workflow
step's outcome through `AgentDispatchOutcome::Executed`. Condition 3 is fired. The protocol has
one real instance and three of `WF-006`'s four named kinds still have none, so what is
established is that the protocol is instantiated, not that it is shared. Conditions 1 and 2 are
open questions this record does not answer.

---
id: ARC-HARNESS-001
type: architecture
title: An autonomous harness is owned per component, and the seam falls between what should run and whether it can run now
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - ecosystem
  - ownership
  - harness
  - execution
relations:
  - target: ARC-ECOSYSTEM-001
    type: affects
  - target: OD-AGENT-001
    type: relates-to
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-CAPABILITY-003
    type: relates-to
  - target: OD-PLATFORM-001
    type: relates-to
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
---

# An autonomous harness is owned per component, and the seam falls between what should run and whether it can run now

## Question

A design has accumulated for an autonomous engineering harness: a goal above the board, a
typed work graph, a deterministic scheduler, a context builder, an execution supervisor, a
model and effort selector, budget accounting, run history, and a workspace materialization
step that installs an agent's integration files for the length of a run and removes them
afterwards. It was proposed as `crates/autonomous/nomos-*`, packaged as a Nomos capability,
with agent runtimes underneath as adapters.

`ARC-ECOSYSTEM-001` decides ownership by semantics rather than by location, and it does not
answer this. It answers the four products and the two crossings it drew. A harness is not
one subsystem, and asking which of the four owns *it* produces an argument rather than an
answer, because the honest answer differs component by component.

The cost of leaving it open is the cost that record names for itself: work accumulated across
an unstated seam has to be re-litigated. The design had already reached crate names, and a
crate name is the placement made rather than proposed.

## The Sentence That Decides It

> **The scheduler decides what should run. Coordination decides whether it can run now. The
> executor decides how to perform the assigned step.**

Three questions, and each has a different owner because each has a different subject.

*What should run* is a question about a codebase. It cannot be asked without one: its
answer depends on which rules bind, which obligations are unmet, and what a change is
supposed to achieve. `ARC-ECOSYSTEM-001` grants Nomos "change planning and the validation
that execution satisfied it", and that sentence is this question.

*Whether it can run now* is a question about resources. Territory, leases, concurrency,
whether a machine has the executable. Nothing in it mentions software engineering, and the
same machinery would be correct under a product that had nothing to do with code.

*How to perform the step* is a question for whatever performs it — a model, a compiler, a
test runner, a person. It is the executor's, and `OD-AGENT-001`'s reasoning applies to it
directly: an executor that also decides the process is deciding through the least reviewed
surface in the system.

## Ownership, Component By Component

| Component | Owner | Why the semantics require it |
|---|---|---|
| Goal, design and the work graph | Nomos | The node kinds are engineering operations and the edges are rule barriers. A graph whose edges are rule validations cannot be defined without a codebase. |
| Convergence validation | Nomos | "The validation that execution satisfied it" is Nomos's clause in `ARC-ECOSYSTEM-001`, and a goal satisfied is a claim about a codebase. |
| Readiness, ordering and dispatch mechanics | XVPE | Computing an eligible set, breaking ties and dispatching a compatible wave is a task scheduler, which is that record's first named example of a generic primitive. |
| Execution supervision | XVPE | A stalled child, an undrained pipe and a process tree are "a process substrate". Nothing about a hung subprocess is about software engineering. |
| Selection among admissible executors | XVPE mechanism, Nomos requirement | The optimization is generic. What is being optimized for — which quality floor this step needs and what a failure here costs — is engineering. `OD-CAPABILITY-001` already drew this line once: "the registry ranks; the caller spends". |
| Context construction | Nomos content, XVPE transport | Which architecture, rules, facts and findings bear on a step is Nomos's subject matter. Budgeting, deduplication and delivery are not. |
| Handoff across a session boundary | XVPE mechanism, Nomos content and its bound | Persisting a run's state and restoring it into a successor is a continuation primitive; nothing in it is about software engineering, and the same machinery would be correct under a product that had nothing to do with code. What must survive so the successor continues *this* work — the item it holds, the authorities that item's territory actually reaches, and what has already been refused — is engineering. This row differs from context construction in the one way that matters: context is assembled for a step, and a handoff crosses a session, so the successor cannot be assumed to have read anything. That is exactly why the content has a **bound** and not only an owner. `OD-AGENT-001` refuses a handoff that restates the contract, because a document written once and read every session afterwards is how an undecided architecture becomes normative through the least reviewed surface in the system. |
| Budget accounting | XVPE mechanism, Nomos policy | Counting tokens, cost and latency is telemetry. Deciding what a feature is worth spending is an engineering judgment about consequences. |
| Run history | XVPE store, Nomos content | A storage primitive holds it; what is stored — that this step on this subject was verified by this rule — is a fact about a codebase. |
| Workspace materialization | XVPE | Staging, backup, journalling, atomic replacement, rollback and recovery are a package platform. `OD-PACKAGE-001` records that this build has no package at all, so nothing here extends an existing family. |
| The harness as it exists in this repository today | Repository and bootstrap tooling | `AGENTS.md`, `CLAUDE.md`, the skills and the board were written to get this built. Nothing in the list above is built. |

Several entries deliberately say two owners. That is not indecision. It is the same shape
`ARC-ECOSYSTEM-001` used for the crossings it drew: a generic primitive is consumed by
adaptation, and the adapter carries the meaning while the primitive carries the mechanism.
A selector that learns what a rule barrier is in order to rank executors is a primitive that
has been taught software engineering, which that record forbids by name.

The handoff row was added at version 2. Version 1 answered for nine of the ten components
`P11-HARNESS-SEAM` required and said nothing about who owns handoff — while saying something
about handoff below, under what the record does *not* decide, which is a deferral of the
mechanism and was never the ownership answer. A reader looking for the owner found a sentence
answering a different question and stopped, which is precisely the failure this table exists to
prevent for every other component. Nothing mechanical caught it, and the reason is worth
keeping: the item's predicate is a test suite, and a test suite cannot read a `done_when`.

## What This Record Does Not Decide

It decides ownership. It decides no mechanism, and several mechanisms are open items held by
other territories. Reading an answer to any of them out of this record would be reading a
second authority into it.

Where the choice of next work is computed, and whether the board computes it at all, belongs
to `P11-NEXT-WORK`. How engineering readiness is told apart from dispatchability belongs to
`P11-DISPATCH-SPLIT`. How a stall is distinguished from a long run belongs to
`P11-EXEC-IDLE`. What a handoff may *carry* — its content, not its owner, which the table
above now names — belongs to `P11-AGENT-CONTINUATION`. Whether a
named choice may be required rather than preferred belongs to `P11-PREFERENCE-STRENGTH`.
Whether a run's observations may become KWB knowledge belongs to `P11-ECOSYSTEM-UPWARD`, and
this record does not draw that arrow.

## Nothing Here Is Built, And That Is The Present Answer

The table above is a rule for placements not yet made. It is not a description of this
repository, and the distinction matters enough to state rather than imply.

What exists today is bootstrap. The ledger coordinates sessions; the contract file routes an
agent to authority; two skills carry procedures; the contract tests preserve the workspace.
`ARC-ECOSYSTEM-001` already names the ledger and the contract tests as things whose address
is not their ownership, and this record adds the rest of the harness to that list rather than
removing any of it.

So the answer to "is the harness a product surface" is **not yet, and not by accumulation**.
A component becomes one when something is built to the description in the table, under the
owner the table names. Until then the correct reading of every file in this repository that
looks like harness machinery is the sentence `ARC-ECOSYSTEM-001` wrote for exactly this: what
was written to get Nomos built is not thereby part of Nomos.

What would change it, stated so that a later reader does not have to guess: a component
leaves bootstrap when it is required by something other than the development of this
repository. Not when it is good, not when it is reusable, and not when a second project
copies it — `ARC-ECOSYSTEM-001`'s anti-drift clause already refuses reuse as a criterion,
and this is that clause applied to the harness.

## Why Location Will Argue Against This

The prediction is worth writing down because the argument will be made, and it will be made
by someone reading the tree rather than this record.

Every component in the table that gets built first will be built here, in a crate named
`nomos-something`, because that is where the work is and where the tests are. `P11-EXEC-IDLE`
will land wall and idle bounds in `nomos-platform`. A selector, if one is written, will be
written beside the registry it consults. Each of those is a bootstrap placement and none of
them is evidence.

`ARC-ECOSYSTEM-001` supplies the rule that disposes of the argument in advance — current
repository location alone does not make something Nomos — and this record is the reason it
will need to be cited: a harness is the case most likely to be argued from its address,
because its address is the only thing about it that exists.

## Alternatives Considered

**One owner for the whole harness.** Rejected because the components do not share a subject.
Whichever owner is chosen, at least three entries in the table are wrong under it, and a
record that is wrong in three places is cited for the two it got right.

**Defer until something is built.** Rejected on `ARC-ECOSYSTEM-001`'s own reasoning: the
longer work accumulates across an unstated seam, the more of it is re-litigated. The design
reached crate names before this record existed, which is the deferral already failing once.

**Decide it in `AGENTS.md`.** Refused, and this is the second time. `OD-AGENT-001` refused to
put the ecosystem boundary in an instruction file because such a file is read by a machine
every session and reviewed by a person approximately never, which promotes an undecided
architecture to normative status through the least reviewed surface in the repository.
`ARC-ECOSYSTEM-001` exists because of that refusal, and this record exists because that
record left a subsystem it did not reach.

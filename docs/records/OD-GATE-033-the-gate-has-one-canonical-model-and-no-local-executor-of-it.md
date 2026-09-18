---
id: OD-GATE-033
type: decision
title: The gate has one canonical model and no local executor of it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - enforcement
  - availability
relations:
  - target: OD-GATE-027
    type: relates-to
  - target: OD-GATE-006
    type: affects
  - target: OD-GATE-011
    type: relates-to
  - target: OD-GATE-004
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
---

# The gate has one canonical model and no local executor of it

## Question

The gate's authoritative executor is GitHub Actions, and it is currently unavailable —
`OD-GATE-027` measured that and decided what a red check means while it is true. That record's
own Recovery section assumes two things this repository does not have, and this record exists
to decide them.

What is the canonical model of the gate's step set, such that GitHub Actions and a local
executor both project from it rather than one restating the other? And what does a local
execution mean for each step whose participation is host-conditional — including the steps no
local run may claim at all?

## What Was Measured

Taken from `.github/workflows/gate.yml` at this revision, and from a search of `crates/` and
`tests/` for anything that reads it.

The gate is one job on a two-entry matrix (`ubuntu-latest`, `windows-latest`,
`fail-fast: false`) and declares **fifteen** steps. Which legs a step reaches is decided per
step by an `if: matrix.os == '…'` guard, or by no guard at all.

| Step | Guard | Legs | Local caller today |
|---|---|---|---|
| Show toolchain | none | both | none |
| Install cargo-deny | none | both | none |
| Lint | `ubuntu-latest` | Linux | `work finish` |
| Test | `ubuntu-latest` | Linux | none |
| Install compatibility floor toolchain | `ubuntu-latest` | Linux | none |
| Compatibility floor | `ubuntu-latest` | Linux | none |
| Determinism | `windows-latest` | Windows | none |
| Install freestanding targets | `ubuntu-latest` | Linux | none |
| Portability floor | `ubuntu-latest` | Linux | none |
| Boundaries | none | both | none |
| XVPE crossing (cold by construction) | `ubuntu-latest` | Linux | none |
| Required projections | none | both | none |
| Rules | `ubuntu-latest` | Linux | none |
| Supply chain | `ubuntu-latest` | Linux | none |
| Corpus gates (reports what did not run) | `ubuntu-latest` | Linux | none |

Four steps are unguarded and run on both legs. Ten are Linux-only. One, `Determinism`, is
Windows-only. Exactly **one step has a local caller**, and it is `Lint`:
`crates/substrate/nomos-ledger/src/gate_unknown.rs` names `LINT_STEP` and
`nomos_ledger::Derive_Step` reads the workflow to produce its argv, so `work finish` runs the
gate's lint step before an item's predicate.

**What already reads the file, and why that matters.** The derivation is not the only reader,
and none of them restates what they read. `Derive_Step` and `Workflow_Path` parse it;
`crates/host/nomos-cli/tests/gate_step/lint_step.rs` asserts the derivation against the real
file; `step_order.rs` asserts every `cargo install` precedes the first `cargo test`;
`excuses.rs` refuses `continue-on-error` by name; `pinning.rs` requires every `uses:` to be a
commit; and `tests/contract/tests/agent_harness/readers.rs` derives the gate's own command
list from its `run:` lines. The module doc states the reason in as many words: a copy of the
lint command in that crate "is a second source of truth that goes stale the day the workflow
changes, and two guards for one rule is how they come to disagree."

**The gap this record is about.** Nothing reads a guard. A search of `crates/` and `tests/`
for `matrix.os`, and for any prefix `if:`, returns nothing. `Find_Run` moves between steps on
`- name:` and `name:` and returns the first `run:` inside the one it wants, ignoring every
other key — the workflow says so itself at the `Lint` step, and calls that deliberate for
`work finish`'s purpose.

The consequence is exact. An executor projected from the workflow by the existing derivation
would treat `Determinism` as an ordinary step, because the only thing distinguishing it is the
key that reader discards. On a Linux host it would either run a Windows-only step's command or
skip it, and a skip reported as a clean result is the defect the `Portability floor` step
already refuses for its own subject: that step fails when its derived crate list is empty
because "a gate that cannot find its subject must say so rather than report a clean result
over nothing."

`GateUnknown` already carries the refusal vocabulary for the other half — `Unreadable`,
`NoSuchStep` and `NotASingleCommand` — and its doc states the principle this record extends to
guards: "a guessed predicate is the defect this module exists to close," and "unknown gate
coverage is not gate coverage."

## Decision

### 1. The canonical model is `.github/workflows/gate.yml` itself

The workflow file is the model. GitHub Actions and a local executor both project from it, and
neither holds a copy of the step set.

The reason is not that the file is convenient. It is that the file is the only artifact that
is simultaneously what the authoritative executor runs and what every local reader already
parses. Actions executes this file and nothing else, so any other model would be authoritative
about something nobody executes — a declaration whose divergence from the file that runs would
be invisible. And the readers above already treat it as the authority in fact, each of them
written specifically so that it cannot disagree with the file.

A declaration the workflow were rendered from is refused for two reasons, either of which is
sufficient. It puts a second artifact between the policy and the executor, so the gate could be
green while the declaration and the executed file disagreed — `OD-GATE-011`'s defect at the
gate's own root. And the rendered workflow would be a projection nobody owns, which is the
state `OD-GATE-005` records for `diagrams/relations.mmd` and `spec/domain-specification.md`,
and which already fails the gate when it goes stale.

**What this does not claim.** The file is YAML in the Actions dialect, its `run:` bodies are
shell, and its guard vocabulary is GitHub's expression language rather than this workspace's.
The model is the file; the *reading* of it is a closed subset that this workspace implements
and refuses outside of. That subset is currently `name:`/`run:` for one named step, and
decision 2 states what it must grow to.

### 2. A local execution gives every step one of three values

`Executed(exit)` — the step's guard admits this host, and its `run:` derived, so it ran here
and this is what it exited with.

`Unavailable(host)` — the step's guard does not admit this host, so this execution produced no
evidence about it. Reported as unavailable. Never as a pass, and never as a skip whose absence
is unmentioned.

`Refused(cause)` — the guard, or the `run:` body, uses a form outside the subset. `GateUnknown`
already names the three causes for the `run:` half; the guard half needs the same treatment,
and a guard the executor does not implement must refuse the step rather than guess which leg it
selects.

The forbidden fourth value is a subset reported as clean. It is `OD-GATE-001` and
`OD-GATE-020` one level up, and the file's own precedent for refusing it is the `Portability
floor` step.

### 3. Enumerated against this revision

The four unguarded steps — `Show toolchain`, `Install cargo-deny`, `Boundaries`, `Required
projections` — execute on either host. The ten Linux-only steps execute on a Linux local host
and are `Unavailable(Linux)` on Windows. `Determinism` executes on a Windows host and is
`Unavailable(Windows)` on Linux.

**A local run reports the set it did not execute.** The precedent is the `Corpus gates` step,
which exists for exactly this reason: cargo swallows a passing test's output, so the absence of
three corpora from a runner would otherwise be invisible, and `--nocapture` is what makes it
appear. A local run's unavailable set is the same kind of hole and gets the same treatment.

**A local run cannot substitute for the second leg, and must say so.** `Boundaries` and
`Required projections` run on both legs because their subjects vary by host — directory
iteration order, filename case, path separators, and for `Required projections` a content hash
recomputed over the rendered bytes. One host's local run answers that host's question. The
Windows-only step cannot be answered from Linux at all. An executor that reported one clean
result for both legs would be claiming the second leg on evidence it never took.

### 4. Gate availability is a repository-health state, reported from evidence

Availability is a state distinct from pass and fail, and it borrows the vocabulary this
workspace already owns rather than declaring a parallel one — `Applicability` separates
`MissingCapability` and `ProviderUnavailable` from a judgment, and
`SynchronizationState::Unavailable` records that unavailable is neither agreement nor
divergence. `OD-GATE-027` made that decision; this record supplies the mechanism, and does not
restate the decision.

**The surface is a gate execution report.** One report describes one execution: the step set it
projected from, the source revision, the host, each step's value from decision 2, and when it
ran. Availability is then computed rather than fetched. A step a report covers is available on
that report's evidence; a step no report covers is `Unavailable`, and an execution carrying any
`Refused` step is not evidence about that step.

What a reader sees is therefore per step and never a single bit: `executed(0)` and
`unavailable(no execution since <revision>)` are different lines, where today both are one red
check. That difference is the whole content of `OD-GATE-027`, and nothing reports it. The
surface must be reachable from the command line rather than present only in prose — a state
that can be read only by someone who already knows it exists is the shape `OD-GATE-001`
refuses.

**Deliberately not by reading GitHub's run history.** Availability computed from an API would
depend on the network and on a settings page — `OD-GATE-027` names a billing matter as the
cause — so the state could not be computed in the tree that needs it, which is the tree a
session is working in. Availability computed from evidence in hand is computable everywhere,
and it answers the question that matters: not "is the scheduler up" but "what has actually
been executed, and what has not".

A fresh clone with no report genuinely has no gate evidence, and reads `Unavailable` for every
step. That is the correct answer rather than a defect: the alternative is the corpora situation
`tests/contract/` already declares, where a test that cannot find its corpus returns early and
prints `ok`.

`OD-GATE-027`'s retirement is unchanged and is not re-decided here. The first successful hosted
run ends the state by observation rather than by decision.

### 5. What this record authorizes, and what it does not do

It authorizes a second executor that projects from the model in decision 1, produces the values
in decision 2 for every step in decision 3, and reports the surface in decision 4 — filed as
its own item with its own territory, because it touches the CLI and the ledger rather than
these records.

It does not build that executor. It does not narrow the gap `LINT_STEP` records: a per-item
predicate stays authored per item, because a scoped test is what such a predicate is for —
running the whole workspace on every finish costs minutes, and a finish that costs minutes is
one that gets skipped.

## What This Does And Does Not Invalidate

Nothing here weakens `OD-GATE-027`'s position, and the two errors it separates stay separate.
Every commit in the unavailability window carries the verification its own item declared, and
that evidence really was produced. What remains unverified is remote full-gate execution
evidence, and this record adds the second thing it was missing: a way for the steps that
predicate never covered — `Determinism` above all, which no item predicate on this board runs —
to be seen as uncovered rather than assumed covered.

Replay is not owed by default, for the reason `OD-GATE-027` gives. Re-running a week of history
to produce evidence nobody asked for is work this repository declines elsewhere for the same
reason.

## Alternatives Considered

**A declaration the workflow is rendered from.** Rejected in decision 1: it inserts a second
artifact between the policy and the executor, making a green gate compatible with the two
disagreeing, and the rendered file would be a projection nobody owns.

**A shell script that reimplements the steps locally.** Rejected. The second copy is precisely
what `OD-GATE-004` and `OD-GATE-011` refuse, and it would drift from the file `work finish`
already derives from — which is why this item is a decision rather than a script.

**Deriving a local argv for every step by reusing `Derive_Step`.** Rejected: it ignores the
guard by construction, so it would run `Determinism` on Linux or drop it silently. Deriving an
argv and deciding whether that argv should run here are two questions, and only the first
exists today.

**Treating a locally-green run as evidence the hosted gate passed.** Rejected as
`OD-GATE-027`'s error in the other direction. One host cannot answer the other leg's question,
and `Determinism` cannot run on the wrong one at all.

**Reading GitHub's run history to compute availability.** Rejected in decision 4: it makes
repository health depend on a network API and on a settings page, so the state could not be
computed where it is needed.

**Reporting a host-unavailable subset as clean.** Rejected: the `Portability floor` precedent,
and `OD-GATE-001`'s defect with one more layer between the reader and the absence.

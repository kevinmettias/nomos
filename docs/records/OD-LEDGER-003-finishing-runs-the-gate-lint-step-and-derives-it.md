---
id: OD-LEDGER-003
type: decision
title: Finishing runs the gate's lint and rules steps, derived from the gate, and the test step stays scoped
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - work-ledger
  - verification
  - gate
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
  - target: OD-GATE-036
    type: relates-to
---

# Finishing runs the gate's lint and rules steps, derived from the gate, and the test step stays scoped

## Question

`work finish` runs an item's verification predicate and records the item `Done` if it exits
zero. Every item's predicate is a `cargo test` invocation.

The gate is not a `cargo test` invocation. It lints first —
`cargo clippy --workspace --all-targets -- -D warnings` — over a workspace that *denies*
`unwrap_used`, `indexing_slicing` and `arithmetic_side_effects`, because a panic on the
analysis path is a determinism defect rather than a bug: a replay must reach the same panic
at the same step.

So an item could be recorded verified while the gate that follows it was already red, and
the ledger would say the work was checked.

## What Actually Happened

`P9-SKIP` and `P9-PHASE-GAP` were both finished with a clippy error in a file the item had
just written, and both were caught afterwards by a person running clippy by hand. That is
the arrangement the ledger exists to stop relying on, and it is the sentence `OD-LEDGER-001`
already had to write once about territory.

The evidence is not where one would expect it, and the difference matters enough to record.
**Clippy passes on both commits.** Measured at `e121777` and `4c0fd3b` in detached
worktrees, with a confirmed-red control — an injected `values[0]` produced
`error: indexing may panic`, exit 101 — so the measurement was live rather than vacuously
green.

The defect existed only in the working tree, in the window between `work finish` and
`git commit`: 180 seconds for `P9-SKIP`, 185 for `P9-PHASE-GAP`, measured from each item's
recorded `verified_at` against the commit timestamp. It was repaired before the commit was
written.

That makes the case *stronger*, not weaker. A red commit would mean the gate caught it. A
clean commit with a three-minute window means a person caught it, twice, and the ledger
recorded `Done` both times without knowing. It also means the two instances cannot be
replayed from git — a future author must not go looking for a red commit that does not
exist. They are reproduced in `gate_covers_finish.rs` in the shape they actually had: a
predicate that exits zero while the gate's own step does not.

## Decision

**Finishing runs the gate's lint step before the item's predicate, and the step is derived
from the gate rather than written down beside it.**

That is the decision as version 1 made it. Version 2 runs the gate's `Rules` step after the
lint step and before the predicate, under the same three parts below; the amendment at the end
of this record says what changed and what did not.

Three parts, each load-bearing.

**1. Derived, never copied.** `nomos-ledger` reads `.github/workflows/gate.yml` and takes
the `run:` line of the step named `Lint`. A copy of that command inside the crate would be
a second source of truth that goes stale the day the workflow changes, and two guards for
one rule is how they come to disagree. A test asserts that altering the workflow alters
what `finish` runs, which is what fails if somebody later reintroduces the constant.

**2. An underivable gate refuses.** A missing, unreadable, or scripted `run:` line yields
`FinishRefusal::GateUndetermined`, and nothing is run. Deriving an argv from
`cargo clippy && cargo doc` would mean guessing, and a guessed predicate is worse than a
refused one because it looks like it worked. This is the rule this crate already applies one
level up in `ClaimRefusal::UnknownIndependence`: an unanswered question refuses rather than
grants. `GateUndetermined` answers `Judged_The_Work()` false — nobody found out whether the
work passes, and reporting that as failing work would send an author to fix working code.

**3. The lint runs first and short-circuits.** An author told "your tests passed" and "you
cannot land" in one breath reads only the first sentence. A red gate is
`FinishRefusal::GateFailed`, separate from `PredicateFailed` because the remedy differs: a
failing predicate says the work does not do what the item asked, a failing gate says the
work may be exactly right and still cannot land.

The result is recorded. `VerificationRecord` carries a `gate` field holding the derived argv
and its exit code. Records written before this decision carry `null` there, and are left
that way rather than backfilled — otherwise a reader cannot tell an item finished under the
gate from one finished before the gate was part of finishing.

## What Stays Weaker, And Why

**The gate's test step is not derived.** It runs `cargo test --workspace`; an item's
predicate stays the scoped `cargo test -p …` its author wrote.

This is deliberate, and it is the part a reader should be suspicious of, so it is stated
rather than left to be discovered. A scoped test is what a per-item predicate is *for*.
Running the whole workspace on every finish costs minutes, and the failure mode of a finish
that costs minutes is not a slow ledger — it is an author who stops running `work finish`
and marks the item done another way, which is the failure this system exists to prevent.

So the remaining hole is real: an item can still be finished while a test elsewhere in the
workspace is red. What closes it is the gate itself, at push time, where a full workspace
run belongs. What this decision closes is the narrower and more damaging case — the item
that is recorded `Done` having never been linted at all, which is the case that actually
occurred, twice, in three items.

## Consequences

`work finish` now needs a readable gate workflow. A tree without one cannot finish an item,
which is correct: an item finished in a tree with no gate has been checked against nothing
in particular.

Every test that builds a ledger in a temporary directory now builds a gate beside it. Those
fixtures lint with `cargo --version` rather than the real clippy invocation, because those
tests are about what a predicate's exit code does to an item; what the derived step actually
is, and that it comes from the workflow, is covered separately.

`OD-GATE-001` remains the authority on the corpus step reporting what did not run. This
decides only what `finish` is obliged to run before it writes `Done`.

## Amendment, Version 2: Finishing Also Runs The Gate's Rules Step

Added at version 2. `OD-GATE-036` decided it, in its first part, on five measured landings, two
of which passed their own predicate and left a Blocking finding at `HEAD` for hours.
`P195-WORK-FINISH-RUNS-THE-GATES-RULES-STEP-AFTER-ITS-LINT-STEP` built it.

**What is superseded, and it is one clause.** "Finishing runs the gate's lint step before the
item's predicate" now reads: finishing runs the gate's `Lint` step and then its `Rules` step
before the item's predicate. The other clause of the title, that the test step stays scoped,
stands unchanged, and `OD-GATE-036`'s 7,202.9 s run of the whole workspace is the measurement
that keeps it standing.

**The three parts, carried to the second step.**

- **Derived, never copied.** `nomos-ledger` names the step `RULES_STEP` beside `LINT_STEP` and
  takes both from one reading of the workflow through the same `Derive_Step`. A test rewrites a
  fixture workflow's `Rules` line and finds the new command in what the finish ran and in what it
  recorded; writing the command into the crate as a constant fails it.
- **An underivable step refuses.** A scripted `Rules` step, or one declared with no `run:` line,
  is `GateUndetermined`. Both steps are derived before either runs, so the refusal comes before
  the lint step has spent its minutes and nothing runs at all.
- **The first failure short-circuits.** The order is `Lint`, `Rules`, predicate. Any nonzero
  exit of the `Rules` step, whether 1 for a Blocking finding, 5 for a run that could not be
  assembled or 6 for a run that judged nothing, is `GateFailed`, carrying the step's argv and the
  tail of what it printed, and it judged the work exactly as a red lint step does.

**Where the second step differs from the first, which is absence.** A workflow that declares no
`Rules` step is not undetermined. It has made no claim for a finish to honour, and the same
ledger serves KWB, whose gate declares `Lint`, `Test` and `Contract`, so the finish goes ahead
and records the step as not declared. That required absence to mean only absence: `Derive_Step`
had answered `NoSuchStep` both for a step the workflow never declares and for one declared with
no `run:` line, and the second would have been recorded as absent while it sat in the file
unrun. It now answers `NoSuchStep` only for the first, and gives the second the
`NotASingleCommand` refusal the whole-set reader `Derive_Steps` already gave it. For the lint
step both causes refuse, so nothing a finish did before this version changes.

**Bounded by the item.** The `Rules` step runs under the same `Runner` as the lint step: the
item's own timeout as the wall bound, half of it as the idle bound, in the tree being finished.
`gate run` prints nothing until it is done, so an item whose timeout is less than twice the
step's running time is cut off at the idle bound; `OD-GATE-036` counts the predicates on the
board that are exposed to it.

**Recorded, never backfilled.** `VerificationRecord` carries `rules` beside `gate`: `Ran`, with
the argv and the exit code, or `NotDeclared`. Every record written before this version holds
no such key and is read as `None`, which is left as it is for the reason `gate` was: a reader
must be able to tell a finish that ran the step from one that predates it, and both from a
finish in a workflow that never declared it. The ledger schema version moves to 7, and a binary
copied before this version refuses a board carrying the new key, once, by `OD-LEDGER-008`'s
guard, which `OD-GATE-036` accepted.

**What it costs, measured.** On 2026-10-03, in a detached worktree of this change with its own
warm target directory, under the board's ordinary load (total CPU at 24 to 39 percent and about
6.9 GB available without these runs), a finish of a scratch item whose predicate is
`cargo --version` took 7.8, 10.9 and 2.1 s with the binary from before this change, which runs
the lint step only, and 78.4, 74.7 and 64.3 s with this change's binary. The `Rules` step adds
about a minute to a warm finish here, inside the 52.6 s quiet and 210.5 s loaded range
`OD-GATE-036` measured for the step alone.

**What it catches, measured.** In a scratch copy of this change's tree with `be59c109`'s helper
name reintroduced, the three spellings of `Assert_Well_Formed_Graph_Document` renamed back to
`Assert_Well_Formed_GraphML` as `OD-GATE-036` did at `0000e0e3`, a finish of a scratch item
exited 1 and left the item claimed with no record: the lint step passed, the `Rules` step
exited 1, and the predicate did not run. It took 239.3 s with the copy's target directory cold
and 73.3 s warm. The refusal named the step's command and carried the tail of its output, and
that tail says `476 finding(s), 1 of which can fail a build` but not the finding's own line.
`cargo run` replays the workspace's cached compiler warnings on stderr, after the run's findings
on stdout, and the Blocking line is not the last finding `gate run` prints, so it fell outside
the 2,000 bytes a refusal keeps; `gate run` over the same tree names it, 472nd of 479 lines.
In a scratch tree whose workflow declares `Lint`, `Test` and `Contract`, a finish exited 0 and
its record carries `"rules": "NotDeclared"`. In one whose `Rules` step is an action, a finish
refused as undetermined, exit 4, having run nothing.

**What stands.** The three parts for the lint step, the `gate` field and its rule against
backfill, and "What Stays Weaker, And Why". The hole that section describes, a test red
elsewhere in the workspace, is not closed by this version: `OD-GATE-036`'s second part narrows
it with a rule on how predicates are authored, and nothing a finish runs.

**Consequences.** A finish now judges the tree it runs in with the rule layer, so in a shared
tree a peer's uncommitted Blocking finding refuses an unrelated finish, as a peer's clippy error
already did; finishing from a worktree at the commit to be published is the remedy
`OD-GATE-036` names. The fixtures that build a ledger with a gate beside it still lint with
`cargo --version`, and they declare no `Rules` step, so their finishes record it as not
declared. The cases that run the step are against fixture workflows that declare it, beside a
test that this repository's own workflow still yields a `Rules` argv.

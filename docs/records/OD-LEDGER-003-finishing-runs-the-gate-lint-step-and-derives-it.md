---
id: OD-LEDGER-003
type: decision
title: Finishing runs the gate's lint step, derived from the gate, and the test step stays scoped
status: accepted
version: 1
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
---

# Finishing runs the gate's lint step, derived from the gate, and the test step stays scoped

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
**Clippy passes on both commits.** Measured at `26566c6` and `31f4450` in detached
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

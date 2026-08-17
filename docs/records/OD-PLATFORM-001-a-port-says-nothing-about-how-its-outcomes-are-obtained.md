---
id: OD-PLATFORM-001
type: decision
title: A port that names its outcomes says nothing about how they are obtained
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - platform
  - verification
  - work-ledger
relations:
  - target: OD-GATE-001
    type: relates-to
  - target: D-130
    type: relates-to
---

# A port that names its outcomes says nothing about how they are obtained

## Question

`ExitOutcome` is three variants and not an exit code beside a bool, and its doc comment
says why: a process killed for exceeding its timeout has not failed its predicate, nobody
found out whether the predicate holds, and reporting that as a non-zero exit would turn
"we did not learn anything" into "the check failed". `Produced_A_Verdict` exists to keep
those apart.

The type is right. What decides whether an instance of it is *true* is plumbing the trait
says nothing about, and the one implementation there is got it wrong.

## What Was Measured

`StdProcessLauncher` piped a predicate's stdout and stderr and read neither until the
child had exited. The wait loop called `try_wait`, slept, and never drained.

A pipe is a bounded buffer — 65,536 bytes on this platform, observed directly rather than
assumed, as the exact quantity captured from a child that then wedged. A child that fills
it blocks in `write`. It never exits, so `try_wait` never reports, so the loop runs to the
timeout and kills it. Only then does anything read.

Same program, same 60-second budget, only the volume changed: 840,000 bytes emitted in
about a second timed out after 60 seconds; 4,000 bytes emitted the same way verified in
zero. Reproduced here at 660,000 bytes under a 10-second budget — `TimedOut`, with 65,536
bytes captured out of 660,000.

It was met for real finishing `P10-FIRST-CHECK`, whose predicate is `cargo test
--workspace`: 75,751 bytes of output, 103 seconds when run by hand, and `TimedOut` twice
at 600 seconds under `work finish` with a fully warm cache.

## Why This Is The Wrong Direction To Fail In

`P9-PREDICATE` separated `GateFailed` from `PredicateFailed` because a failing gate and
failing work are different facts, and added `NoVerdict` so that "nobody found out" could
not read as bad work. This defect inverts that carefully drawn line: a run that *did* find
out, and found nothing wrong, is written down as a run nobody got an answer from.

The failing direction is worse still. Failure detail is exactly what makes output large,
so the louder the failure the likelier it is recorded as a timeout rather than as a
failure — the defect is anti-correlated with the runs whose answers matter most.

That it survived four phases is a property of the ledger rather than of the launcher.
Every earlier item scoped its predicate to one or two crates and stayed under the buffer;
`P10-ABANDON-REASON`'s predicate measures 13,852 bytes today, comfortably under. The first
predicate to name the whole workspace was the first to exceed it.

## The Decision

**Both streams are read from the moment the child starts.** A thread per stream, appending
into a buffer shared with the caller. A child cannot finish writing more than a pipeful
unless somebody is taking it away, so this is what makes the wait a wait on the program
rather than a wait on its volume.

**The wait after the child ends is bounded, and is not a join.** This is a second defect,
found while fixing the first rather than reported with it. Once the child has ended its
own ends of the pipes are closed and end of file arrives at once — unless the child handed
a pipe to a background process before exiting, in which case that grandchild holds it open
and reading to end of file waits for a process that was never the one being judged. The
old code did exactly that after killing on timeout, and a test written for it measured
**29 seconds** spent waiting on a grandchild by a launcher whose child had exited
immediately. A thread joined without a bound would have reproduced it precisely.

The trade is stated rather than glossed: within `DRAIN_GRACE` a grandchild's trailing
output can be cut off. What is bought is that the verdict of the process actually under
test is never held hostage to one. A launcher that hangs is a `work finish` that never
returns, which is the same "nobody found out" one level further out.

**Captured output is decoded lossily.** `read_to_string` refuses the entire stream on one
byte that is not UTF-8 and the caller discarded the error, so a predicate printing a stray
byte had the whole of its output recorded as nothing at all — the same shape of loss, at a
smaller scale, as the one this record is about.

## What The Port Still Does Not Say

`ProcessLauncher` is a one-method trait whose doc comment carefully distinguishes "we
could not ask" from "we asked and the answer was no". It says nothing about draining,
because there is nothing in a signature that could say it. The requirement that makes its
outcomes true now lives in the tests of one implementation.

`D-130` defers `nomos-platform-xvpe` until Phase 5. When a second implementation of this
port arrives it will be free to reintroduce this defect exactly, and nothing outside
`nomos-platform-std`'s own test module would notice. Writing the guard as a shared
conformance suite over the trait — rather than as tests against one struct — is the thing
that would close this properly, and it is not built here: there is one implementation
today, and a conformance suite with a single conformer proves nothing it did not assume.

## Consequences

Four tests in `nomos-platform-std`, and each control was watched failing rather than
reasoned about:

- A program emitting 660,000 bytes under a short budget reports `Exited { code: 7 }`. Red
  before the fix, reporting `TimedOut`.
- Its transcript equals what the program wrote, byte for byte. Red before the fix at
  65,536 bytes of 660,000. Weakening the fix to drain the pipe but keep only the first
  65,536 bytes leaves the outcome test green and turns this one red — which is what makes
  the two controls independent rather than one assertion written twice. A launcher that
  kept a prefix would have traded a false timeout for a false transcript, and the tail is
  where a failing run puts its reason.
- A program that genuinely exceeds its budget still reports `TimedOut`. Green before the
  fix and required to stay so, because every other assertion here is satisfied by a
  launcher that simply waits forever. Disabling the timeout turns this one red alone.
- A child that exits while a grandchild holds the pipe does not hold the launcher.
  Restoring the unbounded wait turns this one red alone, at 29.4 seconds.

`P10-FIRST-CHECK`'s predicate drops the `-q` it carried only to fit the buffer. Its
recorded `output_tail` was a run of empty doctest sections rather than the real counts —
evidence that fitted rather than evidence that told anyone anything.

## Amendment: One Non-Verdict Was Two Facts Wearing The Same Name

`TimedOut` answered "nobody found out" for two different reasons that call for different
remedies: a predicate genuinely still working when the wall bound expired, and a predicate
that produced nothing at all while something else — a hung reader, a deadlocked test case,
a child blocked writing into a pipe nobody drained — kept it from ever finishing. `wait` and
`fix the reader` are not the same action, and a value that cannot say which was needed sends
every holder to the same one: raise the timeout. `P11-EXEC-IDLE` is the item that measured
this against the board, in `P10-LAUNCHER-PIPE`'s own history: a predicate that passes in 103
seconds by hand was recorded as `TimedOut` twice at 600 seconds because the launcher read no
pipe until the child exited. The drain was fixed by this record's original decision; the
classification was not.

**A command now carries two bounds.** `Command::timeout` is unchanged — the wall bound,
regardless of whether the process is producing anything. `Command::idle_timeout` is new:
how long the process may go without producing any new output before it is judged to have
stalled. `Command::New` starts the two equal, so a caller that never asks for the
distinction gets exactly the wait it asked for before — the two bounds expire together and
the process is judged once, at the wall bound, exactly as `TimedOut` always meant.
`Command::With_Idle_Timeout` is what gives a caller a shorter idle bound; without it, the
idle bound cannot fire ahead of the wall bound, because it cannot be shorter than the wall
bound. Progress resets it: every poll compares how much has been captured on either stream
against the last poll, and any increase moves the idle clock back to now.

**`ExitOutcome::Stalled { idle_elapsed }` is new, sitting beside the unchanged `TimedOut`.**
`TimedOut` keeps its exact shape and meaning: the process was still within its idle bound —
producing, or default-configured so the two bounds coincide — when the wall bound expired.
`Stalled` is only reachable when a caller asked for an idle bound shorter than the wall
bound and the process went silent for the whole of it. The two are driven apart by a test in
`nomos-platform-std`: one child that produces nothing under a one-second idle bound and a
thirty-second wall bound reports `Stalled` in well under ten seconds; the control — the same
wall-genuinely-slow program this record's original test already used, now given a five-second
idle bound it never approaches — still reports `TimedOut`, proving the idle bound does not
turn honest, ongoing work into a false stall.

**`ExitOutcome` stays `Copy`, and that bounded what `Stalled` could carry.**
`nomos-ledger`'s own finishing code destructures `output.outcome` by value and, on the
non-`Exited` arm, reads `output.outcome` a second time to build `FinishRefusal::NoVerdict` —
which only compiles because `ExitOutcome` is `Copy`. A variant holding a `String` or a
`Vec<String>` would end that silently, in a file this item's territory does not include and
must not widen into. `Stalled` therefore carries only `idle_elapsed: std::time::Duration` —
itself `Copy` — rather than the output or the argv. That is not evidence discarded: both are
already retained exactly as they were for every other outcome, on `ProcessOutput.stdout` /
`.stderr`, captured up to the moment of the kill regardless of how the wait ended, and the
argv was never the launcher's to lose — it is the caller's own `Command`, held by whoever
already built it. What `Stalled` needed to add was the one fact that lived only inside the
wait loop: how long the silence had gone on. Verified directly rather than assumed: this
distinction survives, unmodified, into what `nomos-ledger` prints for a predicate that never
answered —
`P11-EXEC-IDLE's predicate produced no verdict (Stalled { idle_elapsed: 45s }), so whether
the work is finished is still unknown`, against `... (TimedOut), so ...` for the wall-bound
case — with zero changes to `nomos-ledger`, because `{outcome:?}` was already how
`FinishRefusal::NoVerdict` was described.

**Termination now reaches the process tree, not only the process this launcher started.**
A direct `Child::kill` on Windows is `TerminateProcess` against one handle; it does not
cascade to whatever that process itself started, and does not need to for most predicates —
but a `cargo test` killed this way leaves its compiled test binary running underneath it,
which is the process that was actually hanging. `taskkill /T /F /PID` is used instead: given
a pid it walks the tree rooted there and terminates every process in it. Chosen as a runtime
branch — `if cfg!(windows)`, not `#[cfg(windows)]` — because this workspace's gate lints only
its Linux leg on the measured strength of there being no conditionally-compiled code
anywhere for that leg to miss; a compile-time `#[cfg]` here would have made that no longer
true for the one function that needed platform-specific behavior. `taskkill` failing to run
at all — wrong platform, not on `PATH`, the process already gone — falls back to the direct
`Child::kill`, unchanged from before this amendment. Proven with a test spawning `cmd`
running `ping` as `cmd`'s own child: killing only `cmd` would leave `ping` writing to a
marker file for the whole of its own thirty seconds; this launcher's kill stops the writes
within the same poll the wait loop ends on.

**What this leaves open.** `nomos-ledger` does not ask for an idle bound shorter than the
wall bound anywhere it runs a predicate, so a real `work finish` still classifies a stall
that develops partway through a long, otherwise-quiet run as `TimedOut` rather than
`Stalled` — the idle bound only earns its keep for a caller that configures it, and wiring a
useful default into the ledger's own predicate execution is a decision about how much
silence a real predicate should tolerate, not a fact about the launcher, and is left to
whichever item takes it up. The tree-kill mechanism is Windows-only, matching where this
crate's own tests actually run and where this repository is developed; a Unix mechanism
would need process-group setup at spawn time through APIs this workspace's zero-`#[cfg]`
property does not yet have a precedent for threading through cleanly, and is not built here
for the same reason the original conformance-suite gap above is not closed: one
implementation was measured, and a mechanism written for a platform nobody tested it against
would be a claim rather than a finding.

## Status

Accepted for the implementation that exists. The port-level question is open and named
above rather than left implicit: the property is unstated in the trait, unenforced for any
future implementation, and guarded only where it was fixed. `P11-EXEC-IDLE` narrowed one
part of that gap — the non-verdict `ExitOutcome` reports is no longer one value doing two
jobs — without closing it: the conformance-suite question from the original decision is
still open, and the idle bound this amendment adds is available to a caller rather than
applied on the caller's behalf.

---
id: OD-LEDGER-015
type: decision
title: The three verbs that change the board took no lock, and what a lock may span is the recording of a verdict and not the reaching of it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-LEDGER-003
    type: relates-to
  - target: OD-LEDGER-009
    type: relates-to
  - target: OD-PLATFORM-001
    type: relates-to
---

# The three verbs that change the board took no lock, and what a lock may span is the recording of a verdict and not the reaching of it

## Question

`FileLedger::With_Lock` acquires the cross-process lock, reads the ledger, hands the document
to a modification and writes it back. Its doc comment opened with four words:

> Every mutation goes through here.

`Claim`, `Renew` and `Release` are every mutation there is. None of them went through it. Each
did its own `self.Load()`, mutated the document in memory, and called `self.Save()` with
nothing acquired anywhere in between.

The stronger version of the finding is the one worth writing down. `With_Lock` had **no
callers at all**. Searching the workspace for the identifier returned the definition and one
line in `tests/contract/surface/nomos-ledger.txt` recording that it is public. It was a
correct mechanism, documented at length, exercised by nothing, describing an arrangement that
did not exist.

## What Was Actually At Risk

`work/ledger.json` is one JSON document holding every item. A read-modify-write over a whole
document loses, not a field, but every change the other writer made to any item — because
what gets written back is a snapshot taken before that writer existed. Two sessions
overlapping meant one of the two updates vanished, and the loser was told its operation had
succeeded.

This board is worked by several agent sessions at once, by design. `OD-LEDGER-001` says two
items are concurrently claimable exactly when their territories are disjoint, and the whole
point of that rule is that disjoint work proceeds in parallel. So the arrangement that makes
the ledger useful is also the arrangement that made this reachable, and — this is the part
that makes it worth an item rather than a comment — the two writers in the race are precisely
the ones the rule *permits*. `T-1` and `T-2` sharing no territory is not a conflict the
ledger has any reason to refuse. It was still a lost write.

What it looks like from outside is the ledger simply not having recorded something. There is
no error, no conflict, no corrupt file, and nothing in `git diff` to see: the file reads as
whatever the winner wrote, which is a perfectly valid document. A lost `Release` is an agent
that ran its verification, was told the item was recorded as done, and left an item still
marked claimed. A lost `Renew` is worse to diagnose than to suffer — it presents as a lease
that ran out early, which reads as an agent that died, so whoever investigates investigates
the wrong thing.

**No incident is known to have been caused by this.** It was found by reading `store.rs` while
designing `P10-STALE-WRITER`, not by a board that lost work. The window per operation is a
load, a decision and a save — milliseconds — and nothing here claims it was being hit
routinely. What is not speculative is the mechanism: the lock several concurrent sessions rely
on was not being taken by the code that writes.

It is a different defect from `P10-STALE-WRITER`, which is why the two are separate items with
the same symptom. That one is a binary from before a field existed dropping the field it does
not understand. This one is a current binary dropping changes it never read. Same missing
data, unrelated causes, and folding them together would have produced a fix for one wearing
the other's evidence.

## The Constraint That Makes This Non-Trivial

`Finish` runs the item's verification predicate — `cargo test --workspace`, minutes of it —
and `OD-LEDGER-003` added the gate's own lint step in front of it. A cross-process lock held
across that would stop every other session on the machine for the duration, and each of them
would sit in `LOCK_WAIT_LIMIT` for twenty seconds and then fail. "Take the lock around
finishing" is not a smaller version of the right answer; it is a worse defect than the one
being fixed.

The resolution is that these are two different spans and only one of them is a
read-modify-write.

Reaching a verdict is `Finish` in `finish.rs`: load the document, find the predicate, run the
gate step, run the predicate. It reads and it never writes. **Recording** a verdict is the
single call to `ledger.Release(item, holder, ReleaseOutcome::Finished(record))` at the end of
it, and that is the read-modify-write. Locking `Release` therefore leaves the minutes exactly
where they were — outside — and puts the milliseconds inside. `finish.rs` needed no change at
all, and this item reserved none of it.

There is a race left in `Finish` and it is a different one: it reads the item, spends minutes
not holding anything, and then writes. Somebody else may have released the item in between.
That is not a lost update — `Release` re-checks the holder *inside* the acquisition and
refuses if the claim is no longer this holder's — so the outcome is a refusal rather than a
silent overwrite. A refusal after a long predicate is a cost; a silent overwrite would be a
defect. It is left as it is, deliberately, and named here so the next reader does not have to
rediscover that it was considered.

## The Decision

**The three verbs perform their read, their decision and their write inside one acquisition,
through the mechanism that already existed.**

Not a second mechanism beside it. `With_Lock` was already correct, and duplicating a canonical
mechanism is how two of them come to disagree about which one holds the lock.

What sits between the verbs and `With_Lock` is a private `Decide_Under_Lock`, because the
verbs have a shape `With_Lock` alone cannot carry: they answer with a `ClaimRefusal`, and a
refusal is an *answer*. Sending it out through `With_Lock`'s error channel would put "agent-b
holds this" and "the file will not parse" into one type, which is exactly the conflation
`OD-LEDGER-009` had to undo when every load and save failure in these three functions was
being reported as `NoSuchItem`. So the refusal rides out as the modification's value and only
genuine store failures use the error. The helper is written once and called three times,
because three copies of "take the lock, read the clock, decide, write" are three chances for
one of them to stop taking the lock, which is the defect being fixed.

Two smaller things were settled on the way, and both are consequences of moving the span
rather than separate improvements.

**The clock is read inside the acquisition.** All three verbs used to read `now` before the
load. Once the load waits for a lock, a claim that waited on a contended lock would be granted
a lease measured from before the wait — shortened by however long it waited — and would judge
other holders' leases against a time that had already passed. One reading, taken after the
lock is held, is handed to the decision.

**The write happens only if the document changed.** Every one of these verbs has refusal paths
that decide against changing anything: held by somebody else, no such item, not claimable.
`With_Lock` used to save unconditionally, which under this arrangement would mean a refused
claim rewrites the roadmap. Worse, `Save` validates, so a document that is already invalid on
disk would turn "your claim is held by agent-b" into "the ledger is unusable" — a store
failure reported in place of the answer to the caller's question, which is the shape of
`OD-LEDGER-009` all over again. Comparing what was read against what the modification produced
is cheap and makes the refusal paths read-only, which is what they always were.

## What Is Deliberately Not Locked

`Conflicts` reads and answers and never writes. `Save` replaces the file atomically, so a
reader sees one whole document or another whole document and never half of one, and there is
no read-modify-write for a concurrent write to land inside. Locking it would make every
listing queue behind every writer to answer exactly the same question. Its answer can be stale
by the time the caller acts on it, and acting on it goes through `Claim`, which re-asks under
the lock. `Load` and `Validate_Current` are the same case.

## What This Gives Up

The stale takeover. `With_Lock` returns `(T, Option<StaleTakeover>)` and its comment says a
caller that drops it has made a choice which the signature makes visible in review. The three
verbs drop it, and this is that choice being made rather than overlooked.

They have nowhere to put it. `Reservation` and `ClaimRefusal` are both public, and adding a
field or a variant to either is a change to the crate's public surface —
`tests/contract/surface/nomos-ledger.txt`, which this item did not reserve and could not edit.
What is lost is a diagnostic, not consistency: a lock is broken only after `LOCK_STALE_AFTER`
of fifteen minutes, and the document a takeover finds is whole because `Save` is atomic. What
the operator does not get told is that their predecessor died mid-update. That is worth
surfacing and it is worth a surface change, and it is a different item than this one.

## Reproducing It

An acceptance test for a lost update has to produce the interleaving, not hope for it. Two
writers merely started at the same time reproduce this by luck; a test that reproduces by luck
goes green on a slower machine with the defect still in place, and would have been worth less
than the doc comment it replaced.

`Test_Two_Concurrent_Claims_Should_Both_Survive` and its two siblings for `Release` and
`Renew` run two OS threads against one ledger directory and fix the order rather than racing
it. The second writer does not start until the first has read the document it is about to
write back; the first does not write until the second has either finished or been kept waiting
for 750 milliseconds. Both outcomes are legitimate, and they are what the two arrangements
look like from outside — without the lock the second writer completes inside the window and
the first then writes over it; with the lock the second is still waiting when the window
closes, and reads the first writer's document when it finally gets in. The wait that is
*meant* to expire is why the harness uses a condition variable rather than a `Barrier`.

The seam is a `FileSystem`, not a hook in the store. `FileLedger` already takes the filesystem
it reads through, for a reason its own doc comment gives — a caller reaching for `std::fs`
would put the store beyond a test's control, which is `OD-PLATFORM-001`'s rule applied one
crate down. The test supplies an ordinary implementation that happens to stop one registered
thread after handing back the bytes. Nothing test-only was added to `store.rs`, and no
production code knows the tests exist.

The harness asserts that its own seam fired. A run in which nothing was held is a run that
interleaved nothing, however green it is, and would be the same vacuity `OD-SPEC-007` had to
guard against one system over.

## The Controls

**Before the fix.** The three tests were written first and run against the unchanged store:
34 passed, 3 failed, with

```
assertion `left == right` failed: the second writer was told its claim was granted and the
ledger does not have it: one writer wrote back a document it had read before the other one
existed
  left: None
 right: Some("agent-b")
```

and the corresponding two for the release and the renewal. `left: None` is the second writer's
claim being absent from the file the first writer wrote.

**After the fix.** 37 passed, 0 failed, in 0.95 seconds — three overlapping 750-millisecond
windows.

**The negative control, on the fixed code.** `Decide_Under_Lock` was changed to do the load,
the decision and the save directly instead of through `With_Lock`, changing nothing else: the
clock still read once, the write still conditional, the verbs untouched. 34 passed, 3 failed,
with the same three messages verbatim. So what those tests hold is the acquisition itself and
not some other part of the rearrangement. The acquisition was restored and the suite returned
to 37 passed.

## Status

Closed by `P10-LOCK-BYPASS`. No public item of `nomos-ledger` was added, removed or changed;
`tests/contract/surface/nomos-ledger.txt` is untouched, and `With_Lock` keeps its signature and
its `(T, Option<StaleTakeover>)` contract. The doc comment on it no longer describes an
arrangement that does not exist, and it now also says what must never be wrapped around it.

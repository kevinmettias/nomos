---
id: OD-LEDGER-021
type: decision
title: A verb that changes the board is one the store owns, and add was outside the door
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-015
    type: relates-to
  - target: OD-LEDGER-009
    type: relates-to
  - target: OD-LEDGER-010
    type: relates-to
  - target: OD-LEDGER-014
    type: relates-to
---

# A verb that changes the board is one the store owns, and add was outside the door

## Question

`nomos work add` printed its success line, exited 0, and the item was never on the board.

`AGENTS.md` rests the whole working loop on the opposite of that. Step 3 says **exit 0 is the
only thing that means you have it**, and step 2 says to put an item on the board before
claiming one. An add that returns 0 over a write that did not survive makes the first
sentence false at the step that precedes the one it is written on.

`OD-LEDGER-015` recorded this failure once already, for `Claim`, `Renew` and `Release`. This
is a second instance, reached by a different route, in a verb that record did not name.

## What Was Observed

Two `work add` invocations ran back to back from one script at unix 1786386100, against a
freshly built schema-3 binary. Both printed their success line and both exited 0:

```
added P11-SPENT-RECORD-ID reserving 6 path(s)
added P11-REEXPORT-SURFACE reserving 5 path(s)
```

`P11-SPENT-RECORD-ID` is on the board. `P11-REEXPORT-SURFACE` was in no commit, `git log -S`
over `work/ledger.json` finds it in none, and it was absent from the working tree minutes
later.

The window is locatable and does not depend on remembering what happened. Commit `a2c0fed`,
written by another session while those adds ran, carries a `work/ledger.json` holding the
first of the two adds and not the second. The file on disk had already lost it before that
commit was taken, so the loss is between the two adds and not in anybody's staging.

## The Mechanism, Established Rather Than Inferred

`Add` lived in the command layer, in `crates/host/nomos-cli/src/work.rs`. It read the whole
document with `ledger.Load()`, checked the identifier, pushed the item and wrote the whole
document back with `ledger.Save()` — with nothing held anywhere in between. It never called
`With_Lock`, and it is not on `FileLedger` at all.

That is the exact shape `OD-LEDGER-015` removed from three verbs. It survived in a fourth
because those three were fixed **by name**: the record calls them "the three verbs that change
the board", and putting an item on the board was not counted as changing it. It is. The
document `add` writes back is the whole board, so an add that read before a peer's claim
writes its own snapshot back over that claim.

The loss is symmetric, and this is the part that made the incident hard to read. Whichever of
the two writers saves last wins whole. So the same single defect appears as a lost *claim*
when the add saves last, and as an add that returned exit 0 and was never on the board when
the peer's locked verb saves last. The second is what was observed here; the first is what the
test reproduces, because the harness controls which writer is held.

The peer's verb was not at fault and did nothing wrong. It took the lock, loaded inside it,
decided and wrote — correctly. A lock excludes only the writers that take it, and the add did
not, so the lock had nothing to say about it.

### The two candidates that were ruled out

Both were plausible enough to check before writing any code, and neither is what happened.

**A second lock identity under `NOMOS_WORK_DIR`.** Ruled out at the construction site. The
ledger path and the lock path are derived from one `directory` argument in the same
expression — `directory.join("ledger.json")` and `directory.join("ledger.lock")` — so the
lock always accompanies the file it guards. Pointing the variable elsewhere selects a
different ledger *and* its own lock; it cannot split one from the other.

**A lock taken but not honoured across processes on this platform.** Ruled out at the
primitive. `FileLock` excludes by `OpenOptions::create_new`, which is `CREATE_NEW` on Windows
and `O_CREAT | O_EXCL` on Unix — chosen for exactly this reason and recorded in its own doc
comment. Its contention tests pass on this platform.

Neither was needed. The writer bypassed the door.

## The Decision

### 1. `add` is a verb that changes the board, and the store owns it

`FileLedger::Add` performs the read, the duplicate check and the write inside one lock
acquisition, through `Decide_Under_Lock` — the same door `Claim`, `Renew`, `Release`,
`Take_Over` and `Decline` go through. The command layer keeps the argument parsing and the
reporting, which is all a command layer should have had.

### 2. The duplicate check moves inside the lock with the write it guards

Not incidental to moving the code, and worse than the lost item if left behind. Outside the
lock, two sessions adding one identifier both read a board without it and are both told it is
theirs. The document that results carries the identifier twice, `Validate` calls that invalid,
and every operation loads before it does anything — so the next agent to touch the board, on
any item, is refused by a ledger that will not load. Two callers were each told they
succeeded and the board is unusable.

### 3. A refusal that was distinguishable stays distinguishable

`AddRefusal` is `add`'s own vocabulary rather than a borrowed `ClaimRefusal` arm. Adding an
item takes no territory, judges no lease and consults no other holder, so every arm of a
claim's refusal would be a sentence about a different question — the mis-subject
`OD-LEDGER-014` measured.

It has three arms because the command already gave three exit codes and routing the write
through the store must not collapse them:

| Arm | Exit | What the caller does next |
|---|---|---|
| `AlreadyPresent` | 4 | choose another identifier |
| `WouldBeInvalid` | 1 | correct the item — commonly, reserve something |
| `LedgerUnusable` | 5 | stop and fetch a person |

`WouldBeInvalid` exists only because of this. `Save` refuses an invalid document with
`LedgerError::Invalid`, and carrying every `LedgerError` out as `LedgerUnusable` would have
turned "your territory is empty" into "the ledger is unusable" — `OD-LEDGER-009`'s conflation
arriving by a new route, at the one refusal an author reaches by writing a plausible item.
`AGENTS.md` states the empty-territory rule, so it is a refusal authors meet.

### 4. `Decide_Under_Lock` is generic over what its caller refuses with

It was concretely `ClaimRefusal` while the only callers were claim verbs. Rather than give
`add` an arm to borrow, or let it reach for `With_Lock` directly — which is the per-verb copy
that function exists to prevent, and precisely how this defect happened — the door stays
single and each verb brings its own vocabulary. The only bound is being able to say "the store
itself failed", expressed as `From<&LedgerError>`.

This is the part that addresses the class rather than the instance. `OD-LEDGER-015` fixed
three verbs by name and the fourth was written outside the door afterwards. A door that
refuses only the vocabularies it was born knowing is a door the next verb routes around.

### 5. The fix is not a wider lock wait

Widening `LOCK_WAIT_LIMIT`, or retrying, changes how often two writers collide. It does not
change whether a writer that never takes the lock can be excluded by one, which is the
property. A remedy that improves the odds would have made the next instance rarer and no less
possible, and rarer is worse here: this one took a peer's commit to diagnose.

### 6. It is not closed by advice either

Telling authors in `AGENTS.md` to re-read the board after adding would document the defect as
a working practice. The contract being repaired *is* that exit 0 is the answer; a step that
verifies exit 0 concedes it is not.

## What This Does Not Do

`add` still does not refuse an item whose territory somebody already holds, and that is
`OD-LEDGER-010`'s decision, untouched. Opening an item on held ground is how this board is
used. What is now guaranteed is that the write survives, not that the ground is free.

It also does not close `P11-SPENT-RECORD-ID`: an identifier already published can still be
reserved, because the check compares against open items only. That is a different question
about what `add` should refuse, and this record is about whether what it accepted was written
down.

## Controls

Three tests in `crates/substrate/nomos-ledger/tests/exclusion_holds.rs`, using the
`Two_Writers` interleaving harness that `OD-LEDGER-015`'s own tests are built on. The harness
holds one writer between its read and its write, so the ordering is constructed rather than
raced — a test that reproduced by luck would go green on a slower machine with the defect
still there.

- `Test_An_Add_Should_Not_Erase_A_Claim_Taken_While_It_Ran`. Watched failing before the fix,
  with the claim granted to `agent-b` gone from the board it was written to.
- `Test_Two_Concurrent_Adds_Of_One_Identifier_Should_Not_Both_Be_Accepted`. Watched failing
  with the duplicate check moved back outside the lock and everything else unchanged, which is
  the negative control for decision 2 specifically.
- `Test_An_Item_That_Would_Not_Validate_Should_Refuse_As_The_Authors_Mistake`, for decision 3.

The exit-code contract in decision 3 is held from outside by the seven existing tests in
`crates/host/nomos-cli/tests/add_guarantees_what_it_says.rs`, which pin 0, 1 and 4 through the
real binary and were not modified. Two of them fail if `WouldBeInvalid` is dropped.

## Status

Accepted. Implemented across `P11-ADD-NOT-DURABLE` — the store, the record and the tests —
and `P11-ADD-CALL-SITE`, which carries the two files the first item's territory did not
reserve. The second item exists because there is no `work edit` and widening a claim
mid-flight is what makes a declared territory worthless; the two territories are disjoint and
were held at once, which is the arrangement `OD-LEDGER-001` describes.

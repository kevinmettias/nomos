---
name: nomos-task
description: Run one unit of work in this repository end to end through the ledger - observe the board, claim an item, implement inside its territory, verify, finish, commit, re-read. Use whenever you are about to change this repository and are not already holding a claim, and when authoring a new item so its territory is right the first time.
---

# Taking a task through the ledger

`AGENTS.md` is the contract; this is the procedure. Read the contract first if you have not.

The loop has nine steps and five of them have a failure that looks like something else.
Those are the reason this is a skill rather than a paragraph.

## 1. Re-derive the state

```
git log --oneline -5
git status --short
```

Other sessions work this tree concurrently. Anything your task description says about the
board, the tree, or which item is free may already be false — including a statement that
the tree is clean. Uncommitted files that are not yours belong to somebody live: leave them
alone, and never `git add -A`.

## 2. Build the binary, and re-copy it

The ledger verbs are the `nomos` binary, and running them through `cargo run` breaks in two
ways: the predicate is usually `cargo test`, which rebuilds the binary that is the running
process, and a polling loop fights your own builds for the target directory lock.

Build, then copy the binary somewhere outside the tree and run the copy.

**Re-copy after every pull or rebuild.** A copy built before a ledger field existed cannot
write that field back. A build carrying the guard refuses the verb outright; one copied
before it drops the field silently at exit 0. `nomos work validate` prints the file's schema
beside the build's, so one command tells you which copy you are running.

## 3. Choose an item, or author one

`nomos work list` shows the board and why an item cannot be taken. Prefer an item whose
territory does not overlap what anybody holds.

If you are authoring one, the territory is the part that is hard to change later — there is
no `work edit`, and widening mid-claim is the failure this repository has already had
twice. Reserve, in addition to the code you will edit:

- the record identifier under `docs/records/` if the work makes a decision, **and** the
  registration file under `crates/spec/nomos-spec-store/records/` that makes it governing;
- the snapshot under `tests/contract/surface/` for the crate whose public API moves;
- `tests/contract/tests/corpus_gates.rs` if you add a test to any corpus-gated file.

Check the record identifier is unused before adding the item. Nothing else will, and two
items reserving one identifier turns a shared guard red for a reason that reads like
contention.

An item that reserves nothing is refused. A territory of `.` reserves the whole repository
and blocks every other claim; never author one.

## 4. Claim it, and branch on the exit code

**Exit 0 is the only thing that means you hold it.** The listing column is a snapshot and
means nothing seconds later, and the refusal and success lines share vocabulary, so a text
match reads a refusal as a success.

Refused-because-taken is retryable and you should pick up something else. Ledger-unusable
is not retryable and means stop and fetch a person. `README.md` has the codes; the point
here is that collapsing them into non-zero makes the first indistinguishable from the
second.

If you must work before a claim frees, do not leave the changes in the tree — a peer's
`work finish` runs the workspace lint, and your uncommitted change fails their item for
your reason. Park it as a patch and re-apply after the claim lands.

## 5. Read only what this item needs

Route through `AGENTS.md`. An item about one crate does not need the whole architecture,
and reading it anyway is how a context fills up with what the task did not use.

## 6. Implement inside the territory

The smallest change that satisfies `done_when`. Do not fix the neighbouring defect you
noticed — add an item for it; that is what the board is for. Do not introduce a second
authority for something already governed by a record, a test, or a generated projection.

A decision that outlives the item goes in a record, and the record is not governing until
its registration file exists.

## 7. Verify before finishing

Run the workspace lint yourself, then the item's own predicate, then the tests your change
actually reaches. `work finish` runs the lint too and refuses on a red gate *after* you
have spent the run, so doing it by hand is purely for speed.

A green run is weaker than it looks in two places: `cargo test` stops early without
`--no-fail-fast`, so the failure count you read may be one of several, and a test that
cannot find its corpus passes having read nothing.

## 8. Finish through the ledger

`work finish` records the item done only if the predicate exits zero, and keeps apart the
predicate failing, the predicate not running at all, and there being no predicate. Never
hand-edit an item to done: the three answers collapse into one and the item claims a check
that never ran.

## 9. Commit, then re-read the board

Commit the paths you touched, explicitly, including `work/ledger.json`. Write the message
to a file and use `git commit -F` — the shell here splits long prose bodies into arguments
and fails after `git add` has already run.

Then re-read the board. It has changed while you worked.

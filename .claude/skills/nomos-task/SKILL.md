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

## 3. Claim what AGENTS.md step 2 names, or author one

Which item to claim is step 2's answer, not this skill's: `nomos work list` prints it on
the `next:` line, computed from the whole board rather than picked by eye. Read it there.

`next:` orders only by id and does not weigh territory overlap with a live session —
claiming is declared, not enforced, so nothing stops it from naming an item whose files a
peer already holds. Claiming a different eligible item with cleanly disjoint territory
instead is still sound judgment; the line names an answer, not a rule against a better one.

If the board names none, you are authoring one. The territory is the part that is hard to
change later — there is no `work edit`, and widening mid-claim is the failure this
repository has already had twice. Reserve, in addition to the code you will edit:

- the record identifier under `docs/records/` if the work makes a decision, **and** the
  registration file under `crates/spec/nomos-spec-store/records/` that makes it governing;
- the snapshot under `tests/contract/surface/` for the crate whose public API moves;
- `tests/contract/tests/corpus_gates.rs` if you add a test to any corpus-gated file.

Those four are instances of one rule, not a list to memorise, and the rule is what finds the
fifth: **territory follows the acceptance predicate's dependency cone, not the sites where
the failure appears.** The predicate names a test, the test reads inputs, and any input whose
modification may be required to satisfy it is territory. Grep the target tests for
`include_str!`, `include_bytes!`, `read_to_string`, `env::var` and any crate-local register
or fixture module, then follow each to the file it lands on.

Two items were declined in one day for missing exactly that. The first reserved two files
while its failures lived in five. The second reserved all five failing test files and none of
the data they read -- `crates/spec/nomos-spec-ingest/tests/family_counts/measurement.rs` takes
every expectation it checks from an `include_str!` of `tests/corpus/families/counts.json`, the
file holding the numbers. Both
authors grepped; the second grepped every failure site and stopped there.

A file that only *reads* the same input is not territory. It is a non-regression check the
`done_when` names, and grounds for declining rather than widening if it turns out to need an
edit.

**A Rust corollary: `pub` is not reachability.** A `pub` item inside a private module is
crate-private to every consumer. The crate's own snapshot under `tests/contract/surface/` is
the oracle -- a name absent from it cannot be named by another crate, whatever its own
modifier says. An item was
authored on the premise that a host could call a `pub fn` it had no path to, and was declined
before it was ever claimed.

Nothing enforces any of this. Two instances justify stating a rule, not building one to check
it, and that is deliberate rather than an omission.

Check the record identifier is unused before adding the item. Nothing else will, and two
items reserving one identifier turns a shared guard red for a reason that reads like
contention.

An item that reserves nothing is refused. A territory of `.` reserves the whole repository
and blocks every other claim; never author one.

**The predicate is authored in the same call, and its scope is the item's claim.** Territory
says which files the item may change; the argv after `--` says what will be run to establish
that it changed them correctly. A predicate establishes *this item's* `done_when`. It does not
inherit the independent obligations a crate or package happens to own, unless those
obligations are themselves part of what the item claims.

That is scope matching, not breadth minimising. `cargo test -p <crate>` is exactly right for
an item whose claim really is that the crate remains valid; what is wrong is a verification
scope wider than the obligation scope the item declared, whatever shape the command has. It is
also a different argument from the one
`crates/substrate/nomos-ledger/src/gate_unknown.rs` makes for keeping the gate's test step
authored per item. That one is about cost, minutes spent on every finish. This one is about
attribution: a predicate wider than the claim goes red for somebody else's reason, and the
item cannot finish although its own acceptance condition passes.

Two items paid for that in one day. The first declared both spec crates while claiming a
single thing — that a preservation test stops reporting a violation once the corpus is
present. Run with the corpus exported it failed instead on a volume count over a directory
outside this repository, and it finished corpus-unset with its real obligation discharged by
hand, so the ledger's machine-recorded verification did not prove what the item claimed. The
second declared the whole contract-tests package and changed one markdown file. It could not
finish because `public_surface` was red on a peer's unblessed snapshot, while `agent_harness`
— the obligation that governs skill text, and the only one the item claimed — was green
throughout.

Find the scope where the territory was found, in the same dependency cone. Where a single test
binary is the obligation, `cargo test -p <package> --test <target>` names it and nothing else.
The item that added this paragraph declared
`cargo test --no-fail-fast -p nomos-contract-tests --test agent_harness`, because
`tests/contract/tests/agent_harness.rs` is what judges skill text and no other obligation in
that package was any part of its claim.

No mechanism checks this, for the reason given above: two instances justify a rule and not a
predicate-scope validator, a scope algebra, or automatic dependency analysis.

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

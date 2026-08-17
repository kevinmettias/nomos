---
id: OD-LEDGER-027
type: decision
title: A verification record names HEAD as the tree it ran against, and work show reports when that has moved
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - verification
  - identity
relations:
  - target: OD-ANALYSIS-001
    type: relates-to
  - target: OD-STORE-002
    type: relates-to
  - target: OD-LEDGER-003
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
---

# A verification record names HEAD as the tree it ran against, and work show reports when that has moved

## Question

`VerificationRecord` carries `argv`, `exit_code`, `output_tail`, `verified_at` and `gate` —
what ran, what it answered, when, and whether the gate agreed. Nothing in it says what tree
that predicate ran against. An item finished at unix `N`, and the tree moved under it after —
a peer lands a module split, a record is amended, the item's own files are edited by the next
item that reserves them — and `work show` reports exactly what it reported before. "Verified"
and "was verified once, against something else" are the same three words.

`OD-ANALYSIS-001` already decided this question once, for a different identity: a key
carrying the wrong thing about a workspace state defeats the reuse the key exists for, and
the fix there was not to add a coarser answer but to separate *what a fact is computed from*
from *what tree it was measured against*, and to keep the second as provenance beside the
fact rather than folded into its identity. `OD-STORE-002` names the same split again, for a
different family, and gives it a name: `revision`, carried as data, excluded from identity,
"the revision it was read at." A `VerificationRecord` is not a derived identity — it does not
need a revision-independent equality — but it is exactly the "provenance, not identity" half
of that split, missing its provenance.

## What Identifies A Tree

Three candidates, each cheap to state and each with a cost the others do not have.

**A git revision — `HEAD`, read directly.** Cheap: reading `.git/HEAD` and following one
loose ref costs a file read or two, not a subprocess. It is what CI and every reader already
speaks — `git log`, a commit hash in a PR, `git diff <rev>`. Its cost is a real one: it is
silent about a dirty working tree, which is the normal state of this repository while an item
is being finished — the record is written before the commit that carries the item's own
change, so the working tree at `verified_at` is, in the ordinary case, ahead of what `HEAD`
names.

**A digest over the item's own reserved territory.** Honest about exactly what the predicate
could have been affected by, and no more — it does not claim to see a file the item's
territory never named. Its cost is symmetric with its honesty: it says nothing about the rest
of the workspace, which is where a peer's breakage actually comes from. Every predicate on
this board is a `cargo test -p <crate>` invocation, and a crate under test reads every file in
it, not only the ones an item's territory happened to name — so a digest scoped to an item's
own territory would not see a shared file in the same crate moving under an item that never
touched it, which is precisely the "peer lands a module split" case `done_when` names first.

**A digest over the whole working tree.** The strongest of the three — it sees everything the
predicate could possibly have run against. Its cost is that it sees things the predicate could
not have run against too: it moves on every untracked scratch file, every build artifact
outside `.gitignore`, every file a concurrent session is mid-edit on in a directory this item
never reserved, unless the scope is curated by hand — and curating that scope is a second,
unstated territory declaration living beside the one the ledger already has.

## Decision

**`VerificationRecord::revision` is `HEAD`, read directly from `.git/HEAD` (following one
loose ref if `HEAD` names a branch rather than a commit), through the same `FileLedger`
filesystem abstraction `Finish` already reads the gate's workflow through.**

Chosen for what it does not give up rather than only for what it buys: it answers the
question this field exists for — has the *committed* tree moved under this item since it was
verified — which is exactly the shape of all three examples in `done_when`'s own list (a
peer's module split, a record amendment, the next item's edit to this item's files): each is
a commit that lands after this one's `verified_at`, and `HEAD` moving is precisely what
detects a commit landing.

What it gives up is stated rather than hidden: it cannot see the working tree's own dirty
state, so two verifications against the same `HEAD` with different uncommitted diffs compare
as identical. A territory digest would close exactly that gap and open the wider one above; a
whole-tree digest would close both gaps and manufacture noise from every scratch file this
repository's own working style produces continuously — `AGENTS.md`'s scratch-file guidance
exists because that noise is real, not hypothetical. `HEAD` is the answer that is honest about
the one thing every reader already checks by hand today — "did anything land after this" —
and declines to pretend to answer the finer question neither of the other two answers cleanly
either.

**`None` on any read failure, with no distinction between the failures.** No `.git` at the
tree the predicate ran in, `HEAD` naming a ref this build does not chase through
`.git/packed-refs`, or any other I/O error, all collapse to `None`. A reader cannot tell them
apart, and that is deliberate: none of them is a case this field claims to answer, and
inventing a value for any of them — guessing, or falling back to a subprocess `git`
invocation that might not be installed — would be worse than admitting there is nothing here.
This is the same shape `gate: Option<GateOutcome>` already takes, for the same reason: a
`None` left visible rather than backfilled.

**Never backfilled.** A record written before this field existed keeps `revision: None`
forever. Computing one after the fact — from the commit nearest `verified_at`, say — would
manufacture the exact claim this field exists to stop: that a tree was measured when nothing
measured it.

## Staleness Is Reported

`done_when` is explicit that a field nothing reads is a comment with a serializer.
`work show` now prints one of four lines for a `Done` item's verification, computing the
tree's *current* revision the same way `Finish` computed the recorded one:

- **No revision recorded** — the record predates this field, or `HEAD` could not be resolved
  when it ran. Distinguishing these two would need a reason nothing else here carries; both
  read as "this field has nothing to say."
- **Still describes this tree** — the recorded revision equals the current one.
- **`STALE`** — the recorded revision and the current one differ, naming both.
- **Cannot tell** — a revision was recorded, but the current tree's revision could not be
  read. Silence here would read as agreement, which is the one thing this field must not do
  by omission.

## Where The Change Actually Landed, Against What The Item Said

The item named `crates/substrate/nomos-ledger/src/verification_record.rs` and
`crates/substrate/nomos-ledger/src/finish/mod.rs`. Both had already moved by the time this
item was claimed: `verification_record.rs` is `crates/substrate/nomos-ledger/src/verification/record.rs`
— `verification.rs` at the crate root declares `mod record;` and `mod predicate;` and
re-exports both — and `finish/mod.rs` is `crates/substrate/nomos-ledger/src/finish.rs`, a
top-level file that declares the `finish` module's own submodules (`finishing`, `gate_step`,
`running`, and the rest) the same way `store.rs` and `verification.rs` do. This is the same
shape `OD-LEDGER-022` found and named for its own territory: a crate split after a territory
description was written moved the code the description pointed at, one level, without moving
the description. The field and the resolution both landed in `finish.rs`, where `Finish`
itself and the record it builds already live.

The item also named `crates/host/nomos-cli/src/work.rs`. The function `work show` prints
through — `Print_History`, and the verification line inside it — lives in its submodule
`crates/host/nomos-cli/src/work/listing.rs`, reached from `work.rs` by `mod listing;`.
Exactly the shape `OD-LEDGER-022` names for `report.rs` beside it. The staleness reporting
above landed in `listing.rs`.

The two crates each read `HEAD` themselves, independently, rather than sharing one reader.
`nomos-ledger`'s reading is private to `finish.rs`; making it available to `nomos-cli` would
have meant adding it to `crates/substrate/nomos-ledger/src/lib.rs`'s `pub use finish::{...}`
list, a file neither this item's territory nor `OD-LEDGER-022`'s named. `OD-LEDGER-022` took
that step for `RefusalLayer`, because there the alternative was reimplementing a match over
every `ClaimRefusal` variant in a second crate — a duplication with real drift risk, since a
variant added to one arm and missed in the other fails silently. Here the duplicated logic is
under fifteen lines, has no variants to miss, and both readings are exercised by the same kind
of fixture (a temporary directory `StdFileSystem` reads through), so the two are kept apart
deliberately rather than merged into a widened territory neither crate's item declared.

## What This Costs

Every predicate that runs `Finish` now reads one or two extra files. Both are local reads
through a filesystem abstraction already open for the gate's workflow; the cost is not
measurable against a `cargo test` invocation that is already minutes long.

## What Holds It

`crates/substrate/nomos-ledger/tests/exclusion_holds/persistence.rs`'s
`Fully_Populated` fixture carries a `Some` revision through a round trip, alongside every
other optional field a `Test_The_Ledger_Should_Round_Trip_Losslessly`-shaped guard depends on
being present. `crates/substrate/nomos-ledger/src/item/tests.rs`'s
`Test_A_Field_Added_To_An_Item_Should_Raise_The_Schema_Version` is unaffected by this field —
it counts `LedgerItem`'s own top-level keys, and `revision` is nested inside `verified`,
which was already one of them — so `SCHEMA_VERSION`'s bump here is enforced by
`deny_unknown_fields` and `Test_A_Ledger_Newer_Than_This_Build_Should_Say_So_Rather_Than_Malformed`-shaped
guards in `persistence.rs`, not by that counting test.

## What Would Make This Stale

A change to how this repository commits work that made `HEAD` no longer the boundary between
"before this item" and "after it" — for instance, finishing an item after its commit rather
than before. Nothing in this item's territory changes that ordering, and nothing here assumes
it will not change; if it does, this record's claim that `HEAD` at `verified_at` names "the
tree before this item's own change" stops holding and the record needs revisiting, not the
field.

## What This Record Does Not Decide

Not a subprocess `git` integration, and not a whole-tree or territory-scoped digest — both are
named above and both were rejected for the concrete costs stated, not because a digest is
categorically wrong; a later item with a different question (bit-for-bit reproducibility, say)
may need one of them and would decide that on its own terms.

Not a shared `Current_Revision` helper across `nomos-ledger` and `nomos-cli`. Two small,
independently-tested readings were kept over one shared, exported one, for the reason stated
above; a third caller needing the same read would be the point at which sharing it stops
costing more than duplicating it.

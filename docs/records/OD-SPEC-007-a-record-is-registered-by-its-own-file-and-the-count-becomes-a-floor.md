---
id: OD-SPEC-007
type: decision
title: A record is registered by its own file, and the count assertion becomes a floor
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - governing-records
  - concurrency
relations:
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-SPEC-005
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-LEDGER-001
    type: affects
---

# A record is registered by its own file, and the count assertion becomes a floor

## Question

`OD-LEDGER-007` named `crates/spec/nomos-spec-store` a structural serializer: a path every
record writer is forced to reserve, which does not resolve when one of them finishes because
the next one will reserve it too. Three symbols in two files forced it. `RECORDS` and
`GOVERNING_RECORD_IDS` in `governing.rs` were two hand-maintained lists a new record had to
join, and `governing_records_are_present.rs` asserted the second one's length against a
literal. Two items writing two *different* records edited the same two files, by
construction.

`OD-LEDGER-007` declined the obvious remedy and gave a reason rather than a schedule:
deriving both lists from `docs/records` would make
`Test_Every_Canonical_Record_On_Disk_Should_Be_Governing` compare that directory against
itself and pass having checked nothing. `OD-SPEC-005` exists because six governing records
were written as files and never reached the store, and that guard is the only thing that
catches it.

The objection is correct. It is also narrower than it looks.

## Answering The Vacuity Objection Directly

`OD-LEDGER-007` §"Why The Structural Remedy Is Not Applied Here" refuses one specific
derivation: one whose **input is `docs/records`**. That is fatal to the derivation it names
and to no other. What makes the guard a check is not that its second side is hand-typed into
one file — it is that the second side is **authored independently of the directory it is
compared against**.

So the second side stays authored, and stops being one file.

A record becomes governing by having a registration under
`crates/spec/nomos-spec-store/records/`, one file per record, named for the identifier:

```
# The registration for OD-LEDGER-007. Writing the record file does not make a record
# governing; this file does.
path: docs/records/OD-LEDGER-007-a-record-excludes-nobody-and-two-other-files-serialize-the-board.md
```

The identifier is the file stem and appears nowhere inside it, because an `id:` key would be
a second place for the identity to be wrong. The body names the path, because a slug is not
derivable from an identifier and renaming one should touch that record's own registration and
nothing else.

A build script reads that directory — **and only that directory** — and generates the two
tables into `OUT_DIR`. `docs/records` is never enumerated by anything that produces
`GOVERNING_RECORD_IDS`; the repository root reaches the generator only so that a registration
naming a file that is not there can be refused.

The guard's two sides are therefore what they always were. Side A is every file under
`docs/records` declaring `authority: canonical-normative-record`. Side B is a set a human
wrote down. The author still performs two independent acts, and forgetting the second is
still what the guard catches: writing `docs/records/OD-STORE-001-….md` and not writing
`records/OD-STORE-001.record` puts the identifier on side A and not on side B, and `unseeded`
is non-empty. `OD-SPEC-005`'s six records would fail today exactly as they failed then.

What changes is the *locality* of the declaration, not its independence. That is the whole of
the trade, and it is the point: two record writers now write two different files, so a record
excludes nobody but its own writer — in the store as well as in `docs/records`, which is what
`P10-RECORD-LOCK` bought one level down and `OD-LEDGER-007` found defeated one level up.

There is one thing this arrangement cannot prove about itself, and it is written here rather
than left to be discovered. No test in a consistent tree can detect a later "simplification"
of the build script into globbing `docs/records`, because in a consistent tree the two
directories yield the same identifiers and every assertion still passes. What guards it is
structural — the reader takes its directory as a parameter, and
`Test_The_Reader_Should_Enumerate_The_Directory_It_Is_Given` hands it a directory of one
while the repository holds thirty-two — plus a textual check over the call sites,
`Test_The_Build_Script_Should_Not_Enumerate_The_Record_Directory`. That check says in its own
doc comment that it is textual and is not evidence about behaviour. A textual check is a
textual check, and calling it more than that would be the same overclaim this record is
about.

## Why The Declaration Site Stays Written By Hand

`RECORDS` and `GOVERNING_RECORD_IDS` keep their `const` items in `governing.rs` verbatim.
Only the initializers become `include!(concat!(env!("OUT_DIR"), …))`.

This is load-bearing and it was settled by running the real scanner rather than by reading
it. `nomos_rules::Universes_In` matches `syn::Item::Const` with a slice type and public
visibility and **never inspects the initializer**, so an `include!` in initializer position
produces a `DeclaredUniverse` identical to the literal slice's — same path, same name, same
`kind: Constant`, same claimed mirror — and `tests/contract` needs no change at all, snapshot
included. An `include!` at *item* position is `syn::Item::Macro` and is dropped: the universe
returns **zero** rows, `Test_The_Declared_Table_Should_Match_What_Is_Derived` reports the row
as vanished, and deleting that row to make it green would leave the workspace's most-cited
declared universe uncounted forever. The same probe showed a `static` is not recognised
either.

The generated fragments are bare `&[…]` expressions, which the scanner reports `Unparseable`
rather than empty, and `Check_Completeness_Mirrors` turns `Unparseable` into a finding. So
they go to `OUT_DIR` and never into `src/`.

The prediction held: `tests/contract/surface/nomos-spec-store.txt` and
`tests/contract/tests/completeness_universes.rs` are byte-for-byte unchanged, and this item
reserved neither. One thing the probe could not have predicted was found by compiling, and it
is recorded here because the next person to hit it will lose an hour. `Without_Test_Modules`
in `tests/contract/src/gates.rs` blanks from a `#[cfg(test)]` marker to the **next** brace
block in the file, whatever that block happens to be. `#[cfg(test)] mod registration;` carries
no braces, so placed among the other `mod` lines in `lib.rs` it swallowed
`pub use authoring::{…}`, and `Test_Every_Crates_Public_Surface_Should_Match_Its_Snapshot`
reported eleven exports missing and a re-export it could not resolve — a snapshot failure with
no relation to anything this item changed. The declaration was moved to the end of `lib.rs`,
where nothing braced follows it, with a comment saying why. That is a workaround at the call
site, not a fix: the scanner is wrong for any crate that puts a brace-less `#[cfg(test)]` item
above a braced one, and `tests/contract` was outside this item's territory.

## What The Literal Count Was Buying

`assert_eq!(GOVERNING_RECORD_IDS.len(), 31)` was the last file forcing every record writer
into one crate, and it was not decoration. It bought two things, and they are not the same
thing.

**The deliberate step.** `OD-SPEC-005` and `OD-LEDGER-001` both describe raising it as the
cost adding a governing record is meant to carry. That cost is exactly what this item exists
to remove, and removing it is not a loss: the deliberate step survives as the registration
file. It is still a separate artifact, still authored by hand, still visible in the diff as a
file somebody added. It is no longer *shared*, which was never part of what it bought — that
was a side effect of where it happened to live.

**The coordinated deletion.** This is the real one. A record file and its declaration deleted
*together* pass both directions of `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`,
because the two sets still agree — they agree about a record that is gone. Nothing else in
this workspace sees that. The literal count saw it, and it is worth being precise about how:
it saw it by requiring the deleter to remember a **third** artifact. It worked by
forgetfulness, not by derivation.

No derived count can replace it. A count derived from the registrations falls when a
registration is deleted; a count derived from `docs/records` falls when a record is deleted;
neither notices when both fall together. The only thing that can notice is an artifact
outside both — which is what the literal was.

## The Decision

**A record is registered by its own file, and the count assertion becomes a floor.**

```rust
const FEWEST_GOVERNING_RECORDS: usize = 32;

assert!(GOVERNING_RECORD_IDS.len() >= FEWEST_GOVERNING_RECORDS, "…");
```

The floor keeps the coordinated-deletion guarantee by exactly the mechanism the literal used:
a deletion has to be accompanied by lowering it, and a deleter who forgets goes red. It gives
up the ceiling. Adding a record no longer costs an edit to this file — which is the item —
and adding one is no longer visible here at all.

The floor is a third artifact for the same reason the literal was, and it is *not* raised on
addition. It is lowered by a deliberate removal, and it may be raised for free by any item
that already has this file open for another reason.

## What This Costs

**Drift.** The floor's guarantee is exact only while the count sits on it. Once the count has
risen above, a coordinated deletion inside the slack is caught by nothing here. The literal
had no slack. This is the one guarantee that is genuinely weaker after this change, it is
weaker by an amount equal to the distance between the count and the floor, and nothing forces
that distance to close. A record deleted from a workspace of sixty, with the floor still at
thirty-two, is a deletion this workspace will not notice.

The alternative considered was an upper bound as well — a declared slack, failing when the
count exceeds the floor by more than some number. It was rejected: it reintroduces the shared
edit on an unpredictable schedule, so instead of every record writer colliding, one
unforeseeable record writer in every N collides with all the others. A rule that fires
unpredictably is worse to author against than one that fires always.

**One assertion narrows.** `Test_The_Records_Should_Be_Present_As_Disposed_Content` compares
`count(source_documents)` against `GOVERNING_RECORD_IDS.len()`. That incidentally caught
`RECORDS` and `GOVERNING_RECORD_IDS` disagreeing, because they were two lists kept by hand —
and they had already drifted in one respect: the two lists were in different orders,
`OD-LEDGER-001` and `OD-SPEC-004` transposed between them, harmlessly because both are sets.
They are now generated from one input and cannot disagree at all. The second subject becomes
impossible by construction rather than checked, which is stronger, and the assertion keeps the
subject its message actually claims. The remaining hazard, two registrations naming one
document, is a refusal in the reader rather than a test. Its doc comment says so, because
leaving the old one would have been an overclaim.

**A build script.** `nomos-spec-store` acquires one, and code that used to be read by rustc is
now written by a program. It has no dependencies and must keep none, its reader is under unit
test including every one of its refusals, and it refuses rather than skips on all of them — a
registration skipped instead of refused is a governing record that quietly stops governing,
which is `OD-SPEC-005`'s defect wearing a build script's clothes. The reader lives in
`src/registration.rs` and is reached from `build.rs` by `#[path]` rather than copied, because
two readers of one format are two answers waiting to disagree — which this workspace has
written down twice already.

**The register does not empty today.** `OD-LEDGER-007`'s `KNOWN_SERIALIZERS` reads declared
territory, not edited files, and twelve open items were authored under `OD-LEDGER-001`'s rule
as it stood. Removing the entry in the commit that dissolves the coupling would redden
`Test_Every_Universal_Reservation_Should_Be_Declared`, because those twelve territories still
reserve the crate. The entry stays and its *reason* is restated: it no longer describes a rule
that binds new items, it describes twelve territories authored before this record, and it
comes out when the last of them is finished or re-authored. A register that over-reports is as
useless as one that under-reports, and the reason is the part that would have been false.

Measured on the board the day this was decided: twelve open record writers, sixty-six pairs,
sixty-six blocked, forty-five of them by nothing but the two declared serializers, and twelve
by nothing but this crate. Those twelve are what this buys, once the reservations follow it
out. The other thirty-three are held by `tests/contract` as well and are not this record's
subject.

## The Controls, And The One That Is Not A Test

Every control below was observed failing before it was relied on, in the manner `OD-SPEC-005`
recorded its three.

**The guard did not go vacuous.** `Test_A_Record_Whose_Registration_Is_Missing_Should_Be_Unseeded`
runs the real comparison with the real canonical set and the governing set minus one element
and asserts the missing identifier is reported. `Test_A_Registration_With_No_Record_Should_Be_A_Phantom`
is the other direction. Both call the same extracted `Disagreements` the assertion calls, so a
control claiming the guard would have caught something is exercising the guard rather than a
copy written beside it.

**Side B is the registration directory.** `Test_The_Governing_List_Should_Be_The_Registration_Directory`
reads the `.record` stems at test time and compares them against `GOVERNING_RECORD_IDS`, with
its own vacuity guard.

**The refused derivation, exhibited.** `Test_Comparing_The_Directory_Against_Itself_Would_Check_Nothing`
performs the comparison `OD-LEDGER-007` declined — both sides from `docs/records`, including a
record nobody registered — and asserts it comes back clean. It asserts nothing about the
arrangement in place. It exists so the next person to propose globbing `docs/records` finds a
test that already says what would happen.

**The reader refuses rather than skips**, on a missing directory, an empty one, a file that is
not a registration, a stem that is not an identifier, a body with no `path:` line, one with
two, an unknown key, a path outside `docs/records`, a path that is not on disk, and two
registrations naming one record. Each is a unit test asserting `Err` and not a short `Ok`.

**The control that cannot be a test.** Deleting a registration file and watching
`GOVERNING_RECORD_IDS` shrink requires a rebuild, and no `#[test]` causes one. It was run
twice by hand and both results are written down here rather than asserted by something that
did not do them.

*Registration removed, record left on disk.* `records/OD-GATE-001.record` was removed and
`cargo test -p nomos-spec-store --test governing_records_are_present --no-fail-fast` run: 12
passed, 4 failed. `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing` failed naming
`OD-GATE-001` as unseeded — **this is the proof the guard did not go vacuous** — and
`Test_Every_Governing_Record_Should_Resolve_By_Id` failed on the floor at 31 against 32. The
two set-level controls failed as collateral, which is what they should do in a tree whose two
sides disagree. `Test_The_Governing_List_Should_Be_The_Registration_Directory` stayed green,
correctly: both it and the generated list lost the entry together. The file was restored and
the suite returned to green.

*Record and registration removed together.* Both `docs/records/OD-GATE-001-….md` and
`records/OD-GATE-001.record` were removed: 15 passed, **1 failed**. Every other assertion in
the file passed, including *both directions* of
`Test_Every_Canonical_Record_On_Disk_Should_Be_Governing` and
`Test_The_Governing_List_Should_Be_The_Registration_Directory` — because the sets agree about
a record that is gone. Only the floor failed. That is the "What The Literal Count Was Buying"
section demonstrated rather than argued, and it is the reason the floor exists. Both files
were restored.

## Status

Closed by `P10-SEED-GRAIN`. `OD-LEDGER-001` is amended at version 4: an item that will write a
canonical record claims its record identifier and its own registration file, and no longer the
store's governing list.

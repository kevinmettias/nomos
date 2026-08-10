---
id: OD-LEDGER-007
type: decision
title: A record excludes nobody, and the two files that still serialize the board are declared
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
  - target: OD-LEDGER-004
    type: relates-to
  - target: OD-GATE-002
    type: relates-to
---

# A record excludes nobody, and the two files that still serialize the board are declared

## Question

`P10-RECORD-LOCK` set out to stop a record reservation from serializing the board. Before
it, every open item reserved `docs/records`, which contains every record anybody could
write, so one claim refused all eight others. It re-authored each item to reserve the
record identifier it would allocate, and it left behind an acceptance test:
`Test_Two_Items_Writing_Different_Records_Should_Be_Claimable_At_Once`, asserted against the
repository's own `work/ledger.json`.

That test went red on 2026-08-09 with no code change. Five open items reserved a record and
no two of them could be held at once, so the search for a pair found none and both
assertions failed on their own precondition rather than on their subject.

The reason is not who authored what. It is a rule.

## Two Rules, Each Forcing A Shared Edit

**Seeding.** `OD-LEDGER-001`'s third authoring rule requires that anything writing a
canonical record also reserve `crates/spec/nomos-spec-store`, because a canonical record
must be added to `RECORDS` and `GOVERNING_RECORD_IDS` in `governing.rs` and the literal
count in `governing_records_are_present.rs` raised, or the workspace goes red. Two items
writing two *different* records therefore edit the same two files. They cannot be
independent, by construction — and the rule that makes it so was added in the same body of
work as the guarantee it defeats.

**Snapshots.** `P9-PUBLIC-API` checks each crate's public surface into
`tests/contract/surface`, and `corpus_gates.rs` declares a per-file test count. So an item
that widens any API, or adds any test to a corpus-gated file, writes under `tests/contract`
— and items reserve the whole directory in order to write one file inside it. That is
exactly the shape `P10-RECORD-LOCK` named for records, one level up, arriving independently
and later.

`OD-LEDGER-004` measured both without drawing the conclusion. It lists `crates/spec` and
`crates/spec/nomos-spec-store` at six blocked pairs and `tests/contract` at five, and reads
them as ordinary code contention. For record-writing items specifically they are not: the
overlap is guaranteed by an authoring rule, so the acceptance test passed only for as long
as some pair on the board happened to be disjoint for unrelated reasons. The property was
measured true once and was never structurally maintained.

## The Decision

**The acceptance property is restated to the one the mechanism can keep: a record excludes
nobody but its own writer. What else two record writers share is measured separately, and
every path *all* of them are forced to reserve must be declared.**

Three tests carry it, in `crates/substrate/nomos-ledger/tests/records_do_not_serialize.rs`.

**`Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer`** reduces every open record
writer to the records it reserves and claims all of them through the real ledger on that
projection. All, not a pair: a pair could be independent by accident. This is exactly what
`P10-RECORD-LOCK` bought and it holds for a structural reason — an item reserves
`docs/records/<ID>` and two identifiers are two paths.

**`Test_Every_Universal_Reservation_Should_Be_Declared`** is the new guard. A path two
record writers share is *contention*: two items that both change the CLI, which is what
territory is for and which resolves when one of them finishes. A path **all** of them
reserve is a *rule*, and it does not resolve — the next record writer will reserve it too.
Those are enumerated in `KNOWN_SERIALIZERS` with what forces each, and a third one arriving
fails the suite.

**`Test_Every_Declared_Serializer_Should_Still_Serialize`** is the other direction. A
register that over-reports is as useless as one that under-reports, so an entry that stops
serializing anything has to come out in the commit that earned it.

The register is a debt list, not an exemption list. It is expected to empty.

## Why The Structural Remedy Is Not Applied Here

The structural fix for seeding is to derive `RECORDS` and `GOVERNING_RECORD_IDS` from
`docs/records` rather than hand-maintaining them. That would remove the shared edit
entirely: adding a record would touch the record file and nothing else.

It is not done here, for a reason rather than for time. The hand-maintained list *is* a
declared universe, and the literal count assertion is the deliberate cost of adding a
governing record. `OD-SPEC-005` exists because six governing records were written as files
and never reached the store — the declaration is what catches that, and deriving the list
would make `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing` compare a directory
against itself and pass having checked nothing. Replacing a universe with a derivation is
`OD-COMPLETENESS-001`'s subject and overlaps `P10-MIRROR-DISAGREEMENT`, which is open and
holds the same question about `DECLARED_RULES`. Two items deriving two universes by two
methods is how they come to disagree, so it wants coordinating rather than doing twice.

The structural fix for snapshots is finer reservation: an item that changes one crate's API
reserves `tests/contract/surface/<crate>.txt`, not the directory. Nothing prevents that
today — territory is file-resolution and already compares paths by containment. What is
*not* done here is forcing it, because forcing it means re-authoring four items that
belong to other work, and `P10-SEEDING-SERIALIZES` explicitly refuses to manufacture a pair
by narrowing somebody else's territory.

## What This Costs

The board serializes and will keep serializing. At the time of this record: five open
record writers, ten pairs, ten blocked — seven of them by nothing but the two declared
serializers, and three by ordinary code contention as well. Those seven are the debt.

What is bought is that the number is measured on every run and printed beside its
denominator, the two causes are told apart, and a third cause cannot arrive quietly. What
is given up is the claim that the board is parallel today, which was never true for a
reason anybody had arranged.

## Status

Closed by `P10-SEEDING-SERIALIZES`.

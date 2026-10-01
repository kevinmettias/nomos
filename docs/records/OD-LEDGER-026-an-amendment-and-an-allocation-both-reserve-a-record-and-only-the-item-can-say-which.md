---
id: OD-LEDGER-026
type: decision
title: An amendment and an allocation both reserve a record, and only the item can say which
status: closed
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - records
  - concurrency
relations:
  - target: OD-LEDGER-025
    type: affects
  - target: OD-LEDGER-016
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
  - target: OD-LEDGER-014
    type: relates-to
---

# An amendment and an allocation both reserve a record, and only the item can say which

## Question

`OD-LEDGER-025` made `work add` refuse a territory naming a record identifier this repository
had already published, because an identifier is allocated once and an author who reached for a
spent number had no way to move it afterwards. That is right for the case it was built for.

It is wrong for the other reason an item names a published record. Amending one is ordinary
work here — `ARC-ECOSYSTEM-001` is at version 2, `OD-CAPABILITY-001` is at version 2, and
`D-134` was amended in place rather than superseded because six of its seven decisions were
untouched and cited by working code. An item that amends a record **must** reserve it, because
the amendment edits that file and territory is the only thing keeping a second writer off it.

So the guard refused the reservation that the discipline requires.

## What Was Measured

Against `ARC-HARNESS-001`, on 2026-08-10. Reserving `docs/records/ARC-HARNESS-001` was refused
by identifier stem; reserving the full published filename was refused too, because
`OD-LEDGER-016` folds a record filename onto the identifier it carries and both spellings reach
the same subject. No spelling got an amendment through.

Two costs, both already on the board rather than hypothetical.

`P11-ECOSYSTEM-UPWARD` reserves `docs/records/ARC-ECOSYSTEM-001` and its `done_when` is version
3 of that record. It was authored before `OD-LEDGER-025` landed and could not be authored the
day after, so the guard forbade an item the board was already carrying.

`ARC-HARNESS-001` owes an ownership row for handoff, which `P11-HARNESS-SEAM`'s `done_when`
required among its named components and the record does not carry. No item could be opened to
add it, because `add` refused to reserve the record that item would edit. That omission
survived `work finish` for a reason worth keeping: the predicate is a test suite, and a test
suite cannot read a `done_when`.

## The Decision

**The item declares which act it is, and the store refuses on the declaration rather than on
the identifier.**

`FileLedger::Add` takes a second territory beside the published set: the records the item says
it is *editing* rather than allocating. A reservation the item declared is exempt from the
published comparison. A reservation it did not declare is refused exactly as before.

Three consequences follow, and each is a thing the guard must not do.

**It is declared, never inferred.** An identifier carries no evidence of which act is meant, so
anything the store could read off a path would be a convention rather than a rule — and
`OD-LEDGER-013` already refused the shape where the outcome turns on which of two equivalent
spellings an author happened to write. Both spellings work here for the same reason.

**Declaring does not exempt the open-item comparison.** Two items amending one record are two
writers on one file, which is precisely what territory serializes. The declaration answers
*which act this is*, never *whether somebody else is already doing it*. `AddRefusal::RecordReserved`
is unchanged and still refuses an amendment.

**A declaration that names nothing is refused.** `AddRefusal::AmendmentNotPublished` exists
because without it the declaration would be the cheapest possible defeat of the guard: declare
every reservation an amendment and no identifier is ever spent again. It also catches the
ordinary mistake, an author who mistyped a number, who would otherwise get an item that
allocates while claiming to amend. Its exit code is `ValidationError` and not `Conflict` —
nothing is contended, the identifier is free, and an agent told `Conflict` would go looking for
a holder that does not exist.

The refusal that remains says which of the two cases it is. `"A record identifier is allocated
once; choose the next free one"` was the whole of that sentence before, and it is advice that
renumbers a record which should not move whenever the author meant to amend.

## Where The Declaration Lives, And Why Not On The Item

`--amends <record>` reserves what it declares. An author writes the record once, and an item
that declared an amendment without reserving the file it edits is not expressible.

The declaration is passed beside the item and is **not** a `LedgerItem` field. It decides
whether the add is refused and has no reader afterwards, so putting it on the item would add a
field to a document two live sessions share — and `OD-LEDGER-008` prices exactly that: a build
older than a field reads the file, ignores the key it does not know, and writes the document
back without it, at exit 0. A field worth that cost has to be worth reading back, and this one
is not.

What is lost is that the board does not record, afterwards, that an item amended rather than
allocated. That is stated here rather than left to be discovered: `work add` says it on the
success line, once, and nothing keeps it.

## What Was Considered And Rejected

**Remove the guard.** `P11-SPENT-RECORD-ID` measured what its absence cost — three open items
on `OD-LEDGER-020`, two on `OD-LEDGER-021`, five declined and re-authored to clear them,
because there is no `work edit`. The guard is not the defect.

**Let the author reserve the registration file instead of the record.** Both spellings are
refused today, and `OD-LEDGER-016` is the reason: the identifier and the file it names are one
subject. A rule that turned on which of two paths an author happened to write is the kind of
convention this board does not enforce.

**Infer the amendment from the identifier being published.** This is the tempting one, and it
is wrong in one direction that matters: it makes every collision look like an amendment. The
author who reached for a spent number is exactly the author who did not mean to amend, and
under this rule they are told nothing at all.

**Refuse the amendment and require a superseding record instead.** Rejected because it is a
rule about how decisions are written, smuggled in as a rule about a ledger verb. `D-134` was
amended rather than superseded on the merits of the decision, and the board is not where that
question belongs.

## What Holds It

`crates/substrate/nomos-ledger/tests/exclusion_holds.rs`. The acceptance and the contention are
asserted separately rather than through one assertion either could satisfy, because a single
one is satisfied by an `add` that ignores the declaration entirely:

- a published record declared as an amendment is accepted, driven in **both** spellings;
- an amendment of a record another open item reserves is still refused, naming that item;
- an amendment of a record nobody published is refused as such;
- a published identifier reserved *without* a declaration is still refused by its file, which
  is `OD-LEDGER-025`'s own assertion, narrowed to the case it now covers and otherwise
  unchanged.

## Status

Closed. `P11-AMEND-GRAIN` carries it — the third spelling of the item, after
`P11-AMEND-RESERVATION` reserved only the CLI and `P11-AMEND-TERRITORY` reserved `store.rs`
while the sentence it had to change had moved to `add_refusal.rs`. The second of those was
correct when it was authored, and this record's own subject is why it could not be repaired in
place.

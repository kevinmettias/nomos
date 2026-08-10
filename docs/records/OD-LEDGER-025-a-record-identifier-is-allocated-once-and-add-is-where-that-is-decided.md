---
id: OD-LEDGER-025
type: decision
title: A record identifier is allocated once, and add refuses a spent one by naming what spent it
status: closed
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - records
  - concurrency
relations:
  - target: OD-LEDGER-016
    type: relates-to
  - target: OD-LEDGER-021
    type: relates-to
  - target: OD-LEDGER-014
    type: relates-to
---

# A record identifier is allocated once, and add refuses a spent one by naming what spent it

## Question

`work add` refused a duplicate item identifier and nothing else. An author reserving a record
identifier somebody had already published — or one another open item had already spoken for —
was told nothing, and the only guard was remembering to look.

`P10-ADD-PROMISE` closed by narrowing the doc comment that promised a territory check rather
than by adding one, so the promise came out of the prose and the check never went in.

## What Was Measured

Five collisions, and one of them happened to this item.

**Two on published identifiers.** `P10-DERIVED-FACT` reserved `docs/records/OD-ANALYSIS-002`,
published by `0bcaf6d` under `P10-DEPENDENT-EDGE`. `P10-REQUEST-LAYOUT` reserved
`docs/records/OD-SPEC-011`, published by `515fc26` under `P10-VOCABULARY-REFUSAL`. Both items
are Declined now, so no open item carries a spent identifier today; the mechanism that let it
happen was untouched by that.

**Three on unspent ones.** Three open items reserved `OD-LEDGER-020` and two reserved
`OD-LEDGER-021`, each authored by a session taking the next free `OD-LEDGER` number. That
reddened `Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` against the repository's own
board, and with it every predicate naming `-p nomos-ledger`. Clearing the two required
declining and re-authoring four items, because there is no `work edit`. Then, during the
clearing, `P11-DISPATCH-SPLIT` and this item's first reissue both took `OD-LEDGER-022` — one
session's `add` landing between another's board read and its own — and the guard went red a
second time for the same reason.

**The two halves fail differently, and only one is visible to any existing check.** The guard
above reddens on two *open* items sharing an identifier. One open item on a *published* one
excludes nobody, so nothing reddens at all: the author discovers it mid-claim, when there is
no longer a way to move it.

## The Decision

### 1. Two refusals, because there are two remedies

`AddRefusal::RecordPublished { identifier, file }` and
`AddRefusal::RecordReserved { identifier, item }`.

A published identifier is spent forever and the only fix is choosing another number. A
reserved one belongs to an item that may yet be retired, so the author has a second option
that does not exist in the first case. One message for both would send half of each group to
the wrong remedy — the mis-subject `OD-LEDGER-014` measured one verb over.

Each names what spent it rather than only that something did. An author told
"`OD-LEDGER-025` is taken" has to go and find out by what, and the thing that answers that is
a filename or an item identifier.

### 2. The store decides both, and the command layer supplies the repository

`OD-LEDGER-021` put `add` behind one lock and left the command layer its input and its
reporting. Both comparisons are therefore decided in `FileLedger::Add`, inside
`Decide_Under_Lock`, and `Add` takes the published record set as an argument.

The open-item comparison **has** to be there. Two sessions each taking the next free number
read a board without the other's item and are both told it is free, which is exactly how
`OD-LEDGER-022` went twice. A check against a document loaded before the lock is the check
that just failed.

The published comparison does not need the lock — a file on disk is not racing the board —
and it is inside it anyway, because the argument is already in hand and a second code path
that decides the same kind of question differently is worth more than the microseconds.

What the store does **not** do is discover the repository. `nomos-ledger` is a general
exclusion ledger over territories; it already stretches as far as it should by knowing what a
record filename folds to, and teaching it to walk a source tree would make a substrate crate
an authority on this repository's directory layout. So `nomos-cli` enumerates `docs/records`
and hands the result in. A caller with nothing to declare passes an empty territory, which is
honest — it is saying it does not know — and the open-item half still holds.

### 3. Both comparisons go through `Territory::Intersect`

Not a containment rule written beside it. `Intersect` folds a record filename onto the
identifier it carries, which is what makes `docs/records/OD-LEDGER-020` and
`docs/records/OD-LEDGER-020-a-slug.md` one subject without anything in `add` knowing the
grammar. `OD-LEDGER-016` is that decision and this is its second caller.

It is applied one authored path at a time, so the refusal can name which path collided —
`Intersection::Overlaps` carries subject digests, and a digest is not a sentence anybody can
act on.

### 4. Only record identifiers, and only open items

**Only record identifiers.** `add` does not refuse overlapping territory in general and must
not start: items overlap constantly and claims are what serialize them. What is guarded is
the one reservation an author cannot recover from mid-claim.

**Only open items.** A `Done` or `Declined` item's territory is history, in the sense
`Is_Open` already gives that word. Refusing against closed items would make every allocated
number a permanent claim, and since almost every item ever finished reserved a record, the
next author could allocate nothing at all.

### 5. `ExitCode::Conflict`, not `Usage`

The item's `done_when` asked for "a usage refusal rather than a store error", and the
operative contrast is with `StoreError` — this is the author's to fix, not a broken ledger.
Between the two candidates the decision is `Conflict`, the code `AlreadyPresent` already
gives a taken item identifier, because the reason given is the same one: an identifier
somebody else holds, resolved by choosing another.

`Usage` is the parser's code for a malformed invocation. An agent that saw it would go and
inspect its own argument syntax, which is not the fix, and the invocation here is perfectly
well formed. The refusal text is what tells the two record cases apart; the exit code tells
an agent what *kind* of thing happened, and this is a kind that already had one.

## What Was Considered And Rejected

**Enumerating `docs/records` inside the store.** It would put both halves in one place, and
`FileSystem` has no directory listing to do it with — `Read_To_String`, `Replace_Atomically`
and `Exists` are the whole port. Adding one is a band-15 change, and the reason not to is not
that it was out of this item's territory: it is decision 2's, that a general ledger should not
become a repository scanner.

**Probing the registration file instead.**
`crates/spec/nomos-spec-store/records/<ID>.record` is an exact path, so `Exists` alone would
answer without any listing. Rejected: it would hardcode one crate's directory into
`nomos-ledger`, which is a worse coupling than the one it avoids, and it would miss a record
file written but not yet registered.

**Refusing against closed items too.** Decision 4. It reads as the safer choice and refuses
the whole board.

**A doc comment telling authors to check.** `P10-ADD-PROMISE` already did that, and the five
collisions above all happened afterwards.

## What Holds It

Six tests in `crates/substrate/nomos-ledger/tests/exclusion_holds.rs`, of which three are
controls:

- a published identifier is refused and the **file** is named;
- an identifier another open item reserves is refused and the **item** is named, including
  when the two authors spelled it differently — one as the bare identifier, one as the file
  they were about to write;
- the two refusals are different values *and* different sentences;
- **an unspent identifier is still accepted**, beside published ones — including
  `OD-LEDGER-0071` alongside a reservation of `OD-LEDGER-007`, because an ordinal is compared
  as a whole component and a prefix rule would say `007` was taken;
- **a closed item's reservation reserves nothing**, for both `Done` and `Declined`;
- **ordinary shared territory is still accepted**, which is the guard against this check
  quietly becoming the territory refusal `add` deliberately does not make.

No item on the board carries a spent or duplicated identifier as this lands, so the check is
proved by its own tests rather than by what it happens to catch today.

## Status

Closed by P11-ADD-IDENTIFIER-GUARD, which is `P11-SPENT-RECORD-ID` reissued twice — once for
`OD-LEDGER-020` and once for `OD-LEDGER-022`, both collisions of the kind it exists to
prevent.

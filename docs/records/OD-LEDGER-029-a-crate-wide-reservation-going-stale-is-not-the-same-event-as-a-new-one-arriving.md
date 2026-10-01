---
id: OD-LEDGER-029
type: decision
title: A crate-wide reservation going stale is not the same event as a new one arriving
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-LEDGER-011
    type: relates-to
  - target: OD-LEDGER-028
    type: relates-to
---

# A crate-wide reservation going stale is not the same event as a new one arriving

## Question

`OD-LEDGER-028` named its own expiry condition in advance: `crates/host/nomos-cli`'s entry
in `KNOWN_SERIALIZERS` would go stale "either open item reaching `Done`, if the other by
then no longer reserves the whole crate." `P10-VACUITY-HOME` reached `Done` while
`P11-NEXT-WORK` was in progress, leaving `P10-SERVICE-SEAM` as the only open item still
reserving the whole crate — one, not two — and
`Test_Every_Declared_Serializer_Should_Still_Serialize` went red exactly the way that
record predicted it would.

A second, unrelated fact arrived in the same window. `P11-NEXT-WORK`, in progress on the
same board, reserves two specific files rather than a whole crate:
`crates/host/nomos-cli/src/work.rs`, the ledger CLI dispatcher its `next:` line is added
to, and `tests/contract/surface/nomos-ledger.txt`, the one public-surface snapshot its new
export changes — `OD-LEDGER-011`'s own settled shape for that territory. Both collide with
`P10-SERVICE-SEAM`'s whole-crate and whole-directory reservations, for the reason
`OD-LEDGER-028` already accepted: an item asking whether the crate needs an adapter seam
cannot yet name which file a not-yet-chosen implementation would touch. Because
`P11-NEXT-WORK` is the only other currently-open record writer with satisfied
dependencies, both collisions are universal by
`Test_Every_Universal_Reservation_Should_Be_Declared`'s own definition — reserved by every
open record writer, not merely contended between two of them.

## Decision

**All three facts are decided together, because they are one register and one commit, not
three.**

`crates/host/nomos-cli`'s existing entry is removed. Its coupling does not resolve into
ordinary contention — `P10-SERVICE-SEAM` alone reserving the whole crate is one item's
territory, which is what territory is for, not a structural serializer. Restated only if a
second open item again reserves the crate whole with no narrower file to name.

Two entries replace it, one per file `P11-NEXT-WORK` actually touches inside
`P10-SERVICE-SEAM`'s two broad reservations: `crates/host/nomos-cli/src/work.rs` and
`tests/contract/surface/nomos-ledger.txt`.

**Each is keyed to the narrower of the two colliding paths, not the directory or crate
`P10-SERVICE-SEAM` names.** `Covers` — the direction
`Test_Every_Declared_Serializer_Should_Still_Serialize` checks — is coarser-reserves-finer:
a reservation `Covers` a declared path only when its own normalized length is no longer
than the declared path's. `P10-SERVICE-SEAM`'s whole-crate reservation covers a narrow
declared file; a narrow declared file trivially covers itself when `P11-NEXT-WORK`'s own
territory is exactly that file. Declaring the *directory* instead would have covered
`P10-SERVICE-SEAM` the same way but not `P11-NEXT-WORK`, whose own path is longer than the
directory it sits inside — `Reserving` would then count one writer, not two, and the entry
would read as declared-but-not-serializing the moment it was written. This asymmetry is
deliberate: `OD-LEDGER-011`'s own history is why `Covers` is directional rather than the
symmetric `Paths_Collide` the universal-reservation search uses — a symmetric reading would
count a narrow reservation as forever reserving the directory around it, and the register
could never empty by narrowing, only by an item disappearing.

## What This Costs

Unchanged from `OD-LEDGER-007` and `OD-LEDGER-028`: every record-writing item whose
verification predicate reaches `cargo test -p nomos-ledger` carries this exclusion until it
clears. What is new here is the register changing composition rather than only growing —
the first time an entry born of one pairing has gone stale while a second, unrelated
pairing arrived inside the same commit's window, and the first time an entry is keyed to
one side's narrow file rather than the shared crate or directory name, because that is what
`Covers`'s own direction requires when the two colliding territories are not the same
spelling.

## What Was Considered And Rejected

**Narrowing `P11-NEXT-WORK`'s own territory instead of declaring the coupling.** Available
here in a way it was not for `OD-LEDGER-007` or `OD-LEDGER-028` — this record's author
holds `P11-NEXT-WORK` and could narrow it without touching anybody else's open item.
Rejected because there is nothing left to narrow: both files already name the one thing
each part of the item's work touches, the same shape `OD-LEDGER-011` settled as correct
for a public-surface snapshot. The collision is not this item's territory being wider than
its work; it is `P10-SERVICE-SEAM`'s being exactly as wide as `OD-LEDGER-028` already
accepted a not-yet-implemented architectural question needs to be.

**Declaring `crates/host/nomos-cli` and `tests/contract/surface` — the directory spellings
— instead of the two narrow files.** Tried first and found to reproduce
`Test_Every_Declared_Serializer_Should_Still_Serialize`'s failure one layer over: `Covers`
does not hold in `P11-NEXT-WORK`'s direction for a path longer than the one declared, so a
directory-keyed entry counts only `P10-SERVICE-SEAM` and reads as stale immediately. The
narrow-file keying is not a style choice; it is what makes the entry actually still
serialize under the check that asks.

**Leaving the stale entry in place until `P10-SERVICE-SEAM` itself resolves.** Rejected
because `Test_Every_Declared_Serializer_Should_Still_Serialize` is the control that exists
precisely to catch an over-reporting register, and leaving a known-stale entry in place to
avoid a second edit is the failure mode that test was built against, restated as a
convenience.

**Waiting for the two new collisions to resolve on their own, the way ordinary two-item
contention does.** They will not while both items stay open: each is universal across every
currently-open record writer with satisfied dependencies, which by the register's own
definition is a rule, not contention, and a rule does not resolve when one party finishes
— the next record writer to arrive with a file under either path inherits it too.

## Controls

| Weakening | What it produces |
|---|---|
| leave `crates/host/nomos-cli` declared | `Test_Every_Declared_Serializer_Should_Still_Serialize` fails, an over-reporting register `OD-LEDGER-011`'s own history warns against |
| leave either new collision undeclared | `Test_Every_Universal_Reservation_Should_Be_Declared` fails, the exact defect this register exists to catch |
| declare the directory spelling instead of the narrow file | `Test_Every_Declared_Serializer_Should_Still_Serialize` fails again, one layer over: `Covers` does not hold in `P11-NEXT-WORK`'s direction for the coarser path |
| narrow `P11-NEXT-WORK` below the one file each part of its work touches | a territory authored to guess at independence rather than to measure it |

## What Would Make This Stale

The same condition `OD-LEDGER-028` named, applied to each new entry: `P10-SERVICE-SEAM` or
`P11-NEXT-WORK` reaching `Done` while the other no longer reserves the colliding path, or a
third open item forcing the same path, which changes an entry's reason rather than
removing it.

## Status

Closed by `P11-CRATE-SERIALIZER-4`.

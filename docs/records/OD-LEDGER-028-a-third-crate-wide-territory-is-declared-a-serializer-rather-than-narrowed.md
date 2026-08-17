---
id: OD-LEDGER-028
type: decision
title: A third crate-wide territory is declared a serializer rather than narrowed
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
---

# A third crate-wide territory is declared a serializer rather than narrowed

## Question

`OD-LEDGER-007` established the register: a path two record writers share is contention,
which territory is for and which resolves when one of them finishes; a path *every* open
record writer shares is a rule, and it does not resolve on its own, so `KNOWN_SERIALIZERS`
in `crates/substrate/nomos-ledger/tests/records_do_not_serialize/serializers.rs` names it
with what forces it. `OD-LEDGER-011` closed the two entries that register opened with, and
the file's own doc comment since then has read the empty list as a state worth naming
rather than an absence.

`Test_Every_Universal_Reservation_Should_Be_Declared` went red with no code change:
`crates/host/nomos-cli` is reserved by every currently open record-writing item, and the
register does not say so. Confirmed against a fully committed tree — every uncommitted
change in the working copy at the time, `stash`ed away, still reproduced the same failure
— so this is not a symptom of anybody's in-flight work. It is the board.

Two open items force it: `P10-VACUITY-HOME` and `P10-SERVICE-SEAM`, both reserving
`crates/host/nomos-cli` whole. Both are asking a question about the crate's own shape.
`P10-VACUITY-HOME` asks where a guarantee that must hold for every judging command in this
crate should live — across `check.rs`, `spec.rs`, `work.rs` and whatever the next command
adds. `P10-SERVICE-SEAM` asks whether the crate needs a seam between choosing a platform,
running a verb and rendering an outcome, at all six of its current modules and whichever
carries the answer. Neither item has chosen an implementation. Naming a narrower file for
either today would not measure the item's actual scope; it would guess at one, and the
guess could not be corrected later — territory has no `work edit`.

## Decision

**`crates/host/nomos-cli` is declared in `KNOWN_SERIALIZERS`, naming both items and why
each reserves the whole crate rather than a file in it.**

Chosen over narrowing `P10-VACUITY-HOME`'s or `P10-SERVICE-SEAM`'s territory, which
`OD-LEDGER-007` already refused to do to unblock a different pair, for the same reason:
narrowing somebody else's open item's territory is a decision made about work that is not
this item's, on behalf of a holder who has not made it. It is also, here, not obviously
correct — both items' own `why` sections describe questions the whole crate is the honest
scope of, not territory authored wider than the work needs.

The register's own guidance, in the test that found this — "add it with what forces it, or
remove the coupling" — names both paths as available. This record takes the first because
the second is not available cheaply: removing it means one of two open items answering an
architectural question before it has done the work that answers it.

## What This Costs

Every record-writing item whose verification predicate reaches `cargo test -p nomos-ledger`
carries this exclusion until it clears. That is not new — `OD-LEDGER-007` already named the
same shape for the first two entries and called it the board's debt rather than a defect in
the mechanism. What is new is that the register briefly read as empty and was not; a
contributor who trusted the doc comment's silence over running the suite would have found
this the way two sessions did, independently, roughly twelve hours apart.

## Consequences

`crates/substrate/nomos-ledger/tests/records_do_not_serialize/serializers.rs`'s
`KNOWN_SERIALIZERS` carries one entry, and its file-level doc comment stops describing the
register as empty.

## What Holds It

`Test_Every_Universal_Reservation_Should_Be_Declared` and
`Test_Every_Declared_Serializer_Should_Still_Serialize`, the same two tests `OD-LEDGER-007`
named, run over the repository's real board rather than a fixture.

## What Would Make This Stale

Either open item reaching `Done`, if the other by then no longer reserves the whole crate.
Either open item being re-authored to a narrower territory by whoever holds or next claims
it — a decision this record does not make and is not the one to make. Or a third open item
forcing the same path, which changes the entry's reason rather than removing it.

## What This Record Does Not Decide

It does not decide whether `P10-VACUITY-HOME` or `P10-SERVICE-SEAM` is right to reserve the
whole crate as a matter of good territory authoring — only that neither is the accident
`OD-LEDGER-011` closed two of, and that deciding otherwise is not this item's to make.

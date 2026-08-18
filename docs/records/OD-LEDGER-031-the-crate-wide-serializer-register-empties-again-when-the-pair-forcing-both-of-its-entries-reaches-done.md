---
id: OD-LEDGER-031
type: decision
title: The crate-wide serializer register empties again when the pair forcing both of its entries reaches Done
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
  - target: OD-LEDGER-029
    type: relates-to
---

# The crate-wide serializer register empties again when the pair forcing both of its entries reaches Done

## Question

`OD-LEDGER-029` declared two `KNOWN_SERIALIZERS` entries — `crates/host/nomos-cli/src/work.rs`
and `tests/contract/surface/nomos-ledger.txt` — each forced by `P10-SERVICE-SEAM` and
`P11-NEXT-WORK` together, both open at the time the record was written. `P10-SERVICE-SEAM`
was since abandoned and re-authored as `P10-SERVICE-SEAM-2`, which reached `Done`;
`P11-NEXT-WORK` reached `Done` in a separate window. Neither of the two items the entries
name is open any longer, and `Test_Every_Declared_Serializer_Should_Still_Serialize` fails
on both entries, for the exact condition `OD-LEDGER-029`'s own "What Would Make This Stale"
section named in advance: "`P10-SERVICE-SEAM` or `P11-NEXT-WORK` reaching `Done` while the
other no longer reserves the colliding path."

Confirmed against a fully committed HEAD (`5901f2a`) with `nomos work show` for both items
and no uncommitted tree changes: `P10-SERVICE-SEAM` reads `declined` (abandoned, re-authored
as `P10-SERVICE-SEAM-2`), and `P10-SERVICE-SEAM-2` and `P11-NEXT-WORK` both read `done`.

## Decision

**Both entries are removed, and nothing replaces them.**

Unlike `OD-LEDGER-029`, which paired one entry's removal with two arriving in the same
window, no third open item currently forces either path — the board holds zero `Ready` or
`Claimed` items, so no open record writer collides with anything. The register empties to
nothing, the direction `OD-LEDGER-007` named as its intended trajectory and the state
`crates/substrate/nomos-ledger/tests/records_do_not_serialize/serializers.rs`'s own doc
comment described once before, between `OD-LEDGER-011` and `OD-LEDGER-028`.

## What This Costs

Nothing new. An entry leaving the register removes an exclusion rather than adding one;
every item whose predicate reaches `cargo test -p nomos-ledger` is freed of this particular
coupling. `OD-LEDGER-007`'s framing holds: the register is a debt list, and this is a
repayment, not a new debt.

## What Holds It

`Test_Every_Declared_Serializer_Should_Still_Serialize`, the same test `OD-LEDGER-029`
named, run over the repository's real board. `Test_Every_Universal_Reservation_Should_Be_Declared`,
`Test_An_Undeclared_Serializer_Should_Be_Found` and
`Test_A_Run_Should_Report_Whether_The_Board_Is_Parallel` are not evidence for or against
this decision: all three currently fail independently of it, because the board holds zero
open record writers at all — the `writers.len() >= 2` guard is a board-timing fact rather
than a code or record defect, the same shape `OD-LEDGER-030` named for a different pair of
tests, and it is unrelated to whether these two entries still serialize.

## What Would Make This Stale

A pair of open items again forcing `crates/host/nomos-cli/src/work.rs`,
`tests/contract/surface/nomos-ledger.txt`, or any other path every open record writer
shares — the same condition that produced the entries this record removes.

## Status

Closed by `P13-SERIALIZER-REGISTER-EMPTY`.

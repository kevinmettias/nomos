---
id: OD-CONNECTOR-002
type: decision
title: A connector fixture is recorded below its own translation, and an absent one fails as loud as a missing corpus
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ecosystem
  - connectors
  - evidence
  - verification
relations:
  - target: ARC-CONNECTOR-001
    type: relates-to
  - target: OD-CONNECTOR-001
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
---

# A connector fixture is recorded below its own translation, and an absent one fails as loud as a missing corpus

## Question

`ARC-CONNECTOR-001` draws the third crossing and names four invariants a connector's
interface must hold, but a connector is not only an interface; it is also the first thing in
this workspace whose only way to be observed while developing or testing it is a live
external endpoint, credentials, and whatever the vendor happens to return on the day the
test runs. Nothing already governing says what a connector test is allowed to run against
instead, where a recording is taken relative to the vendor-to-canonical translation
`ARC-CONNECTOR-001` already places in the seam, what a replayed pass is entitled to claim in
`EvidenceClass` terms, or what happens when a test wants a recording that is not there.

`OD-GATE-001` already answered the second and fourth questions once, for a different corpus:
a test that cannot find its input must not report `ok`, because a check that cannot find its
subject and a check that agrees with it are otherwise indistinguishable, and CI holds none of
the three corpora that decision was written against. A connector test that cannot reach a
live vendor, or cannot find a recording standing in for one, is the same shape a fourth time,
and the wrong default — return early, print `ok` — is exactly as available here as it was
there. Deciding this before the first connector exists is the same reasoning
`ARC-CONNECTOR-001` gave for itself: the seam where a recording would be taken is the seam
nobody has drawn yet, and the first connector written against a live endpoint would draw it
by not needing to.

## Decision

**A vendor interaction is recorded and replayed, and the recording is taken below the
vendor-to-canonical translation** — on the vendor side of the seam `ARC-CONNECTOR-001`
already draws, not the canonical side. Concretely: the recording is the vendor's own
response, in vendor schema, exactly as the generic connector substrate received it before
handing it to the translation layer — not the `Finding` or canonical fact the translation
layer produces from it.

The reason is the two-sided failure `ARC-CONNECTOR-001`'s layering already implies but does
not have to say once record-and-replay is named. A fixture taken **above** the translation —
a stored canonical fact, standing in for what the translation layer would have produced —
cannot detect a defect in the translation itself, because replaying it never runs the
translation at all; it only exercises whatever reads the canonical fact afterward, which
`ARC-ECOSYSTEM-001` already governs and which is not what a connector test exists to check.
A fixture taken **below** the translation — the vendor's own response — is read by the
translation layer on every replay exactly as it would read a live response, so a defect
introduced in that layer is caught the same way it would be caught against the live vendor,
and a vendor API version migration is checked by re-recording the fixture and running the
same translation over it, which is one of the reasons this record exists to name in advance.

Recording below the translation means the fixture is, unavoidably, in vendor wire format —
the vendor's field names, status enumerations and response shape, exactly as
`ARC-CONNECTOR-001`'s second invariant already says only the translation layer may know.
This is not a new exposure. The fixture is data the translation layer's own tests hold, read
only by that layer, in the same way `ARC-CONNECTOR-001` already permits that layer and no
other to import the vendor's schema types; a recorded vendor response sitting in that
layer's test fixtures is the same permission applied to a stored request instead of a live
one. Nothing above the translation layer may hold, name, or branch on the fixture's vendor
field names any more than it may hold the vendor's SDK object under `OD-CONNECTOR-001`; the
fixture format is confined to exactly the code the vendor schema itself was already confined
to.

**A replayed pass is `Verified` evidence that the translation layer maps this specific,
previously-captured vendor response to the canonical fact the recording is paired with, and
it is not `Observed` evidence, or any other evidence, about the live vendor system at the
time the test ran.** `Verified`, per
`crates/contracts/nomos-contracts/src/finding/evidence.rs`, is "checked by a mechanism that
would have detected the negation" — replay is exactly that mechanism for the translation
code: if the translation regressed, the same recorded input would produce a different
canonical fact than the one the test expects, and the test would fail. That is the whole
claim a green replay makes. It is not `Observed` — "directly observed at runtime" — because
nothing about a replayed test observes the vendor at runtime; the vendor's response was
observed once, when the recording was taken, and every replay since checks this build's
translation code against a fixed, historical input, not the vendor's current behavior. A
suite of green connector tests must never be read as a live-system health check, as
confirmation that the vendor's API still answers the same way today, or as a substitute for
the `Observed` evidence `ARC-CONNECTOR-001`'s third invariant already assigns to state
produced by an actual live call. The two are different claims, and this record keeps them
different so a passing replay is never later cited for the one it does not make.

**A connector test that cannot find its recording must fail, not report `ok`.** This is
`OD-GATE-001`'s decision restated in this setting rather than assumed to carry into it: a
test that returns early because its fixture is absent is indistinguishable, from its exit
code alone, from a test that read the fixture and agreed with it, and CI will not have a
live vendor endpoint, a token, or the fixture corpus by default any more than it has the
three corpora `OD-GATE-001` was written against. Where `OD-GATE-001` chose not to fail the
build for an absent v14, archive, or Rust corpus — because most machines will never have
them, and a test that failed for their absence would make every gate red by default — a
connector's fixture corpus is not that case: it ships inside this repository's own tree, the
same way any other test fixture does, so its absence on a machine that should have it is a
defect in that checkout or that commit, not an expected condition of an ordinary
contributor's machine. A missing connector fixture is a hard failure, not a silent `ok` and
not a declared, tolerated hole.

## What This Binds

Every fixture a connector's tests replay is a captured vendor response, taken below the
vendor-to-canonical translation layer `ARC-CONNECTOR-001` places in the seam, so that replay
exercises the translation on every run.

A replayed test's pass is `Verified` evidence about the translation layer against the
fixture it replayed, and nothing stronger; it is never read, cited, or reported as `Observed`
evidence about the live vendor system, and never as evidence that the vendor's current
behavior matches the recording.

A connector test that cannot find the fixture it needs fails loudly. It does not return
early, and it does not print `ok`.

## What This Does Not Bind

It does not define a fixture file format, a recording tool, a corpus-gate table for
connector fixtures, or where fixtures live in the tree. Those are implementation the first
connector's tests are checked against, the same way `ARC-CONNECTOR-001` left its quarantine
crate's name and `OD-CONNECTOR-001` left its canonical service's name to whoever builds one.

It does not decide whether a live-endpoint integration test may exist alongside the replayed
suite, opt-in and excluded from the default run. Nothing here forbids one; this record only
says a replayed test's pass is not entitled to stand in for what a live test would show, and
that the default suite must not silently skip.

It does not touch `ARC-CONNECTOR-001`'s four invariants or `OD-CONNECTOR-001`'s
write-omission rule. Both are unchanged; this record adds the recording seam and the
evidence and absence rules beside them.

## Controls

| Weakening | What it produces |
|---|---|
| the fixture is a stored canonical fact rather than a vendor response | replay never runs the translation, and a translation defect passes every time |
| a replayed pass is reported or read as `Observed` evidence about the vendor | a stale or synthetic fixture is cited as proof the live system currently behaves that way |
| a connector test returns early and prints `ok` when its fixture is missing | the same silent hole `OD-GATE-001` measured at 68 tests, reopened for a fourth corpus this repository controls and has no excuse for losing |
| the fixture format leaks vendor field names into a canonical type above the translation | the same leak `ARC-CONNECTOR-001`'s second invariant already refuses, reached through the fixture instead of the live response |

## Conflicts With Existing Decisions

`ARC-CONNECTOR-001` is untouched. This record places the recording seam inside the layering
it already draws and applies its second invariant to the fixture the same way that invariant
already applies to a live response; the four invariants themselves are unchanged.

`OD-CONNECTOR-001` is untouched. It governs the outward direction of this seam; this record
governs the inward one, and neither reopens the other.

`OD-GATE-001` is untouched. Its decision — declare the size of a hole rather than fail a
build that cannot reach a corpus outside this repository — is not reopened. This record
draws the opposite conclusion for a fixture corpus that ships inside this repository's own
tree, which is the distinction `OD-GATE-001`'s own reasoning already turns on: absence is
loud precisely when the subject was supposed to be there.

## What This Record Does Not Do

No connector is built, no fixture is recorded, and no recording tool or format is written.
This record does not claim any fixture exists today or that a corpus-gate table for one is
in place; it only settles, before the first connector's tests are written, where the
recording is taken, what a replayed pass may claim, and what an absent fixture must do, so
that none of the three is decided by whichever test is first instead of by this record.

## Status

Closed by `P12-CONNECTOR-REPLAY`, which placed the recording below the translation layer
and set the evidence and absence rules around it. It discharges the fixture-and-evidence
question for connector tests ahead of the first connector; a fixture file format, a
recording tool and a corpus-gate table remain for that first connector's own tests to
define.

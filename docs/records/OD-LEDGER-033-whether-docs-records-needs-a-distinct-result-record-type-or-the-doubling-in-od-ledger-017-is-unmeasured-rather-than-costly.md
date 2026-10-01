---
id: OD-LEDGER-033
type: decision
title: Whether docs/records needs a distinct result-record type, or the doubling in OD-LEDGER-017 is unmeasured rather than costly
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - records
  - traceability
relations:
  - target: OD-LEDGER-017
    type: relates-to
  - target: OD-LEDGER-006
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
---

# Whether docs/records needs a distinct result-record type, or the doubling in OD-LEDGER-017 is unmeasured rather than costly

## Question

An external design review (`nomos_spec_and_work_ledger_arch.txt`, read against this
workspace rather than accepted on read) proposes splitting a completed unit of work into
four artifact kinds — task, change record, architecture-decision record, and result
record — so that a durable decision (why something is true), a semantic delta (how the
intent moved and why), and a delivery report (what was actually built, against which
commit) never share one file. `docs/records/` has one type today: a governing record,
`type` drawn from the same closed set `LedgerItem::kind` uses (architecture, decision,
correction, validation, cleanup), that carries decision content and delivery narrative in
the same file whenever the two coincide. `OD-LEDGER-017` is the concrete instance named by
`P13-RESULT-RECORD-QUESTION`: it states a decision (all six `WORK-LEDGER` requirements
bind), narrates what was measured (216 v14 task files, the review that produced the six
requirements), and reports a verdict per requirement — in one record, one `status`, one
`version`.

Whether that doubling is a real cost or a distinction without a difference was unmeasured
before this record. No prior record here argued either way.

## What Was Measured

**Revision churn, across the whole set.** `docs/records/*.md` carries a `version` field
per `D-129`'s front matter, and it is not always `1`:

| `version` | Count |
|---:|---:|
| 1 | 107 |
| 2 | 27 |
| 3 | 4 |
| 4 | 2 |
| 5 | 1 |

34 of 141 records — about a quarter — have been revised at least once. Two records sit at
`version: 3` and are load-bearing for this question specifically: `OD-GATE-014` and
`OD-GATE-015`, the ScopeSelector/RuleSelector and BaselinePolicy/SuppressionPolicy
open-decision records `ARC-ROADMAP-001`'s near-term tier depends on. Both are
"is this built now, or does it wait for a trigger" records, and both have been revised
twice — not because the decision changed, but because a `Correction`-kind item updated
their `Status` section each time a landing changed which trigger had fired.
`P13-GATE-014-015-RUN-CALLER-LANDED` is one such correction: it left `OD-GATE-014`'s
decision exactly where it stood (ScopeSelector still waits) and rewrote only the sentence
recording that `OD-GATE-015`'s trigger had newly fired.

**This is exactly the doubling the reviewed design names**, and it is real: these two
records' narrative half moves on a different schedule than their decision half. The
question is whether the current mechanism for that movement — a version-bumped edit to
the same file — is a cost worth a second artifact type, or is already the cheapest correct
answer.

**What the alternative would cost.** A separate result/change-record type would not remove
the need to touch something every time a trigger fires; it would move that touch from a
`Status` paragraph in the existing record to an entry in a second, append-only file, and
require both to be read together to answer "is this decision still open, and what has
happened toward its trigger." The record file's own `git log -p` already **is** that
append-only history — `OD-LEDGER-033-...`'s own predecessor sentences are recoverable from
`OD-GATE-014`'s three committed revisions without a second file existing to hold them. The
brainstorming document's own later section concedes as much for its own proposal ("Git
remains authoritative for... exact line changes... patch content"), and the two-file design
it argues for is aimed at a system where the durable source is *not* version-controlled
prose — which `docs/records/` already is.

**What corrections actually cost, measured.** The `Correction`-kind items that perform
these status updates are narrowly territoried — one or two files, a specific `Status`
paragraph — and they finish in the same shape as any other item: claim, edit, verify,
finish. Nothing in the ledger shows a correction blocked, delayed, or made unsafe by
sharing a file with the decision it updates. `OD-LEDGER-006` already generalized the one
real defect in this neighborhood — a single-valued field silently overwriting prior
history (`abandoned`, `displaced`) — to a list, precisely so that repeated events on one
subject do not lose earlier ones. The same fix, applied to a record's own body, is already
what a version-bumped git history gives for free.

## Decision

**Declined.** No separate result-record or change-record type is built. `docs/records/`
keeps its single governing-record type, and a record whose narrative half moves faster
than its decision half — `OD-GATE-014` and `OD-GATE-015` are the two instances measured
here — continues to absorb that movement as a versioned edit to its own `Status` section,
recoverable through `git log` rather than through a second committed artifact.

This is not a claim that no record will ever need independent narrative and decision
histories. It is a claim that the 34-of-141 revision rate measured here, concentrated in
two records revised twice each for a stated and legible reason, is evidence of the
mechanism working rather than evidence of it failing.

**What would reverse this.** A decision record whose narrative half needs to be read
*without* also reading its current decision — for example, because the decision changed in
a way that makes an earlier narrative entry actively misleading rather than merely
superseded, and `git log` is not how downstream readers (agents building a projection, or
`nomos spec render`) are expected to reach the store. Or a record accumulating enough
narrative revisions that its `Status` section stops being a paragraph and starts being a
log a reader has to reconstruct by eye across several `version`s — a shape `OD-LEDGER-001`
already reached at `version: 5`, which is the nearest live candidate to watch.

## Status

Accepted, drawn by `P13-RESULT-RECORD-QUESTION` against evidence in this repository's own
record set, not against the reviewed design's abstract case for the split. Revisit on
either trigger named above.

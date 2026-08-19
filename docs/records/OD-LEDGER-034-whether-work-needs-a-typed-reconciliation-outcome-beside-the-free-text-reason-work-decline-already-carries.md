---
id: OD-LEDGER-034
type: decision
title: Whether work needs a typed reconciliation outcome beside the free-text reason work decline already carries
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - reconciliation
  - traceability
relations:
  - target: OD-LEDGER-017
    type: relates-to
  - target: OD-LEDGER-019
    type: relates-to
---

# Whether work needs a typed reconciliation outcome beside the free-text reason work decline already carries

## Question

An external design review (`nomos_spec_and_work_ledger_arch.txt`) recommends a typed
reconciliation-outcome vocabulary — `not_started`, `partially_implemented`,
`implemented_unverified`, `implemented_verified`, `superseded`, `not_applicable` — for a
task claimed against a repository where the requested behavior turns out to already
exist. Its argument: collapsing "already implemented" into either `done` (asserting
verification that did not happen) or silent abandonment loses information a later reader
needs.

This repository's nearest mechanism is `work decline`, whose own CLI text says it "ends an
item that turned out not to be work — superseded by another item, or refused by a record
since it was written" (`OD-LEDGER-019` is the record that split it from `abandon` on
exactly that ground), carried only in a free-text `--reason`. Grepping the workspace for
`reconciliation`, `already_implemented`, `partially_implemented` finds matches only in
`nomos-spec-ingest`'s v14/v15 corpus-overlay reconciliation
(`crates/spec/nomos-spec-ingest/src/reconciliation/`) — a different domain, restoring lost
document families from an archived corpus, not a ledger item discovering pre-existing
code. Nothing in `nomos-ledger` or `nomos-cli` carries this vocabulary.

## What Was Measured

Every `Declined` item on the board, read in full: **61 items**, `state.Declined.reason`
read for each one.

**Zero cite "the requested behavior already existed in the codebase."** None. The reasons
cluster into a small number of recurring, already-named failure modes instead:

| Pattern | Approximate count | Example |
|---:|---:|---|
| Territory mis-scoped or found short mid-claim, re-authored under a new id | ~28 | `P10-DECLINED-DEPENDENCY-2`, `P13-PACKAGE-MANIFEST` |
| Superseded by a reissue of the same subject (authoring error, corrected predicate, record-id collision) | ~20 | `P11-SPENT-RECORD-ID`, `P12-DUPLICATE-AUTHORITY-2` |
| Verification predicate unsatisfiable or missing, item re-authored with a working one | ~9 | `P13-NAMING-V15-CASING`, `P12-NO-PRIVILEGED-SURFACE-2` |
| Genuinely not work (a probe, a test artifact) | 1 | `T-PROBE-FREE` |
| Dependency on an item that itself declined | ~3 | `P10-EDGE-CONSTRAINTS`, `P12-NO-PRIVILEGED-SURFACE` |

The scenario the reviewed vocabulary targets — a task-ledger stood up against a
*pre-existing, previously untracked* codebase, so early claims routinely discover work
that already happened before the ledger existed — is the shape `OD-LEDGER-017` measured in
the *v14* Go-era task set (216 tasks, migrated onto code with real prior history). It is
not this ledger's shape: `work/ledger.json` was built incrementally from `P1-MODEL`
forward, alongside the code each item's `done_when` describes, so there is structurally no
period during which capability could accumulate untracked ahead of the board. The nearest
thing this board has to "discovered something already true" is a `Correction`-kind item
finding a *stale claim* in a record or comment — which already has a typed home, the
`Correction` `kind` itself, and a `why` that states what was found wrong.

## Decision

**Declined.** No typed reconciliation-outcome field is added to `work decline` or
`work finish`. The vocabulary answers a reconciliation problem — pre-existing,
previously-untracked implementation discovered under a tracked task — that has not
occurred on this board in 61 measured instances, and does not have a structural reason to
occur given how this ledger was built.

**What would reverse this.** A real instance of an item claimed whose `done_when` is then
found already satisfied by code nobody tracked to a prior item — most plausibly if a whole
subsystem were ever imported wholesale (a vendored crate, an acquired codebase, a merge of
a long-lived side branch) rather than built item-by-item as this workspace has been so
far. Until that occurs, `work decline`'s free-text `--reason` is sufficient because the
population it would need to distinguish between has exactly one measured member.

## Status

Accepted, drawn by `P13-RECONCILIATION-OUTCOME-QUESTION` against a full read of the 61
`Declined` items on the board at the commit this record lands on, not against the reviewed
design's general case for the vocabulary.

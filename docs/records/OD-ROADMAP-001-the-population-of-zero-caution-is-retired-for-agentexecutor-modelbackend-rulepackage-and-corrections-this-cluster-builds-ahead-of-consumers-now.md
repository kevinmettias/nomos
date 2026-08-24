---
id: OD-ROADMAP-001
type: decision
title: The population-of-zero caution is retired for AgentExecutor, ModelBackend, RulePackage and corrections; this cluster builds ahead of consumers now
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - roadmap
  - package
  - corrections
  - agent
  - model
  - architecture
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
  - target: OD-PACKAGE-010
    type: relates-to
  - target: OD-PACKAGE-011
    type: relates-to
  - target: OD-PACKAGE-012
    type: relates-to
  - target: OD-CORRECTIONS-001
    type: relates-to
---

# The population-of-zero caution is retired for AgentExecutor, ModelBackend, RulePackage and corrections; this cluster builds ahead of consumers now

## Question

This workspace has repeatedly applied the same caution to the same cluster of decisions:
before typing a schema for a manifest, a routing shape, a task envelope, or a
classification layer, wait for a second real instance to check the shape's field
boundaries against, so the shape is transcribed from something real rather than invented.
`OD-PACKAGE-006` named it first, for `KNOWN_PROVIDERS`. `OD-PACKAGE-008` tested it against
a real second and then a real fourth rule and found it load-bearing for `RulePackage`.
`OD-CORRECTIONS-001` applied it to `CorrectionCandidate` generation and classification.
`OD-PACKAGE-010` was built only after the user overrode it once, explicitly and narrowly,
for `ModelBackendPackage`/`AgentExecutorPackage`'s first manifest maturity alone.
`OD-PACKAGE-011` and `OD-PACKAGE-012` then re-applied the same caution to everything the
first override did not name — routing, dispatch, task envelopes, capability-class
authorization — each time correctly, each time requiring the user to intervene again to
get past it.

That pattern is itself the defect this record exists to end. A caution that has to be
overridden by the same person for the same cluster of work, one record at a time, on
every session that reaches it, is not protecting anything the first override did not
already settle — it is a cost paid repeatedly for a decision made once. The user said so
directly: this has been a recurring friction, and the standing instruction is to stop
re-deriving it. This record is that standing instruction, recorded so a future session
reads it instead of re-litigating the caution from the affected records' own older text.

## The Decision

**For the AgentExecutor / ModelBackend / RulePackage / corrections cluster named below, do
not wait for a second real consumer, a second real instance, or a real case to check a
shape against before building it.** Build the shapes these records named and deferred —
task envelopes, `PrepareChangeContext`, `ContextualizeFinding`, model routing profiles and
selectors, `RulePackage`'s manifest and lifecycle, `CorrectionCandidate` generation and
classification, capability-class authorization and audit — directly, from the corpus
requirement text and this workspace's own existing types, the same way every other piece
of this workspace was built before a population-of-zero caution existed to gate it.

This supersedes, specifically:

- `ARC-ROADMAP-001` constraint 2 (`OD-PACKAGE-008` remains authoritative; no `RulePackage`
  manifest scheduled) and constraint 3 (`EGRAPH` waits for a second real consumer).
- `OD-PACKAGE-006`'s remaining wait for a third same-language provider crate before
  `KNOWN_PROVIDERS` gains a self-registering mechanism.
- `OD-PACKAGE-008`'s conclusion that the wait for `RulePackage`'s manifest crate is
  load-bearing.
- `OD-PACKAGE-010`'s scope boundary confining the first `ModelBackendPackage`/
  `AgentExecutorPackage` maturity to model selection alone, and its deferral of
  `MODEL-ROUTE-037`'s catalog-entry shape and `038` through `049`.
- `OD-PACKAGE-011`'s and `OD-PACKAGE-012`'s conclusions that no real second case exists yet
  for model routing, dispatch, or capability-integrated agent/model backend routing.
- `OD-CORRECTIONS-001`'s conclusion that `CorrectionCandidate` generation, classification/
  ranking, `COR-005`'s rerun-and-compare half, and oscillation detection wait for a real
  trigger.

## What This Does Not Do

It does not retire engineering discipline. Every shape built under this override still
compiles, still passes its tests, still passes `cargo clippy --workspace --all-targets` and
the real gate, and still gets a real, tested exercise — a unit test standing in for the
"real caller" this cluster of records previously required to exist in production code. What
is retired is specifically the requirement that a second real *external* consumer exist
before the shape may be typed at all; a shape's own test suite is now sufficient warrant.

It does not touch any decision outside this named cluster. `OD-CAPABILITY-007` and
`OD-ANALYSIS-007`, which apply an analogous population caution to unrelated subsystems
(provider-decline semantics, program-semantics capability selection), are untouched — this
record supersedes what it names, not the pattern wherever it appears.

It does not un-happen the real technical findings these records made. `RulePackage`'s
contents list genuinely mixing manifest-shaped fields with data corpora and executable
fixtures, `MODEL-ROUTE-037`'s nine catalog-entry states genuinely naming a design taxonomy
rather than transcribing a closed fact, `nomos-model-package`'s genuine lack of a real
backend or executor implementation — all of that stays true and stays useful context for
*how* to build the deferred shape well. What changes is the answer to "build it now or
wait," not the measurements that answer informed.

## Status

Accepted. Standing, not scoped to one session — a future session reading `ARC-ROADMAP-001`,
`OD-PACKAGE-006`, `OD-PACKAGE-008`, `OD-PACKAGE-010`, `OD-PACKAGE-011`, `OD-PACKAGE-012`, or
`OD-CORRECTIONS-001` should read this record alongside them rather than re-apply the
caution those records' own older text still describes. Each of those records is amended in
the same change that adds this one, pointing back here rather than leaving this record as
the only place the supersession is stated.

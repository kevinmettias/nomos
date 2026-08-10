---
id: OD-TRACE-001
type: decision
title: A corpus requirement is related to this build by an assessment committed here, because the corpus is not on CI
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - traceability
  - requirements
  - corpus
  - completeness
  - verification
relations:
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
  - target: OD-PACKAGE-001
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
  - target: D-132
    type: relates-to
  - target: D-134
    type: relates-to
---

# A corpus requirement is related to this build by an assessment committed here, because the corpus is not on CI

## Question

The v14 corpus carries 363 requirements in 61 families at `authority:
canonical-normative-record`, and `D-132` puts the corpus above the plan. Nothing in this
repository relates any of them to the thing built to satisfy them.

## What Was Measured

Every requirement id under `01_authoring/artifacts/requirements` was resolved against every
file `git ls-files` reports. Fourteen are named in a governing record, the ledger, or crate
source, and the deliberate ones among those were put there by `P10-PACKAGE-SEAM`.

A low number is the expected shape for a young build against a large corpus, and on its own
it would be a fact rather than a defect. What makes it a defect is what the six hand-checked
comparisons found.

Three were already satisfied, exactly:

- `EvidenceClass` carries precisely `EVID-001`'s eight classes, `Authoritative` through
  `HumanAsserted`, and does not collapse them into a confidence number, which `EVID-001`
  also forbids.
- `FactVariant` carries precisely `CAP-002`'s five variants.
- `CAP-003` requires provider selection to compare guarantees rather than availability, and
  `selection.rs` compares `Guarantee` on every axis and documents that it is deliberately
  not a score.

Three diverge, each by a single element, and none says so:

- `Blocker` carries six of `WORK-LEDGER-005`'s seven typed causes, in corpus order, dropping
  `StaleProbeArtifact`.
- `LedgerItem` carries twelve of `WORK-LEDGER-001`'s thirteen fields, dropping `priority`.
- `Applicability` carries six of `CHK-003`'s seven reporting categories, dropping
  agent-required.

That is the finding. **Met and unmet have the same shape from outside** — an identifier that
appears nowhere, beside code that may or may not honour it. Six comparisons cost a full
manual audit each, and three of the six came back met, which is the worst ratio a hand check
can have: most of the work bought no change.

This repository refuses precisely this class of ignorance everywhere else. `tests/contract`
asserts the README's tables against the workspace in both directions. `OD-COMPLETENESS-001`
records that a completeness guard is only as complete as its universe. `D-134` makes a
universe declare its own mirror. The corpus that governs the entire product is the one
authority with no guard at all.

## The Decision

**An assessment of a corpus requirement is a declared entry committed to this repository,
compared against the workspace by a test, and it is not derived from the corpus at check
time.**

An entry names the requirement id, the verdict, and the site in the workspace the verdict is
about. Four verdicts, and the fourth is the one that does work:

- **Met** — the site satisfies the requirement, and the entry names where.
- **Diverges** — the site is derived from the requirement and departs from it deliberately;
  the entry names the governing record carrying the reason. A divergence with no record is
  not a verdict, it is the state this record exists to end.
- **NotBinding** — the requirement is read as not reaching this build, with a record saying
  why a corpus requirement does not bind the thing built to enforce it.
- **Unassessed** — the default, held by the absence of an entry.

## Why Not A Projection Of The Corpus

This is the argument that decided it, and it is not an aesthetic one.

`OD-GATE-001` records that a test which cannot find its corpus returns early and prints
`ok`. Three corpora live outside this repository and CI has none of them. A traceability
surface computed from the corpus at check time would therefore be green on every pull
request by being unable to look — and it would be green about the *entire* corpus, which is
a far larger lie than the sixty-eight tests that hole already covers.

So the corpus is required to *author* an entry and to *re-verify* one. It must not be
required to *run the guard*. The guard checks what is committed: that every entry names a
site that exists, that every `Diverges` entry names a record that exists and is registered,
and that the assessed set only grows. A corpus-gated test may compare entries against the
corpus when the corpus is present, and inherits the `OD-GATE-001` hole honestly rather than
hiding inside it.

`nomos-spec-project`'s `traceability-matrix` profile is not this. Like all eighteen profiles
it relates the store to itself, and it stays that way.

## Why Not A Citation In A Comment

A doc comment naming a requirement id is not a universe and nothing reads it, which is the
failure `OD-COMPLETENESS-001` already recorded one scale down. A comment can also say `Met`
about code that stopped being met three commits ago, and nothing anywhere goes red.

Entries may still be surfaced at the site — `D-134`'s form, where a universe declares its
mirror next to itself — but the site note is a rendering of the entry, never the authority
for it.

## Unassessed Is A State, Not A Gap

349 requirements have not been looked at, and any scheme requiring all of them answered
before any of them is answered will not be adopted. So the registry takes the shape
`OD-SPEC-007` already chose for governing records: **a floor rather than a count.**

Absence of an entry means nobody looked. That is a readable state, distinct from met and
distinct from waived, and it is the honest description of most of the corpus today. The
guard asserts that the assessed set does not shrink, so an entry cannot be quietly deleted
to make a divergence disappear.

## What This Costs

An entry can go stale. The guard can see that a named site has vanished; it cannot see that
the code at that site drifted out of satisfying the requirement while the path stayed valid.
Semantic drift is meaning, and this workspace has no type for it — the same limit
`ARC-ECOSYSTEM-001` states about its own boundary.

That is the trade and it is worth taking. A stale `Met` entry is wrong in one requirement.
The status quo is unreadable in all 363, and the only instrument for telling met from unmet
is an audit that costs the same whichever answer it reaches.

## What This Record Does Not Do

It builds no registry. No crate changes, no test is added, and no requirement is assessed
here. It decides the shape the mechanism must have and, more importantly, the shape it must
not have.

The first two entries are already open as items: `P10-LEDGER-CORPUS` for the
`WORK-LEDGER` family, and `P10-AGENT-REQUIRED` for `CHK-003`. Whichever lands first carries
the registry, which is why this record does not pick the file it lives in.

## Status

Closed by `P10-CORPUS-TRACE`.

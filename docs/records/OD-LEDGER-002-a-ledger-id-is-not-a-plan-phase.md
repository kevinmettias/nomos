---
id: OD-LEDGER-002
type: decision
title: A ledger id's number is the ledger's own, and P8 is not the plan's Phase 8
status: closed
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - naming
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: D-132
    type: relates-to
---

# A ledger id's number is the ledger's own

## Question

Every item in `work/ledger.json` is named `P<n>-<WORD>`. The approved plan is also organised
in numbered phases. A reader takes the two numbers to be the same number, and for the first
four they broadly were.

They are not the same now. `work list` shows six `P8-*` items, five of them done, and every
one is a follow-up to the analysis kernel that `P7-*` built. The plan's Phase 8 is a rules
engine — `nomos-rules`, `nomos-applicability`, `nomos-findings`, `nomos-gates`, a rules core
with three checks, and `nomos check` and `nomos gate`. None of it exists: no such crate is a
workspace member, and the binary has exactly one group, `work`.

So a reader of this ledger in three months concludes Phase 8 is nearly finished, and it has
not been started.

## What The Ledger Actually Holds

| Prefix | Items | What they built |
|---|---|---|
| `P1` | 2 | `nomos-spec-model`: the types, the id grammar, the canonical normalizer, and the gate proving it reproduces v14's real hash |
| `P2` | 5 | The specification store, the deterministic bundle, ingest I0–I3, the preservation rules, and the governing records authored into the store |
| `P3` | 11 | The corpus work: overlay, the lost families, siblings, lineage, archaeology, row typing, and the measured counts. One declined |
| `P4` | 1 | `nomos-spec-project`: profiles and renderers |
| `P7` | 6 | The analysis kernel and its substrate: capability, analysis, the Rust provider, the store, the workspace, and the vertical slice |
| `P8` | 6 | Follow-ups to `P7`, every one of them. Composition, fact identity, document kinds, a second provider, selection, and a contract's home |
| `P9` | 8 | Findings from an audit of the tree against what it claims, and the question `P8-SELECTION` left open |

There are no `P5` or `P6` items. Not deferred and not declined — absent. `P9-PHASE-GAP`
carries that separately, because the gap is a different question from the numbering: `D-129`
asserts an authoring surface that does not exist, and nothing records that it was skipped.

## The Decision

**A ledger id's number is a batch ordinal in the ledger's own sequence. It is not a plan
phase and has not been one since `P7`.**

The ledger is authoritative for its own ids. `D-132` already establishes the direction of
authority in the other place the two artefacts meet: the plan is a game plan — a reasoning
trace that motivated the work — and where it disagrees with a reading taken from the
corpus, the reading wins and the plan is superseded rather than corrected. The same holds
here. A number in this ledger describes what this ledger did.

That is a legend, not a change. What was wrong was that nobody had written it down.

## Why Nothing Is Renumbered

Renaming the six `P8-*` items would make the ledger agree with the plan's numbering at the
cost of five commit messages that name those ids in their subject lines and bodies —
`P8-COMPOSE`, `P8-PIN`, `P8-KIND`, `P8-SECOND-PROVIDER`, `P8-SELECTION`, `P8-CONTRACT-HOME`
— plus two decision records and an amendment that cite them as evidence. Git history is the
one artefact here that cannot be edited without rewriting it.

So the choice is between a ledger that disagrees with a document not in this repository, and
a ledger that disagrees with its own history. The first is a legend problem and this record
is the legend. The second would be a falsification.

## What Could Not Be Checked From Here

The plan is not in this repository. `docs/` holds only `records/`, and the sections cited
when this was raised — `§A3`, `§B5`, `§C2`, and `D-128` — are not among them.

So the statements above about *the ledger* are checked against `work/ledger.json` and the
tree. The statement about *the plan's* Phase 8 rests on the citation being accurate, and is
recorded as such. What is independently verifiable, and is the part that matters, is that
none of the five named crates is a workspace member and neither named command exists —
whatever phase number that work carries.

## What Would Close This Properly

The plan in the repository, or a stated pointer to where it lives and at which revision.
`ARC-SPECDB-001` makes the specification a database precisely so that a document of record
is not a file somebody has a copy of. The plan is a document of record about this build and
it is reachable from here only by trust, which is the same defect one level out.

Not opened as an item, because where the plan should live is a decision about the
specification system rather than about the ledger, and it belongs with `P9-READ-SURFACE`'s
question about what this repository can answer without a corpus.

## Status

Closed by `P9-PHASE-NUMBERS`. No item was renamed and the ledger validates unchanged.

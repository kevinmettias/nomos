---
id: OD-SPEC-011
type: decision
title: An unknown relation type is refused by name rather than by a foreign key
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-store
  - diagnostics
  - records
relations:
  - target: OD-SPEC-006
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
---

# An unknown relation type is refused by name rather than by a foreign key

## Question

The seeded relation vocabulary is five terms — `supersedes`, `superseded_by`, `affects`,
`affected_by` and `relates-to` — and `relations.relation_type` carries a foreign key onto
`relation_types`. A record declaring a sixth term is therefore refused, and that refusal is
the mechanism working: `RELATION_TYPES` says so in the comment that added `relates-to`.

What it was not is diagnosable. `Seed_Governing_Records` surfaced it as

```
StoreError::Sql("FOREIGN KEY constraint failed")
```

which is the same eleven words for every foreign key the schema has, and names neither the
record that carried the term nor the term itself.

## What Was Measured

While landing `OD-LEDGER-014`, one record declared `type: amends` for an edge onto
`OD-LEDGER-001`. Every test in this crate seeds a store before it can assert anything, so
that one bad word in one record produced:

| | |
|---|---|
| suites failing | 2 |
| tests failing | 18 |
| failures naming the record | 0 |
| failures naming the term | 0 |
| failures naming an admissible term | 0 |

Eighteen unrelated-looking failures, including `Test_Seeding_Should_Be_Idempotent` and
`Test_No_Governing_Record_Should_Carry_A_Carriage_Return`, whose names point at three
different subsystems and none of which is where the defect was. The author's next three
questions — which record, which term, what instead — were each answerable only by reading
`governing.rs`.

## The Decision

### 1. The vocabulary is checked where the record is known

`Refuse_Unknown_Relation` runs in the second seeding pass, over each record's declared
relations, and returns `StoreError::Record { path, cause }`. That variant already exists for
exactly this shape and its documentation already says why: a document that does not read
names the document, "because in either case the caller's next question is which file".

The database layer cannot do this. By the time SQLite refuses, the record's path has been out
of scope for two call frames — the constraint knows a term and a row and has never seen a
filename. So this is not a check duplicated from the schema; it is the same rule stated where
the context needed to explain it still exists. The foreign key remains, and remains the
backstop for any path that does not come through seeding.

It is raised before `Reference_Node`, so a record on its way out does not leave a placeholder
node behind it.

### 2. The refusal reports the vocabulary from the table

`Admissible_Relations` builds the list from `RELATION_TYPES` rather than restating it. A
restated list is a second authority that goes stale silently, and this one is read by an
author in the middle of fixing the very thing it describes — the worst moment for it to be
wrong. A term added to the table appears in the refusal with no second edit, and
`Test_The_Reported_Vocabulary_Should_Come_From_The_Table` asserts the count rather than a
fixed string so it survives the table changing.

### 3. Unknown terms stay refused, and the fix is the diagnosis

This is the part worth stating, because the obvious reading of the incident is the wrong one.
The record that failed wanted `amends`, which is a real relation and arguably the honest one.
Admitting it would have made the seed pass.

It is refused anyway. `ADR-ARTIFACT-GRAPH-002`'s vocabulary arrives with the corpus, and
widening the seed table now would put an invented answer where a recorded one belongs — the
reasoning `RELATION_TYPES` already carries for its tier, applied to its membership. The
refusal message says so, so the next author reads why the table is short at the moment they
are tempted to extend it, and reaches for an existing term where that is honest. In the
measured case `affects` was honest and was used.

The defect was never that the store refused. It was that it refused anonymously.

## What Was Considered And Rejected

**Widen `RELATION_TYPES` when a record needs a term.** Rejected by decision 3. It answers a
question the corpus already answers, and each addition is one more term the real vocabulary
has to be reconciled against later.

**Validate in `Put_Relation`.** The natural home by proximity and the wrong one by
information: `Put_Relation` receives an identifier, a term and a target, and would report the
same anonymous failure one layer up. The path is what the author needs and only the seeding
loop has it.

**Collect every bad term and report them together.** Attractive, and refused because seeding
is a transaction against a store that must not be left half-written. Stopping at the first
bad record keeps the failure one record deep, which is the property `done_when` asks for when
it says one record with one bad term must not read as eighteen failures.

## What This Does Not Do

- It does not change which relations are admissible, or the schema, or the foreign key.
- It does not touch the projections. A record refused at seeding never reaches a renderer.
- It does not make every `StoreError::Sql` diagnosable. This closes the one cause a record
  author can actually cause; the others are reached by code rather than by content.

## Controls

| Weakening | What it produces |
|---|---|
| leave the foreign key to report it | 18 failures across 2 suites, 0 of them naming the record or the term |
| restate the vocabulary in the message | a list that goes stale, read by an author fixing the thing it describes |
| admit unknown terms so the seed passes | an invented vocabulary where `ADR-ARTIFACT-GRAPH-002`'s recorded one belongs |
| refuse everything | the first relation stops the seed, which is worse than the foreign key it replaced — the reason `Test_Every_Seeded_Term_Should_Be_Admitted` exists |
| assert only that the message names the record | green for a message that names the record and nothing else, which every refusal here already does |

## Status

Closed by `P10-VOCABULARY-REFUSAL`.

---
id: OD-SPEC-013
type: decision
title: A submission is a node, a field is a sequence of attributed rows, and a gap is a row of its own
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - substrate
  - feature-lifecycle
  - schema
relations:
  - target: OD-SPEC-008
    type: relates-to
  - target: OD-SPEC-009
    type: relates-to
  - target: OD-SPEC-010
    type: relates-to
  - target: OD-SPEC-011
    type: relates-to
  - target: ARC-SPECDB-002
    type: relates-to
---

# A submission is a node, a field is a sequence of attributed rows, and a gap is a row of its own

## Question

`OD-SPEC-008` made `FeatureRequest`, `DesignSpec` and `FeatureResult` born structured and
deferred the physical layout, handing it one constraint: only tables something writes to
exist, because a table nothing writes to looks like a feature in a schema dump and is not one.
`OD-SPEC-009` fixed the seam — one accept function, transports never validate. `OD-SPEC-010`
stated the rule set and bound this record to a layout that can express an attributed value
sequence per field and a decision gap as a row.

So the layout is the last open question in the set, and it is asked with three answers already
fixed. What remains is which tables exist, what a field is, and what identity a submission has.

## Why This Record Is Not `OD-SPEC-011`

`OD-SPEC-009` and `OD-SPEC-010` both cite `OD-SPEC-011` as this record, six times between them,
because both were written while that identifier was reserved for it by `P10-REQUEST-LAYOUT`.
The identifier was published for a different decision — an unknown relation type refused by
name — and two work items had reserved one identifier, which nothing checks.

Those two records are amended to cite `OD-SPEC-013`, and this section exists so the amendment
is not silent. A reader who finds a stale citation elsewhere should treat `OD-SPEC-013` as the
layout record and `OD-SPEC-011` as the relation-type refusal, which is what each of them
argues.

## Decision

### 1. A submission is a node

A `FeatureRequest`, `DesignSpec` or `FeatureResult` is a row in `nodes`, with `kind` of
`feature-request`, `design-spec` or `feature-result`.

It is not a new subject table, and that is the load-bearing half of this record. `nodes` is
already the store's answer to *what is a thing that can be referred to*: it carries the
`node_id` identity, and `node_aliases`, `node_history`, `relations` and `lineage` all address
it. A parallel subject table would need a parallel answer to each of those, and the first one
that mattered would be `node_history` — which `OD-SPEC-010` already requires for promotion
from `draft` to `accepted`, by name, because its `reason` column is `NOT NULL` with a
non-empty check and therefore refuses an acceptance nobody justified.

A submission that was not a node would have to either grow its own history table or reach into
the node one without being a node, and the second is the shape `ARC-SPECDB-001` exists to keep
out of this store.

### 2. Three tables, and what decides the split

| Table | What it holds |
|---|---|
| `submissions` | the scalars that are structural rather than submitted |
| `submission_values` | every field value, attributed and ordered |
| `submission_gaps` | one decision gap per row |

The split is not by convenience. A column is admissible exactly when the thing it holds is
**not** something a later reader could clarify, infer or decide — because those are the four
origins `OD-SPEC-010` requires every *value* to carry, and a column has room for one value and
no origin.

So `submissions` carries `node_uid`, `kind`, `form_contract_version`, `state`, and the two
halves of `provenance` — who submitted it and through which surface. Each is a fact about the
submission event, fixed when the accept function ran, and none of them is a thing anybody
clarifies later. `state` is the one worth naming explicitly: `draft` and `accepted` are not
submitted content, nobody types them, and giving `state` an origin would invite a submission
that claims its own acceptance.

Everything `OD-SPEC-010` lists per kind — `goal`, `behaviour`, `acceptance`, `invariants`,
`answers`, `alternatives`, `selected`, `architecture_delta`, `implements`, `evidence`,
`deviations`, `owed` — is a `submission_values` row. So is `title`, which the universal table
lists and which is a sentence somebody wrote and may later rewrite.

### 3. A field is a sequence of rows, and a column would make the rule set unimplementable

`submission_values` carries `submission_uid`, `field`, `ordinal`, `origin`, `value`,
`value_hash`, `supersedes_hash` and `recorded_at`. `origin` is checked against `submitted`,
`clarified`, `inferred` and `decided`. The current reading of a field is its latest ordinal;
every earlier value stays.

The column alternative — one column per field on a `feature_requests` table — is refused, and
on correctness rather than on taste. `OD-SPEC-010` requires that a later value supersede an
earlier one **for reading** and never replace it **in storage**, and that every value carry
which of four origins it has. A column holds one value and no origin. Implementing the rule
set over columns means either a second table of superseded values, which is this table with an
apologetic name, or overwriting, which is the weakening `OD-SPEC-010`'s controls table names
as destroying the one thing a request exists to preserve.

`supersedes_hash` is `normative_statements.supersedes_hash` applied to a second kind of row,
which `OD-SPEC-010` asks for by name. Two supersession models in one store would be two
answers to what a superseded value is.

### 4. An absence is a value, not a missing row

`OD-SPEC-010` requires that a required field be satisfied by content or by an explicit
statement that there is none, and never by emptiness — an empty `deviations` means nobody
looked, `deviations: none` means somebody looked.

The layout keeps those distinguishable by making the second a row like any other, with
`value` holding the stated absence and an origin naming who stated it. The honest submission
has a row; the abandoned one has none. Had absence been modelled as a nullable column, the two
would be one `NULL`, which is the collapse that rule exists to prevent.

### 5. A gap is a row, closed only by a citation

`submission_gaps` carries `submission_uid`, `question`, `blocks` — the fields it blocks —
`severity` checked against `blocking` and `non-blocking`, and `closed_by`, which is `NULL`
while the gap is open and otherwise holds the citation that closed it.

`closed_by` is a citation and not a boolean, and that is the whole of what makes the rule
enforceable rather than advisory. `OD-SPEC-010` says a gap is never closed by supplying the
value it blocks; a boolean `closed` column could be set by whoever supplied it, and nothing
would record that the question had been answered. A citation column can only be filled by
naming a governing record or a recorded decision, and the value that arrives alongside it
takes origin `decided`.

A `blocking` gap with `closed_by IS NULL` refuses acceptance. A `non-blocking` one does not,
and travels to the design and the result rather than being dropped at each hand-off.

### 6. `answers` and `implements` join the seed vocabulary

The lifecycle edges are `relations` rows, which is what `OD-SPEC-010` assumes. They are not
writable today: `relations.relation_type` is a foreign key onto `relation_types`, the seeded
vocabulary is `supersedes`, `superseded_by`, `affects`, `affected_by` and `relates-to`, and
`OD-SPEC-011` made an unknown term refuse by name. So an `answers` edge is refused, and the
rule set that depends on it could not be exercised.

Both terms are added at tier `seed`, with inverses `answered_by` and `implemented_by`.

This is the `relates-to` precedent rather than a new liberty. That term was added when
governing records used a vocabulary term the table lacked and the foreign key refused them;
the alternative was rewriting the relations as something they were not, and a wrong edge in
the graph this system exists to keep honest is worse than a vocabulary one term short. Here
the terms are not invented either — `OD-SPEC-008` names the lifecycle and `OD-SPEC-010` rule 4
names both edges and states what each must resolve to.

They stay `seed`. `ADR-ARTIFACT-GRAPH-002`'s vocabulary arrives with the corpus and supersedes
this whole table, and guessing a tier would put an invented answer where a recorded one
belongs.

## What This Does Not Decide

It does not add domain, range or cardinality to `relation_types`. Adding `answers` and
`implements` makes the edges writable; it does not stop one joining a suite to a table row.
`OD-SPEC-010` rule 4 constrains the submission and cannot constrain the graph, and closing that
is `P10-DECLINED-DEPENDENCY`'s neighbour rather than this record's — the item holding it is
`P10-EDGE-CONSTRAINTS`.

It does not decide the intake surface, which is `OD-SPEC-009`, nor the rule set, which is
`OD-SPEC-010`. It does not govern document-first objects: governing records keep the
preservation machinery `ARC-SPECDB-001` exists for.

It does not make the store durable by itself. The committed durable form is the bundle text,
per `OD-SPEC-008`, and these three tables travel in it like every other.

## Controls

| Weakening | What it produces |
|---|---|
| a column per field | the origin has nowhere to live, and supersession becomes overwriting |
| a nullable column for a field that may be empty | the honest submission and the abandoned one become one `NULL` |
| a `closed` boolean on a gap | the question is closed by whoever supplied the value, and nothing records that it was answered |
| a submission as its own subject table | `node_history`, `relations` and `lineage` each need a second answer, and promotion loses the table that refuses a reasonless acceptance |
| an origin on `state` | a submission can assert its own acceptance |
| a quarantine or draft-holding table | refused already by `OD-SPEC-010`; a table whose writer is a bug |
| inventing lifecycle edge names beyond the two the records name | the corpus vocabulary arrives and the invented terms are what has to be unpicked |

## Status

Accepted. The tables, the field shape, the gap shape, the identity and the two vocabulary
terms are decided. Relation constraints, the intake surface and the rule set are not, and are
`P10-EDGE-CONSTRAINTS`, `OD-SPEC-009` and `OD-SPEC-010` respectively.

---
id: OD-SPEC-012
type: decision
title: A declared constraint says which node kinds a relation type joins, and how many a node may carry
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-store
  - specification-bundle
  - records
relations:
  - target: OD-SPEC-010
    type: relates-to
  - target: OD-SPEC-011
    type: relates-to
  - target: OD-LEDGER-020
    type: relates-to
---

# A declared constraint says which node kinds a relation type joins, and how many a node may carry

## Question

`relation_types` carried a name, a tier and the name of its inverse, and nothing else.
`relations` enforced only that both endpoints already existed as nodes and that the triple
was unique. `OD-SPEC-010` rule 4 requires that an `implements` edge resolve to an accepted
design and an `answers` edge to an accepted request, and its own text says why that rule
cannot be the whole answer: it constrains the submission that writes the edge, not the
graph the edge lands in. Nothing stopped an `implements` edge joining a suite to a table
row, because nothing in the schema said which kinds of node `implements` was ever about.

`P10-EDGE-CONSTRAINTS`, twice re-authored, named the defect and stalled on what depended on
it. `P10-SUBMISSION-LAYOUT` landing closed that dependency; this record is
`P10-EDGE-CONSTRAINTS-2` closed.

## What Was Measured

The seeded vocabulary is nine rows — five concepts, four of them paired with an inverse:
`supersedes`/`superseded_by`, `affects`/`affected_by`, `relates-to` (its own inverse),
`answers`/`answered_by`, `implements`/`implemented_by`. Every governing record this build
seeds declares `type: decision` or `type: architecture` in its front matter and nothing
else — a query over the seeded corpus found no third value. The busiest node's `relates-to`
count, summed across both directions the self-paired type writes, was 21.

## The Decision

### 1. A relation type declares domain, range and cardinality together, or it is refused

`SpecificationStore::Put_Relation_Type` now takes `domain: &[&str]`, `range: &[&str]` and
`max_per_node: u32` as required arguments beside the name and tier it always took. An empty
domain, an empty range or a zero cardinality is `StoreError::UnconstrainedRelationType`,
raised before anything is written. A relation type that admits everything is not a
constraint, so a caller that declares nothing is refused at registration rather than
handed a row that would go on admitting everything downstream — the shape `OD-SPEC-011`
already used for an unknown term, applied to an unconstrained one.

Domain and range are nullable in neither direction: a type joins a *set* of node kinds at
each end, which is why they are declared as lists rather than as two more single-valued
columns. `relations` stays a graph between kinds, not a graph between exact node identities
— `answers` admits every `design-spec`, not one named design.

### 2. Domain, range and cardinality are per named direction, not per pair

`answers` and `answered_by` are two rows in `relation_types`, each with its own domain,
range and cardinality, rather than one constraint read backwards. They are not
symmetric — `answers` runs from a `design-spec` to the `feature-request` it answers, and
`answered_by` runs the other way — so a single shared declaration would have to pick a
direction and silently apply it to both. Stating each direction on its own row is the same
choice `relation_types.inverse_of` already made for the vocabulary itself: two rows, paired,
rather than one row asked to mean two things depending on which way it is read.

### 3. The check runs where the edge is written, and is exempt for one case: an unresolved placeholder

`Write_Relation` resolves both endpoints, looks up the relation type's declared constraint,
and refuses before inserting if an endpoint's kind is not admitted at its role or if the new
edge would push the writing node's count of that type past its cap. Both refusals —
[`StoreError::RelationEndpoint`] and [`StoreError::RelationCardinality`] — name the relation
type, the endpoint or node involved, what it is, and what would have satisfied the check:
`RelationEndpoint` carries the admitted kinds, `RelationCardinality` carries the declared
cap.

The one exemption is deliberate: an endpoint minted by `Reference_Node` (`nodes.authority =
EXTERNAL`) carries the sentinel kind `unknown` because nothing has ingested it yet, and
`unknown` is not a fact about the node — it is the absence of one. Checking a placeholder's
kind against a declared domain or range would be checking a fact that does not exist yet,
and admitting the sentinel into every type's domain and range as if it were a real kind
would have made the constraint report success for a question it never actually asked. So
the check on that endpoint is deferred, not weakened: `Seed_Governing_Records`'s own
forward-reference mechanism (`SeedReport.references`, and `OD-SPEC-011`'s reporting of it)
still marks the placeholder visibly, and a later commit that resolves it writes the real
node and the real kind then arrives for the *next* edge that endpoint takes part in. The
cardinality check is not exempted the same way, because it counts edges leaving the real
`from` endpoint — which, by the time an edge can be written at all, already resolved to a
concrete row — rather than judging the placeholder's kind.

An edge to an identifier no node holds at all still writes nothing silently, exactly as
before this record: that contract belongs to `OD-SPEC-011` and the sibling-suite tests that
depend on it, and this record does not touch it. Re-inserting an edge that already exists
does not count a second time against the cap, the same idempotence `INSERT OR IGNORE`
already gives every other edge in this store.

### 4. The five seed types declare real constraints, not placeholders

| type | domain | range | max per node |
|---|---|---|---|
| `supersedes` / `superseded_by` | decision, architecture | decision, architecture | 16 |
| `affects` / `affected_by` | decision, architecture | decision, architecture | 64 |
| `relates-to` | decision, architecture | decision, architecture | 128 |
| `answers` | design-spec | feature-request | 8 |
| `answered_by` | feature-request | design-spec | 8 |
| `implements` | feature-result | design-spec | 8 |
| `implemented_by` | design-spec | feature-result | 8 |

The record kinds (`decision`, `architecture`) are the two values this build's own governing
records actually carry, not an invented wider set — the same restraint `OD-SPEC-011` already
applied to the vocabulary itself applies here to what each term admits. The lifecycle
kinds (`design-spec`, `feature-request`, `feature-result`) are `SubmissionKind`'s three
labels, and the domain/range pairing on `answers`/`implements` is `OD-SPEC-010` rule 4's own
statement of what those edges must resolve to — read here as a graph constraint rather than
restated as a submission-time check, because it is now able to be one, and `submission.rs`'s
rule 4 remains the separate guarantee it always was: that the *cited target* is not merely
the right kind but an *accepted* submission of it. A relation type constraint cannot express
acceptance state; it was never asked to.

The cardinalities are not predictions of a ceiling. `relates-to`'s cap of 128 sits well
above the busiest measured node (21) with room for the corpus to grow; the lifecycle types'
cap of 8 is generous against every resubmission scenario this build's own tests exercise.
Each cap exists to catch a joined-the-wrong-node mistake becoming an unbounded pile of
edges, not to assert where real usage will top out — `ADR-ARTIFACT-GRAPH-002`'s vocabulary
is free to set its own numbers when it supersedes this table.

### 5. Domain and range travel as JSON columns, not as a second table

`relation_types` gained `domain_kinds_json`, `range_kinds_json` and `max_per_node`, rebuilt
in migration 7 rather than widened by `ALTER TABLE ADD COLUMN`: SQLite only allows a
`NOT NULL` column added that way to carry a caller-invented default, and a default here
would hand every existing row a constraint nobody declared — the exact permissiveness this
record exists to close off. There is nothing to carry across regardless: `relation_types` is
populated by application code once the schema is in place, never by a migration, so the
table always holds zero rows when its own migration runs.

A JSON column rather than a child table keyed on `(relation_type, role, node_kind)`: nothing
else in this schema joins against a relation type's admitted kinds, so a table would exist
to be scanned start to finish exactly once per check, in a size in the tens of node kinds at
most. `nodes.kind` itself is free text with no table of its own for the same reason. Both
lists are sorted before they are serialized, so the same set of kinds always writes the same
bytes regardless of the order a caller listed them in — the bundle's byte-identical round
trip depends on that the same way it depends on every other ordering in this store being by
natural key rather than by insertion order.

### 6. The bundle carries the constraint, not just the row

`nomos-spec-bundle`'s `RelationType` gained `domain: Vec<String>`, `range: Vec<String>` and
`max_per_node: u32`. The exporter decodes the two JSON columns, the importer re-encodes them,
and `columns::COVERAGE`'s `relation_types` entry names all three — the guard that would have
caught a bundle quietly missing this the way `OD-SPEC-011`'s own motivating column-coverage
mechanism was built to. `crates/spec/nomos-spec-bundle/tests/round_trip/populated.rs`'s fixed
fixture now declares a non-trivial constraint on its own synthetic type (`verifies`, domain
`concept`, range `requirement`, cap 4), so the whole existing round-trip suite exercises real
values rather than only a row's presence, and
`Test_A_Relation_Types_Constraint_Should_Survive_The_Round_Trip` asserts those three values
by name after a full export/import cycle.

## What Happens When `ADR-ARTIFACT-GRAPH-002` Lands

The corpus's real relation vocabulary supersedes `RELATION_TYPES` in `governing.rs`, exactly
as `OD-SPEC-011` already says it will. This record adds nothing that decision has to work
around: every term the real vocabulary introduces registers through the same
`Put_Relation_Type` this record requires, declaring its own domain, range and cardinality
the same way the five seed types do here. Nothing about the mechanism is seed-specific — it
is seed *content* that a wider vocabulary replaces, not a mechanism a wider vocabulary has to
grow into.

## What Was Considered And Rejected

**Infer domain and range from the edges already present.** Rejected by `done_when` itself:
an inferred constraint can only ever be as wide as what has already been written, so it
could never refuse the *first* bad edge of a new kind — exactly the shape `implements`
joining a suite to a table row would have taken if nobody had joined them yet.

**Leave the constraint on the submission, as `OD-SPEC-010` rule 4 already does.** That rule's
own text says it cannot be the whole answer: it runs once, at acceptance, over the fields one
submission cites. It says nothing about an edge written any other way — through
`Seed_Governing_Records`, through a future authoring surface, or by hand against the store —
and a constraint that only one caller obeys is not a constraint the graph holds.

**Admit the placeholder sentinel `unknown` into every type's declared range.** Rejected in
decision 3: it would make the check pass by widening what every type admits rather than by
deferring judgment on a fact that does not exist yet, and it would have to be removed from
every declaration the day a real `unknown`-shaped kind was ever wanted for something else.

**A child table keyed on `(relation_type, role, node_kind)`.** Considered and set aside in
decision 5 for the same proportionality `nodes.kind` already argues: nothing else in this
schema needs to join against admitted kinds, so a table for it would be scanned wholesale on
every check for no query a table earns its keep by answering.

## Controls

| Weakening | What it produces |
|---|---|
| infer the constraint from edges already written | never refuses the first edge of a new, wrong kind |
| leave it to `OD-SPEC-010` rule 4 alone | silent for every edge not written through `Accept_Submission` |
| admit `unknown` into every domain and range | the check passes by definition on the one endpoint it most needs to defer judgment on |
| a shared constraint per pair instead of per direction | `answers` and `answered_by` cannot both be stated correctly, because they join in opposite roles |
| no cardinality cap, domain and range only | `relates-to` keeps the property this record closes for count while leaving it open for volume |

## Status

Closed by `P10-EDGE-CONSTRAINTS-2`.

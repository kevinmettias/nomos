---
id: OD-SPEC-008
type: decision
title: Feature requests, designs and results are born structured, and their documents are projections
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - substrate
  - feature-lifecycle
relations:
  - target: ARC-SPECDB-002
    type: relates-to
  - target: OD-SPEC-006
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
---

# Feature requests, designs and results are born structured, and their documents are projections

## Question

A typed layer of `FeatureRequest`, `DesignSpec` and `FeatureResult` cannot be started until
one thing is recorded, because the answer is the schema written down. Authored files with
typed front matter and a derived store index is a different schema from structured records
whose markdown is generated, and the difference reaches every row.

An earlier attempt on this item reached the opposite answer to the one below, on the
evidence available then. It is worth saying why that answer is not simply overturned by
preference.

## What A Feature Request Is

Naming this first, because the substrate question cannot be answered about an object nobody
has defined, and because this repository already has two things a reader will confuse it
with.

A **feature request** is a structured submission recording what somebody asked for, before
anybody has decided how or whether to answer it. It carries the intent, the goal, the
behaviour wanted, how the asker will know it worked, and what must not break. It carries
what was asked separately from what was later clarified, and it names the decisions it needs
that nobody has taken yet.

It is **not a work item**. `work/ledger.json` holds items, and an item reserves territory, is
claimed by exactly one holder, and is refused if it reserves nothing. A feature request
reserves nothing, is held by nobody, and may produce many items or none.

It is **not a decision record**. A record argues a case and the argument is the artifact. A
feature request makes no argument; it states a want and the constraints on satisfying it.

A **design specification** is the answer to a feature request that has been chosen: the
alternatives considered, the one selected, the architecture delta, and what acceptance will
require. A **feature result** is what was actually built against that design, including where
it deviated and what it left owed.

The three are one lifecycle and are decided together here, because splitting them across
substrates would put the split in the schema of every row that joins them.

## Decision

Feature requests, design specifications and feature results are **born structured**. The
canonical form of each is the structured record. Every markdown, YAML, JSON and HTML document
about them is a projection, and no such document is authored by hand.

This is `ARC-SPECDB-002` applied rather than a new rule. That record decides the axis: an
object is born structured when its schema can be stated before its content exists. These
three pass that test unambiguously. The questions a feature request answers — what do you
want, why, what should happen, how will you know it worked, what must it not break — exist
before any particular request does. They can be written as a form, and a form written in
advance is exactly what the test asks for.

Governing records fail the same test and stay document-first, which is why this decision does
not disturb them.

## Why The Authored-File Answer Is Not Taken

The earlier answer — an authored file under `docs/records` with typed front matter, and the
store as a derived index — rested on three grounds. Two of them have moved.

The first was that nothing persists a specification database, so what survives an invocation
is the file. That is still true of the build and is not an argument about these objects.
`SpecificationStore::Open` exists and is called by nothing; the only construction site in the
host is `In_Memory`. The store is not durable because nothing needed it to be, and this
decision is the thing that needs it. A fact about what has been wired is weak evidence about
what should be.

The second was that deciding per artifact would put two substrates in one build and encode
the split in every typed row. That objection is correct and `ARC-SPECDB-002` was written to
answer it. It is fatal to a split drawn on artifact kind. The split taken here is drawn on a
property of the objects, is stated as a rule that applies to objects nobody has proposed yet,
and is therefore not the failure that objection describes.

The third ground was that the price of the store option was out of date in the buyer's
favour. That one held up, and it points the other way.

## The Price, As OD-SPEC-007 Left It

`OD-SPEC-006` quoted three costs for making the store the durable substrate: stop seeding
from `include_str!` and seed from something committed that is not the record files, with the
bundle named as the candidate; make the files under `docs/records` rendered outputs with a
freshness check; and design a guard, because `OD-SPEC-005` refuses deriving the seeded list
from the directory it is meant to govern.

`OD-SPEC-007` paid the third in general form. It established that what makes that guard a
check is not that its second side is hand-typed into one file, but that the second side is
**authored independently of the directory it is compared against**. Per-record registration
files keep the independence and remove the shared edit. The guard is no longer an open design
question; it is a solved shape with a working instance in this repository.

The first two costs are the price of **converting governing records**, and this decision does
not convert them. An object family that starts in the store pays neither: there is no seed to
change, because feature requests were never compiled into the binary, and there is no
directory of hand-authored files to turn into rendered outputs, because none was ever
written. Those costs stay owed by whoever later proposes moving governing records, and stay
quoted at `OD-SPEC-007`'s price rather than `OD-SPEC-006`'s.

What this decision does owe is durability, and that is its real cost, stated plainly: a
born-structured object whose store is rebuilt in memory on every invocation has no substrate
at all.

## What Durable Means Here, And What Git Holds

One consequence has to be settled now rather than discovered, because getting it wrong is
expensive and the failure is not obvious in a single session.

The durable committed form is **text**, not the SQLite file. A binary store in git cannot be
diffed, cannot be reviewed, and cannot be merged — and this repository is worked by
concurrent sessions whose whole coordination model assumes two of them can touch one tree.
Committing the database would make every pair of concurrent feature-request writes a
conflict no tool can resolve.

So the layering is: the structured record is canonical; its committed durable form is the
deterministic text serialization `nomos-spec-bundle` already produces; SQLite is a derived
working index that any checkout can rebuild; and markdown, YAML, JSON and HTML are
projections of the record, governed by the freshness machinery that already exists.

`OD-SPEC-006` named the bundle as the candidate for exactly this role. This decision takes it
up for the objects it governs and leaves it a candidate for the ones it does not.

## What This Binds

It binds the schema of the typed layer. `FeatureRequest`, `DesignSpec` and `FeatureResult`
are defined as structured schemas carrying their own version, validated at write time, with
the obligations `ARC-SPECDB-002` puts on every born-structured object: a versioned schema, a
refusal rather than an acceptance when a submission is incomplete, separation of what was
originally submitted from what was later clarified or inferred, and a freshness proof on
every projection that gets committed.

It binds the direction of generation. A `.md`, `.yaml`, `.json` or `.html` document about one
of these objects is an output. Editing one is editing a build artefact, and the surface that
writes them is not an editor.

## What This Does Not Bind

It does not touch governing records. They remain document-first, and `OD-SPEC-006` is not
amended.

It does not touch the imported corpora, the preservation rules, or the round-trip machinery.
Those govern document-first objects and keep governing them.

It does not touch `work/ledger.json`. A feature request is not an item, the ledger is not a
specification store, and nothing here makes one answerable to the other.

It does not decide the physical table layout. That is a separate item, and it inherits a
constraint already written into the schema this repository has: only tables something writes
to exist, because a table nothing writes to looks like a feature in a schema dump and is not
one. Seven tables landing empty ahead of a writer would be that failure committed
deliberately.

It does not decide the intake surface — form, CLI, API or MCP — nor the validation rule set,
including the conditional rules a form contract needs. Those are separate items with their
own predicates. This record decides only where the truth lives.

## Consequences

The typed layer can be started, which was the thing blocked.

These objects skip the round-trip proof, because there is no arbitrary document to prove
recoverable. They do not skip proof: they take on a freshness proof instead, and the gate
already runs one such step.

Two substrates now exist in one build, deliberately, with a written rule for which side any
object falls on and an argument for why that rule is not a list.

## Status

Accepted. The typed layer is unblocked; durability, physical layout, intake and validation
are separate items and none of them is decided here.

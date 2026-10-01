---
id: OD-SPEC-014
type: decision
title: Whether the spec store's graph-and-projection engine becomes shared infrastructure KWB depends on, and what would decide it
status: open
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - ecosystem
  - xvpe
  - kwb
relations:
  - target: ARC-ECOSYSTEM-001
    type: affects
  - target: D-130
    type: relates-to
  - target: D-135
    type: relates-to
  - target: D-136
    type: relates-to
  - target: D-137
    type: relates-to
---

# Whether the spec store's graph-and-projection engine becomes shared infrastructure KWB depends on, and what would decide it

## Question

`nomos-spec-store` holds a graph of nodes tagged by kind, authority and representation,
normative statements with recorded and recomputed hashes, relations, lineage and source
provenance. `nomos-spec-project` projects that graph through eighteen declared profiles into
markdown, YAML, JSON, HTML and mermaid. Both are domain-neutral machinery today — they know
about `kind`/`authority`/`representation` tags and section filters, not about architecture
decisions specifically.

`D-136` rebuilds KnowledgeWorkbench in Rust. Its own declared domain — per `AGT-005`, already
a normative statement in this store — is provenance-preserving claims, decisions and
contradictions, which is the same shape this engine already stores and projects. `D-137`
reserved `KnowledgeReferenceId` in `nomos-contracts` as the seam a KWB citation would use, and
its own text says nothing yet produces or consumes one.

Whether that shape genuinely generalizes past Nomos's own use of it, and whether KWB should be
built on the same store-and-projection core rather than its own, is undecided.

## Current Position

Nothing is extracted and nothing depends on anything. `nomos-spec-store` stays exactly what it
is today: a store seeded by this repository's own governing records and, optionally, the v14
corpus — self-referential to Nomos. `KnowledgeReferenceId` sits in `nomos-contracts` reserved
and unconsumed, exactly as `D-137` left it.

The one real tension, noted when this was first discussed: Nomos's `authority` column means
"canonical per Nomos's own governance," singular and self-referential. KWB's domain is
multi-source provenance — possibly-disagreeing claims from sources Nomos never governs — which
is closer to the divergence-detection `nomos-spec-ingest` already does between a statement's
recorded hash and its recomputed one than to a single canonical authority. Reusing the engine
as-is would not be reuse; it would need its authority and kind vocabulary generalized first,
which is design work nobody has scoped.

## What Would Decide It

Two independent gates, either of which currently blocks acting on this regardless of the
merits:

`D-130` refuses any Nomos dependency on XVPE before Phase 5, by any path. This repository
cannot take the dependency even if the design question were already settled.

`D-122`'s sibling-suite bar requires two products demonstrating identical domain-neutral
semantics before code moves to XVPE — one product's shape is never sufficient. Nomos is the
only product using this engine today; KWB does not exist yet in Rust. The comparison this
record would need cannot be run until KWB's rewrite reaches the point of actually needing a
knowledge-graph store and a multi-format projection layer of its own, and its shape can be
measured against this one rather than assumed from it.

## Status

Open. Revisit when KWB's Rust rewrite reaches the point of needing its own graph-and-lineage
storage, or at Phase 5 when `D-130`'s gate lifts — whichever comes first. Recorded here so the
intent is legible rather than lost between sessions, not because either gate has moved.

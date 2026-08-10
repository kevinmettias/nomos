---
id: D-129
type: decision
title: The specification store is the identity substrate; markdown is an editing surface
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - specification-system
  - documentation-architecture
relations:
  - target: ADR-DOC-001
    type: supersedes
  - target: ARC-SPECDB-001
    type: affects
---

# The specification store is the identity substrate; markdown is an editing surface

## Decision

The specification store holds identity. A `node_id` is stable across databases and travels
in a bundle; a file path is navigation and may change without changing what a thing is.

Markdown remains the surface people author through, via a round trip that reads a record
out as markdown and writes the edit back. It stops being the substrate in which identity
lives.

This supersedes ADR-DOC-001, which made markdown files with constrained YAML front matter
the canonical authored source.

## Rationale

ADR-DOC-001 was right about prose. Architecture rationale, requirements, tradeoffs and
open questions are explanatory text, markdown represents them directly, and diffs stay
reviewable. None of that is disputed here and none of it is being reversed — markdown is
still what an author types.

What ADR-DOC-001 could not give is a place for the relationship between two revisions of
the same content to live. It says "every important authored artifact needs a stable ID and
typed relations", and that is exactly the requirement a filesystem cannot enforce: a file
tree has no way to refuse a reorganization that drops 282 rows, because at the file level
nothing was dropped. v15.0 satisfied every constraint ADR-DOC-001 states and still lost the
content.

Leaving both records live would leave two answers to what the source of truth is, so this
one names the supersession explicitly rather than quietly disagreeing.

## Consequences

Identity survives renames, and a rename is an ordinary edit rather than a migration.

An author still edits markdown, but the edit is a transaction against the store: claim,
stage, preview, commit. The preview is mandatory and names what the edit changes,
including whether normative wording moved.

ADR-DOC-001 stays in the store as a superseded node. Its prose is restored from the v14
corpus in Phase 3; until then it is present by identity and by this edge, not by content.

## What Is Not Built (amended at version 2)

*Answered by the version 3 section below. Kept as written, because the gap it records is what
made the work schedulable and a record that quietly stops mentioning its own gap teaches
nobody anything.*

The round trip does not exist. Nothing reads a record out of the store as markdown, and
nothing writes an edit back. There is no claim, stage, preview or commit against the store
for an authored record; the preview this record calls mandatory has never run.

So the second paragraph of the decision describes an intent, and until now it read as a
description of the system. That is the specific failure mode this whole build is organised
against — a governing record asserting a capability, validated, in the store, and nothing
exercising it — and the honest correction is to say so in the record rather than to leave a
reader to discover it from the absence of a command.

What is built is the first paragraph, and it is load-bearing: the store holds identity, a
`node_id` is stable across databases and travels in a bundle, and `P3-*` restored content
against those identities across five revisions. Nothing here is retracted. The supersession
of ADR-DOC-001 stands, because it is about where identity lives and not about how an author
types.

What follows from the gap, and is the reason it is worth recording rather than merely
fixing: **markdown files under `docs/records` are still the substrate in practice.** These
records are authored as files, seeded into the store from those files by `include_str!`, and
edited as files. The store holds a copy. Identity is stable because the seed says so, not
because an authoring transaction ever produced it — and the direction of that dependency is
the opposite of what this record decides.

`P9-AUTHORING` carries the round trip. It is a phase of work rather than a defect to patch,
and stating the gap is what makes it schedulable.

## What Is Built (amended at version 3)

The round trip exists. `nomos spec markdown` renders a record out of the store's own rows —
the declared front matter and the block rows, never the ingested blob — and `nomos spec
preview` and `nomos spec commit` write an edit back as one transaction. The preview is
mandatory in the only way that survives a careless caller: committing takes a preview value
that nothing outside the module can construct, so an unpreviewed commit is not a lapse of
discipline but an unwritable program. The preview names whether normative wording moved, from
the normalizer over the blocks and from the recorded statements where a corpus has supplied
them, and says which of the two it used.

Identity survives the round trip, and a rename is an ordinary edit: the document row is
updated rather than replaced, so the block surrogates every lineage row hangs from are
untouched, and retyping `id:` in a staged file is refused because identity is not a property
of the file.

What this does not do is move the substrate, and that half is now recorded rather than
implied. Markdown files under `docs/records` are still what this repository authors in,
because nothing persists a specification database: the store is assembled per invocation from
records compiled into the binary. `OD-SPEC-006` states that on the evidence, says what would
have to change for the store to become durable, and names the obstacle already recorded
against the obvious route.

So the second paragraph of the decision now describes the system. What changed underneath it
is the evidence rather than the dependency: the store used to hold a copy of unknown
fidelity, and it now holds a projection asserted byte for byte, in both directions, over every
record in it.

## Alternatives Considered

Amending ADR-DOC-001 in place was rejected. The reasoning that produced it is sound and
worth keeping legible; overwriting it would destroy the record of why the earlier answer
was reasonable.

Leaving ADR-DOC-001 accepted and adding a narrower record about identity was rejected.
Two accepted records disagreeing about the substrate is the ambiguity this decision exists
to remove.

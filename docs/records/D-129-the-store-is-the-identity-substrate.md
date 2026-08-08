---
id: D-129
type: decision
title: The specification store is the identity substrate; markdown is an editing surface
status: accepted
version: 1
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

## Alternatives Considered

Amending ADR-DOC-001 in place was rejected. The reasoning that produced it is sound and
worth keeping legible; overwriting it would destroy the record of why the earlier answer
was reasonable.

Leaving ADR-DOC-001 accepted and adding a narrower record about identity was rejected.
Two accepted records disagreeing about the substrate is the ambiguity this decision exists
to remove.

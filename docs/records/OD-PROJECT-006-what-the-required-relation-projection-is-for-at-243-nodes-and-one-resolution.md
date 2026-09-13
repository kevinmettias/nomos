---
id: OD-PROJECT-006
type: decision
title: What the required relation projection is for at two hundred and forty-seven nodes and one resolution
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - projection
  - spec
relations:
  - target: OD-PROJECT-002
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
  - target: OD-SPEC-015
    type: relates-to
---

# What the required relation projection is for at two hundred and forty-seven nodes and one resolution

## Question

`diagrams/relations.mmd` is one of the two committed projections and one of the two
`spec freshness` requires. It is large, it has one resolution, and nearly one commit in five
pays to re-render it. The `README` opens by claiming a canonical, multi-resolution model of
software, and this projection has exactly one resolution — there is no aggregation anywhere in
selection, no group-by and no rollup, so nothing can show how the `OD-LEDGER` decisions stand
to the `OD-GATE` ones without drawing every node.

`OD-PROJECT-002` decided which projections are required and `OD-PROJECT-003` what committing
one costs. Neither asked whether the required one can be read.

## What Was Measured

Measured 2026-09-13 over the committed diagram, its own sidecar, and this repository's git history.

**The artifact.** 1,879 lines, 90,749 bytes, 247 distinct nodes, 1,646 edges, two subgraphs.
`OD-SPEC-015` measured that 93 per cent of the edges carry the single label `relates-to`, which
is paired with itself, so each authored pair is drawn once in each direction with the same
word.

**Its cost, and where that cost actually is.** 155 of the 844 commits in the last thirty days
touched this file — 18 per cent, nearly one in five. But rendering it takes **495
milliseconds**, and rendering the domain specification beside it takes 654. The re-render is
not what costs; `OD-GATE-005` requires that re-render to come from a worktree holding exactly
the record set the commit publishes, so **the tax is constructing that worktree and building a
binary in it**, about a minute and a half, and it is paid per commit that touches
`docs/records/` whatever the diagram's size.

**Its narrowing mechanisms.** `Filter::identifier_prefix` exists and **no shipped profile uses
it.** No aggregation of any kind exists in selection.

**What its sidecar holds.** `diagrams/relations.mmd.nomos-projection.json` carries a
`content_digest`, an `inputs_digest`, the section counts, and **one hash per input — 1,646 of
them**, each naming an edge by identity.

## The Decision

**The diagram stays required, exactly as it is. What reads it is `spec freshness`, not a
person — and the tax it is blamed for is not a function of its size, so making it smaller
would buy nothing.**

### What reads it

The sidecar answers this directly and it is worth stating plainly, because the question assumes
the other answer: the required relation projection is a **derived, committed manifest that the
store is compared against**. `spec freshness` reads the body, recomputes it from the store, and
separates two failures that share exit code 8 — the store moved under an unchanged body, and
somebody typed into the body. Its 1,646 per-input hashes are what make that comparison
per-edge rather than per-file.

Nobody reads 1,646 arrows, and at this size no mermaid renderer is likely to draw them
usefully. **That is not a defect in the artifact; it is a category error in the question.** Its
readability was never what made it worth requiring. What made it worth requiring is that it is
*derived*, so it cannot disagree with the store without something noticing.

### Why a coarser required projection would save nothing

This is the answer the measurement forces, against the intuition.

Rendering is 495 milliseconds. The tax is the worktree and the build, and `OD-GATE-005` imposes
that on any commit touching a record — for the reason that record gives, which is that the
shared tree holds other sessions' unlanded records and is never the tree a commit publishes.
A ten-node diagram would cost precisely the same to re-render, because the cost is not in the
rendering.

So replacing the required projection with a coarser one would change what the artifact shows
without changing what it costs, and would weaken the comparison: a rollup has fewer inputs to
hash, so a per-edge change could stop being visible. Paying the same tax for a weaker check is
the wrong trade in both directions at once.

### Why the requirement does not move off it either

`OD-PROJECT-002` required it so a stale one is caught, and nothing else catches one. Removing
the requirement would mean `spec freshness --require diagram-set` no longer asks, and a
diagram that had silently stopped matching the store would be discovered by whoever next
happened to look. The recurring red CI this projection is associated with is the check working,
not the check misfiring.

### What is actually missing, and it is not this artifact

A reader who wants to see how the `OD-LEDGER` decisions stand to the `OD-GATE` ones has
nothing, and that want is real. But it is a want for **a second artifact nobody requires**, not
a change to this one, and it needs something that does not exist: aggregation in selection.
`identifier_prefix` alone cannot express it — it narrows to one family, which draws that
family's internal edges and every edge leaving it as a dangling end, rather than rolling
families into nodes and edges between them.

That is filed as its own item. It is deliberately not required, for the reason above: a
projection is required because it is a re-render obligation, and a reading aid that nobody
compares against the store has no obligation to discharge.

## What This Record Does Not Do

It does not change any profile, the required set, selection code, or `Filter`.

It does not reopen `OD-GATE-005`'s rule about which tree a projection is rendered from, or
`OD-PROJECT-002`'s required set. It measures the cost that rule imposes and says where that
cost is, which is a different thing from disputing it.

It does not claim the diagram is readable. It claims readability is not what it is for, and
names what is.

It does not build the coarser view. `diagrams/relations.mmd` is re-rendered by the item that
lands this record only because registering a record changes the record set — never for a
content reason of its own.

## Status

Accepted. The required relation projection is a derived manifest the store is compared
against, read by `spec freshness` and not by a person; its 1,646 per-input hashes are the
comparison, its size is not a defect, and the re-render tax it is blamed for is the worktree
and the build that `OD-GATE-005` requires rather than the 495 milliseconds of rendering — so a
coarser required projection would pay the same cost for a weaker check. The multi-resolution
view the question really wants is a second, unrequired artifact needing aggregation selection
does not have, and is filed separately.

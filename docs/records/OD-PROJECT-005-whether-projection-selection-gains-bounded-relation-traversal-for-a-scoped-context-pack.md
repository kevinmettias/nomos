---
id: OD-PROJECT-005
type: decision
title: Whether projection selection gains bounded relation traversal for a scoped context pack
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - projection
  - spec
  - records
relations:
  - target: OD-PROJECT-002
    type: relates-to
  - target: OD-SPEC-015
    type: relates-to
  - target: OD-SPEC-012
    type: relates-to
---

# Whether projection selection gains bounded relation traversal for a scoped context pack

## Question

`implementation-context-pack` is the one shipped profile whose declared format is `contextpack`
and whose output is `context/implementation.json` — the shape meant to be an agent's focused
context. It is the least focused profile shipped.

Its three sections are every node of kind `decision`, every normative statement with no filter
at all, and every prose block. Nothing scopes it to a task, a subject or a question. So an
agent needing the record that governs the thing it is about to change must either read the
whole decision set or already know the identifier — and not knowing the identifier is the
condition a context pack exists to remove.

## What Was Measured

Measured 2026-09-13.

**The pack's three sections, verbatim from the profile.** `nodes` filtered to
`kind: decision`; `statements` with no filter; `blocks` filtered to `kind: prose`. No subject,
no scope, no question.

**Selection cannot do better today.** `Filter` is a flat conjunction of equality, prefix,
either-column and semi-join clauses over one query, and no code path in `select` follows an
edge. `subject-dossier`, the nearest thing to a scoped projection, selects the subject's own
node, its own statements and its own relation rows — and `Gather_Relations` returns
`(from, relation_type, to, tier, suite)`, five identities and no content. A neighbour's content
is never retrieved and a second hop does not exist.

**The scale the pack would carry.** `spec/domain-specification.md`, the committed projection of
the same store's records half, is **5,162,557 bytes over 63,195 lines** for 232 records —
about 22 KB of rendered content per record.

**Reachability, over the real record graph.** 232 records, treating every edge as undirected
because `relates-to` is its own inverse:

| from any record | median reached | mean | max | share of corpus |
|---|---|---|---|---|
| 1 hop | 7 | 8.0 | 36 | 3% |
| 2 hops | 43 | 45.7 | 161 | 19% |
| 3 hops | 144 | 142.8 | 238 | **62%** |

Degree is median 6, mean 7.0, max 35 — `ARC-ECOSYSTEM-001`.

**Following only the directional terms reaches nothing.** Restricted to
`affects`/`affected_by`/`supersedes`, the median reach is **1 — the record itself — at one, two
and three hops alike**, and **177 of 232 records (76%) have no directional edge at all.**

## The Decision

**Yes. Selection gains bounded relation traversal, at exactly one hop, following every relation
term, emitting the subject's content inlined and each reached neighbour cited by identity,
title and declared status, ordered by identity.**

Each of the five open questions is answered by a measurement rather than a preference.

### Which terms are followed: all of them, `relates-to` included

This looks like the question with the most room in it and has none. `OD-SPEC-015` measured
`relates-to` at 93 per cent of authored edges; the traversal measurement above is the other
half of that fact. A traversal that followed only the directional terms would reach **the
record it started from and nothing else**, for three quarters of the corpus, at any hop count.

So "follow the meaningful edges and skip the vague ones" is not available. There is one graph,
`relates-to` is nearly all of it, and a traversal either follows it or does not exist.

### The stop condition: one hop, and the bound is the decision rather than a parameter

A hop bound stops bounding almost immediately. One hop reaches 3 per cent of the corpus; two
reaches 19 per cent and ranges from 7 records to 161 depending on where it starts; three
reaches 62 per cent and, from the best-connected records, everything. The graph's diameter is
small because `relates-to` is symmetric and dense, which is exactly what makes a second hop
worthless as a bound.

A node budget is refused for the same measurement. A budget would only be needed above one hop,
and above one hop it must truncate a set that varies more than twenty-fold by starting point —
so two agents asking about neighbouring records would get packs neither of them could compare,
and which one got truncated would depend on a graph property neither of them chose.

One hop is therefore fixed, not configurable. A caller wanting a neighbour's neighbourhood asks
about the neighbour, which is a second question with its own honest answer, rather than a
parameter that silently changes what a pack means.

### Declared status participates, as a report and never as a filter

`Filter` gained `status` this week, and ten of this repository's governing questions declare
`status: open`.

A pack that filtered reached records to `accepted` would hide precisely the unsettled questions
an implementer most needs to know are unsettled. One that filtered to `open` would hide the
settled ground the work stands on. Both are worse than not asking.

So status does not gate traversal. It is **carried on each citation**, so a pack says of every
neighbour whether the thing it governs is decided or still open — which is the same reasoning
`P95-A-RECORDS-DECLARED-STATUS-CANNOT-BE-SELECTED` acted on for projections generally: the
field existed and nothing could read it back, so open questions rendered as settled decisions.

### Emission order: by identity, never by discovery

Every existing gather ends in an explicit `Ordered_By` over an identity column — `n.node_id`,
`s.statement_id`, `f.node_id, r.relation_type, t.node_id`. Reached content is emitted by
`node_id` ascending, extending that rule rather than adding a second one.

The alternative, discovery order, would make the output depend on frontier iteration, and two
renders of one store would stop being byte-identical — which the determinism contract the
freshness sidecar rests on does not permit. Ordering by identity also means a pack's diff
between two renders shows what changed rather than what moved.

### Inlined subject, cited neighbours

The subject's own content is inlined. Each reached neighbour is cited by identity, title and
declared status, plus the section anchor the citation points at — not inlined.

At about 22 KB of rendered content per record, a median one-hop neighbourhood of 7 inlined is
around 155 KB, which would be usable. The maximum is 36, around 790 KB, which would not be —
and the record that hits it, `ARC-ECOSYSTEM-001`, is one of the ones most worth asking about.
Inlining would make a pack's size a function of its subject's popularity rather than of the
question, and the agent least able to afford a large pack is the one asking about the
best-connected record.

Citation by identity is also what the asking agent actually lacked. It did not need the
neighbour's prose; it needed to know the neighbour exists and what it is called. A second call
fetches one.

## What This Record Does Not Do

It does not change any profile, including `implementation-context-pack`. It does not add a
filter field, change selection code, or change a renderer.

It does not decide the spelling of the traversal — whether it is a new content kind, a section
option, or a profile-level declaration — or what the citation's JSON looks like. Those are the
building item's, against the format `contextpack` already has.

It does not extend traversal to the corpus half of the store. Every measurement here is over
`docs/records/`, and a corpus node's edge population has not been measured.

It does not reopen `OD-SPEC-015`. That `relates-to` carries 93 per cent of the edges is taken
as measured there and used here; nothing about this decision asks it to be otherwise.

## Status

Accepted. Selection gains bounded relation traversal at exactly one hop, following every term
because following only the directional ones reaches nothing for 76 per cent of records,
stopping at one hop because two already ranges from 7 records to 161 and three reaches 62 per
cent of the corpus, carrying declared status on each citation rather than filtering by it,
emitting by identity so two renders stay byte-identical, and citing neighbours rather than
inlining them so a pack's size follows its question and not its subject's popularity.
